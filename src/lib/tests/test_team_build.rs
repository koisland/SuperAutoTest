use std::rc::Rc;

use crate::{
    battle::{
        state::{Status, TeamFightOutcome},
        stats::Statistics,
        team::Team,
    },
    pets::{names::PetName, pet::Pet},
};

use super::common::test_ant_team;

#[test]
fn test_create_team_standard_size() {
    let team = Team::new(
        &[
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        5,
    );

    assert!(team.is_ok())
}

#[test]
fn test_create_team_large_size() {
    let team = Team::new(
        &[
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        10,
    );

    assert!(team.is_ok())
}

#[test]
fn test_create_team_invalid_size() {
    let team = Team::new(
        &[
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        3,
    );

    assert!(team.is_err())
}

#[test]
fn test_team_restore() {
    let mut team = test_ant_team();
    let original_team = team.clone();

    let mut enemy_team = test_ant_team();

    let mut outcome = team.fight(&mut enemy_team);
    while outcome == TeamFightOutcome::None {
        outcome = team.fight(&mut enemy_team);
    }

    // Teams faints.
    // Not equal to original team copy.
    assert_eq!(team.all().len(), 0);
    assert_ne!(team, original_team);

    // Restore pets on team.
    team.restore();

    // Team restored to original state.
    assert_eq!(team, original_team);
}

#[test]
fn test_team_swap() {
    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Snake).unwrap(),
            Pet::try_from(PetName::Hippo).unwrap(),
        ],
        5,
    )
    .unwrap();

    team.swap_pets(
        &mut team.nth(0).unwrap().borrow_mut(),
        &mut team.nth(1).unwrap().borrow_mut(),
    );

    assert_eq!(team.nth(0).unwrap().borrow().name, PetName::Hippo);
    assert_eq!(team.nth(1).unwrap().borrow().name, PetName::Snake)
}

#[test]
fn test_team_push() {
    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Snake).unwrap(),
            Pet::try_from(PetName::Hippo).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
        ],
        5,
    )
    .unwrap();

    // Push pet at pos 0 (Snake) a space back to pos 1.
    team.push_pet(0, -1, None).unwrap();

    assert_eq!(team.nth(1).unwrap().borrow().name, PetName::Snake);

    // Push pet at pos 2 (Dog) two spaces forward to pos 0.
    team.push_pet(2, 2, None).unwrap();

    assert_eq!(team.nth(0).unwrap().borrow().name, PetName::Dog);

    // Two push triggers made + (Default: [StartBattle, StartTurn]).
    assert_eq!(team.triggers.len(), 4);
    let push_trigger_snake = team.triggers.get(2).unwrap();
    let push_trigger_dog = team.triggers.back().unwrap();

    // Get weak references to pets.
    let dog = Rc::downgrade(&team.friends.get(0).unwrap());
    let snake = Rc::downgrade(&team.friends.get(2).unwrap());

    // Snake
    assert!(
        push_trigger_snake.status == Status::Pushed
            && push_trigger_snake
                .affected_pet
                .as_ref()
                .unwrap()
                .ptr_eq(&snake)
    );
    // Dog
    assert!(
        push_trigger_dog.status == Status::Pushed
            && push_trigger_dog.affected_pet.as_ref().unwrap().ptr_eq(&dog)
    );
}

#[test]
fn test_team_swap_stats() {
    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Snake).unwrap(),
            Pet::try_from(PetName::Hippo).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
        ],
        5,
    )
    .unwrap();

    assert_eq!(
        team.nth(0).unwrap().borrow().stats,
        Statistics {
            attack: 6,
            health: 6
        }
    );
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 4,
            health: 5
        }
    );
    team.swap_pet_stats(
        &mut team.nth(0).unwrap().borrow_mut(),
        &mut team.nth(1).unwrap().borrow_mut(),
    )
    .unwrap();

    assert_eq!(
        team.nth(0).unwrap().borrow().stats,
        Statistics {
            attack: 4,
            health: 5
        }
    );
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 6,
            health: 6
        }
    );

    assert_eq!(
        team.nth(2).unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    team.swap_pet_stats(
        &mut team.nth(1).unwrap().borrow_mut(),
        &mut team.nth(2).unwrap().borrow_mut(),
    )
    .unwrap();

    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    assert_eq!(
        team.nth(2).unwrap().borrow().stats,
        Statistics {
            attack: 6,
            health: 6
        }
    );
}