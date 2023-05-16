use crate::{error::SAPTestError, regex_patterns::*};
use itertools::Itertools;
use log::info;

const TABLE_ENTRY_DELIM: &str = "|-";

pub fn get_page_info(url: &str) -> Result<String, SAPTestError> {
    info!(target: "wiki_scraper", "Retrieving page info for {url}.");
    Ok(ureq::get(url).call()?.into_string()?)
}

/// Remove any Fandom icon names from a block of text.
/// * Ex. `{IconSAP|Turtle}` -> `Turtle`
pub fn remove_icon_names(line: &str) -> String {
    let mut final_line = line.to_string();
    let final_line_copy = final_line.clone();

    for cap in RGX_ICON_NAME.captures_iter(&final_line_copy) {
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
