use crate::common::{game::Pack, pet::Pet};
use itertools::Itertools;
use log::info;
use regex::Regex;
use reqwest::blocking;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

lazy_static! {
    static ref RGX_TIER: Regex = Regex::new(r#"<!--\sTIER\s(\d)\s-->"#).unwrap();
    static ref RGX_PET_NAME: Regex = Regex::new(r#"pet\s=\s\{\{IconSAP\|(.*?)\|size"#).unwrap();
    static ref RGX_PET_STATS: Regex =
        Regex::new(r#"attack\s=\s(?P<attack>\d+)\s\|\shealth\s=\s(?P<health>\d+)"#).unwrap();
    static ref RGX_PET_PACK: Regex = Regex::new(r#"(\w+pack)+"#).unwrap();
    static ref RGX_PET_EFFECT_TRIGGER: Regex = Regex::new(r#"\| ''+(.*?)''+"#).unwrap();
    static ref RGX_PET_EFFECT: Regex = Regex::new(r#"→\s(.*?)\n"#).unwrap();
    static ref RGX_ICON_NAME: Regex =
        Regex::new(r#"\{\{IconSAP\|(.*?)[\|\}]+.*?([\w\|]*=[\w\.]+)*"#).unwrap();
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
    info!(target: "wiki_scraper", "Retrieving page info for {url}...");
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
    final_line.to_string().replace('}', "").replace("'''", "")
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
            info!(target: "wiki_scraper", "On {}...", pet_name);

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
                .unwrap_or("None".to_string());

            let pet_effect = RGX_PET_EFFECT
                .captures_iter(line)
                .map(|cap| {
                    cap.get(1).map_or("None".to_string(), |effect| {
                        parse_icon_names(&effect.as_str())
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
