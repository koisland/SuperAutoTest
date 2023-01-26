//! This library provides a testing framework for the game [Super Auto Pets](https://teamwoodgames.com/).
//!
//! # Example
//! ```rust
//! use sapt::{Pet, PetName, PetCombat, Food, FoodName, Team};
//!
//! let mut pet = Pet::from(PetName::Ant);
//! let mut enemy_pet = Pet::from(PetName::Ant);
//!
//! // Give food to pets.
//! pet.item = Some(Food::from(FoodName::Melon));
//! pet.item = Some(Food::from(FoodName::Melon));
//!
//! // And fight individually.
//! pet.attack(&mut enemy_pet);
//!
//! // Or, create a team.
//! let mut team = Team::new(&[Some(pet)], 5).unwrap();
//! let mut enemy_team = Team::new(&[Some(enemy_pet)], 5).unwrap();
//!
//! // And fight as a team.
//! team.fight(&mut enemy_team);
//! ```
//!
//! # Details
//! * [SQLite](https://www.sqlite.org/index.html) and the Rust wrapper for it, [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/), are used to store and query game information.
//!     * This information is scraped and parsed from the Super Auto Pets Fandom Wiki
//!
//!
//! # Shops
//! * This is currently not implemented.
//! * Consider using the Python package [sapai](https://github.com/manny405/sapai) if shop functionality is required.
#![warn(missing_docs)]

#[macro_use]
extern crate lazy_regex;

pub mod battle;
pub mod db;
pub mod foods;
pub mod pets;

pub use crate::battle::{
    effect::Effect,
    state::{Outcome, Statistics},
    team::Team,
    team_effect_apply::EffectApply,
};
pub use crate::foods::{food::Food, names::FoodName};
pub use crate::pets::{combat::PetCombat, names::PetName, pet::Pet};

mod error;
mod graph;
mod regex_patterns;
mod wiki_scraper;

#[cfg(test)]
mod tests;

const SCRAPER_SOURCES: &str = "./config/sources.json";
const DB_CREATE_SQL: &str = "./src/lib/db/sql/create_tables.sql";
const DB_INSERT_PET_SQL: &str = "./src/lib/db/sql/insert_pet.sql";
const DB_INSERT_FOOD_SQL: &str = "./src/lib/db/sql/insert_food.sql";
const DB_FNAME: &str = "./sap.db";
