use itertools::Itertools;
use log::{error, info};
use std::{error::Error, str::FromStr};

use crate::regex_patterns::{
    RGX_ATK, RGX_DMG, RGX_DMG_REDUCE, RGX_END_OF_BATTLE, RGX_END_TURN, RGX_HEALTH, RGX_ONE_USE,
    RGX_RANDOM, RGX_START_TURN, RGX_SUMMON_ATK, RGX_SUMMON_HEALTH,
};
use crate::FoodName;
use crate::{
    db::{pack::Pack, record::FoodRecord},
    error::SAPTestError,
    regex_patterns::*,
    wiki_scraper::common::{get_page_info, remove_icon_names},
};

const TABLE_STR_DELIM: &str = "|-";
const SINGLE_USE_ITEMS_EXCEPTIONS: [&str; 2] = ["Pepper", "Sleeping Pill"];
const HOLDABLE_ITEMS_EXCEPTIONS: [&str; 3] = ["Coconut", "Weakness", "Peanuts"];
const ONE_GOLD_ITEMS_EXCEPTIONS: [&str; 1] = ["Sleeping Pill"];
const DEFAULT_FOOD_COST: usize = 3;

#[derive(Debug, PartialEq, Eq)]
pub enum FoodTableCols {
    Name,
    Tier,
    Effect,
    GamePack(Pack),
}

