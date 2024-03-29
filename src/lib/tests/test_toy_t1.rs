use itertools::Itertools;

use crate::{
    toys::{names::ToyName, toy::Toy},
    Pet, PetName, Statistics, Team, TeamEffects, TeamShopping, TeamViewer,
};

use super::common::test_ant_team;

#[test]
fn test_toy_balloon() {
    let mut team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::Balloon).unwrap());

    let first_ant = team.first().unwrap();
    let first_ant_stats = first_ant.read().unwrap().stats;

    // First turn. Then second to break toy.
    team.open_shop().unwrap().close_shop().unwrap();
    team.open_shop().unwrap().close_shop().unwrap();

    assert_eq!(
        first_ant.read().unwrap().stats,
        first_ant_stats
            + Statistics {
                attack: 1,
                health: 1
            }
    );
}

#[test]
fn test_toy_tennis_ball() {
    let mut team = Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
        ],
        5,
    )
    .unwrap();
    team.set_seed(Some(123));

    let mut enemy_team = team.clone();

    let enemy_pets = enemy_team.all();

    assert_eq!(
        enemy_pets
            .iter()
            .map(|pet| pet.read().unwrap().stats)
            .collect_vec(),
        vec![
            Statistics {
                attack: 2,
                health: 3
            };
            2
        ]
    );

    // Manually add tennis ball toy.
    team.toys.push(Toy::try_from(ToyName::TennisBall).unwrap());

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    // Both enemy pets take one damage at start of abttle from tennis ball.
    assert_eq!(
        enemy_pets
            .iter()
            .map(|pet| pet.read().unwrap().stats)
            .collect_vec(),
        vec![
            Statistics {
                attack: 2,
                health: 2
            };
            2
        ]
    );
}
