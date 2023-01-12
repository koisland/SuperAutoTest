use crate::common::{
    battle::{
        state::Statistics,
        team::{Battle, Team},
    },
    foods::{food::Food, names::FoodName},
    pets::names::PetName,
    tests::common::{
        test_ant_team, test_crab_team, test_dodo_team, test_mosq_team, test_solo_hedgehog_team,
        test_summon_team,
    },
};

use crate::LOG_CONFIG;

use super::common::test_elephant_peacock_team;

#[test]
fn test_battle_honey_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let pets = test_ant_team();
    let enemy_pets = test_ant_team();

    let mut team = Team::new("self", &pets);
    let mut enemy_team = Team::new("enemy", &enemy_pets);

    // Give last pet honey on first team.
    if let Some(last_pet) = &team.friends.borrow_mut()[2] {
        last_pet.borrow_mut().item = Some(Food::new(&FoodName::Honey))
    }

    let winner = team.fight(&mut enemy_team, None).unwrap().clone();

    assert_eq!(winner, team);
}

#[test]
fn test_battle_summon_team() {
    let pets = test_summon_team();
    let enemy_pets = test_summon_team();

    let mut team = Team::new("self", &pets);
    let mut enemy_team = Team::new("enemy", &enemy_pets);

    // First pets are crickets
    // Horse is 3rd pet.
    assert_eq!(team.get_next_pet().unwrap().borrow().name, PetName::Cricket);
    assert_eq!(
        enemy_team.get_next_pet().unwrap().borrow().name,
        PetName::Cricket
    );
    assert_eq!(team.get_idx_pet(2).unwrap().borrow().name, PetName::Horse);
    assert_eq!(
        enemy_team.get_idx_pet(2).unwrap().borrow().name,
        PetName::Horse
    );
    assert_eq!(
        team.get_next_pet().unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );
    assert_eq!(
        enemy_team.get_next_pet().unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );

    // After one turn.
    team.fight(&mut enemy_team, Some(1));

    // Cricket dies and zombie cricket is spawned.
    // Horse provides 1 attack.
    assert_eq!(
        team.get_next_pet().unwrap().borrow().name,
        PetName::ZombieCricket
    );
    assert_eq!(
        enemy_team.get_next_pet().unwrap().borrow().name,
        PetName::ZombieCricket
    );
    assert_eq!(
        team.get_next_pet().unwrap().borrow().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    assert_eq!(
        enemy_team.get_next_pet().unwrap().borrow().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
}

#[test]
fn test_battle_mosquito_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let pets = test_mosq_team();
    let enemy_pets = test_ant_team();

    let mut team = Team::new("self", &pets);
    let mut enemy_team = Team::new("enemy", &enemy_pets);

    let winner = team.fight(&mut enemy_team, None).unwrap().clone();

    // Mosquitoes kill any team before game starts.
    assert_eq!(winner, team);

    for pet in team.get_all_pets().iter() {
        // Mosquitoes are unhurt
        assert_eq!(
            pet.borrow().stats,
            Statistics {
                attack: 2,
                health: 2,
            }
        )
    }
}

#[test]
fn test_battle_hedgehog_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let pets = test_solo_hedgehog_team();
    let enemy = test_ant_team();

    let mut team = Team::new("self", &pets);
    let mut enemy_team = Team::new("enemy", &enemy);

    let winner = team.fight(&mut enemy_team, Some(1));

    assert!(winner.is_none())
}

#[test]
fn test_battle_elephant_peacock_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let pets = test_elephant_peacock_team();
    let enemy_pets = test_ant_team();

    let mut team = Team::new("self", &pets);
    let mut enemy_team = Team::new("enemy", &enemy_pets);

    assert_eq!(
        team.get_idx_pet(1).unwrap().borrow().stats,
        Statistics {
            attack: 2,
            health: 5
        }
    );
    team.fight(&mut enemy_team, Some(1));
    // Lvl.1 elephant deals 1 dmg once to pet at back.
    // Lvl.1 peacock gains 4 atk.
    assert_eq!(
        team.get_idx_pet(1).unwrap().borrow().stats,
        Statistics {
            attack: 6,
            health: 4
        }
    );
}

#[test]
fn test_battle_crab_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let pets = test_crab_team();
    let enemy_pets = test_ant_team();

    let mut team = Team::new("self", &pets);
    let mut enemy_team = Team::new("enemy", &enemy_pets);

    assert_eq!(
        team.get_next_pet().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 1
        }
    );
    assert_eq!(
        team.get_idx_pet(1).unwrap().borrow().stats,
        Statistics {
            attack: 2,
            health: 50
        }
    );
    team.fight(&mut enemy_team, Some(1));
    // Crab at lvl. 1 copies 25 from big ant at pos 2.
    // Gets hit for 2 dmg.
    assert_eq!(
        team.get_next_pet().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 23
        }
    );
}

#[test]
fn test_battle_dodo_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let pets = test_dodo_team();
    let enemy_pets = test_ant_team();

    let mut team = Team::new("self", &pets);
    let mut enemy_team = Team::new("enemy", &enemy_pets);

    assert_eq!(
        team.get_next_pet().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 5
        }
    );
    // Dodo atk at lvl. 1 is 3.
    // 3 * 0.33 = 1.
    assert_eq!(
        (team.get_idx_pet(1).unwrap().borrow().stats.attack as f32 * 0.33).round(),
        1.0
    );
    team.fight(&mut enemy_team, Some(1));

    assert_eq!(
        team.get_next_pet().unwrap().borrow().stats,
        Statistics {
            attack: 4,
            health: 3
        }
    );
}
