use crate::api::routes;

use rocket::{Build, Rocket};
use rocket_sync_db_pools::{database, rusqlite};

#[database("sap_sqlite_db")]
pub struct SapDB(rusqlite::Connection);

#[launch]
pub fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![routes::home, routes::pets, routes::foods])
        // Attach DB connection pool.
        .attach(SapDB::fairing())
}
