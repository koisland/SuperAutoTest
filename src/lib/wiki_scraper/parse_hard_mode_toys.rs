use std::str::FromStr;

use itertools::Itertools;
use log::error;

use crate::{
    db::record::ToyRecord, error::SAPTestError, regex_patterns::RGX_TIER_TABLE,
    toys::names::ToyName,
};

use super::{
    common::{clean_link_text, get_page_info, TABLE_ENTRY_DELIM},
    parse_pet::extract_pet_effect_info,
    IMG_URLS,
};

pub fn parse_single_hard_mode_toy_row(
    rec: &str,
    curr_tier: usize,
    toys: &mut Vec<ToyRecord>,
) -> Result<(), SAPTestError> {
    let cleaned_rec = clean_link_text(rec);
    // Expect three lines name, trigger, and effect preceded by '|'.
    let Some((name, trigger, effect)) = cleaned_rec.lines().filter_map(|line| line.strip_prefix('|').map(|line| line.trim())).take(3).collect_tuple() else {
        return Err(SAPTestError::ParserFailure {
            subject: "Missing Toy Fields".to_string(),
            reason: format!("Unable to get name, trigger, or effect in: {cleaned_rec}"),
        });
    };

    let img_url = IMG_URLS
        .get(name)
        .map(|data| data.url.clone())
        .unwrap_or_else(String::default);

    let (effect_stats, n_triggers, temp_effect) = extract_pet_effect_info(Some(effect));
    toys.push(ToyRecord {
        name: ToyName::from_str(name)?,
        tier: curr_tier,
        effect_trigger: Some(trigger.to_owned()),
        effect: Some(effect.to_owned()),
        effect_atk: effect_stats.attack.try_into()?,
        effect_health: effect_stats.health.try_into()?,
        // All hard mode effects have no level. Set to 1.
        lvl: 1,
        source: None,
        img_url,
        n_triggers,
        temp_effect,
        hard_mode: true,
    });
    Ok(())
}

pub fn parse_hard_mode_toys_table(table: &str, tier: usize, toys: &mut Vec<ToyRecord>) {
    // Get rows in table.
    for rec in table.split(TABLE_ENTRY_DELIM) {
        if let Err(err) = parse_single_hard_mode_toy_row(rec, tier, toys) {
            error!(target: "wiki_scraper", "{err}")
        }
    }
}

pub fn parse_hard_mode_toy_info(url: &str) -> Result<Vec<ToyRecord>, SAPTestError> {
    let response = get_page_info(url)?;
    let mut toys: Vec<ToyRecord> = vec![];

    for cap in RGX_TIER_TABLE.captures_iter(&response) {
        // Find tier and contents of table.
        if let (Some(tier), Some(table)) = (
            cap.get(1)
                .and_then(|mtch| mtch.as_str().parse::<usize>().ok()),
            cap.get(2).map(|mtch| mtch.as_str()),
        ) {
            parse_hard_mode_toys_table(table, tier, &mut toys)
        }
    }

    Ok(toys)
}
