//! A testing framework for the game [Super Auto Pets](https://teamwoodgames.com/).
//!
//! Game information is scraped and parsed from the [Super Auto Pets Wiki](https://superautopets.wiki.gg/wiki/Super_Auto_Pets_Wiki) before being stored in a [SQLite](https://www.sqlite.org/index.html) database.
//!
//! ### Teams
//! Build a [`Team`] and simulate battles between them.
//!
//! Then visualize the results in `.dot` format!
//!  ```
//! use saptest::{
//!     Pet, PetName, Food, FoodName,
//!     Team, TeamCombat, Position, create_battle_digraph
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
//! team.set_item(&Position::First, Food::try_from(FoodName::Garlic).ok());
//! enemy_team.set_item(&Position::First, Food::try_from(FoodName::Garlic).ok());
//!
//! // And fight!
//! team.fight(&mut enemy_team).unwrap();
//!
//! // Create a graph of the fight.
//! println!("{}", create_battle_digraph(&team, false));
//! ```
//!
//! ```bash
//! digraph {
//!     rankdir=LR
//!     node [shape=box, style="rounded, filled", fontname="Arial"]
//!     edge [fontname="Arial"]
//!     0 [ label = "Ant_0 - The Fragile Truckers_copy" ]
//!     1 [ label = "Ant_0 - The Fragile Truckers", fillcolor = "yellow" ]
//!     2 [ label = "Ant_3 - The Fragile Truckers", fillcolor = "yellow" ]
//!     3 [ label = "Ant_4 - The Fragile Truckers_copy" ]
//!     0 -> 1 [ label = "(Attack, Damage (0, 2), Phase: 1)" ]
//!     1 -> 0 [ label = "(Attack, Damage (0, 2), Phase: 1)" ]
//!     1 -> 2 [ label = "(Faint, Add (1, 1), Phase: 1)" ]
//!     0 -> 3 [ label = "(Faint, Add (1, 1), Phase: 1)" ]
//! }
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
//!         &Position::First, // From first.
//!         &Entity::Pet, // Pet
//!         &Position::First // To first position, merging if possible.
//!     ).unwrap()
//!     .sell(&Position::First).unwrap()
//!     .move_pets(
//!         &Position::First, // From first pet
//!         &Position::Relative(-2), // To 2nd pet behind.
//!         true // And merge them if possible.
//!     ).unwrap()
//!     .freeze_shop(&Position::Last, &Entity::Pet).unwrap()
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
//!     Position, Effect, Statistics,
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
//!     TRIGGER_START_BATTLE, // Effect trigger
//!     Target::Friend, // Target
//!     Position::Adjacent, // Positions
//!     Action::Gain(GainType::DefaultItem(FoodName::Melon)), // Action
//!     Some(1), // Number of uses.
//!     false, // Is temporary.
//! );
//! let mut custom_pet = Pet::custom(
//!     "MelonBear",
//!     Statistics::new(50, 50).unwrap(),
//!     &[custom_effect],
//! );
//! // Fight two pets individually as well.
//! // Note: Effects don't activate here.
//! pet.attack(&mut custom_pet);
//! ```
//!
//! ### Logging
//! Enable logging with a crate like [`simple_logger`](https://docs.rs/simple_logger/latest/simple_logger/#).
//!
//! ### Config
//! To configure the global [`SapDB`]'s startup, create a `.saptest.toml` file in the root of your project.
//! * Specify page version for pets, foods, and tokens to query.
//! * Toggle recurring updates on startup.
//! * Set database filename.
//!
//! Read more under the [`db`] module.

#![warn(missing_docs)]
// TODO: Split errors into smaller categories?
#![allow(clippy::result_large_err)]

#[macro_use]
extern crate lazy_regex;

use lazy_static::lazy_static;
use std::fs::read_to_string;

pub mod db;
pub mod effects;
pub mod error;
pub mod foods;
pub mod pets;
pub mod shop;
pub mod teams;
pub mod toys;
pub mod visualization;

#[doc(inline)]
pub use crate::effects::{
    effect::{Effect, Entity, EntityName},
    state::{ItemCondition, Position},
    stats::Statistics,
};
#[doc(inline)]
pub use crate::teams::{combat::TeamCombat, effects::TeamEffects, team::Team, viewer::TeamViewer};

use crate::config::{LibConfig, CONFIG_PATH, DEFAULT_CONFIG};
#[doc(inline)]
pub use crate::db::{query::SAPQuery, setup::SapDB};
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
#[doc(inline)]
pub use crate::toys::{names::ToyName, toy::Toy};

#[doc(inline)]
pub use crate::visualization::{digraph::create_battle_digraph, tsv::create_battle_df};

#[doc = include_str!("../../README.md")]
mod config;
mod regex_patterns;
#[cfg(test)]
mod tests;
mod wiki_scraper;

const DB_FNAME: &str = "./sap.db";
const ENV_SAPTEST_CONFIG: &str = "CONFIG_SAPTEST";

lazy_static! {
    #[doc(hidden)]
    static ref CONFIG: LibConfig = {
        // Read in env var for saptest config file if one provided. Otherwise, use default.
        let config = std::env::var(ENV_SAPTEST_CONFIG).unwrap_or(CONFIG_PATH.to_string());
        read_to_string(config)
        .map_or_else(
            |_| DEFAULT_CONFIG,
            |toml_str| toml::from_str(&toml_str).unwrap()
        )
    };

    #[doc(hidden)]
    /// Global pooled database.
    pub static ref SAPDB: SapDB = SapDB::new(
        CONFIG.database.filename.as_ref().unwrap_or(&DB_FNAME.to_owned())
    ).unwrap();
}
