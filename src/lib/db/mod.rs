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
//! * This database is initialized as a global, static type using the [`lazy_static`] crate.
//! * To create a new test database, use the [`SapDB`](crate::SapDB) struct:
//!     ```
//!     let db = saptest::SapDB::new("./test_sap.db");
//!     ```
//!
//! ### Configuration
//! To modify [`SapDB`](struct@crate::SapDB) behavior, create a `.saptest.toml`.
//! * Specific page version to query.
//!     * All pages on the Fandom wiki are version controlled and have an associated id.
//!         * ex. <https://superautopets.fandom.com/wiki/Pets?oldid=4883>
//!     * In the case that a page is altered with incorrect information, this can be used to find a valid version.
//!     * Leaving this blank will default to the latest version.
//!         * ex. <https://superautopets.fandom.com/wiki/Pets>
//! * Toggle recurring updates on startup.
//!     * By default, the database is updated on startup.
//! * Database filename.
//!
//! ```toml
//! [database]
//! # https://superautopets.fandom.com/wiki/Pets
//! # pets_version = ?
//!
//! # https://superautopets.fandom.com/wiki/Food
//! # foods_version = ?
//!
//! # https://superautopets.fandom.com/wiki/Tokens
//! # tokens_version = ?
//!
//! filename = "./sap.db"
//! update_on_startup = false
//! ```

/// Game packs.
pub mod pack;
/// Record types.
pub mod record;
/// Database setup functions.
pub mod setup;
/// Database helper functions.
pub mod utils;
