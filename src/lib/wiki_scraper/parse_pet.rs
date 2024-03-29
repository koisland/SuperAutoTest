use itertools::Itertools;
use log::{error, info};
use std::{convert::TryInto, error::Error, str::FromStr};

use crate::{
    db::{pack::Pack, record::PetRecord},
    error::SAPTestError,
    regex_patterns::*,
    wiki_scraper::{
        common::{clean_link_text, get_page_info},
        IMG_URLS,
    },
    PetName, Statistics,
};

const DEFAULT_PET_COST: usize = 3;

/// Numeric regex helper function.
fn num_regex(pattern: &LRegex, string: &str) -> Option<usize> {
    if let Some(cap) = pattern.captures(string) {
        cap.get(1)
            .map(|mtch| mtch.as_str().parse::<usize>().unwrap())
    } else {
        None
    }
}

/// Parse a block of Fandom wiki source text for a pet's stats.
/// * Original text: `attack = 2 | health = 1`
/// * Regex: `attack\s=\s(?P<attack>\d+)\s\|\shealth\s=\s(?P<health>\d+)`
pub fn parse_pet_stats(line: &str) -> Result<(usize, usize), SAPTestError> {
    if let Some(pet_stats) = RGX_PET_STATS.captures(line) {
        // Pattern requires both attack and health. Safe to unwrap.
        let atk_str = pet_stats.name("attack").unwrap().as_str();
        let health_str = pet_stats.name("health").unwrap().as_str();
        if let (Ok(atk), Ok(health)) = (atk_str.parse::<usize>(), health_str.parse::<usize>()) {
            Ok((atk, health))
        } else {
            Err(SAPTestError::ParserFailure {
                subject: "Pet Stats".to_string(),
                reason: format!("Unable to parse attack ({atk_str}) and/or health ({health_str})",),
            })
        }
    } else {
        Err(SAPTestError::ParserFailure {
            subject: "Pet Stats".to_string(),
            reason: format!("Unable to find pet stats on line: {line}"),
        })
    }
}

/// Parse a block of Fandom wiki source text for a pet's packs.
/// * Original text: `starpack`
/// * Regex: `(\w+pack)+`
///
/// Each pack is matched to a `Pack` enum:
/// * `Star`
/// * `Puppy`
/// * `Turtle`
/// * `Weekly`
/// * `Unknown`
///
pub fn parse_pet_packs(line: &str) -> Vec<Pack> {
    RGX_PET_PACK
        .captures_iter(line)
        .filter_map(|cap| {
            Pack::from_str(cap.get(1).unwrap().as_str().trim_end_matches("pack")).ok()
        })
        .collect_vec()
}

/// Parse a block of Fandom wiki source text for a pet's effect trigger.
/// * Original text: `| '''Trigger'''`
/// * Regex: `\|\s*'''(.*?)'''`
pub fn parse_pet_effect_trigger(line: &str) -> Option<String> {
    if let Some(Some(effect_trigger)) = RGX_PET_EFFECT_TRIGGER.captures(line).map(|cap| cap.get(1))
    {
        Some(effect_trigger.as_str().to_string())
    } else {
        None
    }
}

/// Parse a block of Fandom wiki source text for a pet's effects.
/// * Original text:
///     * `| '''Trigger''' → Effect\n`
/// * Regex:
///     * Switches based on if pet effect found.
///         * For animals like Sloth or Tiger.
///     * Found: `→\s(.*?)\n`
///     * Otherwise: `\|\s([^[=]]*?\.*)\n`
pub fn parse_pet_effects(line: &str, pet_effect_found: bool) -> Vec<String> {
    // Use triggerless capture pattern to get pet effects that lack '→' (effect trigger).
    let pet_effect_captures = if pet_effect_found {
        RGX_PET_EFFECT.captures_iter(line)
    } else {
        RGX_PET_EFFECT_TRIGGERLESS.captures_iter(line)
    };

    pet_effect_captures
        .filter_map(|cap| {
            cap.get(1)
                .map(|effect| effect.as_str().replace(['\'', '"'], ""))
                .filter(|effect| !effect.is_empty())
        })
        .collect_vec()
}

