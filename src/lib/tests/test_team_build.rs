use std::rc::Rc;

use itertools::Itertools;

use crate::{
    effects::state::{Status, Target},
    pets::{names::PetName, pet::Pet},
    teams::{
        combat::TeamCombat,
        team::{Team, TeamFightOutcome},
        viewer::TeamViewer,
    },
    Position, Statistics, TeamEffects,
};

use super::common::test_ant_team;

#[test]
fn test_create_team_standard_size() {
    let team = Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
        ],
        5,
    );

    assert!(team.is_ok())
}

#[test]
fn test_create_team_large_size() {
    let team = Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
        ],
        10,
    );

    assert!(team.is_ok())
}

#[test]
fn test_create_team_invalid_size() {
    let team = Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
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
        assert!(
            restored_pet.as_ref().unwrap().borrow().stats
                == original_pet.as_ref().unwrap().borrow().stats
        );
        for (effect_restored, effect_original) in restored_pet
            .as_ref()
            .unwrap()
            .borrow()
            .effect
            .iter()
            .zip_eq(original_pet.as_ref().unwrap().borrow().effect.iter())
        {
            assert!(effect_restored.uses == effect_original.uses)
        }
    }
}

#[test]
fn test_team_push() {
    let mut team = Team::new(
        &[
            Some(Pet::try_from(PetName::Snake).unwrap()),
            Some(Pet::try_from(PetName::Hippo).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
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
    let dog = Rc::downgrade(team.friends.get(0).unwrap().as_ref().unwrap());
    let snake = Rc::downgrade(&team.friends.get(2).unwrap().as_ref().unwrap());

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
fn test_team_add_pet() {
    // Create a starting team with max size of 3.
    let mut team = Team::new(
        &[
            Some(Pet::try_from(PetName::Snake).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
        ],
        3,
    )
    .unwrap();

    // Add pet at end.
    team.add_pet(Pet::try_from(PetName::Ant).unwrap(), 2, None)
        .unwrap();
    // Attmepting to add again is overflow of pets.
    assert!(team
        .add_pet(Pet::try_from(PetName::Ant).unwrap(), 3, None)
        .is_err());
    // This is counted as a fainted pet.
    assert_eq!(team.fainted.len(), 1);
    assert_eq!(team.friends.len(), 3);
    // Extend size of team.
    team.max_size = 8;

    assert_eq!(team.friends.iter().filter(|slot| slot.is_none()).count(), 0);
    // Adding pets within bounds of team's max size will extend empty slots as well as add the pet.
    team.add_pet(Pet::try_from(PetName::Ant).unwrap(), 6, None)
        .unwrap();

    // Three empty slots created.
    assert_eq!(team.friends.iter().filter(|slot| slot.is_none()).count(), 3);

    // Adding a pet to a pos with another pet with the same name will not merge them.
    // Will append after the pos.
    {
        let fourth_pos = team.friends.get(3).unwrap();
        assert!(fourth_pos.is_none());
    }
    team.add_pet(Pet::try_from(PetName::Ant).unwrap(), 2, None)
        .unwrap();
    {
        // Pet now at position.
        let fourth_pos = team.friends.get(3).unwrap();
        assert!(fourth_pos.is_some());
    }

    // Adding a pet to a position greater than the max size will error.
    assert!(team
        .add_pet(Pet::try_from(PetName::Ant).unwrap(), 9, None)
        .is_err())
}

#[test]
fn test_clear_team() {
    let pet = Some(Pet::try_from(PetName::Ant).unwrap());
    let mut dead_pet = pet.clone();
    dead_pet.as_mut().unwrap().stats.health = 0;

    let mut empty_pos_front_team =
        Team::new(&[None, pet.clone(), None, pet.clone(), None], 5).unwrap();
    empty_pos_front_team.clear_team();
    assert!(
        empty_pos_front_team
            .friends
            .get(0)
            .unwrap()
            .as_ref()
            .unwrap()
            .borrow()
            .pos
            == Some(0)
            && empty_pos_front_team
                .friends
                .get(2)
                .unwrap()
                .as_ref()
                .unwrap()
                .borrow()
                .pos
                == Some(2)
            && empty_pos_front_team.friends.len() == 3
    );

    let mut dead_pet_second_pos_team =
        Team::new(&[None, dead_pet.clone(), None, pet.clone(), None], 5).unwrap();
    dead_pet_second_pos_team.clear_team();
    assert!(
        dead_pet_second_pos_team
            .friends
            .get(0)
            .unwrap()
            .as_ref()
            .unwrap()
            .borrow()
            .pos
            == Some(0)
            && dead_pet_second_pos_team.friends.len() == 1
    );

    let mut all_dead_pets_team =
        Team::new(&[None, dead_pet.clone(), None, dead_pet.clone(), None], 5).unwrap();
    all_dead_pets_team.clear_team();
    assert!(all_dead_pets_team.friends.len() == 0);

    let mut empty_team = Team::new(&[None, None, None, None, None], 5).unwrap();
    empty_team.clear_team();
    assert!(empty_team.friends.len() == 0);
}

#[test]
fn test_get_nearest_pet() {
    let pet = Some(Pet::try_from(PetName::Ant).unwrap());

    let team = Team::new(&[pet.clone(), pet.clone(), None, pet.clone(), None], 5).unwrap();
    {
        let pets = team
            .get_pets_by_pos(
                team.first(),
                &Target::Friend,
                &Position::Nearest(-2),
                None,
                None,
            )
            .unwrap();
        assert_eq!(pets.len(), 2)
    }

    {
        let enemy_team = team.clone();
        let pets = team
            .get_pets_by_pos(
                team.first(),
                &Target::Either,
                &Position::Nearest(3),
                None,
                Some(&enemy_team),
            )
            .unwrap();
        assert_eq!(pets.len(), 3)
    }
}

#[test]
fn test_start_battle_order() {
    // log4rs::init_config(build_log_config()).unwrap();
    fn create_caterpillar_example(caterpillar_stats: Statistics) -> (Team, Team) {
        let dolphin = Pet::try_from(PetName::Dolphin).unwrap();
        let mut caterpillar = Pet::try_from(PetName::Caterpillar).unwrap();
        // Caterpillar attack (?, ?) determines if dolphin (4,3) hits before caterpillar transforms.
        caterpillar.stats = caterpillar_stats;
        caterpillar.set_level(3).unwrap();

        let team = Team::new(&[Some(caterpillar)], 5).unwrap();
        let enemy_team = Team::new(&[Some(dolphin)], 5).unwrap();
        (team, enemy_team)
    }

    {
        // Caterpillar has lower attack than dolphin.
        // Transforms after dolphin snipe.
        let (mut team, mut enemy_team) = create_caterpillar_example(Statistics {
            attack: 3,
            health: 4,
        });
        team.trigger_start_battle_effects(&mut enemy_team).unwrap();
        // Caterpillar survives transforming into unhurt butterfly.
        let butterfly = team.first().unwrap();
        assert!(
            butterfly.borrow().stats
                == Statistics {
                    attack: 1,
                    health: 1
                }
                && butterfly.borrow().name == PetName::Butterfly
        )
    }

    {
        // Caterpillar has higher attack than dolphin
        // Transforms before dolphin snipe
        let (mut team, mut enemy_team) = create_caterpillar_example(Statistics {
            attack: 5,
            health: 4,
        });
        team.trigger_start_battle_effects(&mut enemy_team).unwrap();
        // Butterfly faints.
        assert!(team.first().is_none());
        let butterfly = team.friends.first().unwrap().as_ref().unwrap();
        assert!(
            butterfly.borrow().stats
                == Statistics {
                    attack: 1,
                    health: 0
                }
                && butterfly.borrow().name == PetName::Butterfly
        )
    }
}
