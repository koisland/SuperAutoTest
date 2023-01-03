#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rocket;

mod api;
mod common;
mod db;
mod wiki_scraper;

use log::info;
use rusqlite::Connection;
use std::error::Error;

use crate::api::server;
use crate::db::{
    query::{update_food_info, update_pet_info},
    setup::{create_tables, get_connection},
};

pub const LOG_CONFIG: &str = "./config/log_config.yaml";
pub const SCRAPER_SOURCES: &str = "./config/sources.json";
pub const DB_CREATE_SQL: &str = "./src/db/sql/create_tables.sql";
pub const DB_INSERT_PET_SQL: &str = "./src/db/sql/insert_pet.sql";
pub const DB_INSERT_FOOD_SQL: &str = "./src/db/sql/insert_food.sql";
pub const DB_FNAME: &str = "./sap.db";

pub fn update_db() -> Result<(), Box<dyn Error>> {
    // Get database connection.
    let conn: Connection = get_connection()?;

    create_tables(&conn)?;
    update_pet_info(&conn)?;
    update_food_info(&conn)?;

    Ok(())
}

pub fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file(LOG_CONFIG, Default::default())?;

    // Update pets and foods on startup.
    update_db()?;
    info!(target: "db", "Successfully updated all tables.");

    // Launch rocket and pass database connection.
    server::main()?;

    Ok(())
}
