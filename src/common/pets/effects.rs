use std::str::FromStr;

use crate::{
    common::{
        battle::{
            effect::{Effect, EffectType},
            state::{Action, Condition, CopyAttr, Position, Statistics, Target},
            trigger::*,
        },
        foods::{food::Food, names::FoodName},
        pets::{names::PetName, pet::Pet},
    },
    db::{setup::get_connection, utils::map_row_to_pet},
};

#[allow(dead_code)]
/// Maps a pet to its effects.
pub fn get_pet_effect(
    pet: &PetName,
    pet_stats: &Statistics,
    effect_stats: Statistics,
    lvl: usize,
    n_triggers: usize,
) -> Option<Effect> {
    match pet {
        PetName::Ant => Some(Effect {
            trigger: TRIGGER_SELF_FAINT,
            target: Target::Friend,
            position: Position::Any,
            action: Action::Add(effect_stats),
            uses: Some(n_triggers),
            effect_type: EffectType::Pet,
        }),
        PetName::Mosquito => Some(Effect {
            trigger: TRIGGER_START_BATTLE,
            target: Target::Enemy,
            position: Position::Any,
            action: Action::Remove(effect_stats),
            uses: Some(n_triggers),
            effect_type: EffectType::Pet,
        }),
        PetName::Cricket => {
            let zombie_cricket = Box::new(Pet {
                name: PetName::ZombieCricket,
                tier: 1,
                stats: effect_stats,
                lvl,
                effect: None,
                item: None,
                pos: None,
            });
            Some(Effect {
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Summon(Some(zombie_cricket)),
                uses: Some(n_triggers),
                effect_type: EffectType::Pet,
            })
        }
        PetName::Horse => Some(Effect {
            trigger: TRIGGER_ANY_SUMMON,
            target: Target::Friend,
            position: Position::Trigger,
            action: Action::Add(effect_stats),
            uses: Some(n_triggers),
            effect_type: EffectType::Pet,
        }),
        PetName::Crab => Some(Effect {
            trigger: TRIGGER_START_BATTLE,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Copy(
                CopyAttr::PercentStats(effect_stats),
                Position::Condition(Condition::Healthiest),
            ),
            uses: Some(n_triggers),
            effect_type: EffectType::Pet,
        }),
        PetName::Dodo => {
            let mut add_stats = pet_stats.clone();
            add_stats.mult(&effect_stats);

            Some(Effect {
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Specific(1),
                action: Action::Add(add_stats),
                uses: Some(n_triggers),
                effect_type: EffectType::Pet,
            })
        }
        PetName::Elephant => Some(Effect {
            trigger: TRIGGER_SELF_ATTACK,
            target: Target::Friend,
            position: Position::Specific(-1),
            action: Action::Remove(effect_stats),
            uses: Some(n_triggers),
            effect_type: EffectType::Pet,
        }),
        PetName::Flamingo => Some(Effect {
            trigger: TRIGGER_SELF_FAINT,
            target: Target::Friend,
            position: Position::Range(-2..=-1),
            action: Action::Add(effect_stats),
            uses: Some(n_triggers),
            effect_type: EffectType::Pet,
        }),
        PetName::Hedgehog => Some(Effect {
            trigger: TRIGGER_SELF_FAINT,
            target: Target::Either,
            position: Position::All,
            action: Action::Remove(effect_stats),
            uses: Some(n_triggers),
            effect_type: EffectType::Pet,
        }),
        PetName::Peacock => Some(Effect {
            trigger: TRIGGER_SELF_HURT,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(effect_stats),
            uses: Some(n_triggers),
            effect_type: EffectType::Pet,
        }),
        PetName::Rat => {
            let dirty_rat = Box::new(Pet {
                name: PetName::DirtyRat,
                tier: 1,
                stats: effect_stats,
                lvl: 1,
                effect: None,
                item: None,
                pos: None,
            });
            Some(Effect {
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Enemy,
                position: Position::Specific(0),
                action: Action::Summon(Some(dirty_rat)),
                // Activates multiple times per trigger.
                uses: Some(lvl),
                effect_type: EffectType::Pet,
            })
        }
        PetName::Spider => {
            let conn = get_connection().expect("Can't get connection.");
            let mut stmt = conn
                .prepare("SELECT * FROM pets where lvl = ? and tier = 3 and pack = 'Turtle' ORDER BY RANDOM() LIMIT 1")
                .unwrap();
            let pet_record = stmt
                .query_row([lvl.to_string()], map_row_to_pet)
                .expect("No row found.");
            let name = PetName::from_str(&pet_record.name).expect("Can't get pet.");

            let summoned_pet = Box::new(Pet::new(name, Some(effect_stats), lvl).unwrap());
            Some(Effect {
                effect_type: EffectType::Pet,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Summon(Some(summoned_pet)),
                uses: Some(n_triggers),
            })
        }
        PetName::Badger => {
            let mut effect_dmg_stats = pet_stats.clone();
            effect_dmg_stats.mult(&effect_stats);

            Some(Effect {
                effect_type: EffectType::Pet,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Either,
                position: Position::Multiple(vec![Position::Specific(1), Position::Specific(-1)]),
                action: Action::Remove(effect_dmg_stats),
                uses: Some(n_triggers),
            })
        }
        PetName::Blowfish => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_SELF_HURT,
            target: Target::Enemy,
            position: Position::Any,
            action: Action::Remove(effect_stats),
            uses: Some(n_triggers),
        }),
        PetName::Camel => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_SELF_HURT,
            target: Target::Friend,
            position: Position::Specific(-1),
            action: Action::Add(effect_stats),
            uses: Some(n_triggers),
        }),
        PetName::Dog => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_ANY_SUMMON,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(effect_stats),
            uses: Some(n_triggers),
        }),
        PetName::Dolphin => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_START_BATTLE,
            target: Target::Enemy,
            position: Position::Condition(Condition::Illest),
            action: Action::Remove(effect_stats),
            uses: Some(n_triggers),
        }),
        PetName::Kangaroo => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_AHEAD_ATTACK,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(effect_stats),
            uses: Some(n_triggers),
        }),
        PetName::Ox => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_AHEAD_FAINT,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Multiple(vec![
                Action::Add(effect_stats),
                Action::Gain(Box::new(Food::from(FoodName::Melon))),
            ]),
            uses: Some(n_triggers),
        }),
        PetName::Sheep => {
            let ram = Box::new(Pet {
                name: PetName::Ram,
                tier: 1,
                stats: effect_stats,
                lvl: 1,
                effect: None,
                item: None,
                pos: None,
            });
            Some(Effect {
                effect_type: EffectType::Pet,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Summon(Some(ram)),
                // Multiple ways to do this with Action::Multiple as another option.
                // Hard-coded unless more regex parsing shenanigans.
                uses: Some(2),
            })
        }
        // For tiger, create new effect trigger, EffectActivated
        _ => None,
    }
}
