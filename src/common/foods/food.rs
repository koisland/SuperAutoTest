use std::{convert::TryFrom, error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    common::{
        battle::{effect::Effect, state::Statistics},
        foods::{effects::get_food_effect, names::FoodName},
        pets::pet::{MAX_PET_STATS, MIN_PET_STATS},
    },
    db::{setup::get_connection, utils::map_row_to_food},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Food {
    pub name: FoodName,
    pub ability: Effect,
    pub holdable: bool,
    pub temp: bool,
    pub cost: usize,
}

impl From<&FoodName> for Food {
    fn from(value: &FoodName) -> Self {
        Food::new(value).unwrap()
    }
}

impl From<FoodName> for Food {
    fn from(value: FoodName) -> Self {
        Food::new(&value).unwrap()
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

        let effect_atk: usize = food_record.effect_atk.try_into()?;
        let effect_health: usize = food_record.effect_health.try_into()?;

        let effect = get_food_effect(
            name,
            Statistics {
                attack: isize::try_from(effect_atk)?.clamp(MIN_PET_STATS, MAX_PET_STATS),
                health: isize::try_from(effect_health)?.clamp(MIN_PET_STATS, MAX_PET_STATS),
            },
            food_record.single_use.then_some(1),
            food_record.n_targets,
            food_record.end_of_battle,
        );

        Ok(Food {
            name: *name,
            ability: effect,
            temp: food_record.single_use,
            holdable: food_record.holdable,
            cost: food_record.cost,
        })
    }
}
