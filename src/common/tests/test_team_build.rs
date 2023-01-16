use crate::common::{
    battle::team::Team,
    pets::{names::PetName, pet::Pet},
};

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
