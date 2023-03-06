use std::str::FromStr;

use log::info;

use crate::{
    error::SAPTestError,
    regex_patterns::{RGX_COL_DESC_CATEG, RGX_COL_WORD, RGX_MULT_TABLE},
};

use super::common::get_page_info;

#[derive(Debug, Clone, Copy)]
pub(crate) enum WordType {
    Prefix,
    Noun,
}

impl std::fmt::Display for WordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TeamWord {
    pub word_type: WordType,
    pub word: String,
}

impl FromStr for WordType {
    type Err = SAPTestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Prefix" | "Prefixes" => Ok(Self::Prefix),
            "Nouns" | "Noun" => Ok(Self::Noun),
            _ => Err(SAPTestError::ParserFailure {
                subject: "Team Names Word".to_string(),
                reason: format!("Invalid category ({s}) for word types."),
            }),
        }
    }
}

pub(crate) fn parse_word_table(tbl: &str, words: &mut Vec<TeamWord>) {
    // Get category word type.
    if let Some(Ok(categ)) = RGX_COL_DESC_CATEG
        .find(tbl)
        .and_then(|mtch| mtch.as_str().strip_prefix("|+"))
        .map(WordType::from_str)
    {
        // Find matches and push.
        for word in RGX_COL_WORD
            .captures_iter(tbl)
            .filter_map(|cap| cap.get(0).map(|mtch| mtch.as_str().trim_matches('|')))
            .filter(|word| !word.is_empty())
        {
            words.push(TeamWord {
                word_type: categ,
                word: word.trim().to_string(),
            })
        }
    } else {
        info!(target: "wiki_scraper", "Table has no valid word category for team names: {tbl}");
    }
}
/// Parse team name info into a list of `Pet`s.
pub(crate) fn parse_names_info(url: &str) -> Result<Vec<TeamWord>, SAPTestError> {
    let response = get_page_info(url)?;
    let mut words = vec![];

    for tbl in RGX_MULT_TABLE
        .captures_iter(&response)
        .filter_map(|cap| cap.get(0).map(|mtch| mtch.as_str()))
    {
        parse_word_table(tbl, &mut words)
    }

    Ok(words)
}
