use serde::{Deserialize, Serialize};

use crate::common::{
    battle::{
        effect::{Effect, EffectType},
        state::{Action, Position, Statistics, Target},
        trigger::*,
    },
    foods::names::FoodName,
    pets::{names::PetName, pet::Pet},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Food {
    pub name: FoodName,
    pub ability: Effect,
}

impl From<FoodName> for Food {
    fn from(value: FoodName) -> Self {
        Food::new(&value)
    }
}

#[allow(dead_code)]
fn get_food_effect(name: &FoodName) -> Effect {
    match name {
        FoodName::Chili => Effect {
            target: Target::Enemy,
            // Next enemy relative to position.
            position: Position::Specific(1),
            action: Action::Remove(Statistics {
                attack: 0,
                health: 5,
            }),
            uses: None,
            effect_type: EffectType::Food,
            trigger: TRIGGER_SELF_ATTACK,
        },
        FoodName::Coconut => Effect {
            target: Target::Friend,
            position: Position::OnSelf,
            // Negate 150 health hit. Pretty much invulnerability.
            action: Action::Negate(Statistics {
                attack: 0,
                health: 150,
            }),
            uses: Some(1),
            effect_type: EffectType::Food,
            trigger: TRIGGER_NONE,
        },
        FoodName::Garlic => Effect {
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Negate(Statistics {
                attack: 0,
                health: 2,
            }),
            uses: None,
            effect_type: EffectType::Food,
            trigger: TRIGGER_NONE,
        },
        FoodName::Honey => {
            let bee = Box::new(Pet {
                name: PetName::Bee,
                tier: 1,
                stats: Statistics {
                    attack: 1,
                    health: 1,
                },
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
            action: Action::Add(Statistics {
                attack: 4,
                health: 0,
            }),
            uses: None,
            effect_type: EffectType::Food,
            trigger: TRIGGER_NONE,
        },
        FoodName::Melon => Effect {
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Negate(Statistics {
                attack: 0,
                health: 20,
            }),
            uses: Some(1),
            effect_type: EffectType::Food,
            trigger: TRIGGER_NONE,
        },
        FoodName::Mushroom => Effect {
            target: Target::Friend,
            position: Position::Trigger,
            // Replace during runtime.
            action: Action::Summon(None),
            uses: Some(1),
            effect_type: EffectType::Food,
            trigger: TRIGGER_SELF_FAINT,
        },
        FoodName::Peanuts => Effect {
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(Statistics {
                attack: 150,
                health: 0,
            }),
            uses: None,
            effect_type: EffectType::Food,
            trigger: TRIGGER_NONE,
        },
        FoodName::Steak => Effect {
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(Statistics {
                attack: 20,
                health: 0,
            }),
            uses: Some(1),
            effect_type: EffectType::Food,
            trigger: TRIGGER_NONE,
        },
    }
}

#[allow(dead_code)]
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
