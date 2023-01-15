use itertools::Itertools;
use log::{error, info};
use std::{error::Error, str::FromStr};

use crate::{
    common::regex_patterns::{
        RGX_ATK, RGX_DMG, RGX_DMG_REDUCE, RGX_END_OF_BATTLE, RGX_END_TURN, RGX_HEALTH, RGX_ONE_USE,
        RGX_RANDOM, RGX_START_TURN, RGX_SUMMON_ATK, RGX_SUMMON_HEALTH,
    },
    db::{pack::Pack, record::FoodRecord},
    wiki_scraper::{
        common::{get_page_info, remove_icon_names},
        error::WikiParserError,
        regex_patterns::*,
    },
};

const TABLE_STR_DELIM: &str = "|-";
const SINGLE_USE_ITEMS_EXCEPTIONS: [&str; 2] = ["Pepper", "Sleeping Pill"];
const HOLDABLE_ITEMS_EXCEPTIONS: [&str; 3] = ["Coconut", "Weakness", "Peanuts"];

#[derive(Debug, PartialEq, Eq)]
pub enum FoodTableCols {
    Name,
    Tier,
    Effect,
    GamePack(Pack),
}

impl FromStr for FoodTableCols {
    type Err = WikiParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Name" => Ok(FoodTableCols::Name),
            "Tier" => Ok(FoodTableCols::Tier),
            "Effect" => Ok(FoodTableCols::Effect),
            "Turtle Pack" => Ok(FoodTableCols::GamePack(Pack::Turtle)),
            "Puppy Pack" => Ok(FoodTableCols::GamePack(Pack::Puppy)),
            "Star Pack" => Ok(FoodTableCols::GamePack(Pack::Star)),
            _ => Err(WikiParserError {
                reason: format!("Unknown column {s}."),
            }),
        }
    }
}

/// Clean text removing:
/// * Links `[[...|...]]`
/// * Icon names `{IconSAP|...}`.
pub fn clean_link_text(text: &str) -> String {
    let mut text_copy = text.to_string();

    for capture in RGX_FOOD_LINK_NAME.captures_iter(text) {
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
            text_copy = RGX_FOOD_LINK_NAME
                .replacen(&text_copy, 1, food_name)
                .to_string();
        }
    }
    remove_icon_names(&text_copy).trim_matches('|').to_string()
}

/// Get the largest table and its values contained by `{|...|}` and split it into rows.
pub fn get_largest_table(page_info: &str) -> Result<Vec<&str>, Box<WikiParserError>> {
    let largest_block = page_info
        .split("\n\n")
        .max_by(|blk_1, blk_2| blk_1.len().cmp(&blk_2.len()))
        .ok_or(WikiParserError {
            reason: "Largest text block not found.".to_string(),
        })?;

    if let Some(Some(largest_table)) = RGX_TABLE.captures(largest_block).map(|cap| cap.get(0)) {
        Ok(largest_table.as_str().split(TABLE_STR_DELIM).collect_vec())
    } else {
        Err(Box::new(WikiParserError {
            reason: "Can't find main table following format: {|...|}.".to_string(),
        }))
    }
}

/// Get table column names from the header row of a `fandom-table`.
///
/// These are mapped to `FoodTableCols`.
pub fn get_cols(cols_str: &str) -> Result<Vec<FoodTableCols>, WikiParserError> {
    let cols: Option<Vec<FoodTableCols>> = RGX_COLS
        .captures_iter(cols_str)
        .filter_map(|capt|
            // Get capture and remove newlines and !.
            // !Name\n -> Name
            capt.get(1).map(|mtch| mtch.as_str().trim_matches(|c| c == '\n' || c == '!')))
        .map(|colname| FoodTableCols::from_str(colname).ok())
        .collect();

    cols.ok_or(WikiParserError {
        reason: format!("One or more cols is unknown in col_str: {}.", cols_str),
    })
}

