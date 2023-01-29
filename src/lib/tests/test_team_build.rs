use crate::{
    battle::{
        state::{Statistics, Status, TeamFightOutcome},
        team::Team,
    },
    pets::{names::PetName, pet::Pet},
};

use super::common::test_ant_team;

#[test]
fn test_create_team_standard_size() {
    let team = Team::new(
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
            Some(Pet::from(PetName::Snake)),
            Some(Pet::from(PetName::Hippo)),
        ],
        5,
    )
    .unwrap();

    team.swap_pets(0, 1).unwrap();

    assert_eq!(team.nth(0).unwrap().name, PetName::Hippo);
    assert_eq!(team.nth(1).unwrap().name, PetName::Snake)
}

#[test]
fn test_team_invalid_swap() {
    let mut team = Team::new(
        &[
            Some(Pet::from(PetName::Snake)),
            Some(Pet::from(PetName::Hippo)),
        ],
        5,
    )
    .unwrap();

    assert!(team.swap_pets(0, 3).is_err());
}

#[test]
fn test_team_push() {
    let mut team = Team::new(
        &[
            Some(Pet::from(PetName::Snake)),
            Some(Pet::from(PetName::Hippo)),
            Some(Pet::from(PetName::Dog)),
        ],
        5,
    )
    .unwrap();

    // Push pet at pos 0 (Snake) a space back to pos 1.
    team.push_pet(0, -1, None).unwrap();

    assert_eq!(team.nth(1).unwrap().name, PetName::Snake);

    // Push pet at pos 2 (Dog) two spaces forward to pos 0.
    team.push_pet(2, 2, None).unwrap();

    assert_eq!(team.nth(0).unwrap().name, PetName::Dog);

    // Two push triggers made + (Default: [StartBattle, StartTurn]).
    assert_eq!(team.triggers.len(), 4);
    // Snake
    assert!(
        team.triggers.get(2).unwrap().status == Status::Pushed
            && team.triggers.get(2).unwrap().to_idx == Some(1)
    );
    // Dog
    assert!(
        team.triggers.back().unwrap().status == Status::Pushed
            && team.triggers.back().unwrap().to_idx == Some(0)
    );
}

#[test]
fn test_team_swap_stats() {
    let mut team = Team::new(
        &[
            Some(Pet::from(PetName::Snake)),
            Some(Pet::from(PetName::Hippo)),
            None,
            None,
            Some(Pet::from(PetName::Dog)),
        ],
        5,
    )
    .unwrap();

    assert_eq!(
        team.nth(0).unwrap().stats,
        Statistics {
            attack: 6,
            health: 6
        }
    );
    assert_eq!(
        team.nth(1).unwrap().stats,
        Statistics {
            attack: 4,
            health: 5
        }
    );
    team.swap_pet_stats(0, 1).unwrap();

    assert_eq!(
        team.nth(0).unwrap().stats,
        Statistics {
            attack: 4,
            health: 5
        }
    );
    assert_eq!(
        team.nth(1).unwrap().stats,
        Statistics {
            attack: 6,
            health: 6
        }
    );

    assert_eq!(
        team.nth(4).unwrap().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    team.swap_pet_stats(1, 4).unwrap();

    assert_eq!(
        team.nth(1).unwrap().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    assert_eq!(
        team.nth(4).unwrap().stats,
        Statistics {
            attack: 6,
            health: 6
        }
    );
}
