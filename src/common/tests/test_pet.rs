use crate::common::{
    battle::{
        state::{Action, Statistics},
        trigger::*,
    },
    foods::{food::Food, names::FoodName},
    pets::{
        combat::{BattleOutcome, Combat},
        effects::get_pet_effect,
        names::PetName,
        pet::Pet,
    },
};
use std::collections::VecDeque;

#[test]
fn test_attack_pet() {
    let mut ant_t1 = Pet::from(PetName::Ant);
    let mut ant_t2 = Pet::from(PetName::Ant);

    // Set 2nd ant health to survive attack.
    ant_t2.stats.health = 3;

    let outcome = ant_t1.attack(&mut ant_t2);

    assert!(ant_t1.stats.health == 0 && ant_t2.stats.health == 1);
    // Note stat_diff and idx not checked.
    assert_eq!(
        outcome,
        BattleOutcome {
            friends: VecDeque::from_iter([
                TRIGGER_SELF_FAINT,
                TRIGGER_ANY_FAINT,
                TRIGGER_AHEAD_FAINT,
                TRIGGER_ANY_ENEMY_HURT
            ]),
            opponents: VecDeque::from_iter([
                TRIGGER_SELF_HURT,
                TRIGGER_ANY_HURT,
                TRIGGER_KNOCKOUT,
                TRIGGER_SPEC_ENEMY_FAINT,
                TRIGGER_ANY_ENEMY_FAINT,
            ])
        }
    )
}

#[test]
fn test_create_def_pet() {
    let pet = Pet::from(PetName::Ant);

    assert_eq!(
        pet,
        Pet {
            id: Some("Ant".to_string()),
            name: PetName::Ant,
            tier: 1,
            stats: Statistics {
                attack: 2,
                health: 1,
            },
            lvl: 1,
            effect: get_pet_effect(
                &PetName::Ant,
                &Statistics {
                    attack: 2,
                    health: 1,
                },
                Statistics {
                    attack: 2,
                    health: 1,
                },
                1,
                1
            ),
            item: None,
            pos: None
        }
    )
}

#[test]
fn test_levelup() {
    let mut test_ant = Pet::from(PetName::Ant);

    // Lvl 1 effect adds (2,1)
    assert_eq!(test_ant.lvl, 1);
    if let Action::Add(stats) = &test_ant.effect.as_ref().unwrap().action {
        assert_eq!(
            *stats,
            Statistics {
                attack: 2,
                health: 1
            }
        )
    }

    test_ant.set_level(2).unwrap();

    // Lvl 2 effect adds (4,2)
    assert_eq!(test_ant.lvl, 2);
    if let Action::Add(stats) = &test_ant.effect.as_ref().unwrap().action {
        assert_eq!(
            *stats,
            Statistics {
                attack: 4,
                health: 2
            }
        )
    }
}

#[test]
fn test_invalid_levelup() {
    let mut test_ant = Pet::from(PetName::Ant);
    assert!(test_ant.set_level(5).is_err());
}

#[test]
fn test_create_pet() {
    let test_ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();

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
            effect: get_pet_effect(
                &PetName::Ant,
                &Statistics {
                    attack: 50,
                    health: 50,
                },
                Statistics {
                    attack: 2,
                    health: 1,
                },
                1,
                1
            ),
            item: None,
            pos: None
        }
    )
}

#[test]
fn test_set_remove_item() {
    let mut test_ant = Pet::from(PetName::Ant);

    // Set item to honey.
    test_ant.set_item(Some(Food::from(FoodName::Honey)));
    assert_eq!(
        test_ant.item.as_ref().map(|item| item.name.clone()),
        Some(FoodName::Honey)
    );

    // Remove item.
    test_ant.set_item(None);
    assert_eq!(test_ant.item, None);
}

#[test]
fn test_set_pos() {
    let mut test_ant = Pet::from(PetName::Ant);
    test_ant.set_pos(0);
    assert!(test_ant.pos == Some(0))
}

// #[test]
// fn test_attack_meat() {}

// #[test]
// fn test_attack_melon() {

// }
