use std::{borrow::Cow, str::FromStr};

use itertools::Itertools;
use log::{error, info, warn};

use crate::{
    db::{pack::Pack, record::PetRecord},
    error::SAPTestError,
    regex_patterns::*,
    wiki_scraper::common::{clean_link_text, get_largest_table, get_page_info},
    PetName,
};

use super::IMG_URLS;
const DEFAULT_TOKEN_COST: usize = 0;

/// Cleans token blocks and pads stats to number of spanned cols if has variable X.
/// * X/1 -> X/1|X/1|X/1
pub fn clean_token_block(block: &str) -> Result<String, SAPTestError> {
    let block = if let Some(cap) = RGX_SUMMON_STATS.captures(block) {
        let num_sub_cols = cap.get(1).map(|mtch| mtch.as_str());
        let summon_stats = cap.get(2).map(|mtch| mtch.as_str());

        if let (Some(cols), Some(stats)) = (num_sub_cols, summon_stats) {
            let num_cols = cols.parse::<usize>()?;
            let stats_per_lvl = (0..num_cols).map(|_| format!("|{stats}",)).join("\n");
            RGX_SUMMON_STATS.replace(block, stats_per_lvl)
        } else {
            Cow::Borrowed("")
        }
    } else {
        Cow::Borrowed(block)
    };
    Ok(clean_link_text(
        block.trim_start_matches(|c| c == '|' || c == '\n'),
    ))
}

pub fn parse_single_token(block: &str, pets: &mut Vec<PetRecord>) -> Result<(), SAPTestError> {
    let cleaned_block = clean_token_block(block)?;

    if let Some((name, stats_level_1, stats_level_2, stats_level_3, summon, notes)) = cleaned_block
        .split('|')
        .map(|txt| txt.trim())
        .collect_tuple()
    {
        let url = IMG_URLS
            .get(<&str>::clone(&name))
            .map(|data| data.url.clone())
            .unwrap_or_else(String::default);

        for (lvl, stats) in [stats_level_1, stats_level_2, stats_level_3]
            .into_iter()
            .enumerate()
        {
            // Chick has `X` in stats.
            let stats = stats.replace('X', "0");

            if let Some((Some(attack), Some(health))) = stats
                .split('/')
                .map(|num| num.parse::<usize>().ok())
                .collect_tuple::<(Option<usize>, Option<usize>)>()
            {
                pets.push(PetRecord {
                    name: PetName::from_str(name)?,
                    // All tokens treated as tier 1.
                    tier: 1,
                    attack,
                    health,
                    pack: Pack::Unknown,
                    effect_trigger: Some(summon.to_string()),
                    effect: (!notes.is_empty()).then_some(notes.to_string()),
                    effect_atk: 0,
                    effect_health: 0,
                    n_triggers: 0,
                    temp_effect: false,
                    lvl: lvl + 1,
                    cost: DEFAULT_TOKEN_COST,
                    img_url: url.clone(),
                    is_token: true,
                })
            } else {
                warn!("Failed to parse stats for {name} from string {stats}.")
            }
        }
    } else {
        warn!(target: "wiki_scraper", "Token text doesn't have 6 cols ({cleaned_block}).")
    }

    Ok(())
}

/// Parse token info into a list of `Pet`s.
pub fn parse_token_info(url: &str) -> Result<Vec<PetRecord>, SAPTestError> {
    let response = get_page_info(url)?;
    let mut pets: Vec<PetRecord> = vec![];

    let table = get_largest_table(&response)?;

    for block in table
        .iter()
        .filter(|block| block.starts_with("\n|{{IconSAP"))
    {
        if let Err(err) = parse_single_token(block, &mut pets) {
            error!(target: "wiki_scraper", "{err}")
        };
    }

    info!(target: "wiki_scraper", "Retrieved {} tokens.", pets.len());
    Ok(pets)
}
