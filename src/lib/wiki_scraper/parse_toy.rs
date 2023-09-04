use std::str::FromStr;

use log::error;

use crate::{db::record::ToyRecord, error::SAPTestError, regex_patterns::*, toys::names::ToyName};

use super::{
    common::{clean_link_text, get_page_info},
    parse_pet::{extract_pet_effect_info, parse_pet_effect_trigger, parse_pet_effects},
    IMG_URLS,
};

pub fn parse_single_toy_row(
    rec: &str,
    curr_tier: usize,
    toys: &mut Vec<ToyRecord>,
) -> Result<(), SAPTestError> {
    let cleaned_rec = clean_link_text(rec);
    let Some(name) = RGX_TOY_NAME
        .captures(&cleaned_rec)
        .and_then(|cap| cap.get(1).map(|mtch| mtch.as_str()))
    else {
        return Err(SAPTestError::ParserFailure {
            subject: "Missing Toy Name".to_string(),
            reason: format!("Unable to get pet name in: {cleaned_rec}"),
        });
    };

    // Source can be blank.
    let source = RGX_TOY_SOURCE
        .captures(&cleaned_rec)
        .and_then(|cap| cap.get(1).map(|mtch| mtch.as_str()));

    // Reuse pet effects parsing fns as format is similar.
    let toy_effect_trigger = parse_pet_effect_trigger(&cleaned_rec);
    let toy_effects = parse_pet_effects(&cleaned_rec, toy_effect_trigger.is_some());

    let url = IMG_URLS
        .get(name)
        .map(|data| data.url.clone())
        .unwrap_or_else(String::default);

    // If effect at a level is missing, still create three records.
    for lvl in 0..3 {
        let toy_lvl_effect = toy_effects.get(lvl).cloned();
        let (effect_stats, n_triggers, temp_effect) =
            extract_pet_effect_info(toy_lvl_effect.as_deref());

        toys.push(ToyRecord {
            name: ToyName::from_str(name)?,
            tier: curr_tier,
            effect_trigger: toy_effect_trigger.clone(),
            effect: toy_lvl_effect,
            effect_atk: effect_stats.attack.try_into()?,
            effect_health: effect_stats.health.try_into()?,
            lvl: lvl + 1,
            source: source.map(|source| source.to_owned()),
            img_url: url.to_owned(),
            n_triggers,
            temp_effect,
            hard_mode: false,
        })
    }
    Ok(())
}

pub fn parse_toys_table(table: &str, tier: usize, toys: &mut Vec<ToyRecord>) {
    // Get rows in table.
    for rec_cap in RGX_TOY_ROW.captures_iter(table) {
        // If no toy records, skip.
        let Some(rec) = rec_cap.get(0).map(|mtch| mtch.as_str()) else {
            continue;
        };

        if let Err(err) = parse_single_toy_row(rec, tier, toys) {
            error!(target: "wiki_scraper", "{err}")
        }
    }
}

pub fn parse_toy_info(url: &str) -> Result<Vec<ToyRecord>, SAPTestError> {
    let response = get_page_info(url)?;
    let mut toys: Vec<ToyRecord> = vec![];

    for cap in RGX_TIER_TABLE.captures_iter(&response) {
        // Find tier and contents of table.
        if let (Some(tier), Some(table)) = (
            cap.get(1)
                .and_then(|mtch| mtch.as_str().parse::<usize>().ok()),
            cap.get(2).map(|mtch| mtch.as_str()),
        ) {
            parse_toys_table(table, tier, &mut toys)
        }
    }

    Ok(toys)
}
