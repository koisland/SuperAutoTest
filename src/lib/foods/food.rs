use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    battle::effect::Effect, db::record::FoodRecord, error::SAPTestError, foods::names::FoodName,
    SAPDB,
};

/// A Super Auto Pets food.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
impl TryFrom<FoodName> for Food {
    type Error = SAPTestError;

    fn try_from(value: FoodName) -> Result<Self, Self::Error> {
        Food::new(&value)
    }
}

impl TryFrom<&FoodName> for Food {
    type Error = SAPTestError;

    fn try_from(value: &FoodName) -> Result<Self, Self::Error> {
        Food::new(value)
    }
}

impl Display for Food {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}: {}]", self.name, self.ability)
    }
}

impl Food {
    /// Create a `Food` from `FoodName`.
    pub fn new(name: &FoodName) -> Result<Food, SAPTestError> {
        let food_record: FoodRecord = SAPDB
            .execute_food_query("SELECT * FROM foods WHERE name = ?", &[name.to_string()])?
            .into_iter()
            .next()
            .ok_or(SAPTestError::QueryFailure {
                subject: "No Food Effect".to_string(),
                reason: format!("No food record for {name}"),
            })?;
        let effect = Effect::try_from(&food_record)?;

        Ok(Food {
            name: name.clone(),
            ability: effect,
            temp: food_record.single_use,
            holdable: food_record.holdable,
            cost: food_record.cost,
        })
    }
}
