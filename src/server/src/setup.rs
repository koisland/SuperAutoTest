#[macro_use]
extern crate rocket;

mod route_battle;
mod route_foods;
mod route_home;
mod route_pets;
mod utils;

const DB_FILE: &str = "./sap.db";
const LOG_CONFIG: &str = "./config/log_config.yaml";

use sapt::db::{
    query::{update_food_info, update_pet_info},
    setup::{create_tables},
};

use log::info;
use rusqlite::{Connection, OpenFlags};
use std::{error::Error, future::Future};
use rocket::{Build, Rocket, tokio, fairing::AdHoc};
use rocket_sync_db_pools::{database, rusqlite};

#[database("sap_sqlite_db")]
pub struct SapDB(rusqlite::Connection);

#[launch]
pub fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount(
            "/",
            routes![
                route_home::home,
                route_pets::pets,
                route_foods::foods,
                route_battle::battle
            ],
        )
        .attach(AdHoc::on_ignite("init_log", |rocket| async move {
            log4rs::init_file(LOG_CONFIG, Default::default()).expect("No log config provided.");
            rocket
        }))
        .attach(AdHoc::on_ignite("init_db", |rocket| async move {
            update_db().unwrap();
            rocket
        }))
        // Attach DB connection pool.
        .attach(SapDB::fairing())
}

fn update_db() -> Result<(), Box<dyn Error>> {
    // Create database connection.
    let conn = Connection::open(DB_FILE)?;

    // Create tables.
    create_tables(&conn)?;
    // Tokio panics for blocking request otherwise.
    tokio::task::spawn_blocking(move || {
        update_pet_info(&conn).unwrap();
        update_food_info(&conn).unwrap();
    });

    info!(target: "db", "Successfully updated all tables.");
    Ok(())
}
