use std::rc::Rc;

use itertools::Itertools;

use crate::{
    battle::{
        team::{Team, TeamFightOutcome},
        team_viewer::TeamViewer,
    },
    effects::state::Status,
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

    let mut outcome = team.fight(&mut enemy_team).unwrap();
    while outcome == TeamFightOutcome::None {
        outcome = team.fight(&mut enemy_team).unwrap();
    }

    // Teams faints.
    // Not equal to original team copy.
    assert_eq!(team.all().len(), 0);
    assert_ne!(team, original_team);

    // Restore pets on team.
    team.restore();

    // Team restored to original state.
    // Note references will not be equivalent.
    assert_ne!(team, original_team);

    // But stats, starting triggers, and effect uses are equivalent.
    assert_eq!(team.triggers, original_team.triggers);

    for (restored_pet, original_pet) in team.friends.iter().zip_eq(original_team.friends.iter()) {
        assert!(restored_pet.borrow().stats == original_pet.borrow().stats);
        for (effect_restored, effect_original) in restored_pet
            .borrow()
            .effect
            .iter()
            .zip_eq(original_pet.borrow().effect.iter())
        {
            assert!(effect_restored.uses == effect_original.uses)
        }
    }
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

    // Two push triggers made.
    assert_eq!(team.triggers.len(), 2);
    let push_trigger_snake = team.triggers.front().unwrap();
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
