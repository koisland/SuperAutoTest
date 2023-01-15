use crate::common::{
    battle::{state::Statistics, team::Battle},
    foods::{food::Food, names::FoodName},
    pets::names::PetName,
    tests::common::{test_ant_team, test_cricket_horse_team, test_mosq_team},
};

use crate::LOG_CONFIG;

#[test]
fn test_battle_ant_honey_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_ant_team("self");
    let mut enemy_team = test_ant_team("enemy");

    // Give last pet honey on first team.
    let last_pet = team.friends.borrow()[2].clone().unwrap();
    last_pet
        .borrow_mut()
        .set_item(Some(Food::new(&FoodName::Honey).unwrap()));

    let winner = team.fight(&mut enemy_team, None).unwrap().clone();

    assert_eq!(winner, team);
}

#[test]
fn test_battle_cricket_horse_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_cricket_horse_team("self");
    let mut enemy_team = test_cricket_horse_team("enemy");

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
            health: 2
        }
    );
    assert_eq!(
        enemy_team.get_next_pet().unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );

    // After one turn.
    team.fight(&mut enemy_team, Some(2));

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

    let mut team = test_mosq_team("self");
    let mut enemy_team = test_ant_team("enemy");

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
