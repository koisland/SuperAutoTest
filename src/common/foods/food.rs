use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{
    common::{
        battle::{
            effect::{Effect, EffectType},
            state::{Action, Position, Statistics, Target},
            trigger::*,
        },
        foods::names::FoodName,
        pets::{
            names::PetName,
            pet::{Pet, MAX_PET_STATS, MIN_PET_STATS},
        },
    },
    db::{setup::get_connection, utils::map_row_to_food},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Food {
    pub name: FoodName,
    pub ability: Effect,
}

impl From<FoodName> for Food {
    fn from(value: FoodName) -> Self {
        Food::new(&value).unwrap()
    }
}

#[allow(dead_code)]
fn get_food_effect(name: &FoodName, effect_stats: Statistics, uses: Option<usize>) -> Effect {
    match name {
        FoodName::Chili => Effect {
            target: Target::Enemy,
            // Next enemy relative to position.
            position: Position::Specific(1),
            action: Action::Remove(effect_stats),
            uses: None,
            effect_type: EffectType::Food,
            trigger: TRIGGER_SELF_ATTACK,
        },
        FoodName::Coconut => Effect {
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Invincible,
            uses,
            effect_type: EffectType::Food,
            trigger: TRIGGER_NONE,
        },
        FoodName::Garlic => Effect {
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Negate(effect_stats),
            uses: None,
            effect_type: EffectType::Food,
            trigger: TRIGGER_NONE,
        },
        FoodName::Honey => {
            let bee = Box::new(Pet {
                name: PetName::Bee,
                tier: 1,
                stats: effect_stats,
                lvl: 1,
                effect: None,
                item: None,
                pos: None,
            });
            Effect {
                target: Target::Friend,
                position: Position::Trigger,
                action: Action::Summon(Some(bee)),
                uses: None,
                effect_type: EffectType::Food,
                trigger: TRIGGER_SELF_FAINT,
            }
        }
        FoodName::MeatBone => Effect {
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(effect_stats),
            uses: None,
            effect_type: EffectType::Food,
            trigger: TRIGGER_NONE,
        },
        FoodName::Melon => Effect {
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Negate(effect_stats),
            uses,
            effect_type: EffectType::Food,
            trigger: TRIGGER_NONE,
        },
        FoodName::Mushroom => Effect {
            target: Target::Friend,
            position: Position::Trigger,
            // Replace during runtime.
            action: Action::Summon(None),
            uses,
            effect_type: EffectType::Food,
            trigger: TRIGGER_SELF_FAINT,
        },
        FoodName::Peanuts => Effect {
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Kill,
            uses: None,
            effect_type: EffectType::Food,
            trigger: TRIGGER_NONE,
        },
        FoodName::Steak => Effect {
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(effect_stats),
            uses,
            effect_type: EffectType::Food,
            trigger: TRIGGER_NONE,
        },
        FoodName::Weakness => Effect {
            effect_type: EffectType::Food,
            trigger: TRIGGER_SELF_HURT,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Remove(effect_stats),
            uses: None,
        },
    }
}

#[allow(dead_code)]
impl Food {
    /// Create a `Food` from `FoodName`.
    pub fn new(name: &FoodName) -> Result<Food, Box<dyn Error>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM foods WHERE name = ?")?;
        let food_record = stmt.query_row([name.to_string()], map_row_to_food)?;

        // TODO: Change Statistics to be isize.
        let effect_atk: usize = food_record.effect_atk.try_into()?;
        let effect_health: usize = food_record.effect_health.try_into()?;

        let effect = get_food_effect(
            name,
            Statistics {
                attack: effect_atk.clamp(MIN_PET_STATS, MAX_PET_STATS),
                health: effect_health.clamp(MIN_PET_STATS, MAX_PET_STATS),
            },
            food_record.single_use.then_some(1),
        );

        Ok(Food {
            name: name.clone(),
            ability: effect,
        })
    }
}
