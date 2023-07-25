use itertools::Itertools;
use log::error;
use std::{error::Error, str::FromStr};

use crate::{
    db::{pack::Pack, record::FoodRecord},
    error::SAPTestError,
    regex_patterns::*,
    wiki_scraper::{
        common::{clean_link_text, get_page_info, TABLE_ENTRY_DELIM},
        parse_food::{get_effect_attack, get_effect_health, get_random_n_effect, is_turn_effect},
        IMG_URLS,
    },
    FoodName,
};

pub fn parse_one_ailment_entry(
    ailment_info: &str,
    ailments: &mut Vec<FoodRecord>,
) -> Result<(), Box<dyn Error>> {
    // Split ailment info by '|'. Skipping first element which will be newline.
    let Some((name, effect, _)) = ailment_info.split('|').skip(1).map(|txt| txt.trim()).collect_tuple() else {
        return Err(format!("No name or ability for text: {ailment_info}").into());
    };
    let img_url = IMG_URLS
        .get(name)
        .map(|data| data.url.clone())
        .unwrap_or_else(String::default);

    let (random, n_targets) = get_random_n_effect(effect)?;
    let turn_effect = is_turn_effect(effect);
    let effect_atk = get_effect_attack(effect)?;
    let effect_health = get_effect_health(effect)?;

    ailments.push(FoodRecord {
        name: FoodName::from_str(name)?,
        // All ailments cannot be obtained from shops.
        tier: 0,
        effect: effect.to_owned(),
        pack: Pack::Unknown,
        // All holdable and uses don't decay.
        holdable: true,
        single_use: false,
        // No ailment exists out of battle.
        end_of_battle: true,
        random,
        n_targets,
        effect_atk,
        effect_health,
        turn_effect,
        // No cost for ailment.
        cost: 0,
        img_url,
        is_ailment: true,
    });
    Ok(())
}

pub fn parse_ailment_info(url: &str) -> Result<Vec<FoodRecord>, SAPTestError> {
    let response = get_page_info(url)?;

    let mut ailments: Vec<FoodRecord> = vec![];

    let Some(ailment_tbl) = RGX_TABLE.captures(&response).and_then(|cap| cap.get(0).map(|mtch| mtch.as_str())) else {
        return Err(SAPTestError::ParserFailure { subject: String::from("Missing Ailment Table"), reason: format!("No ailment table found at {url}.") });
    };

    // Remove icon labels.
    let cleaned_ailment_tbl = clean_link_text(ailment_tbl);
    // Skip first item (header) and filter any empty rows.
    for entry in cleaned_ailment_tbl
        .split(TABLE_ENTRY_DELIM)
        .skip(1)
        .filter(|entry| !entry.is_empty() || *entry == "|}")
    {
        if let Err(error_msg) = parse_one_ailment_entry(entry, &mut ailments) {
            error!(target: "wiki_scraper", "{error_msg}", )
        };
    }

    Ok(ailments)
}
