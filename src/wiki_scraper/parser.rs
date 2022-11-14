use itertools::Itertools;
use log::{error, info};
use regex::Regex;
use reqwest::blocking;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::common::{food::Food, game::Pack, pet::Pet};

lazy_static! {
    static ref RGX_TIER: Regex = Regex::new(r#"<!--\sTIER\s(\d)\s-->"#).unwrap();
    static ref RGX_PET_NAME: Regex = Regex::new(r#"pet\s=\s\{\{IconSAP\|(.*?)\|size"#).unwrap();
    static ref RGX_PET_STATS: Regex =
        Regex::new(r#"attack\s=\s(?P<attack>\d+)\s\|\shealth\s=\s(?P<health>\d+)"#).unwrap();
    static ref RGX_PET_PACK: Regex = Regex::new(r#"(\w+pack)+"#).unwrap();
    static ref RGX_PET_EFFECT_TRIGGER: Regex = Regex::new(r#"\| ''+(.*?)''+"#).unwrap();
    // TODO: Misses animals with no triggers. (Tiger)
    static ref RGX_PET_EFFECT: Regex = Regex::new(r#"→\s(.*?)\n"#).unwrap();
    static ref RGX_ICON_NAME: Regex =
        Regex::new(r#"\{\{IconSAP\|(.*?)[\|\}]+.*?([\w\|]*=[\w\.]+)*"#).unwrap();
    static ref RGX_FOOD_LINK_NAME: Regex = Regex::new(r#"\[\[(.*?)\]\]"#).unwrap();
}

#[derive(Deserialize, Debug)]
pub struct SAPWikiSources {
    pub pets: String,
    pub food: String,
}

pub fn read_wiki_url<P: AsRef<Path>>(path: P) -> Result<SAPWikiSources, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let urls = serde_json::from_reader(reader)?;
    Ok(urls)
}

pub fn get_page_info(url: &str) -> Result<String, Box<dyn Error>> {
    info!(target: "wiki_scraper", "Retrieving page info for {url}.");
    Ok(blocking::get(url)?.text()?)
}

fn parse_icon_names(line: &str) -> String {
    let mut final_line = line.to_string();
    let final_line_copy = final_line.clone();

    for cap in RGX_ICON_NAME.captures_iter(&final_line_copy) {
        // If an arg exists for icon, capture it.
        let icon_arg = cap.get(2).map(|cap| cap.as_str()).unwrap_or("");
        // If no name arg exists for icon, substitute icon name.
        let icon_name = if (icon_arg.is_empty()) | (!icon_arg.contains("name=")) {
            cap.get(1)
                .map(|cap| cap.as_str())
                .unwrap_or("Missing")
                .to_string()
        } else {
            icon_arg.replace("name=", "").to_string()
        };
        // Replace first instance in string.
        final_line = RGX_ICON_NAME
            .replacen(&final_line, 1, icon_name)
            .to_string();
    }
    // Remove remaining }, if any.
    // Remove '''
    final_line.replace('}', "").replace("'''", "")
}

pub fn parse_pet_info(url: &str) -> Result<Vec<Pet>, Box<dyn Error>> {
    let response = get_page_info(url)?;
    let mut pets: Vec<Pet> = vec![];
    let mut curr_tier: usize = 1;

    for line in response.split("\n\n") {
        // Update the pet tier.
        if RGX_TIER.is_match(line) {
            curr_tier = RGX_TIER
                .captures(line)
                .and_then(|cap| cap.get(1))
                .and_then(|tier| tier.as_str().parse::<usize>().ok())
                .unwrap();
            info!(target: "wiki_scraper", "On tier {curr_tier} animals...");
        }
        // If a pet name is found.
        if RGX_PET_NAME.is_match(line) {
            let pet_name = RGX_PET_NAME
                .captures(line)
                .and_then(|cap| cap.get(1).map(|cap| cap.as_str()))
                .unwrap();

            let pet_stats = RGX_PET_STATS.captures(line).unwrap();
            // TODO: Default to 0 on parse error.
            let pet_atk: usize = pet_stats
                .name("attack")
                .map_or(0, |m| m.as_str().parse().unwrap_or(0));
            let pet_health: usize = pet_stats
                .name("health")
                .map_or(0, |m| m.as_str().parse().unwrap_or(0));

            let pet_packs = RGX_PET_PACK
                .captures_iter(line)
                .map(|cap| match cap.get(1).unwrap().as_str() {
                    "starpack" => Pack::Star,
                    "puppypack" => Pack::Puppy,
                    "turtlepack" => Pack::Turtle,
                    "weeklypack" => Pack::Weekly,
                    _ => Pack::Unknown,
                })
                .collect_vec();

            let pet_effect_trigger = RGX_PET_EFFECT_TRIGGER
                .captures(line)
                .and_then(|cap| {
                    cap.get(1).map_or(Some("None".to_string()), |cap| {
                        Some(parse_icon_names(cap.as_str()))
                    })
                })
                .unwrap_or_else(|| "None".to_string());

            let pet_effect = RGX_PET_EFFECT
                .captures_iter(line)
                .map(|cap| {
                    cap.get(1).map_or("None".to_string(), |effect| {
                        parse_icon_names(effect.as_str())
                    })
                })
                .collect_vec();

            let pet = Pet::new(
                pet_name,
                curr_tier,
                pet_atk,
                pet_health,
                &pet_packs,
                &pet_effect_trigger,
                &pet_effect,
            );
            pets.push(pet)
        }
    }
    info!(target: "wiki_scraper", "Retrieved {} pets.", pets.len());
    Ok(pets)
}

pub fn parse_food_info(url: &str) -> Result<Vec<Food>, Box<dyn Error>> {
    let response = get_page_info(url)?;
    let mut foods: Vec<Food> = vec![];

    // TODO: Add TableNotFound error to replace unwrap_or with ok_or
    let table = &response
        .split("\n\n")
        .max_by(|x, y| x.len().cmp(&y.len()))
        .unwrap_or("")
        .split("|-")
        .collect_vec();

    for (i, food_info) in table.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let mut food_info_copy = food_info.to_string();

        for capture in RGX_FOOD_LINK_NAME.captures_iter(food_info) {
            // Get last element in link text.
            // ex. |Give one [[Pets|pet]] [[Lemon]]. -> Give one pet Lemon.
            for (i, mtch) in capture.iter().enumerate() {
                // Skip first match which matches everything.
                if i == 0 {
                    continue;
                }
                let food_name = mtch
                    .map_or("", |m| m.as_str())
                    .split('|')
                    .last()
                    .unwrap_or("");
                // Update line copy replacing links wiht food name.
                food_info_copy = RGX_FOOD_LINK_NAME
                    .replacen(&food_info_copy, 1, food_name)
                    .to_string();
            }
        }
        food_info_copy = parse_icon_names(&food_info_copy)
            .replace('\n', "")
            .replace("<br>", "");

        // Remove the first character if is '|'.
        if food_info_copy.chars().next().unwrap_or('_') == '|' {
            let mut food_info_copy_chars = food_info_copy.chars();
            food_info_copy_chars.next();
            food_info_copy = food_info_copy_chars.as_str().to_string();
        }

        // Remove the last character if is '|'.
        if food_info_copy.chars().last().unwrap_or('_') == '|' {
            let mut food_info_copy_chars = food_info_copy.chars();
            food_info_copy_chars.next_back();
            food_info_copy = food_info_copy_chars.as_str().to_string();
        }

        if let Some((mut tier, name, effect, turtle_pack, puppy_pack, star_pack)) =
            food_info_copy.split('|').collect_tuple()
        {
            // Map tiers that are N/A to 0. ex. Coconut which is summoned.
            tier = if tier == "N/A" { "0" } else { tier };

            let available_packs = [Pack::Turtle, Pack::Puppy, Pack::Star];
            let packs = [turtle_pack, puppy_pack, star_pack]
                .iter()
                .enumerate()
                // Access pack by index. Need to be same length. Cannot zip as unstable feature.
                .filter_map(|(i, pack_desc)| {
                    // If pack description doesn't list item in pack, ignore.
                    if pack_desc.contains("Yes") {
                        let pack = available_packs.get(i).unwrap_or(&Pack::Unknown);
                        Some(pack.clone())
                    } else {
                        None
                    }
                })
                .collect_vec();

            // Attempt convert tier to usize.
            let tier_n_conversion = tier.parse::<usize>();
            if let Ok(tier_n) = tier_n_conversion {
                foods.push(Food::new(name, tier_n, effect, &packs[..]));
            } else {
                error!(target: "wiki_scraper", "Unable to convert tier {tier} for {name} to usize.")
            }
        } else {
            error!(target: "wiki_scraper", "Missing fields for {food_info_copy}.");
        }
    }
    info!(target: "wiki_scraper", "Retrieved {} foods.", foods.len());
    Ok(foods)
}
