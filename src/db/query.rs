use log::info;
use std::error::Error;
use std::fs::read_to_string;
use rusqlite::Connection;
use crate::wiki_scraper::parser::{parse_pet_info, read_wiki_url};

pub fn update_pet_info(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let wiki_urls = read_wiki_url(crate::SCRAPER_SOURCES)?;
    // Read in insert or replace SQL.
    let sql_insert_pet = read_to_string(crate::DB_INSERT_PET_SQL)?;
    let mut n_rows_updated: usize = 0;

    let pets = parse_pet_info(&wiki_urls.pets)?;
    // Add each pet.
    for pet in pets.iter() {
        // Creating a new row for each pack a pet belongs to.
        // Each pet constrained by name and pack so will replace if already exists.
        for pack in pet.packs.iter() {
            let n_rows = conn.execute(
                &sql_insert_pet,
                (
                    &pet.name,
                    &pet.tier,
                    &pet.attack,
                    &pet.health,
                    pack.to_string(),
                    &pet.effect_trigger,
                    &pet.effects.get(0).unwrap_or(&"None".to_string()),
                    &pet.effects.get(1).unwrap_or(&"None".to_string()),
                    &pet.effects.get(2).unwrap_or(&"None".to_string()),
                ),
            )?;
            n_rows_updated += n_rows;
        }
    }
    info!(target: "db", "{} rows updated in \"pet\" table.", n_rows_updated);
    Ok(())
}