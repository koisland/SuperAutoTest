//! SQLite [`SapDB`](crate::SapDB) database of game information.
//!
//! This database relies on information from the Super Auto Pets Fandom wiki.
//! * All information is parsed from the following pages:
//!     * [`pets`](https://superautopets.fandom.com/wiki/Pets)
//!     * [`tokens`](https://superautopets.fandom.com/wiki/Tokens)
//!         * Tokens are placed under the `pets` table and given a tier of `0`.
//!     * [`foods`](https://superautopets.fandom.com/wiki/Foods)
//!
//! ### [`SAPDB`](crate::SAPDB)
//! * This database is initialized once as a global, static type using the [`lazy_static`] crate.
//!     * By default, the path is set to `./sap.db` and cannot be changed.
//! * To create a new test database, use the [`SapDB`](crate::SapDB) struct:
//!     ```
//!     let db = saptest::SapDB::new("./test_sap.db");
//!     ```
//! * Only `SELECT *` queries can be executed from the [`execute_pet_query`](crate::SapDB::execute_pet_query) and [`execute_food_query`](crate::SapDB::execute_food_query) methods.
//!

/// Game packs.
pub mod pack;
/// Record types.
pub mod record;
/// Database setup functions.
pub mod setup;
/// Database helper functions.
mod utils;
