use itertools::Itertools;
use log::{info, warn};
use regex::Regex;
use std::error::Error;

use crate::{
    common::{pack::Pack, record::PetRecord},
    wiki_scraper::{
        common::{get_page_info, remove_icon_names},
        error::WikiParserError,
    },
};

lazy_static! {
    static ref RGX_TIER: Regex = Regex::new(r#"<!--\sTIER\s(\d)\s-->"#).unwrap();
    static ref RGX_PET_NAME: Regex = Regex::new(r#"pet\s=\s\{\{IconSAP\|(.*?)\|size"#).unwrap();
    static ref RGX_PET_STATS: Regex =
        Regex::new(r#"attack\s=\s(?P<attack>\d+)\s\|\shealth\s=\s(?P<health>\d+)"#).unwrap();
    static ref RGX_PET_PACK: Regex = Regex::new(r#"(\w+pack)+"#).unwrap();
    static ref RGX_PET_EFFECT_TRIGGER: Regex = Regex::new(r#"\| '''(.*?)'''+"#).unwrap();
    // TODO: Misses animals with no triggers. (Tiger)
    static ref RGX_PET_EFFECT: Regex = Regex::new(r#"→\s(.*?)\n"#).unwrap();
    static ref RGX_PET_EFFECT_TRIGGERLESS: Regex = Regex::new(r#"\|\s([^[=]]*?\.*)\n"#).unwrap();
    static ref RGX_ICON_NAME: Regex =
        Regex::new(r#"\{\{IconSAP\|(.*?)[\|\}]+.*?([\w\|]*=[\w\.]+)*"#).unwrap();
}

/// Parse a block of Fandom wiki source text for a pet's stats.
/// * Original text: `attack = 2 | health = 1`
/// * Regex: `attack\s=\s(?P<attack>\d+)\s\|\shealth\s=\s(?P<health>\d+)`
pub fn parse_pet_stats(line: &str) -> Result<(usize, usize), WikiParserError> {
    if let Some(pet_stats) = RGX_PET_STATS.captures(line) {
        // Pattern requires both attack and health. Safe to unwrap.
        let atk_str = pet_stats.name("attack").unwrap().as_str();
        let health_str = pet_stats.name("health").unwrap().as_str();
        if let (Ok(atk), Ok(health)) = (atk_str.parse::<usize>(), health_str.parse::<usize>()) {
            Ok((atk, health))
        } else {
            Err(WikiParserError {
                reason: format!(
                    "Unable to parse attack ({}) and/or health ({})",
                    atk_str, health_str
                ),
            })
        }
    } else {
        Err(WikiParserError {
            reason: format!("Unable to find pet stats on line: {}", line),
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
        .map(|cap| match cap.get(1).unwrap().as_str() {
            "starpack" => Pack::Star,
            "puppypack" => Pack::Puppy,
            "turtlepack" => Pack::Turtle,
            "weeklypack" => Pack::Weekly,
            _ => {
                warn!(target: "wiki_parser", "New pack found. {:?}", cap);
                Pack::Unknown
            }
        })
        .collect_vec()
}

/// Parse a block of Fandom wiki source text for a pet's effect trigger.
/// * Original text: `| '''Trigger'''`
/// * Regex: `\| ''+(.*?)''+`
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
        })
        .collect_vec()
}

/// Parse a block of Fandom wiki source text to generate a `Pet` and update pets found.
pub fn parse_single_pet(
    block: &str,
    curr_tier: &mut usize,
    pets: &mut Vec<PetRecord>,
) -> Result<(), WikiParserError> {
    // Update the pet tier.
    if RGX_TIER.is_match(block) {
        *curr_tier = RGX_TIER
            .captures(block)
            .and_then(|cap| cap.get(1))
            .and_then(|tier| tier.as_str().parse::<usize>().ok())
            .unwrap_or(0);
        info!(target: "wiki_scraper", "On tier {curr_tier} animals...");
    }
    // If a pet name is found.
    if RGX_PET_NAME.is_match(block) {
        let pet_name = if let Some(Some(name)) = RGX_PET_NAME.captures(block).map(|cap| cap.get(1))
        {
            name.as_str()
        } else {
            return Err(WikiParserError {
                reason: format!("Unable to get pet name in: {block}"),
            });
        };

        let (pet_atk, pet_health) = if let Ok(pet_stats) = parse_pet_stats(block) {
            (pet_stats.0, pet_stats.1)
        } else {
            return Err(WikiParserError {
                reason: format!("No pet stats found in {block}."),
            });
        };

        let pet_packs = parse_pet_packs(block);

        // Remove icon names in line so regex doesn't give false positive.
        let cleaned_line = remove_icon_names(block);
        let pet_effect_trigger = parse_pet_effect_trigger(&cleaned_line);

        let pet_effects = parse_pet_effects(&cleaned_line, pet_effect_trigger.is_some());

        // Create a new pet record for every level.
        for pack in pet_packs.iter() {
            for lvl in 0..3 {
                let pet_lvl_effect = pet_effects.get(lvl).cloned();
                let pet = PetRecord {
                    name: pet_name.to_string(),
                    tier: *curr_tier,
                    attack: pet_atk,
                    health: pet_health,
                    pack: pack.clone(),
                    effect_trigger: pet_effect_trigger.clone(),
                    effect: pet_lvl_effect,
                    lvl: lvl + 1,
                };

                pets.push(pet)
            }
        }
    }
    Ok(())
}

/// Parse pet info into a list of `Pet`s.
pub fn parse_pet_info(url: &str) -> Result<Vec<PetRecord>, Box<dyn Error>> {
    let response = get_page_info(url)?;
    let mut pets: Vec<PetRecord> = vec![];
    let mut curr_tier: usize = 1;

    for block in response.split("\n\n") {
        // Update pets and continue if cannot.
        if parse_single_pet(block, &mut curr_tier, &mut pets).is_err() {
            continue;
        }
    }
    info!(target: "wiki_scraper", "Retrieved {} pets.", pets.len());
    Ok(pets)
}

mod tests {}
