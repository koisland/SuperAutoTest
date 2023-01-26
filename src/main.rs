#[macro_use]
extern crate rocket;

mod cli;
mod server;

use sapt::db::{
    query::{update_food_info, update_pet_info},
    setup::{create_tables, get_connection},
};

use crate::{cli::cli, server::main::main as rocket_main};
use log::info;
use rusqlite::Connection;
use std::error::Error;

const LOG_CONFIG: &str = "./config/log_config.yaml";

pub fn update_db() -> Result<(), Box<dyn Error>> {
    // Get database connection.
    let conn: Connection = get_connection()?;

    create_tables(&conn)?;
    update_pet_info(&conn)?;
    update_food_info(&conn)?;

    info!(target: "db", "Successfully updated all tables.");

    Ok(())
}

pub fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file(LOG_CONFIG, Default::default())?;

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("init", _)) => update_db()?,
        Some(("run", _)) => {
            update_db()?;
            // Launch rocket and pass database connection.
            rocket_main();
        }
        _ => unreachable!("Not a valid option."),
    }

    Ok(())
}
