use std::{borrow::Cow, collections::HashMap, error::Error, str::FromStr};

use itertools::Itertools;
use log::{info, warn};

use crate::{
    db::{pack::Pack, record::PetRecord},
    wiki_scraper::{
        common::get_page_info,
        error::WikiParserError,
        parse_food::{clean_link_text, get_largest_table},
        regex_patterns::{RGX_SUMMON_STATS, RGX_TOKEN_SPAN_COLS},
    },
};
const DEFAULT_TOKEN_COST: usize = 0;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TokenTableCols {
    Name,
    StatsLvl1,
    StatsLvl2,
    StatsLvl3,
    SummonedFrom,
    Notes,
}

impl TokenTableCols {
    pub fn get_cols(table: &[&str]) -> Result<Vec<TokenTableCols>, Box<dyn Error>> {
        let mut cols: Vec<TokenTableCols> = vec![];

        if let (Some(all_cols_str), Some(sub_cols)) = (table.first(), table.get(1)) {
            for col_desc_cap in RGX_TOKEN_SPAN_COLS.captures_iter(all_cols_str) {
                // Safe to unwrap because regex match.
                let row_col = col_desc_cap.get(1).unwrap().as_str();
                let colname_str = col_desc_cap.get(3).unwrap().as_str();

                // Colspan so attack and health.
                if row_col == "col" {
                    for subcol in sub_cols.trim().split('\n') {
                        let mut lvl = subcol.replace("!Level ", "");
                        lvl.insert_str(0, "StatsLvl");

                        cols.push(TokenTableCols::from_str(&lvl)?)
                    }
                } else {
                    // Rowspan so no subcols.
                    cols.push(TokenTableCols::from_str(colname_str)?);
                };
            }
        }

        Ok(cols)
    }
}

impl FromStr for TokenTableCols {
    type Err = WikiParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Name" => Ok(TokenTableCols::Name),
            "StatsLvl1" => Ok(TokenTableCols::StatsLvl1),
            "StatsLvl2" => Ok(TokenTableCols::StatsLvl2),
            "StatsLvl3" => Ok(TokenTableCols::StatsLvl3),
            "Summoned From" => Ok(TokenTableCols::SummonedFrom),
            "Additional Notes" => Ok(TokenTableCols::Notes),
            _ => Err(WikiParserError {
                reason: format!("Unknown column {s}."),
            }),
        }
    }
}

pub fn clean_token_block(block: &str) -> Result<String, Box<dyn Error>> {
    let block = if let Some(cap) = RGX_SUMMON_STATS.captures(block) {
        let num_sub_cols = cap.get(1).map(|mtch| mtch.as_str());
        let summon_stats = cap.get(2).map(|mtch| mtch.as_str());

        if let (Some(cols), Some(stats)) = (num_sub_cols, summon_stats) {
            let mut stats_per_lvl = (0..cols.parse::<usize>()?)
                .into_iter()
                .map(|_| format!("|{stats}"))
                .join("\n");
            stats_per_lvl.push('\n');
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

pub fn parse_single_token(
    block: &str,
    cols: &[TokenTableCols],
    pets: &mut Vec<PetRecord>,
) -> Result<(), Box<dyn Error>> {
    let cleaned_block = clean_token_block(block)?;

    // Trim start to remove | delim so correct number of values.
    let col_vals = cleaned_block.split('|').collect_vec();

    if cols.len() != col_vals.len() {
        return Err(Box::new(WikiParserError {
            reason: format!(
                "Token columns not equal to column values. {} != {}",
                cols.len(),
                col_vals.len()
            ),
        }));
    }
    let col_map_vals: HashMap<&TokenTableCols, &str> = cols
        .iter()
        .zip_eq(col_vals.into_iter().map(|val| val.trim()))
        .collect();

    if let (
        Some(name),
        Some(stats_level_1),
        Some(stats_level_2),
        Some(stats_level_3),
        Some(summon),
        Some(notes),
    ) = (
        col_map_vals.get(&TokenTableCols::Name),
        col_map_vals.get(&TokenTableCols::StatsLvl1),
        col_map_vals.get(&TokenTableCols::StatsLvl2),
        col_map_vals.get(&TokenTableCols::StatsLvl3),
        col_map_vals.get(&TokenTableCols::SummonedFrom),
        col_map_vals.get(&TokenTableCols::Notes),
    ) {
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
                    name: name.to_string(),
                    tier: 0,
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
                })
            } else {
                warn!("Failed to parse stats for {name} from string {stats}.")
            }
        }
    } else {
        warn!(target: "wiki_scraper", "Name and stats for token don't exist.")
    }

    Ok(())
}
/// Parse token info into a list of `Pet`s.
pub fn parse_token_info(url: &str) -> Result<Vec<PetRecord>, Box<dyn Error>> {
    let response = get_page_info(url)?;
    let mut pets: Vec<PetRecord> = vec![];

    let table = get_largest_table(&response)?;
    let cols = TokenTableCols::get_cols(&table)?;

    for block in table
        .get(2..)
        .ok_or(WikiParserError {
            reason: "No largest table in token url content.".to_string(),
        })?
        .iter()
    {
        parse_single_token(block, &cols, &mut pets)?;
    }

    info!(target: "wiki_scraper", "Retrieved {} tokens.", pets.len());
    Ok(pets)
}
