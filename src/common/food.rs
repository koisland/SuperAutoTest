use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use super::{
    effect::{Action, Modify, Outcome, Statistics},
    effect::{Effect, EffectAction, EffectTrigger, EffectType, Position, Target},
    foods::names::FoodName,
    pet::Pet,
    pets::names::PetName,
};

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
            uses: Some(Rc::new(RefCell::new(1))),
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
        FoodName::Honey => {
            let bee = Box::new(Pet {
                name: PetName::Bee,
                tier: 1,
                stats: Rc::new(RefCell::new(Statistics {
                    attack: 1,
                    health: 1,
                })),
                lvl: 1,
                effect: None,
                item: None,
            });
            Effect {
                target: Target::Friend,
                position: Position::None,
                effect: EffectAction::Summon(Some(bee)),
                uses: None,
                effect_type: EffectType::Food,
                trigger: EffectTrigger::Friend(Outcome {
                    action: Action::Faint,
                    position: Some(Position::Specific(0)),
                }),
            }
        }
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
            uses: Some(Rc::new(RefCell::new(1))),
            effect_type: EffectType::Food,
            trigger: EffectTrigger::None,
        },
        FoodName::Mushroom => Effect {
            target: Target::Friend,
            position: Position::None,
            // Replace during runtime.
            effect: EffectAction::Summon(None),
            uses: Some(Rc::new(RefCell::new(1))),
            effect_type: EffectType::Food,
            trigger: EffectTrigger::Friend(Outcome {
                action: Action::Faint,
                position: Some(Position::Specific(0)),
            }),
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
            uses: Some(Rc::new(RefCell::new(1))),
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
