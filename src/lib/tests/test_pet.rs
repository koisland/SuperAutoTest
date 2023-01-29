use crate::{
    battle::{
        effect::Entity,
        state::{Action, Condition, Position, Statistics, Target},
        trigger::*,
    },
    pets::combat::AttackOutcome,
    Effect, Pet, PetCombat, PetName,
};

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
        AttackOutcome {
            friends: vec![
                TRIGGER_SELF_FAINT,
                TRIGGER_ANY_FAINT,
                TRIGGER_AHEAD_FAINT,
                TRIGGER_ANY_ENEMY_HURT
            ],
            opponents: vec![
                TRIGGER_SELF_HURT,
                TRIGGER_ANY_HURT,
                TRIGGER_KNOCKOUT,
                TRIGGER_SPEC_ENEMY_FAINT,
                TRIGGER_ANY_ENEMY_FAINT,
            ]
        }
    )
}

#[test]
fn test_create_def_pet() {
    let mut pet = Pet::from(PetName::Ant);
    pet.seed = 0;

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
            exp: 0,
            effect: vec![Effect {
                owner_idx: None,
                entity: Entity::Pet,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Any(Condition::None),
                action: Action::Add(Statistics::new(2, 1).unwrap()),
                uses: Some(1),
                temp: false
            },],
            item: None,
            pos: None,
            cost: 3,
            seed: 0
        }
    )
}

#[test]
fn test_get_effect() {
    let test_ant = Pet::from(PetName::Ant);

    assert_eq!(
        test_ant.get_effect(1).unwrap(),
        vec![Effect {
            owner_idx: None,
            entity: Entity::Pet,
            trigger: TRIGGER_SELF_FAINT,
            target: Target::Friend,
            position: Position::Any(Condition::None),
            action: Action::Add(Statistics::new(2, 1).unwrap()),
            uses: Some(1),
            temp: false
        },],
    );
    assert!(test_ant.get_effect(4).is_err());
}

#[test]
fn test_levelup() {
    let mut test_ant = Pet::from(PetName::Ant);

    // Lvl 1 effect adds (2,1)
    assert_eq!(test_ant.lvl, 1);
    if let Action::Add(stats) = &test_ant.effect.first().as_ref().unwrap().action {
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
    if let Action::Add(stats) = &test_ant.effect.first().as_ref().unwrap().action {
        assert_eq!(
            *stats,
            Statistics {
                attack: 4,
                health: 2
            }
        )
    }

    // Fails to set.
    assert!(test_ant.set_level(4).is_err())
}

#[test]
fn test_invalid_levelup() {
    let mut test_ant = Pet::from(PetName::Ant);
    assert!(test_ant.set_level(5).is_err());
}

#[test]
fn test_create_pet() {
    let mut test_ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();

    test_ant.seed = 0;

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
            effect: vec![Effect {
                owner_idx: None,
                entity: Entity::Pet,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Any(Condition::None),
                action: Action::Add(Statistics::new(2, 1).unwrap()),
                uses: Some(1),
                temp: false
            },],
            item: None,
            pos: None,
            cost: 3,
            seed: 0
        }
    )
}

#[test]
fn create_pet_token() {
    let mut test_bee = Pet::new(
        PetName::Bee,
        None,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();
    test_bee.seed = 0;

    assert_eq!(
        test_bee,
        Pet {
            id: None,
            name: PetName::Bee,
            tier: 0,
            stats: Statistics {
                attack: 50,
                health: 50,
            },
            lvl: 1,
            exp: 0,
            effect: vec![],
            item: None,
            pos: None,
            cost: 0,
            seed: 0
        }
    );
}

#[test]
fn test_set_pos() {
    let mut test_ant = Pet::from(PetName::Ant);
    test_ant.set_pos(0);
    assert!(test_ant.pos == Some(0))
}

#[test]
fn test_add_experience() {
    let mut test_ant = Pet::from(PetName::Ant);

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
