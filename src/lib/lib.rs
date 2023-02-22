//! A testing framework for the game [Super Auto Pets](https://teamwoodgames.com/).
//!
//! Game information is scraped and parsed from the [Super Auto Pets Fandom Wiki](https://superautopets.fandom.com/f) before being stored in a [SQLite](https://www.sqlite.org/index.html) database.
//!
//! ### Teams
//! Build a [`Team`] and simulate battles between them.
//!  ```
//! use saptest::{
//!     Pet, PetName, Food, FoodName,
//!     Team, TeamCombat, Position
//! };
//!
//! // Create a team.
//! let mut team = Team::new(
//!     &vec![Some(Pet::try_from(PetName::Ant).unwrap()); 5],
//!     5
//! ).unwrap();
//! let mut enemy_team = team.clone();
//!
//! // Set a seed for a team.
//! team.set_seed(Some(25));
//!
//! // Give food to pets.
//! team.set_item(Position::First, Food::try_from(FoodName::Garlic).ok());
//! enemy_team.set_item(Position::First, Food::try_from(FoodName::Garlic).ok());
//!
//! // And fight!
//! team.fight(&mut enemy_team);
//! ```
//! ### Shops
//! Add shop functionality to a [`Team`] and roll, freeze, buy/sell pets and foods.
//! ```
//! use saptest::{
//!     Entity, EntityName, Pet, PetName, Food, FoodName,
//!     Shop, ShopItem, TeamShopping,
//!     Team, TeamViewer, Position,
//!     db::pack::Pack
//! };
//!
//! // All teams are constructed with a shop at tier 1.
//! let mut team = Team::new(
//!     &vec![Some(Pet::try_from(PetName::Ant).unwrap()); 4],
//!     5
//! ).unwrap();
//!
//! // All shop functionality is supported.
//! team.set_shop_seed(Some(1212))
//!     .set_shop_packs(&[Pack::Turtle])
//!     .open_shop().unwrap()
//!     .buy(
//!         &Position::First, // From
//!         &Entity::Pet, // Pets
//!         &Position::First // To first position, merging if possible.
//!     ).unwrap()
//!     .sell(&Position::First).unwrap()
//!     .move_pets(
//!         &Position::First, // From first pet
//!         &Position::Relative(-2), // To 2nd pet behind.
//!         true // And merge them if possible.
//!     ).unwrap()
//!     .freeze_shop(Position::Last, Entity::Pet).unwrap()
//!     .roll_shop().unwrap()
//!     .close_shop().unwrap();
//!
//! // Shops can be built separately and can replace a team's shop.
//! let mut tier_5_shop = Shop::new(3, Some(42)).unwrap();
//! let weakness = ShopItem::new(
//!     Food::try_from(FoodName::Weak).unwrap()
//! );
//! tier_5_shop.add_item(weakness).unwrap();
//! team.replace_shop(tier_5_shop).unwrap();
//! ```
//!
//! ### Pets
//! Build custom [`Pet`]s and [`Effect`]s.
//! ```
//! use saptest::{
//!     Pet, PetName, PetCombat,
//!     Food, FoodName,
//!     Entity, Position, Effect, Statistics,
//!     effects::{
//!         trigger::TRIGGER_START_BATTLE,
//!         actions::GainType,
//!         state::Target,
//!         actions::Action
//!     }
//! };
//!
//! // Create known pets.
//! let mut pet = Pet::try_from(PetName::Ant).unwrap();
//!
//! // A custom pet and effect.
//! let custom_effect = Effect::new(
//!     Entity::Pet,
//!     TRIGGER_START_BATTLE, // Effect trigger
//!     Target::Friend, // Target
//!     Position::Adjacent, // Positions
//!     Action::Gain(GainType::DefaultItem(FoodName::Melon)), // Action
//!     Some(1), // Number of uses.
//!     false, // Is temporary.
//! );
//! let mut custom_pet = Pet::custom(
//!     "MelonBear",
//!     Some("melonbear_1".to_string()),
//!     Statistics::new(50, 50).unwrap(),
//!     &[custom_effect],
//! );
//! // Fight two pets individually as well.
//! // Note: Effects don't activate here.
//! pet.attack(&mut custom_pet);
//! ```
//!
//! ### Logging
//! Enable logging with [`log4rs`](https://docs.rs/log4rs/latest/log4rs/) with [`build_log_config`](crate::logging::build_log_config) to view battle logic and more.
//! ```
//! use saptest::logging::build_log_config;
//!
//! let config = build_log_config();
//! log4rs::init_config(config).unwrap();
//!
//! // Code below.
//! ```
//!
//! ### Config
//! To configure the global [`SapDB`]'s startup, create a `.saptest.toml` file in the root of your project.
//! * Specify page version for pets, foods, and tokens to query.
//! * Toggle recurring updates on startup.
//! * Set database filename.
//!
//! Read more under the [`db`](crate::db) module.

#![warn(missing_docs)]

#[macro_use]
extern crate lazy_regex;

use lazy_static::lazy_static;
use std::fs::read_to_string;

pub mod db;
pub mod effects;
pub mod error;
pub mod foods;
pub mod logging;
pub mod pets;
pub mod shop;
pub mod teams;

#[doc(inline)]
pub use crate::effects::{
    effect::{Effect, Entity, EntityName},
    state::{Condition, Position},
    stats::Statistics,
};
#[doc(inline)]
pub use crate::teams::{combat::TeamCombat, effects::TeamEffects, team::Team, viewer::TeamViewer};

use crate::config::{LibConfig, CONFIG_PATH, DEFAULT_CONFIG};
#[doc(inline)]
pub use crate::db::setup::SapDB;
#[doc(inline)]
pub use crate::foods::{food::Food, names::FoodName};
#[doc(inline)]
pub use crate::pets::{combat::PetCombat, names::PetName, pet::Pet};
#[doc(inline)]
pub use crate::shop::{
    store::{Shop, ShopItem},
    team_shopping::TeamShopping,
    viewer::{ShopItemViewer, ShopViewer},
};

mod config;
mod graph;
mod regex_patterns;
#[cfg(test)]
mod tests;
mod wiki_scraper;

const DB_FNAME: &str = "./sap.db";

lazy_static! {
    #[doc(hidden)]
    static ref CONFIG: LibConfig = read_to_string(CONFIG_PATH)
        .map_or_else(
            |_| DEFAULT_CONFIG,
            |toml_str| toml::from_str(&toml_str).unwrap()
        );

    #[doc(hidden)]
    /// Global pooled database.
    pub static ref SAPDB: SapDB = SapDB::new(
        CONFIG.database.filename.as_ref().unwrap_or(&DB_FNAME.to_owned())
    ).unwrap();
}
