use crate::wiki_scraper::regex_patterns::RGX_ICON_NAME;
use log::info;
use reqwest::blocking;
use serde::Deserialize;
use std::{error::Error, fs::File, io::BufReader, path::Path};

#[derive(Deserialize, Debug)]
pub struct SAPWikiSources {
    pub pets: String,
    pub food: String,
}

pub fn read_wiki_url<P: AsRef<Path>>(path: P) -> Result<SAPWikiSources, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let urls = serde_json::from_reader(reader)?;
    Ok(urls)
}

pub fn get_page_info(url: &str) -> Result<String, Box<dyn Error>> {
    info!(target: "wiki_scraper", "Retrieving page info for {url}.");
    Ok(blocking::get(url)?.text()?)
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
