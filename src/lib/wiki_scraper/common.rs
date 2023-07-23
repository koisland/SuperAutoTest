use crate::{error::SAPTestError, regex_patterns::*};
use itertools::Itertools;
use log::info;

pub const TABLE_ENTRY_DELIM: &str = "|-";

pub fn get_page_info(url: &str) -> Result<String, SAPTestError> {
    info!(target: "wiki_scraper", "Retrieving page info for {url}.");
    Ok(ureq::get(url).call()?.into_string()?)
}

/// Remove any Fandom icon names from a block of text.
/// * Ex. `{IconSAP|Turtle}` -> `Turtle`
pub fn remove_icon_names(line: &str) -> String {
    let mut final_line = line.to_string();

    for cap in RGX_ICON_NAME.captures_iter(line) {
        // If an arg exists for icon, capture it.
        let icon_arg = cap.get(2).map(|cap| cap.as_str()).unwrap_or("");
        // If no name arg exists for icon, substitute icon name.
        let icon_name = if (icon_arg.is_empty()) | (!icon_arg.contains("name=")) {
            cap.get(1)
                .map(|cap| cap.as_str())
                .unwrap_or("Missing")
                .to_string()
        } else {
            icon_arg.replace("name=", "").to_string()
        };
        // Replace first instance in string.
        final_line = RGX_ICON_NAME
            .replacen(&final_line, 1, icon_name)
            .to_string();
    }
    // Remove remaining }, if any.
    final_line.replace('}', "")
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
        Ok(largest_table
            .as_str()
            .split(TABLE_ENTRY_DELIM)
            .collect_vec())
    } else {
        Err(SAPTestError::ParserFailure {
            subject: "Largest Table".to_string(),
            reason: "Can't find main table following format: {|...|}.".to_string(),
        })
    }
}

/// Clean text removing:
/// * Links `[[...|...]]`
/// * Icon names `{IconSAP|...}`.
pub fn clean_link_text(text: &str) -> String {
    let mut text_copy = text.to_string();

    for capture in RGX_LINK_NAME.captures_iter(text) {
        // Get last element in link text.
        // ex. |Give one [[Pets|pet]] [[Lemon]]. -> Give one pet Lemon.
        for (i, mtch) in capture.iter().enumerate() {
            // Skip first match which matches everything.
            if i == 0 {
                continue;
            }
            let icon_name = mtch
                .map_or("", |m| m.as_str())
                .split('|')
                .next()
                .and_then(|name| name.strip_prefix("File:"))
                .and_then(|name| name.strip_suffix(".png"))
                .unwrap_or("");
            let icon_name = icon_name.to_ascii_lowercase();

            // Update line copy replacing links wiht food name.
            text_copy = RGX_LINK_NAME.replacen(&text_copy, 1, icon_name).to_string();
        }
    }
    remove_icon_names(&text_copy).trim_matches('|').to_string()
}
