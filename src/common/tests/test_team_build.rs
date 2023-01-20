use crate::common::{
    battle::{
        state::{Statistics, TeamFightOutcome},
        team::Team,
    },
    pets::{names::PetName, pet::Pet},
};

use super::common::test_ant_team;

#[test]
fn test_create_team_standard_size() {
    let team = Team::new(
        "standard",
        &[
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
        ],
        5,
    );

    assert!(team.is_ok())
}

#[test]
fn test_create_team_large_size() {
    let team = Team::new(
        "big_mode",
        &[
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
        ],
        10,
    );

    assert!(team.is_ok())
}

#[test]
fn test_create_team_invalid_size() {
    let team = Team::new(
        "too_big_for_max_size",
        &[
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
        ],
        3,
    );

    assert!(team.is_err())
}

#[test]
fn test_team_restore() {
    let mut team = test_ant_team("self");
    let original_team = team.clone();

    let mut enemy_team = test_ant_team("enemy");

    let mut outcome = team.fight(&mut enemy_team);
    while outcome == TeamFightOutcome::None {
        outcome = team.fight(&mut enemy_team);
    }

    // Teams faints.
    // Not equal to original team copy.
    assert_eq!(team.get_all_pets().len(), 0);
    assert_ne!(team, original_team);

    // Restore pets on team.
    team.restore();

    // Team restored to original state.
    assert_eq!(team, original_team);
}
