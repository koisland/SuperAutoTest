use crate::api::{route_battle, route_foods, route_home, route_pets};

use rocket::{Build, Rocket};
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
        // Attach DB connection pool.
        .attach(SapDB::fairing())
}
