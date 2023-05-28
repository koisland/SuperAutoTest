//! SQLite [`SapDB`](crate::SapDB) database of game information.
//!
//! This database relies on information from the Super Auto Pets Fandom wiki.
//! * All information is parsed from the following pages:
//!     * [`pets`](https://superautopets.fandom.com/wiki/Pets)
//!     * [`tokens`](https://superautopets.fandom.com/wiki/Tokens)
//!         * Tokens are placed under the `pets` table and given a tier of `0`.
//!     * [`foods`](https://superautopets.fandom.com/wiki/Foods)
//!
//! ### Schema
//! To view via `sqlite`.
//! ```bash
//! sqlite3 sap.db
//! .schema
//! ```
//!
//! #### Names
//! Team Names. ex. `The Super Auto Pets`
//!
//! ```sql
//! CREATE TABLE IF NOT EXISTS names (
//!     id INTEGER PRIMARY KEY,
//!     word_category TEXT NOT NULL,
//!     word TEXT NOT NULL,
//!     CONSTRAINT unq UNIQUE (word_category, word)
//! );
//! ```
//! * word_category
//!     * Categories for team name words.
//!         1. `prefix` for adjectives
//!         2. `noun` for nouns
//! * word
//!     * The word used to build the name.
//!
//! #### Pets
//! Pet records.
//!
//! ```sql
//! CREATE TABLE IF NOT EXISTS pets (
//!     id INTEGER PRIMARY KEY,
//!     name TEXT NOT NULL,
//!     tier INTEGER NOT NULL,
//!     attack INTEGER NOT NULL,
//!     health INTEGER NOT NULL,
//!     pack TEXT NOT NULL,
//!     effect_trigger TEXT NOT NULL,
//!     effect TEXT NOT NULL,
//!     effect_atk INTEGER NOT NULL,
//!     effect_health INTEGER NOT NULL,
//!     n_triggers INTEGER NOT NULL,
//!     temp_effect BOOLEAN NOT NULL,
//!     lvl INTEGER NOT NULL,
//!     cost INTEGER NOT NULL,
//!     img_url TEXT,
//!     is_token BOOLEAN NOT NULL,
//!     CONSTRAINT unq UNIQUE (name, pack, lvl)
//! );
//! ```
//! * `name`
//!     * Name of a pet.
//! * `tier`
//!     * Tier of a pet.
//! * `attack`
//!     * Attack stat.
//! * `health`
//!     * Health stat.
//! * `pack`
//!     * Pack pet belongs to.
//!     * Records are duplicated if a pet exists in multiple packs.
//! * `effect_trigger`
//!     * Effect trigger.
//! * `effect`
//!     * Effect description.
//! * `effect_atk`
//!     * Attack stat of an effect.
//!     * This can be a summon's attack, the amount of attack given, the amount of damage dealt, etc.
//! * `effect_health`
//!     * Health stat of an effect.
//!     * This can be a summon's health, the amount of health given, the amount of damage resisted, etc.
//! * `n_triggers`
//!     * Number of times an effect activates per trigger.
//! * `temp_effect`
//!     * Whether or not the effect persists after the shop phase.
//!     * Example: [`Horse`](crate::PetName::Horse)
//! * `lvl`
//!     * Level of pet.
//! * `cost`
//!     * Cost of pet.
//! * `img_url`
//!     * Current image url displayed on page.
//! * `is_token`
//!     * Is current pet a [token](https://superautopets.fandom.com/wiki/Tokens)?
//!
//! #### Foods
//! Food records.
//!
//! ```sql
//! CREATE TABLE IF NOT EXISTS foods (
//!     id INTEGER PRIMARY KEY,
//!     name TEXT NOT NULL,
//!     tier INTEGER NOT NULL,
//!     effect TEXT NOT NULL,
//!     pack TEXT NOT NULL,
//!     holdable BOOLEAN NOT NULL,
//!     single_use BOOLEAN NOT NULL,
//!     end_of_battle BOOLEAN NOT NULL,
//!     random BOOLEAN NOT NULL,
//!     n_targets INTEGER NOT NULL,
//!     effect_atk INTEGER NOT NULL,
//!     effect_health INTEGER NOT NULL,
//!     turn_effect BOOLEAN NOT NULL,
//!     cost INTEGER NOT NULL,
//!     img_url TEXT,
//!     CONSTRAINT unq UNIQUE (name, pack)
//! );"
//! ```
//! * `name`
//!     * Name of a food.
//! * `tier`
//!     * Tier of a food.
//! * `effect`
//!     * Effect description
//! * `pack`
//!     * Pack food belongs to.
//!     * Records are duplicated if a food exists in multiple packs.
//! * `holdable`
//!     * Whether food can be held.
//! * `single_use`
//!     * Whether food is single use.
//!     * Example: [`Steak`](crate::FoodName::Steak)
//! * `end_of_battle`
//!     * If effect lasts until the end of the battle from the end of the shop phase.
//!     * Example: [`Cupcake`](crate::FoodName::Cupcake)
//! * `random`
//!     * Whether an effect has some randomness associated with it.
//!     * Example: [`Sushi`](crate::FoodName::Sushi)
//! * `n_targets`
//!     * Number of targets an food effect has.
//! * `effect_atk`
//!     * Attack stat of an effect.
//!     * This can be a summon's attack, the amount of attack given, the amount of damage dealt, etc.
//! * `effect_health`
//!     * This can be a summon's health, the amount of health given, the amount of damage resisted, etc.
//! * `turn_effect`
//!     * Whether an effect is turn-based.
//!     * Example: [`Grapes`](crate::FoodName::Grapes)
//! * `cost`
//!     * The cost of the food.
//! * `img_url`
//!     * Current image url displayed on page.
//!
//! ### Conversion
//! * Any record can be converted into [`Food`](crate::Food)s or [Pet](crate::Pet)s.
//! ```rust compile_fail
//! let food: Food = food_record.try_into().unwrap();
//! let pet: Pet = pet_record.try_into().unwrap();
//! ```
//!
//! ### [`SAPDB`](crate::SAPDB)
//! * This database is initialized as a global, static type using the [`lazy_static`] crate.
//!     ```rust no_run
//!     use saptest::SAPDB;
//!     ```
//! * To create a new test database, use the [`SapDB`](crate::SapDB) struct:
//!     ```rust no_run
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
//! # https://superautopets.fandom.com/wiki/Team_Names
//! # names_version = ?
//!
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
/// Database query.
pub mod query;
/// Record types.
pub mod record;
/// Database setup functions.
pub mod setup;
/// Database helper functions.
pub mod utils;
