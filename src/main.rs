#[macro_use]
extern crate lazy_static;

mod common;
mod db;
mod wiki_scraper;

use log::{error, info};
use std::error::Error;

use crate::db::query::{update_food_info, update_pet_info};
use crate::db::setup::{create_tables, get_connection};

pub const SCRAPER_SOURCES: &str = "config/sources.json";
pub const DB_CREATE_SQL: &str = "./src/db/sql/create_tables.sql";
pub const DB_INSERT_PET_SQL: &str = "./src/db/sql/insert_pet.sql";
pub const DB_INSERT_FOOD_SQL: &str = "./src/db/sql/insert_food.sql";
pub const DB_FNAME: &str = "./sap.db";

pub fn update_db() -> Result<(), Box<dyn Error>> {
    let conn = get_connection()?;
    create_tables(&conn)?;
    update_pet_info(&conn)?;
    update_food_info(&conn)?;
    Ok(())
}

pub fn main() {
    log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();

    let res = update_db();
    if res.is_ok() {
        info!(target: "db", "Successfully updated all tables.")
    } else {
        error!(target: "db", "{}", res.unwrap_err())
    }
}
