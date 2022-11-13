use log::error;
use serde_json::to_writer_pretty;
use std::fs::File;
use crate::wiki_scraper::parser::{parse_pet_info, read_wiki_url};

#[allow(dead_code)]
fn write_pet_info(output: &str) {
    if let Ok(wiki_urls) = read_wiki_url(crate::SCRAPER_SOURCES) {
        let res = parse_pet_info(&wiki_urls.pets);
        if let Ok(all_pets) = res {
            let file = File::create(output).expect("Can't create file.");
            to_writer_pretty(file, &all_pets).expect("Unable to serialize pet info.");
        } else {
            error!(target: "scraper", "{:?}", res.unwrap_err())
        }
    }
}