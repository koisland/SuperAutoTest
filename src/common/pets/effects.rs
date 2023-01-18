use std::str::FromStr;

use crate::{
    common::{
        battle::{
            effect::{Effect, EffectType},
            state::{Action, Condition, CopyAttr, Position, Statistics, Target},
            trigger::*,
        },
        foods::{food::Food, names::FoodName},
        pets::{
            names::PetName,
            pet::{Pet, MAX_PET_STATS},
        },
    },
    db::{setup::get_connection, utils::map_row_to_pet},
};

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
                id: None,
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
            add_stats *= effect_stats;

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
                id: None,
                tier: 1,
                stats: effect_stats,
                lvl: 1,
                effect: None,
                item: None,
                pos: None,
            });
            let rats_summoned = vec![Action::Summon(Some(dirty_rat)); lvl];
            Some(Effect {
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Enemy,
                position: Position::OnSelf,
                action: Action::Multiple(rats_summoned),
                // Activates multiple times per trigger.
                uses: Some(n_triggers),
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

            let summoned_pet = Box::new(Pet::new(name, None, Some(effect_stats), lvl).unwrap());
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
            effect_dmg_stats *= effect_stats;

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
                id: None,
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
                action: Action::Multiple(vec![
                    Action::Summon(Some(ram.clone())),
                    Action::Summon(Some(ram)),
                ]),
                uses: Some(n_triggers),
            })
        }
        // TODO: Tier 4/5/6
        PetName::Deer => {
            let bus = Box::new(Pet {
                name: PetName::Bus,
                id: None,
                tier: 1,
                stats: effect_stats,
                lvl,
                effect: None,
                item: Some(Food::from(FoodName::Chili)),
                pos: None,
            });
            Some(Effect {
                effect_type: EffectType::Pet,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Summon(Some(bus)),
                uses: Some(n_triggers),
            })
        }
        PetName::Hippo => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_KNOCKOUT,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(effect_stats),
            uses: None,
        }),
        PetName::Parrot => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_START_TURN,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Copy(
                CopyAttr::Effect(Box::new(None), Some(lvl)),
                Position::Specific(1),
            ),
            uses: None,
        }),
        PetName::Rooster => {
            let mut chick_stats = pet_stats.clone();
            chick_stats *= effect_stats;
            chick_stats.clamp(1, MAX_PET_STATS);

            let chick = Box::new(Pet {
                name: PetName::Chick,
                id: None,
                tier: 1,
                stats: chick_stats,
                lvl,
                effect: None,
                item: None,
                pos: None,
            });
            let n_chicks = vec![Action::Summon(Some(chick)); lvl];
            Some(Effect {
                effect_type: EffectType::Pet,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Multiple(n_chicks),
                uses: Some(n_triggers),
            })
        }
        PetName::Skunk => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_START_BATTLE,
            target: Target::Enemy,
            position: Position::Condition(Condition::Healthiest),
            action: Action::Debuff(effect_stats),
            uses: Some(1),
        }),
        PetName::Turtle => {
            let max_pets_behind: isize = lvl.try_into().unwrap();
            Some(Effect {
                effect_type: EffectType::Pet,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Range(-max_pets_behind..=-1),
                action: Action::Gain(Box::new(Food::from(FoodName::Melon))),
                uses: Some(1),
            })
        }
        PetName::Whale => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_START_BATTLE,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Evolve(lvl, Position::Specific(1)),
            uses: Some(1),
        }),
        PetName::Crocodile => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_START_BATTLE,
            target: Target::Enemy,
            position: Position::Last,
            action: Action::Remove(effect_stats),
            uses: Some(lvl),
        }),
        PetName::Rhino => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_KNOCKOUT,
            target: Target::Enemy,
            position: Position::Specific(0),
            action: Action::Rhino(effect_stats),
            uses: None,
        }),
        // No shops so start of turn.
        PetName::Scorpion => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_START_TURN,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Gain(Box::new(Food::from(FoodName::Peanuts))),
            uses: None,
        }),
        PetName::Shark => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_ANY_FAINT,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(effect_stats),
            uses: None,
        }),
        PetName::Turkey => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_ANY_SUMMON,
            target: Target::Friend,
            position: Position::Trigger,
            action: Action::Add(effect_stats),
            uses: None,
        }),
        PetName::Boar => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_SELF_ATTACK,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(effect_stats),
            uses: None,
        }),
        PetName::Fly => {
            let zombie_fly = Box::new(Pet {
                id: None,
                name: PetName::ZombieFly,
                tier: 1,
                stats: effect_stats,
                lvl,
                effect: None,
                item: None,
                pos: None,
            });
            // Add exception for other zombie flies.
            Some(Effect {
                effect_type: EffectType::Pet,
                trigger: TRIGGER_ANY_FAINT,
                target: Target::Friend,
                position: Position::Trigger,
                action: Action::Summon(Some(zombie_fly)),
                uses: Some(3),
            })
        }
        PetName::Gorilla => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_SELF_HURT,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Gain(Box::new(Food::from(FoodName::Coconut))),
            uses: Some(n_triggers),
        }),
        PetName::Leopard => {
            let mut effect_dmg = pet_stats.clone();
            effect_dmg *= effect_stats;
            Some(Effect {
                effect_type: EffectType::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Any,
                action: Action::Remove(effect_dmg),
                uses: Some(lvl),
            })
        }
        PetName::Mammoth => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_SELF_FAINT,
            target: Target::Friend,
            position: Position::All,
            action: Action::Add(effect_stats),
            uses: None,
        }),
        PetName::Snake => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_AHEAD_ATTACK,
            target: Target::Enemy,
            position: Position::Any,
            action: Action::Remove(effect_stats),
            uses: None,
        }),
        PetName::Tiger => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_AHEAD_FRIEND,
            target: Target::Friend,
            position: Position::Trigger,
            action: Action::Repeat,
            uses: None,
        }),
        _ => None,
    }
}
