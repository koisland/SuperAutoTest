use std::collections::VecDeque;

use crate::{
    effects::{
        actions::{Action, StatChangeType},
        state::{ItemCondition, Position, Target},
        stats::Statistics,
        trigger::*,
    },
    pets::combat::AttackOutcome,
    Effect, Pet, PetCombat, PetName,
};

#[test]
fn test_attack_pet() {
    let mut ant_t1 = Pet::new(
        PetName::Ant,
        Some(Statistics {
            attack: 2,
            health: 1,
        }),
        1,
    )
    .unwrap();
    let mut ant_t2 = Pet::new(
        PetName::Ant,
        Some(Statistics {
            attack: 2,
            health: 1,
        }),
        1,
    )
    .unwrap();

    // Set 2nd ant health to survive attack.
    ant_t2.stats.health = 3;

    let outcome = ant_t1.attack(&mut ant_t2);

    assert!(ant_t1.stats.health == 0 && ant_t2.stats.health == 1);
    // Note stat_diff and idx not checked.
    assert_eq!(
        outcome,
        AttackOutcome {
            friends: VecDeque::from_iter([
                TRIGGER_SELF_ATTACK,
                TRIGGER_SELF_FAINT,
                TRIGGER_ANY_FAINT,
                TRIGGER_AHEAD_FAINT,
                TRIGGER_ANY_ENEMY_HURT
            ]),
            opponents: VecDeque::from_iter([
                TRIGGER_KNOCKOUT,
                TRIGGER_SELF_ATTACK,
                TRIGGER_SELF_HURT,
                TRIGGER_ANY_HURT,
                TRIGGER_SPEC_ENEMY_FAINT,
                TRIGGER_ANY_ENEMY_FAINT,
            ]),
            friend_stat_change: Statistics {
                attack: 0,
                health: 1
            },
            enemy_stat_change: Statistics {
                attack: 0,
                health: 2
            }
        }
    )
}

#[test]
fn test_create_def_pet() {
    assert_eq!(
        Pet::try_from(PetName::Ant).unwrap(),
        Pet::new(PetName::Ant, None, 1).unwrap()
    )
}

#[test]
fn test_get_effect() {
    let test_ant = Pet::try_from(PetName::Ant).unwrap();

    assert_eq!(
        test_ant.get_effect(1).unwrap(),
        vec![Effect {
            owner: None,
            trigger: TRIGGER_SELF_FAINT,
            target: Target::Friend,
            position: Position::Any(ItemCondition::None),
            action: Action::Add(StatChangeType::Static(Statistics::new(1, 1).unwrap())),
            uses: Some(1),
            temp: false
        },],
    );
    assert!(test_ant.get_effect(4).is_err());
}

#[test]
fn test_levelup() {
    let mut test_ant = Pet::try_from(PetName::Ant).unwrap();

    // Lvl 1 effect adds (1,1)
    assert_eq!(test_ant.lvl, 1);
    if let Action::Add(StatChangeType::Static(stats)) =
        &test_ant.effect.first().as_ref().unwrap().action
    {
        assert_eq!(
            *stats,
            Statistics {
                attack: 1,
                health: 1
            }
        )
    }

    test_ant.set_level(2).unwrap();

    // Lvl 2 effect adds (2,2)
    assert_eq!(test_ant.lvl, 2);
    if let Action::Add(StatChangeType::Static(stats)) =
        &test_ant.effect.first().as_ref().unwrap().action
    {
        assert_eq!(
            *stats,
            Statistics {
                attack: 2,
                health: 2
            }
        )
    }

    // Fails to set.
    assert!(test_ant.set_level(4).is_err())
}

#[test]
fn test_invalid_levelup() {
    let mut test_ant = Pet::try_from(PetName::Ant).unwrap();
    assert!(test_ant.set_level(5).is_err());
}

#[test]
fn test_create_pet() {
    let mut test_ant = Pet::new(
        PetName::Ant,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();

    test_ant.seed = Some(0);

    assert_eq!(
        test_ant,
        Pet {
            name: PetName::Ant,
            id: None,
            tier: 1,
            stats: Statistics {
                attack: 50,
                health: 50,
            },
            lvl: 1,
            exp: 0,
            cost: 3,
            effect: vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Any(ItemCondition::None),
                action: Action::Add(StatChangeType::Static(Statistics::new(1, 1).unwrap())),
                uses: Some(1),
                temp: false
            },],
            item: None,
            pos: None,
            seed: Some(0),
            team: None
        }
    )
}

#[test]
fn create_pet_token() {
    let mut test_bee = Pet::new(
        PetName::Bee,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();
    test_bee.seed = Some(0);

    assert_eq!(
        test_bee,
        Pet {
            id: None,
            name: PetName::Bee,
            tier: 1,
            stats: Statistics {
                attack: 50,
                health: 50,
            },
            lvl: 1,
            exp: 0,
            cost: 0,
            effect: vec![],
            item: None,
            pos: None,
            seed: Some(0),
            team: None
        }
    );
}

#[test]
fn test_set_pos() {
    let mut test_ant = Pet::try_from(PetName::Ant).unwrap();
    test_ant.set_pos(0);
    assert!(test_ant.pos == Some(0))
}

#[test]
fn test_swap_pet() {
    let mut pet_1 = Pet::try_from(PetName::Gorilla).unwrap();
    let mut pet_2 = Pet::try_from(PetName::Leopard).unwrap();
    let (pet_1_copy, pet_2_copy) = (pet_1.clone(), pet_2.clone());
    pet_1.swap(&mut pet_2);

    assert_eq!(pet_1_copy, pet_2);
    assert_eq!(pet_2_copy, pet_1);
}

#[test]
fn test_merge_pets() {
    let mut pet = Pet::try_from(PetName::Gorilla).unwrap();
    let other_pet = Pet::try_from(PetName::Gorilla).unwrap();

    assert!(pet.merge(&other_pet).is_ok());
    assert_eq!(pet.stats, Statistics::new(8, 11).unwrap());
    // Gain 1 experience.
    assert_eq!(pet.exp, 1)
}

#[test]
fn test_swap_pet_stats() {
    let mut pet_1 = Pet::try_from(PetName::Gorilla).unwrap();
    let mut pet_2 = Pet::try_from(PetName::Leopard).unwrap();
    assert!(
        pet_1.stats == Statistics::new(7, 10).unwrap()
            && pet_2.stats == Statistics::new(10, 4).unwrap()
    );

    pet_1.swap_stats(&mut pet_2);
    assert!(
        pet_1.stats == Statistics::new(10, 4).unwrap()
            && pet_2.stats == Statistics::new(7, 10).unwrap()
    );
}
#[test]
fn test_add_experience() {
    let mut test_ant = Pet::try_from(PetName::Ant).unwrap();

    // Add single point.
    test_ant.add_experience(1).unwrap();
    assert!(test_ant.exp == 1 && test_ant.lvl == 1);

    // Add three points to reach level 2 an
    test_ant.add_experience(3).unwrap();
    assert!(test_ant.exp == 4 && test_ant.lvl == 2);

    // Add one point to reach level cap.
    test_ant.add_experience(1).unwrap();
    assert!(test_ant.exp == 5 && test_ant.lvl == 3);

    // Additional experience is not allowed
    assert!(test_ant.add_experience(3).is_err())
}
