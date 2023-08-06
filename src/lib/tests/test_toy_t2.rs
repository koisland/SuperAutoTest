use itertools::Itertools;

use crate::{
    tests::common::test_ant_team, FoodName, Statistics, TeamCombat, TeamEffects, TeamShopping,
    TeamViewer, Toy, ToyName,
};

#[test]
fn test_toy_radio() {
    let mut team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::Radio).unwrap());

    let ants = team.all();
    let exp_ants_stats = ants
        .iter()
        .map(|ant| {
            ant.read().unwrap().stats
                + Statistics {
                    attack: 0,
                    health: 1,
                }
        })
        .collect_vec();

    // First turn. Then second to break toy.
    team.open_shop().unwrap().close_shop().unwrap();
    team.open_shop().unwrap().close_shop().unwrap();

    assert_eq!(
        ants.iter()
            .map(|ant| ant.read().unwrap().stats)
            .collect_vec(),
        exp_ants_stats
    );
}

#[test]
fn test_toy_garlic_press() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    let first_ant = team.first().unwrap();
    let second_ant = team.nth(1).unwrap();

    team.toys.push(Toy::new(ToyName::GarlicPress, 1).unwrap());
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();
    assert_eq!(
        first_ant.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Garlic
    );
    assert!(second_ant.read().unwrap().item.is_none());
    team.restore();

    team.toys.push(Toy::new(ToyName::GarlicPress, 2).unwrap());
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();
    assert_eq!(
        first_ant.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Garlic
    );
    assert_eq!(
        second_ant.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Garlic
    );
}
