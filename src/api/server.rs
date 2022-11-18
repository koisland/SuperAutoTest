use crate::api::routes;

use rocket_sync_db_pools::{database, rusqlite};
use std::error::Error;

#[database("sap_sqlite_db")]
pub struct SapDB(rusqlite::Connection);

#[rocket::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let _rocket = rocket::build()
        .mount("/", routes![routes::index, routes::pets, routes::foods])
        // Attach DB connection pool.
        .attach(SapDB::fairing())
        .launch()
        .await?;

    Ok(())
}