/// Parse food info into a list of `Food`s.
pub fn parse_food_info(url: &str) -> Result<Vec<FoodRecord>, Box<dyn Error>> {
    let response = get_page_info(url)?;
    let mut foods: Vec<FoodRecord> = vec![];

    let table = get_largest_table(&response)?;

    // Can safely unwrap here as will catch above.
    let cols = get_cols(table.first().unwrap())?;

    // Skip first table which contains columns.
    for food_info in table.get(1..).expect("No table elements.").iter() {
        let clean_food_info = clean_link_text(food_info.trim());

        if let Some((mut tier, name, effect, turtle_pack, puppy_pack, star_pack)) = cols
            .iter()
            .zip_eq(clean_food_info.split('|').map(|col_info| col_info.trim()))
            .collect_tuple()
        {
            // Map tiers that are N/A to 0. ex. Coconut which is summoned.
            tier.1 = if tier.1 == "N/A" { "0" } else { tier.1 };

            let packs = [turtle_pack, puppy_pack, star_pack]
                .iter()
                .filter_map(|(pack, pack_desc)| {
                    if let FoodTableCols::GamePack(pack_name) = pack {
                        // If pack description doesn't list item in pack, ignore.
                        pack_desc.contains("Yes").then_some(pack_name.clone())
                    } else {
                        None
                    }
                })
                .collect_vec();

            let holdable_item = effect.1.contains(&format!("Give one pet {}", name.1))
                || HOLDABLE_ITEMS_EXCEPTIONS.contains(&name.1);
            let temp_effect = RGX_END_OF_BATTLE.is_match(effect.1);
            // Hardcode single use items for now.
            let single_use = RGX_ONE_USE.is_match(effect.1)
                || SINGLE_USE_ITEMS_EXCEPTIONS.contains(&name.1)
                || temp_effect;

            let (random, n_targets) = if let Some(Some(n_random_target)) =
                RGX_RANDOM.captures(effect.1).map(|cap| cap.get(1))
            {
                let n = n_random_target.as_str();
                (
                    true,
                    match n {
                        "one" | "a" => 1,
                        "two" => 2,
                        "three" => 3,
                        _ => n.parse()?,
                    },
                )
            } else {
                (false, 1)
            };
            let turn_effect = RGX_START_TURN.is_match(effect.1) || RGX_END_TURN.is_match(effect.1);
            let end_of_battle = RGX_END_OF_BATTLE.is_match(effect.1);
            let effect_atk = if let Some(Some(atk_buff)) =
                RGX_ATK.captures(effect.1).map(|cap| cap.get(1))
            {
                atk_buff.as_str().parse::<isize>()?
            } else if let Some(Some(atk_buff)) = RGX_DMG.captures(effect.1).map(|cap| cap.get(1)) {
                atk_buff.as_str().parse::<isize>()?
            } else if let Some(Some(reduced_dmg)) =
                RGX_DMG_REDUCE.captures(effect.1).map(|cap| cap.get(1))
            {
                reduced_dmg.as_str().parse::<isize>()?
            } else if let Some(Some(summon_atk)) =
                RGX_SUMMON_ATK.captures(effect.1).map(|cap| cap.get(1))
            {
                summon_atk.as_str().parse::<isize>()?
            } else {
                0
            };
            let effect_health = if let Some(Some(health_buff)) =
                RGX_HEALTH.captures(effect.1).map(|cap| cap.get(1))
            {
                health_buff.as_str().parse::<isize>()?
            } else if let Some(Some(summon_health)) =
                RGX_SUMMON_HEALTH.captures(effect.1).map(|cap| cap.get(1))
            {
                summon_health.as_str().parse::<isize>()?
            } else {
                0
            };

            // Attempt convert tier to usize.
            if let Ok(tier_n) = tier.1.parse::<usize>() {
                for pack in packs {
                    foods.push(FoodRecord {
                        name: name.1.to_string(),
                        tier: tier_n,
                        // Remove newlines and replace any in-between effect desc.
                        effect: effect.1.replace('\n', " "),
                        pack,
                        holdable: holdable_item,
                        single_use,
                        end_of_battle,
                        random,
                        n_targets,
                        effect_atk,
                        effect_health,
                        turn_effect,
                    });
                }
            } else {
                error!(target: "wiki_scraper", "Unable to convert tier {} for {} to usize.", tier.1, name.1)
            }
        } else {
            error!(target: "wiki_scraper", "Missing fields for {clean_food_info}. Needs {:?}", cols);
        }
    }
    info!(target: "wiki_scraper", "Retrieved {} foods.", foods.len());
    Ok(foods)
}
