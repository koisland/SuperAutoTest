use itertools::Itertools;
use log::{error, info};
use std::{error::Error, str::FromStr};

use crate::regex_patterns::{
    RGX_ATK, RGX_DMG, RGX_DMG_REDUCE, RGX_END_OF_BATTLE, RGX_END_TURN, RGX_HEALTH, RGX_ONE_USE,
    RGX_RANDOM, RGX_START_TURN, RGX_SUMMON_ATK, RGX_SUMMON_HEALTH,
};
use crate::shop::store::MAX_SHOP_TIER;
use crate::FoodName;
use crate::{
    db::{pack::Pack, record::FoodRecord},
    error::SAPTestError,
    regex_patterns::*,
    wiki_scraper::common::get_page_info,
};

use super::common::clean_link_text;
use super::IMG_URLS;

const SINGLE_USE_ITEMS_EXCEPTIONS: [&str; 2] = ["Pepper", "Sleeping Pill"];
const HOLDABLE_ITEMS_EXCEPTIONS: [&str; 2] = ["Coconut", "Peanut"];
const ONE_GOLD_ITEMS_EXCEPTIONS: [&str; 1] = ["Sleeping Pill"];
const DEFAULT_FOOD_COST: usize = 3;

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
    effect.to_lowercase().contains(&name.to_lowercase())
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
    current_tier: usize,
    foods: &mut Vec<FoodRecord>,
) -> Result<(), Box<dyn Error>> {
    let clean_food_info = clean_link_text(food_info.trim());
    let name = RGX_FOOD_NAME
        .captures(&clean_food_info)
        .and_then(|cap| cap.get(1))
        .map(|mtch| mtch.as_str());
    let ability = RGX_FOOD_EFFECT
        .captures(&clean_food_info)
        .and_then(|cap| cap.get(1))
        .map(|mtch| mtch.as_str());
    let packs = RGX_PET_PACK
        .captures_iter(&clean_food_info)
        .filter_map(|cap| cap.get(1).map(|mtch| mtch.as_str()))
        .filter_map(|pack| Pack::from_str(pack.trim_end_matches("pack")).ok())
        .collect_vec();

    let (Some(name), Some(effect)) = (name, ability) else {
        return Err(format!("No name or ability for text: {food_info}").into());
    };

    // Get image url setting no empty string if not found.
    let url = IMG_URLS
        .get(name)
        .map(|data| data.url.clone())
        .unwrap_or_default();

    let holdable_item = is_holdable_item(name, effect);
    let (_, single_use) = is_temp_single_use(name, effect);
    let (random, n_targets) = get_random_n_effect(effect)?;
    let turn_effect = is_turn_effect(effect);
    let end_of_battle = is_end_of_battle_effect(effect);
    let effect_atk = get_effect_attack(effect)?;
    let effect_health = get_effect_health(effect)?;
    let cost = get_food_cost(name);

    // Attempt convert tier to usize.
    for pack in packs {
        foods.push(FoodRecord {
            name: FoodName::from_str(name)?,
            tier: if current_tier > MAX_SHOP_TIER {
                0
            } else {
                current_tier
            },
            // Remove newlines and replace any in-between effect desc.
            effect: effect.trim().replace('\n', " "),
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
            img_url: url.clone(),
            is_ailment: false,
        });
    }
    Ok(())
}

/// Parse food info into a list of `FoodRecord`s.
pub fn parse_food_info(url: &str) -> Result<Vec<FoodRecord>, SAPTestError> {
    let response = get_page_info(url)?;
    let mut foods: Vec<FoodRecord> = vec![];
    let mut curr_tier: usize = 1;

    // Get positions on page of tiers.
    let mut tier_pos: Vec<usize> = RGX_TIER
        .captures_iter(&response)
        .filter_map(|cap| cap.get(1).map(|mtch| mtch.start()))
        .collect();

    // Find no tier positions.
    if let Some(no_tier_pos) = regex!("<!-- No Tier -->")
        .find(&response)
        .map(|mtch| mtch.start())
    {
        tier_pos.push(no_tier_pos)
    }

    for food_row_mtch in RGX_FOOD_ROW
        .captures_iter(&response)
        .filter_map(|cap| cap.get(1))
    {
        let row_pos = food_row_mtch.start();
        // Search by position. Will not find value but returns index of closest index. This index is the curr tier.
        if let Err(calc_curr_tier) =
            tier_pos.binary_search_by(|tier_start_pos| tier_start_pos.cmp(&row_pos))
        {
            curr_tier = calc_curr_tier
        }
        // Parse foods.
        if let Err(error_msg) = parse_one_food_entry(food_row_mtch.as_str(), curr_tier, &mut foods)
        {
            error!(target: "wiki_scraper", "{error_msg}", )
        };
    }

    info!(target: "wiki_scraper", "Retrieved {} foods.", foods.len());
    Ok(foods)
}
