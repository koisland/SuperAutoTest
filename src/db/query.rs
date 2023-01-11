use crate::{
    db::{
        record::{FoodRecord, PetRecord},
        utils::{map_row_to_food, map_row_to_pet},
    },
    wiki_scraper::{common::read_wiki_url, parse_food::parse_food_info, parse_pet::parse_pet_info},
};
use log::info;
use rusqlite::Connection;
use std::error::Error;
use std::fs::read_to_string;

pub fn query_pet(
    conn: &Connection,
    sql: &str,
    params: &[String],
) -> Result<Vec<PetRecord>, rusqlite::Error> {
    let mut pet_stmt = conn.prepare(sql).unwrap();
    let mut pets: Vec<PetRecord> = vec![];

    if let Ok(mut pets_found) = pet_stmt.query(rusqlite::params_from_iter(params)) {
        while let Some(pet_row) = pets_found.next().unwrap_or(None) {
            let pet = map_row_to_pet(pet_row)?;
            pets.push(pet);
        }
    }
    Ok(pets)
}

pub fn query_food(
    conn: &Connection,
    sql: &str,
    params: &[String],
) -> Result<Vec<FoodRecord>, rusqlite::Error> {
    let mut food_stmt = conn.prepare(sql).unwrap();
    let mut foods: Vec<FoodRecord> = vec![];

    if let Ok(mut foods_found) = food_stmt.query(rusqlite::params_from_iter(params)) {
        while let Some(food_row) = foods_found.next().unwrap_or(None) {
            let food = map_row_to_food(food_row)?;
            foods.push(food);
        }
    }
    Ok(foods)
}

pub fn update_pet_info(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let wiki_urls = read_wiki_url(crate::SCRAPER_SOURCES)?;
    // Read in insert or replace SQL.
    let sql_insert_pet = read_to_string(crate::DB_INSERT_PET_SQL)?;
    let mut n_rows_updated: usize = 0;

    let pets = parse_pet_info(&wiki_urls.pets)?;
    // Add each pet.
    for pet in pets.iter() {
        // Creating a new row for each pack and level a pet belongs to.
        // Each pet constrained by name and pack so will replace if already exists.
        let n_rows = conn.execute(
            &sql_insert_pet,
            [
                &pet.name,
                &pet.tier.to_string(),
                &pet.attack.to_string(),
                &pet.health.to_string(),
                &pet.pack.to_string(),
                &pet.effect_trigger
                    .clone()
                    .unwrap_or_else(|| "None".to_string()),
                &pet.effect.clone().unwrap_or_else(|| "None".to_string()),
                &pet.lvl.to_string(),
            ],
        )?;
        n_rows_updated += n_rows;
    }
    info!(target: "db", "{} rows updated in \"pet\" table.", n_rows_updated);
    Ok(())
}

pub fn update_food_info(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let wiki_urls = read_wiki_url(crate::SCRAPER_SOURCES)?;
    // Read in insert or replace SQL.
    let sql_insert_food = read_to_string(crate::DB_INSERT_FOOD_SQL)?;
    let mut n_rows_updated: usize = 0;

    let foods = parse_food_info(&wiki_urls.food)?;
    for food in foods.iter() {
        let n_rows = conn.execute(
            &sql_insert_food,
            [
                &food.name,
                &food.tier.to_string(),
                &food.effect,
                &food.pack.to_string(),
            ],
        )?;
        n_rows_updated += n_rows;
    }
    info!(target: "db", "{} rows updated in \"food\" table.", n_rows_updated);
    Ok(())
}
