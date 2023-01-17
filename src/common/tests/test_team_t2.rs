use petgraph::dot::Dot;

use crate::common::{
    battle::state::{Statistics, TeamFightOutcome},
    pets::names::PetName,
    tests::common::{
        test_ant_team, test_crab_team, test_dodo_team, test_elephant_peacock_team,
        test_flamingo_team, test_hedgehog_team, test_rat_team, test_spider_team,
    },
};

// use crate::LOG_CONFIG;

#[test]
fn test_battle_hedgehog_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_hedgehog_team("self");
    let mut enemy_team = test_ant_team("enemy");

    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team)
    }

    assert_eq!(fight, TeamFightOutcome::Draw);
}

#[test]
fn test_battle_elephant_peacock_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_elephant_peacock_team("self");
    let mut enemy_team = test_ant_team("enemy");

    assert_eq!(
        team.get_idx_pet(1).unwrap().stats,
        Statistics {
            attack: 2,
            health: 5
        }
    );
    team.fight(&mut enemy_team);

    // Lvl.1 elephant deals 1 dmg once to pet at back.
    // Lvl.1 peacock gains 4 atk.
    assert_eq!(
        team.get_idx_pet(1).unwrap().stats,
        Statistics {
            attack: 6,
            health: 4
        }
    );
}

#[test]
fn test_battle_crab_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_crab_team("self");
    let mut enemy_team = test_ant_team("enemy");

    assert_eq!(
        team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 3,
            health: 1
        }
    );
    assert_eq!(
        team.get_idx_pet(1).unwrap().stats,
        Statistics {
            attack: 2,
            health: 50
        }
    );
    team.fight(&mut enemy_team);

    // Crab at lvl. 1 copies 25 from big ant at pos 2.
    // Gets hit for 2 dmg.
    assert_eq!(
        team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 3,
            health: 23
        }
    );
}

#[test]
fn test_battle_dodo_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_dodo_team("self");
    let mut enemy_team = test_ant_team("enemy");

    assert_eq!(
        team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
    // Dodo atk at lvl. 1 is 3.
    // 3 * 0.33 = 1.
    assert_eq!(
        (team.get_idx_pet(1).unwrap().stats.attack as f32 * 0.33).round(),
        1.0
    );
    team.fight(&mut enemy_team);

    assert_eq!(
        team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 4,
            health: 1
        }
    );
}

#[test]
fn test_battle_flamingo_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_flamingo_team("self");
    let mut enemy_team = test_ant_team("enemy");

    assert_eq!(
        team.get_idx_pet(1).unwrap().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    assert_eq!(
        team.get_idx_pet(2).unwrap().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    team.fight(&mut enemy_team);

    // Flamingo faints giving two pets behind (1, 1).
    assert_eq!(
        team.get_idx_pet(0).unwrap().stats,
        Statistics {
            attack: 3,
            health: 2
        }
    );
    assert_eq!(
        team.get_idx_pet(1).unwrap().stats,
        Statistics {
            attack: 3,
            health: 2
        }
    );
}

#[test]
fn test_battle_rat_lvl_1_team() {
    let mut team_lvl_1 = test_rat_team("self", 1);
    let mut enemy_team_lvl_1 = test_rat_team("enemy", 1);

    team_lvl_1.fight(&mut enemy_team_lvl_1);
    team_lvl_1.fight(&mut enemy_team_lvl_1);

    assert_eq!(team_lvl_1.get_next_pet().unwrap().name, PetName::DirtyRat);
    assert_eq!(
        enemy_team_lvl_1.get_next_pet().unwrap().name,
        PetName::DirtyRat
    );
}

#[test]
fn test_battle_rat_lvl_2_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();s

    let mut team_lvl_2 = test_rat_team("self_2", 2);
    let mut enemy_team_lvl_2 = test_rat_team("enemy_self_2", 2);

    // Both rats are level 2.
    assert_eq!(team_lvl_2.get_next_pet().unwrap().lvl, 2);
    assert_eq!(enemy_team_lvl_2.get_next_pet().unwrap().lvl, 2);

    team_lvl_2.fight(&mut enemy_team_lvl_2);
    team_lvl_2.fight(&mut enemy_team_lvl_2);

    // Both rats die and summon two dirty rats.
    assert_eq!(team_lvl_2.get_all_pets().len(), 2);
    assert_eq!(enemy_team_lvl_2.get_all_pets().len(), 2);

    // All pets on both teams are dirty rats.
    for team in [team_lvl_2, enemy_team_lvl_2].iter_mut() {
        for pet_name in team.get_all_pets().iter().map(|pet| pet.name.clone()) {
            assert_eq!(pet_name, PetName::DirtyRat)
        }
    }
}

#[test]
fn test_battle_spider_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_spider_team("self");
    let mut enemy_team = test_spider_team("enemy");

    team.fight(&mut enemy_team);

    // Spiders kill themselves and both spawn a random tier 3 pet from the Turtle pack.
    assert_eq!(team.get_next_pet().unwrap().tier, 3);
    assert_eq!(enemy_team.get_next_pet().unwrap().tier, 3);
}
