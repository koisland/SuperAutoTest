use itertools::Itertools;

use crate::common::{
    battle::state::Statistics,
    foods::{food::Food, names::FoodName},
    pets::names::PetName,
    tests::common::{
        test_ant_team, test_badger_team, test_blowfish_team, test_camel_team, test_dog_team,
        test_dolphin_team, test_kangaroo_team, test_ox_team, test_sheep_team,
    },
};

// use crate::LOG_CONFIG;

#[test]
fn test_battle_badger_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_badger_team("self");
    let mut enemy_team = test_dolphin_team("enemy");

    assert_eq!(team.get_idx_pet(1).unwrap().borrow().stats.health, 5);
    // Dolphin immediately kills badger.
    // Badger's effect triggers dealing 3 dmg to both adjacent pets.
    let steps = team.fight(&mut enemy_team).collect_vec();
    let winner = &steps.last().unwrap().as_ref().unwrap().name;

    assert_eq!(winner.clone(), team.name);
    assert_eq!(team.get_next_pet().unwrap().borrow().stats.health, 2)
}

#[test]
fn test_battle_blowfish_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_blowfish_team("self");
    let mut enemy_team = test_ant_team("enemy");

    assert_eq!(team.get_idx_pet(1).unwrap().borrow().stats.health, 5);

    team.fight(&mut enemy_team).next();

    // One pet dies to blowfish indirect attack.
    // Another dies to elephant attack.
    assert_eq!(enemy_team.get_all_pets().len(), 1);
    // Blowfish takes 1 dmg.
    assert_eq!(team.get_idx_pet(1).unwrap().borrow().stats.health, 4);
}

#[test]
fn test_battle_camel_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_camel_team("self");
    let mut enemy_team = test_ant_team("enemy");

    assert_eq!(team.get_idx_pet(1).unwrap().borrow().stats.health, 6);
    // Ant has 1 health.
    assert_eq!(team.get_idx_pet(2).unwrap().borrow().stats.health, 1);

    team.fight(&mut enemy_team).next();

    // Camel takes 1 dmg from elephant.
    assert_eq!(team.get_idx_pet(1).unwrap().borrow().stats.health, 5);
    // And gives ant 2 hp.
    assert_eq!(team.get_idx_pet(2).unwrap().borrow().stats.health, 3);
}

#[test]
fn test_battle_dog_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_dog_team("self");
    let mut enemy_team = test_ant_team("enemy");

    assert_eq!(team.get_idx_pet(0).unwrap().borrow().name, PetName::Cricket);
    assert_eq!(
        team.get_idx_pet(1).unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    team.fight(&mut enemy_team).next();

    assert_eq!(
        team.get_idx_pet(0).unwrap().borrow().name,
        PetName::ZombieCricket
    );
    // Dog gains (1,1) after Zombie Cricket spawns.
    assert_eq!(
        team.get_idx_pet(1).unwrap().borrow().stats,
        Statistics {
            attack: 4,
            health: 5
        }
    );
}

#[test]
fn test_battle_kangaroo_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_kangaroo_team("self");
    let mut enemy_team = test_ant_team("enemy");

    assert_eq!(
        team.get_idx_pet(1).unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
    team.fight(&mut enemy_team).next();

    // Friend ahead attacks once increasing stats by (2,2)
    assert_eq!(
        team.get_idx_pet(1).unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
}

#[test]
fn test_battle_ox_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_ox_team("self");
    let mut enemy_team = test_ant_team("enemy");

    let ox = team.get_idx_pet(1).unwrap();
    // No item on default lvl.1 ox.
    assert!(ox.borrow().item.is_none());
    assert_eq!(
        ox.borrow().stats,
        Statistics {
            attack: 1,
            health: 3
        }
    );

    team.fight(&mut enemy_team).next();
    team.fight(&mut enemy_team).next();

    // Gets melon armor.
    assert_eq!(ox.borrow().item, Some(Food::new(&FoodName::Melon).unwrap()));
    // And an extra attack.
    assert_eq!(
        ox.borrow().stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );
}

#[test]
fn test_battle_sheep_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_sheep_team("self");
    let mut enemy_team = test_sheep_team("enemy");

    assert_eq!(team.get_all_pets().len(), 1);
    // Sheep faint and summon two ram.
    team.fight(&mut enemy_team).next();

    for team in [team, enemy_team].iter() {
        let pets = team.get_all_pets();

        assert_eq!(pets.len(), 2);

        for pet in pets.iter() {
            assert_eq!(pet.borrow().name, PetName::Ram)
        }
    }
}
