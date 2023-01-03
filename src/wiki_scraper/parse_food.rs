use itertools::Itertools;
use log::{error, info};
use regex::Regex;
use std::error::Error;

use crate::{
    common::{food::FoodRecord, game::Pack},
    wiki_scraper::common::{get_page_info, remove_icon_names},
};

lazy_static! {
    static ref RGX_FOOD_LINK_NAME: Regex = Regex::new(r#"\[\[(.*?)\]\]"#).unwrap();
}

pub fn parse_food_info(url: &str) -> Result<Vec<FoodRecord>, Box<dyn Error>> {
    let response = get_page_info(url)?;
    let mut foods: Vec<FoodRecord> = vec![];

    // TODO: Add TableNotFound error to replace unwrap_or with ok_or
    let table = &response
        .split("\n\n")
        .max_by(|x, y| x.len().cmp(&y.len()))
        .unwrap_or("")
        .split("|-")
        .collect_vec();

    for (table_n, food_info) in table.iter().enumerate() {
        // First section contains pack info.
        if table_n == 0 {
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
        food_info_copy = remove_icon_names(&food_info_copy)
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
                for pack in packs {
                    foods.push(FoodRecord {
                        name: name.to_string(),
                        tier: tier_n,
                        effect: effect.to_string(),
                        pack,
                    });
                }
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
