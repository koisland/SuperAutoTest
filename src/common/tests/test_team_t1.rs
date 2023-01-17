use crate::common::{
    battle::state::{Statistics, TeamFightOutcome},
    foods::{food::Food, names::FoodName},
    pets::names::PetName,
    tests::common::{test_ant_team, test_cricket_horse_team, test_mosq_team},
};
// use crate::LOG_CONFIG;
// use petgraph::{dot::Dot, visit::Dfs};

#[test]
fn test_battle_ant_honey_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_ant_team("self");
    let mut enemy_team = test_ant_team("enemy");

    // Give last pet honey on first team.
    let last_pet = team.friends.get_mut(2).unwrap().as_mut().unwrap();
    last_pet.set_item(Some(Food::new(&FoodName::Honey).unwrap()));

    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team);
    }

    assert_eq!(fight, TeamFightOutcome::Win);
}

#[test]
fn test_battle_cricket_horse_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_cricket_horse_team("self");
    let mut enemy_team = test_cricket_horse_team("enemy");

    // First pets are crickets
    // Horse is 3rd pet.
    assert_eq!(team.get_next_pet().unwrap().name, PetName::Cricket);
    assert_eq!(enemy_team.get_next_pet().unwrap().name, PetName::Cricket);
    assert_eq!(team.get_idx_pet(2).unwrap().name, PetName::Horse);
    assert_eq!(enemy_team.get_idx_pet(2).unwrap().name, PetName::Horse);
    assert_eq!(
        team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
    assert_eq!(
        enemy_team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );

    // After one turn.
    team.fight(&mut enemy_team);
    team.fight(&mut enemy_team);

    // Cricket dies and zombie cricket is spawned.
    // Horse provides 1 attack.
    assert_eq!(team.get_next_pet().unwrap().name, PetName::ZombieCricket);
    assert_eq!(
        enemy_team.get_next_pet().unwrap().name,
        PetName::ZombieCricket
    );
    assert_eq!(
        team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    assert_eq!(
        enemy_team.get_next_pet().unwrap().stats,
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

    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team)
    }
    // Mosquitoes kill any team before game starts.
    assert_eq!(fight, TeamFightOutcome::Win);
    assert_eq!(team.friends.len(), 3);

    for pet in team.get_all_pets().iter() {
        // Mosquitoes are unhurt
        assert_eq!(
            pet.stats,
            Statistics {
                attack: 2,
                health: 2,
            }
        )
    }
}