impl FoodTableCols {
    pub fn get_cols(cols_str: &str) -> Result<Vec<FoodTableCols>, SAPTestError> {
        let cols: Option<Vec<FoodTableCols>> = RGX_COLS
            .captures_iter(cols_str)
            .filter_map(|capt|
                // Get capture and remove newlines and !.
                // !Name\n -> Name
                capt.get(1).map(|mtch| mtch.as_str().trim_matches(|c| c == '\n' || c == '!')))
            .map(|colname| FoodTableCols::from_str(colname).ok())
            .collect();

        cols.ok_or(SAPTestError::ParserFailure {
            subject: "Food Table Columns".to_string(),
            reason: format!("One or more cols is unknown in col_str: {}.", cols_str),
        })
    }
}
impl FromStr for FoodTableCols {
    type Err = SAPTestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Name" => Ok(FoodTableCols::Name),
            "Tier" => Ok(FoodTableCols::Tier),
            "Effect" => Ok(FoodTableCols::Effect),
            "Turtle Pack" => Ok(FoodTableCols::GamePack(Pack::Turtle)),
            "Puppy Pack" => Ok(FoodTableCols::GamePack(Pack::Puppy)),
            "Star Pack" => Ok(FoodTableCols::GamePack(Pack::Star)),
            _ => Err(SAPTestError::ParserFailure {
                subject: "Food Table Columns".to_string(),
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
pub fn get_largest_table(page_info: &str) -> Result<Vec<&str>, SAPTestError> {
    let largest_block = page_info
        .split("\n\n")
        .max_by(|blk_1, blk_2| blk_1.len().cmp(&blk_2.len()))
        .ok_or(SAPTestError::ParserFailure {
            subject: "Largest Table".to_string(),
            reason: "Largest text block not found.".to_string(),
        })?;

    if let Some(Some(largest_table)) = RGX_TABLE.captures(largest_block).map(|cap| cap.get(0)) {
        Ok(largest_table.as_str().split(TABLE_STR_DELIM).collect_vec())
    } else {
        Err(SAPTestError::ParserFailure {
            subject: "Largest Table".to_string(),
            reason: "Can't find main table following format: {|...|}.".to_string(),
        })
    }
}

/// Check whether food effect is random and the number of targets it affects.
pub fn get_random_n_effect(effect: &str) -> Result<(bool, usize), Box<dyn Error>> {
    if let Some(Some(n_random_target)) = RGX_RANDOM.captures(effect).map(|cap| cap.get(1)) {
        let n = n_random_target.as_str();
        Ok((
            true,
            match n {
                "one" | "a" => 1,
                "two" => 2,
                "three" => 3,
                _ => n.parse()?,
            },
        ))
    } else {
        Ok((false, 1))
    }
}

pub fn get_effect_health(effect: &str) -> Result<isize, Box<dyn Error>> {
    Ok(
        if let Some(Some(health_buff)) = RGX_HEALTH.captures(effect).map(|cap| cap.get(1)) {
            health_buff.as_str().parse::<isize>()?
        } else if let Some(Some(summon_health)) =
            RGX_SUMMON_HEALTH.captures(effect).map(|cap| cap.get(1))
        {
            summon_health.as_str().parse::<isize>()?
        } else {
            0
        },
    )
}

pub fn get_effect_attack(effect: &str) -> Result<isize, Box<dyn Error>> {
    Ok(
        if let Some(Some(atk_buff)) = RGX_ATK.captures(effect).map(|cap| cap.get(1)) {
            atk_buff.as_str().parse::<isize>()?
        } else if let Some(Some(effect_atk_buff)) = RGX_DMG.captures(effect).map(|cap| cap.get(1)) {
            effect_atk_buff.as_str().parse::<isize>()?
        } else if let Some(Some(reduced_dmg)) =
            RGX_DMG_REDUCE.captures(effect).map(|cap| cap.get(1))
        {
            reduced_dmg.as_str().parse::<isize>()?
        } else if let Some(Some(summon_atk)) = RGX_SUMMON_ATK.captures(effect).map(|cap| cap.get(1))
        {
            summon_atk.as_str().parse::<isize>()?
        } else {
            0
        },
    )
}

pub fn is_temp_single_use(name: &str, effect: &str) -> (bool, bool) {
    let temp_effect = RGX_END_OF_BATTLE.is_match(effect);
    // Hardcode single use items for now.
    let single_use =
        RGX_ONE_USE.is_match(effect) || SINGLE_USE_ITEMS_EXCEPTIONS.contains(&name) || temp_effect;

    (temp_effect, single_use)
}

pub fn is_holdable_item(name: &str, effect: &str) -> bool {
    effect
        .to_lowercase()
        .contains(&format!("give one pet {}", name.to_lowercase()))
        || HOLDABLE_ITEMS_EXCEPTIONS.contains(&name)
}

pub fn is_turn_effect(effect: &str) -> bool {
    RGX_START_TURN.is_match(effect) || RGX_END_TURN.is_match(effect)
}
pub fn is_end_of_battle_effect(effect: &str) -> bool {
    RGX_END_OF_BATTLE.is_match(effect)
}

pub fn get_food_cost(name: &str) -> usize {
    if ONE_GOLD_ITEMS_EXCEPTIONS.contains(&name) {
        1
    } else if HOLDABLE_ITEMS_EXCEPTIONS.contains(&name) {
        0
    } else {
        DEFAULT_FOOD_COST
    }
}

/// Parse a single wiki food entry and update a list of `FoodRecord`s.
pub fn parse_one_food_entry(
    food_info: &str,
    cols: &[FoodTableCols],
    foods: &mut Vec<FoodRecord>,
) -> Result<(), Box<dyn Error>> {
    let clean_food_info = clean_link_text(food_info.trim());
    let col_info = clean_food_info
        .split('|')
        .map(|col_info| col_info.trim())
        .collect_vec();

    if cols.len() > col_info.len() {
        return Err(Box::new(SAPTestError::ParserFailure {
            subject: "Food Entry".to_string(),
            reason: format!(
                "Missing food entry fields ({} > {}). Required: {:?}",
                cols.len(),
                col_info.len(),
                cols
            ),
        }));
    }
    if cols.len() < col_info.len() {
        return Err(Box::new(SAPTestError::ParserFailure {
            subject: "Food Entry".to_string(),
            reason: format!(
                "New pack added or extra fields provided ({} < {}). Provided: {:?}",
                cols.len(),
                col_info.len(),
                col_info
            ),
        }));
    };

    if let Some((mut tier, name, effect, turtle_pack, puppy_pack, star_pack)) =
        cols.iter().zip_eq(col_info).collect_tuple()
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

        let holdable_item = is_holdable_item(name.1, effect.1);
        let (_, single_use) = is_temp_single_use(name.1, effect.1);
        let (random, n_targets) = get_random_n_effect(effect.1)?;
        let turn_effect = is_turn_effect(effect.1);
        let end_of_battle = is_end_of_battle_effect(effect.1);
        let effect_atk = get_effect_attack(effect.1)?;
        let effect_health = get_effect_health(effect.1)?;
        let cost = get_food_cost(name.1);

        // Attempt convert tier to usize.
        for pack in packs {
            foods.push(FoodRecord {
                name: FoodName::from_str(name.1)?,
                tier: tier.1.parse::<usize>()?,
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
                cost,
            });
        }
    } else {
    }
    Ok(())
}

/// Parse food info into a list of `Food`s.
pub fn parse_food_info(url: &str) -> Result<Vec<FoodRecord>, Box<dyn Error>> {
    let response = get_page_info(url)?;
    let mut foods: Vec<FoodRecord> = vec![];

    let table = get_largest_table(&response)?;

    // Can safely unwrap here as will catch above.
    let cols = FoodTableCols::get_cols(table.first().unwrap())?;

    // Skip first table which contains columns.
    for food_info in table.get(1..).expect("No table elements.").iter() {
        if let Err(error_msg) = parse_one_food_entry(food_info, &cols, &mut foods) {
            error!(target: "wiki_scraper", "{}", error_msg);
            continue;
        };
    }
    info!(target: "wiki_scraper", "Retrieved {} foods.", foods.len());
    Ok(foods)
}
