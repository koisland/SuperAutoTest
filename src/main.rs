#[macro_use]
extern crate lazy_static;

mod common;
mod wiki_scraper;

use common::pet::Pet;
use log::{error, info};
use serde_json::to_writer_pretty;
use std::{fs::File, path::Path};

use crate::wiki_scraper::parser::{parse_pet_info, read_wiki_url};

pub const PETS_JSON: &str = "pets.json";
pub const SAP_WIKI_SOURCES_JSON: &str = "config/sources.json";

fn get_pet_info(output: &str) {
    if let Ok(wiki_urls) = read_wiki_url(SAP_WIKI_SOURCES_JSON) {
        let res = parse_pet_info(&wiki_urls.pets);
        if let Ok(all_pets) = res {
            let file = File::create(output).expect("Can't create file.");
            to_writer_pretty(file, &all_pets).expect("Unable to serialize pet info.");
        } else {
            error!("{:?}", res.unwrap_err())
        }
    }
}

pub fn main() {
    log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();

    if Path::new(PETS_JSON).exists() {
        let file = File::open(PETS_JSON).expect("Unable to read file.");
        let pets: Vec<Pet> = serde_json::from_reader(file).unwrap();
        println!("{:#?}", pets)
    } else {
        get_pet_info(PETS_JSON)
    }
}
