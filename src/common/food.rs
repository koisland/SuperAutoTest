use crate::common::game::Pack;
use serde::{Deserialize, Serialize};

use super::{
    effect::Effect, effect::FoodEffect, effect::Position, effect::{Statistics, Modify}, effect::Target,
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
            uses: None,
        },
        FoodName::Coconut => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            // Negate 150 health hit. Pretty much invulnerability.
            effect: Effect::Negate(Statistics {
                attack: 0,
                health: 150,
            }),
            uses: Some(1),
        },
        FoodName::Garlic => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            effect: Effect::Negate(Statistics {
                attack: 0,
                health: 2,
            }),
            uses: None,
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
            uses: None,
        },
        FoodName::MeatBone => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            effect: Effect::Add(Statistics {
                attack: 4,
                health: 0,
            }),
            uses: None,
        },
        FoodName::Melon => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            effect: Effect::Negate(Statistics {
                attack: 0,
                health: 20,
            }),
            uses: Some(1),
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
            uses: Some(1),
        },
        FoodName::Peanuts => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            effect: Effect::Add(Statistics {
                attack: 150,
                health: 0,
            }),
            uses: None,
        },
        FoodName::Steak => FoodEffect {
            target: Target::OnSelf,
            position: Position::None,
            effect: Effect::Add(Statistics {
                attack: 20,
                health: 0,
            }),
            uses: Some(1),
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

impl Modify for Food {
    fn add_uses(&mut self, n: usize) -> &Self {
        self.ability.uses.as_mut().map(|uses| *uses += n );
        self
    }

    fn remove_uses(&mut self, n: usize) -> &Self {
        self.ability.uses.as_mut().map(|uses| if *uses >= n { *uses -= n } );
        self
    }
}
