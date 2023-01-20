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

/// Maps a `PetName` to its effects.
///
/// Given `Statistic`s, level and number of effect triggers are factored in.
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
                action: Action::Summon(Some(zombie_cricket), None),
                uses: Some(n_triggers),
                effect_type: EffectType::Pet,
            })
        }
        PetName::Horse => Some(Effect {
            trigger: TRIGGER_ANY_SUMMON,
            target: Target::Friend,
            position: Position::Trigger,
            action: Action::Add(effect_stats),
            uses: None,
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
        PetName::Elephant => {
            let n_removes = vec![Action::Remove(effect_stats); n_triggers];
            Some(Effect {
                trigger: TRIGGER_SELF_ATTACK,
                target: Target::Friend,
                position: Position::Specific(-1),
                action: Action::Multiple(n_removes),
                uses: None,
                effect_type: EffectType::Pet,
            })
        }
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
            uses: None,
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
            let rats_summoned = vec![Action::Summon(Some(dirty_rat), None); lvl];
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
                action: Action::Summon(Some(summoned_pet), None),
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
            uses: None,
        }),
        PetName::Camel => {
            let n_adds = vec![Action::Add(effect_stats); n_triggers];
            Some(Effect {
                effect_type: EffectType::Pet,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Friend,
                position: Position::Specific(-1),
                action: Action::Multiple(n_adds),
                uses: None,
            })
        }
        PetName::Dog => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_ANY_SUMMON,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(effect_stats),
            uses: None,
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
            uses: None,
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
            uses: None,
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
                    Action::Summon(Some(ram.clone()), None),
                    Action::Summon(Some(ram), None),
                ]),
                uses: Some(n_triggers),
            })
        }
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
                action: Action::Summon(Some(bus), None),
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
            let n_chicks = vec![Action::Summon(Some(chick), None); lvl];
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
            uses: Some(n_triggers),
        }),
        PetName::Turtle => {
            let max_pets_behind: isize = lvl.try_into().unwrap();
            Some(Effect {
                effect_type: EffectType::Pet,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Range(-max_pets_behind..=-1),
                action: Action::Gain(Box::new(Food::from(FoodName::Melon))),
                uses: Some(n_triggers),
            })
        }
        PetName::Whale => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_START_BATTLE,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Evolve(lvl, Position::Specific(1)),
            uses: Some(n_triggers),
        }),
        PetName::Crocodile => {
            let n_removes = vec![Action::Remove(effect_stats); n_triggers];
            Some(Effect {
                effect_type: EffectType::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Last,
                action: Action::Multiple(n_removes),
                uses: Some(1),
            })
        }
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
                action: Action::Summon(Some(zombie_fly), None),
                uses: Some(n_triggers),
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
            let n_removes = vec![Action::Remove(effect_dmg); n_triggers];
            Some(Effect {
                effect_type: EffectType::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Any,
                action: Action::Multiple(n_removes),
                uses: Some(1),
            })
        }
        PetName::Mammoth => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_SELF_FAINT,
            target: Target::Friend,
            position: Position::All,
            action: Action::Add(effect_stats),
            uses: Some(n_triggers),
        }),
        PetName::Snake => Some(Effect {
            effect_type: EffectType::Pet,
            trigger: TRIGGER_AHEAD_ATTACK,
            target: Target::Enemy,
            position: Position::Any,
            action: Action::Remove(effect_stats),
            uses: None,
        }),
        _ => None,
    }
}
