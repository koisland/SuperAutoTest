use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    battle::effect::Effect,
    db::{setup::get_connection, utils::map_row_to_food},
    foods::names::FoodName,
};

/// A Super Auto Pets food.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Food {
    /// A food name.
    pub name: FoodName,
    /// A food effect.
    pub ability: Effect,
    /// Whether the food is holdable.
    pub holdable: bool,
    /// Whether an effect is temporary.
    pub temp: bool,
    /// The cost of a food.
    pub cost: usize,
}

impl From<&FoodName> for Food {
    fn from(value: &FoodName) -> Self {
        Food::new(value)
            .expect("Unable to create food from foodname. Check sqlite entry for typos/changes.")
    }
}

impl From<FoodName> for Food {
    fn from(value: FoodName) -> Self {
        Food::new(&value)
            .expect("Unable to create food from foodname. Check sqlite entry for typos/changes.")
    }
}

impl Display for Food {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}: {}]", self.name, self.ability)
    }
}

impl Food {
    /// Create a `Food` from `FoodName`.
    pub fn new(name: &FoodName) -> Result<Food, Box<dyn Error>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM foods WHERE name = ?")?;
        let food_record = stmt.query_row([name.to_string()], map_row_to_food)?;
        let effect = Effect::from(&food_record);

        Ok(Food {
            name: name.clone(),
            ability: effect,
            temp: food_record.single_use,
            holdable: food_record.holdable,
            cost: food_record.cost,
        })
    }
}
