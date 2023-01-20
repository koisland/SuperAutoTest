use crate::common::{
    battle::{
        effect::{Effect, EffectType},
        state::{Action, Position, Statistics, Target},
        trigger::*,
    },
    foods::names::FoodName,
    pets::{names::PetName, pet::Pet},
};

pub fn get_food_effect(name: &FoodName, effect_stats: Statistics, uses: Option<usize>) -> Effect {
    match name {
        FoodName::Chili => Effect {
            target: Target::Enemy,
            // Next enemy relative to current pet position.
            position: Position::Specific(-1),
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
                id: None,
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
                action: Action::Summon(Some(bee), None),
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
            action: Action::Summon(
                None,
                Some(Statistics {
                    attack: 1,
                    health: 1,
                }),
            ),
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
