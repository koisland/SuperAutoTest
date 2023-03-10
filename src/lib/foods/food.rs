use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    db::record::FoodRecord, effects::effect::Effect, error::SAPTestError, foods::names::FoodName,
    Entity, SAPDB,
};

/// A Super Auto Pets food.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Food {
    /// A food name.
    pub name: FoodName,
    /// Food tier.
    pub tier: usize,
    /// A food effect.
    pub ability: Effect,
    /// Whether the food is holdable.
    pub holdable: bool,
    /// Whether an effect is temporary.
    pub temp: bool,
    /// The cost of a food.
    pub cost: usize,
    /// Number of targets this food affects.
    pub n_targets: usize,
}
impl TryFrom<FoodName> for Food {
    type Error = SAPTestError;

    fn try_from(value: FoodName) -> Result<Self, Self::Error> {
        Food::new(&value, None)
    }
}

impl TryFrom<&FoodName> for Food {
    type Error = SAPTestError;

    fn try_from(value: &FoodName) -> Result<Self, Self::Error> {
        Food::new(value, None)
    }
}

impl Display for Food {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}: {}]", self.name, self.ability)
    }
}

impl Food {
    /// Create a `Food` from [`FoodName`] and an optional [`Effect`].
    /// # Example
    /// ```
    /// use saptest::{Food, FoodName, Effect};
    /// let food = Food::new(&FoodName::Custom("Air".to_string()), Some(Effect::default()));
    /// assert!(food.is_ok());
    /// ```
    pub fn new(name: &FoodName, effect: Option<Effect>) -> Result<Food, SAPTestError> {
        // Set default.
        let (food_record, effect) =
            if let (FoodName::Custom(_), Some(custom_effect)) = (name, effect.clone()) {
                (
                    FoodRecord {
                        name: name.clone(),
                        single_use: false,
                        holdable: true,
                        cost: 3,
                        ..Default::default()
                    },
                    custom_effect,
                )
            } else {
                let food_record: FoodRecord = SAPDB
                    .execute_query(Entity::Food, &[("name", &vec![name.to_string()])])?
                    // .execute_food_query("SELECT * FROM foods WHERE name = ?", &[name.to_string()])?
                    .into_iter()
                    .next()
                    .ok_or(SAPTestError::QueryFailure {
                        subject: "No Food Effect".to_string(),
                        reason: format!("No food record for {name}"),
                    })?
                    .try_into()?;

                // Allow custom effect for food if provided.
                let effect = if let Some(custom_effect) = effect {
                    custom_effect
                } else {
                    Effect::try_from(&food_record)?
                };
                (food_record, effect)
            };

        Ok(Food {
            name: name.clone(),
            tier: food_record.tier,
            ability: effect,
            temp: food_record.single_use,
            holdable: food_record.holdable,
            cost: food_record.cost,
            n_targets: food_record.n_targets,
        })
    }
}
