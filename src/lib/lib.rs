//! A testing framework for the game [Super Auto Pets](https://teamwoodgames.com/).
//!
//! Game information is scraped and parsed from the [Super Auto Pets Fandom Wiki](https://superautopets.fandom.com/f) before being stored in a [SQLite](https://www.sqlite.org/index.html) database.
//! * Read more under the [`db`](crate::db) module.
//!
//! ### Teams
//! Build a [`Team`] and simulate battles between them.
//!  ```
//! use saptest::{Pet, PetName, Food, FoodName, Team, Position};
//!
//! // Create a team.
//! let mut team = Team::new(
//!     &vec![Pet::try_from(PetName::Ant).unwrap(); 5],
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
//!     Shop, ShopItem, TeamShopping, Team,
//!     Position, Entity, EntityName, FoodName
//! };
//!
//! // All teams are constructed with a shop at tier 1.
//! let mut team = Team::default();
//!
//! // All shop functionality is supported.
//! team.set_shop_seed(Some(1212))
//!     .open_shop().unwrap()
//!     .buy(&Position::First, &Entity::Pet, &Position::First).unwrap()
//!     .sell(&Position::First).unwrap()
//!     .freeze_shop(Position::Last, Entity::Pet).unwrap()
//!     .roll_shop().unwrap()
//!     .close_shop().unwrap();
//!
//! // Shops can be built separately and can replace a team's shop.
//! let mut tier_5_shop = Shop::new(3, Some(42)).unwrap();
//! let weakness = ShopItem::new(
//!     EntityName::Food(FoodName::Weak),
//!     5
//! ).unwrap();
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
//!     battle::{
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

#![warn(missing_docs)]

#[macro_use]
extern crate lazy_regex;

use lazy_static::lazy_static;
use std::sync::Arc;

pub mod battle;
pub mod db;
pub mod error;
pub mod foods;
pub mod pets;
pub mod shop;

#[doc(inline)]
pub use crate::battle::{
    effect::{Effect, Entity, EntityName},
    state::{Condition, Outcome, Position},
    stats::Statistics,
    team::Team,
    team_effect_apply::TeamEffects,
};
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
    #[doc(hidden)]
    /// Global pooled database.
    pub static ref SAPDB: Arc<SapDB> = Arc::new(SapDB::new(crate::DB_FNAME).unwrap());
}
