
use crate::api::routes;

use std::error::Error;
use rocket_sync_db_pools::{database, rusqlite};


#[database("sap_sqlite_db")]
pub struct SAPDB(rusqlite::Connection);

#[rocket::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let _rocket = rocket::build()
        .mount(
            "/", 
            routes![
                routes::index,
                routes::pets,
                routes::foods
                ]
            )
        // Attach DB connection pool.
        .attach(SAPDB::fairing())
        .launch()
        .await?;

    Ok(())
}
