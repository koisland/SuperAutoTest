use crate::common::game::Pack;
use serde::{Deserialize, Serialize};

use super::{
    effect::{Effect, EffectAction, EffectTrigger, EffectType, Position, Target},
    effect::{Modify, Statistics},
    foods::names::FoodName,
    pets::names::PetName,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct FoodRecord {
    pub name: String,
    pub tier: usize,
    pub effect: String,
    pub pack: Pack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Food {
    pub name: FoodName,
    pub ability: Effect,
}

fn get_food_effect(name: &FoodName) -> Effect {
    match name {
        FoodName::Chili => Effect {
            target: Target::Enemy,
            // Next enemy relative to position.
            position: Position::Specific(1),
            effect: EffectAction::Remove(Statistics {
                attack: 0,
                health: 5,
            }),
            uses: None,
            effect_type: EffectType::Food,
            trigger: EffectTrigger::None,
        },
        FoodName::Coconut => Effect {
            target: Target::OnSelf,
            position: Position::None,
            // Negate 150 health hit. Pretty much invulnerability.
            effect: EffectAction::Negate(Statistics {
                attack: 0,
                health: 150,
            }),
            uses: Some(1),
            effect_type: EffectType::Food,
            trigger: EffectTrigger::None,
        },
        FoodName::Garlic => Effect {
            target: Target::OnSelf,
            position: Position::None,
            effect: EffectAction::Negate(Statistics {
                attack: 0,
                health: 2,
            }),
            uses: None,
            effect_type: EffectType::Food,
            trigger: EffectTrigger::None,
        },
        FoodName::Honey => Effect {
            target: Target::OnSelf,
            position: Position::None,
            effect: EffectAction::Summon(
                Some(PetName::Bee),
                Statistics {
                    attack: 1,
                    health: 1,
                },
            ),
            uses: None,
            effect_type: EffectType::Food,
            trigger: EffectTrigger::None,
        },
        FoodName::MeatBone => Effect {
            target: Target::OnSelf,
            position: Position::None,
            effect: EffectAction::Add(Statistics {
                attack: 4,
                health: 0,
            }),
            uses: None,
            effect_type: EffectType::Food,
            trigger: EffectTrigger::None,
        },
        FoodName::Melon => Effect {
            target: Target::OnSelf,
            position: Position::None,
            effect: EffectAction::Negate(Statistics {
                attack: 0,
                health: 20,
            }),
            uses: Some(1),
            effect_type: EffectType::Food,
            trigger: EffectTrigger::None,
        },
        FoodName::Mushroom => Effect {
            target: Target::OnSelf,
            position: Position::None,
            effect: EffectAction::Summon(
                // Replace during runtime.
                None,
                Statistics {
                    attack: 1,
                    health: 1,
                },
            ),
            uses: Some(1),
            effect_type: EffectType::Food,
            trigger: EffectTrigger::None,
        },
        FoodName::Peanuts => Effect {
            target: Target::OnSelf,
            position: Position::None,
            effect: EffectAction::Add(Statistics {
                attack: 150,
                health: 0,
            }),
            uses: None,
            effect_type: EffectType::Food,
            trigger: EffectTrigger::None,
        },
        FoodName::Steak => Effect {
            target: Target::OnSelf,
            position: Position::None,
            effect: EffectAction::Add(Statistics {
                attack: 20,
                health: 0,
            }),
            uses: Some(1),
            effect_type: EffectType::Food,
            trigger: EffectTrigger::None,
        },
    }
}
impl Food {
    /// Create a `Food` from `FoodName`.
    pub fn new(name: &FoodName) -> Food {
        // TODO: Regex to get food effect stats.
        Food {
            name: name.clone(),
            ability: get_food_effect(name),
        }
    }
}
