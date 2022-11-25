use serde::{Deserialize, Serialize};

use crate::common::game::Pack;

use super::{
    effect::Effect, effect::FoodEffect, effect::Position, effect::Statistics, effect::Target,
    foods::names::FoodName, pets::names::PetName,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct FoodRecord {
    pub name: String,
    pub tier: usize,
    pub effect: String,
    pub pack: Pack,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Food {
    pub name: FoodName,
    pub ability: FoodEffect,
}

fn get_food_effect(name: &FoodName) -> FoodEffect {
    match name {
        FoodName::Chili => FoodEffect {
            target: Target::Enemy,
            // Next enemy relative to position.
            position: Position::Specific(1),
            effect: Effect::Remove(Statistics {
                attack: 0,
                health: 5,
            }),
            limit: None,
        },
        FoodName::Coconut => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            // Negate 150 health hit. Pretty much invulnerability.
            effect: Effect::Negate(Statistics {
                attack: 0,
                health: 150,
            }),
            limit: Some(1),
        },
        FoodName::Garlic => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            effect: Effect::Negate(Statistics {
                attack: 0,
                health: 2,
            }),
            limit: None,
        },
        FoodName::Honey => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            effect: Effect::Summon(
                Some(PetName::Bee),
                Statistics {
                    attack: 1,
                    health: 1,
                },
            ),
            limit: None,
        },
        FoodName::MeatBone => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            effect: Effect::Add(Statistics {
                attack: 4,
                health: 0,
            }),
            limit: None,
        },
        FoodName::Melon => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            effect: Effect::Negate(Statistics {
                attack: 0,
                health: 20,
            }),
            limit: Some(1),
        },
        FoodName::Mushroom => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            effect: Effect::Summon(
                // Replace during runtime.
                None,
                Statistics {
                    attack: 1,
                    health: 1,
                },
            ),
            limit: Some(1),
        },
        FoodName::Peanuts => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            effect: Effect::Add(Statistics {
                attack: 150,
                health: 0,
            }),
            limit: None,
        },
        FoodName::Steak => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            effect: Effect::Add(Statistics {
                attack: 20,
                health: 0,
            }),
            limit: Some(1),
        },
    }
}
impl Food {
    pub fn new(name: &FoodName) -> Food {
        // TODO: Regex to get food effect stats.
        Food {
            name: name.clone(),
            ability: get_food_effect(name),
        }
    }
}
