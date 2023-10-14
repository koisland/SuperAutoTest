use std::iter::zip;

use itertools::Itertools;

use crate::{
    tests::common::test_ant_team, FoodName, Statistics, TeamEffects, TeamShopping, TeamViewer, Toy,
    ToyName,
};

#[test]
fn test_toy_television() {
    let mut team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::Television).unwrap());

    let ants = team.all();
    let ant_stats = ants
        .iter()
        .map(|ant| ant.read().unwrap().stats)
        .collect_vec();

    // First turn. Then second to break toy.
    team.open_shop().unwrap().close_shop().unwrap();
    team.open_shop().unwrap();

    for (ant, original_stats) in zip(ants, ant_stats) {
        assert_eq!(
            original_stats
                + Statistics {
                    attack: 2,
                    health: 2
                },
            ant.read().unwrap().stats
        )
    }
}

#[test]
fn test_toy_peanut_jar() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    let first_ant = team.first().unwrap();

    team.toys.push(Toy::new(ToyName::PeanutJar, 1).unwrap());
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    assert_eq!(
        first_ant.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Peanut
    );
}

#[test]
fn test_toy_air_palm_tree() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    let first_ant = team.first().unwrap();

    team.toys.push(Toy::new(ToyName::AirPalmTree, 1).unwrap());
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    assert_eq!(
        first_ant.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Coconut
    );
}
