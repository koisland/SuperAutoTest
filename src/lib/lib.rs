//! This library provides a testing framework for the game [Super Auto Pets](https://teamwoodgames.com/).
//!
//! # Example
//! ```rust
//! use sapt::{Pet, PetName, PetCombat, Food, FoodName, Team, Position};
//!
//! // Create pets.
//! let pet = Pet::try_from(PetName::Ant).unwrap();
//! let enemy_pet = Pet::try_from(PetName::Ant).unwrap();
//!
//! // Create a team.
//! let mut team = Team::new(&vec![pet; 5], 5).unwrap();
//! let mut enemy_team = Team::new(&vec![enemy_pet; 5], 5).unwrap();
//!
//! // Set a seed for a team.
//! team.set_seed(25);
//!
//! // Give food to pets.
//! team.set_item(Position::First, Food::try_from(FoodName::Garlic).ok());
//! enemy_team.set_item(Position::First, Food::try_from(FoodName::Garlic).ok());
//!
//! // And fight as a team.
//! team.fight(&mut enemy_team);
//! ```
//!
//! # Details
//! * [SQLite](https://www.sqlite.org/index.html) and the Rust wrapper for it, [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/), are used to store and query game information.
//!     * This information is scraped and parsed from the Super Auto Pets Fandom Wiki.
//!
//! # Shops
//! * Currently not implemented.
//! * Consider using the Python package [sapai](https://github.com/manny405/sapai) if shop functionality is required.

#![warn(missing_docs)]

#[macro_use]
extern crate lazy_regex;

use lazy_static::lazy_static;
use std::sync::Arc;

pub mod battle;
pub mod db;
pub mod foods;
pub mod pets;

pub use crate::battle::{
    effect::Effect,
    state::{Outcome, Position},
    stats::Statistics,
    team::Team,
    team_effect_apply::EffectApply,
};
pub use crate::db::setup::SapDB;
pub use crate::foods::{food::Food, names::FoodName};
pub use crate::pets::{combat::PetCombat, names::PetName, pet::Pet};

mod error;
mod graph;
mod regex_patterns;
mod wiki_scraper;

#[cfg(test)]
mod tests;

const PET_URL: &str = "https://superautopets.fandom.com/wiki/Pets?action=raw";
const FOOD_URL: &str = "https://superautopets.fandom.com/wiki/Food?action=raw";
const TOKEN_URL: &str = "https://superautopets.fandom.com/wiki/Tokens?action=raw";
const DB_FNAME: &str = "./sap.db";

lazy_static! {
    /// Global pooled database.
    pub static ref SAPDB: Arc<SapDB> = Arc::new(SapDB::new(crate::DB_FNAME).unwrap());
}