/// Extracts effect information.
///
/// **Note: This only gets raw stats**
pub fn extract_pet_effect_info(effect: Option<&str>) -> (Statistics, usize, bool) {
    let effect = effect.unwrap_or("None");

    // Check if end of battle.
    let end_of_battle_effect = RGX_END_OF_BATTLE.is_match(effect);

    // Remove '%' and " of " so pattern for num_regex can work for percentages.
    let pet_effect = effect.replace(" of ", " ").replace('%', "");

    // If a pet has a summon effect, use attack and health stats from effect_stats.
    let parsed_num_effect_stats = if pet_effect.to_lowercase().contains("summon") {
        let raw_summon_stats = (
            num_regex(RGX_SUMMON_ATK, &pet_effect),
            num_regex(RGX_SUMMON_HEALTH, &pet_effect),
        );
        // If any is seen return atk/health values.
        // If only one value given.
        if RGX_ATK_HEALTH.is_match(&pet_effect) {
            (raw_summon_stats.0, raw_summon_stats.0)
        } else if raw_summon_stats.0.is_some() || raw_summon_stats.1.is_some() {
            raw_summon_stats
        } else {
            // Check for percents or single values.
            (
                num_regex(RGX_ATK, &pet_effect),
                num_regex(RGX_HEALTH, &pet_effect),
            )
        }
    } else {
        let raw_stats = (
            num_regex(RGX_ATK, &pet_effect),
            num_regex(RGX_HEALTH, &pet_effect),
        );

        if RGX_ATK_HEALTH.is_match(&pet_effect) {
            (raw_stats.0, raw_stats.0)
        } else if raw_stats.0.is_some() || raw_stats.1.is_some() {
            if RGX_ATK_HEALTH.is_match(effect) {
                (raw_stats.0, raw_stats.0)
            } else {
                raw_stats
            }
        } else {
            // Check for damage dealing effects.
            (num_regex(RGX_DMG, &pet_effect), None)
        }
    };

    let effect_stats = Statistics {
        attack: parsed_num_effect_stats.0.unwrap_or(0).try_into().unwrap(),
        health: parsed_num_effect_stats.1.unwrap_or(0).try_into().unwrap(),
    };

    let n_triggers = num_regex(RGX_N_TRIGGERS, &pet_effect).unwrap_or(1);

    (effect_stats, n_triggers, end_of_battle_effect)
}

/// Parse a block of Fandom wiki source text to generate a `Pet` and update pets found.
pub fn parse_single_pet(
    block: &str,
    curr_tier: usize,
    pets: &mut Vec<PetRecord>,
) -> Result<(), Box<dyn Error>> {
    // If a pet name is found.
    let Some(pet_name) = RGX_PET_NAME
        .captures(block)
        .and_then(|cap| cap.get(1).map(|mtch| mtch.as_str()))
    else {
        return Err(Box::new(SAPTestError::ParserFailure {
            subject: "Missing Pet Name".to_string(),
            reason: format!("Unable to get pet name in: {block}"),
        }));
    };

    let url = IMG_URLS
        .get(pet_name)
        .map(|data| data.url.clone())
        .unwrap_or_default();

    let (pet_atk, pet_health) = parse_pet_stats(block)?;

    let pet_packs = parse_pet_packs(block);

    // Remove icon names in line so regex doesn't give false positive.
    let pet_effect_trigger = parse_pet_effect_trigger(block);
    let pet_effects = parse_pet_effects(block, pet_effect_trigger.is_some());

    // Create a new pet record for every level.
    for pack in pet_packs.iter() {
        for lvl in 0..3 {
            let pet_lvl_effect = pet_effects.get(lvl).cloned();

            let (effect_stats, n_triggers, temp_effect) =
                extract_pet_effect_info(pet_lvl_effect.as_deref());
            let pet = PetRecord {
                name: PetName::from_str(pet_name)?,
                tier: curr_tier,
                attack: pet_atk,
                health: pet_health,
                pack: pack.clone(),
                effect_trigger: pet_effect_trigger.clone(),
                effect: pet_lvl_effect,
                effect_atk: effect_stats.attack.try_into()?,
                effect_health: effect_stats.health.try_into()?,
                n_triggers,
                temp_effect,
                lvl: lvl + 1,
                cost: DEFAULT_PET_COST,
                img_url: url.clone(),
                is_token: false,
            };

            pets.push(pet)
        }
    }
    Ok(())
}

pub fn parse_pets_table(table: &str, tier: usize, pets: &mut Vec<PetRecord>) {
    // Get rows in table.
    for rec_cap in RGX_PET_ROW.captures_iter(table) {
        // If no toy records, skip.
        let Some(rec) = rec_cap.get(0).map(|mtch| clean_link_text(mtch.as_str())) else {
            continue;
        };

        if let Err(err) = parse_single_pet(&rec, tier, pets) {
            error!(target: "wiki_scraper", "{err}")
        }
    }
}

/// Parse pet info into a list of `Pet`s.
pub fn parse_pet_info(url: &str) -> Result<Vec<PetRecord>, SAPTestError> {
    let response = get_page_info(url)?;
    let mut pets: Vec<PetRecord> = vec![];

    for cap in RGX_TIER_TABLE.captures_iter(&response) {
        // Find tier and contents of table.
        if let (Some(tier), Some(table)) = (
            cap.get(1)
                .and_then(|mtch| mtch.as_str().parse::<usize>().ok()),
            cap.get(2).map(|mtch| mtch.as_str()),
        ) {
            parse_pets_table(table, tier, &mut pets)
        }
    }

    info!(target: "wiki_scraper", "Retrieved {} pets.", pets.len());
    Ok(pets)
}
