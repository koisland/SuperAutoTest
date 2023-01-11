use crate::common::{
    battle::{
        effect::{Effect, EffectType},
        state::{Action, Position, Statistics, Target},
        trigger::*,
    },
    pets::{names::PetName, pet::Pet},
};

#[allow(dead_code)]
/// Maps a pet to its effects.
pub fn get_pet_effect(
    pet: &PetName,
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
            action: Action::CopyStatsHealthiest(effect_stats),
            uses: Some(n_triggers),
            effect_type: EffectType::Pet,
        }),
        PetName::Dodo => Some(Effect {
            trigger: TRIGGER_START_BATTLE,
            target: Target::Friend,
            position: Position::Specific(1),
            action: Action::Add(effect_stats),
            uses: Some(n_triggers),
            effect_type: EffectType::Pet,
        }),
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
            target: Target::Enemy,
            position: Position::Range(-2..=-1),
            action: Action::Add(effect_stats),
            uses: None,
            effect_type: EffectType::Pet,
        }),
        PetName::Hedgehog => Some(Effect {
            trigger: TRIGGER_SELF_FAINT,
            target: Target::Enemy,
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
                stats: Statistics {
                    attack: 1,
                    health: 1,
                },
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
        PetName::Spider => None,
        _ => None,
    }
}
