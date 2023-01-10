use crate::common::{
    effect::Statistics,
    food::Food,
    foods::names::FoodName,
    pet::Pet,
    pets::names::PetName,
    team::{Battle, Team},
    tests::common::{ant, test_ant_team, test_mosq_team, test_summon_team},
};

// use crate::LOG_CONFIG;

#[test]
fn test_build_team() {
    let pets: [Option<Pet>; 5] = test_ant_team();
    let team = Team::new("test", &pets);

    assert!(team.is_ok())
}

#[test]
fn test_build_invalid_team() {
    let mut pets: Vec<Option<Pet>> = test_ant_team().into_iter().collect();

    // Make a invalid team of six pets.
    pets.push(Some(ant()));

    assert!(Team::new("test", &pets).is_err());
}

#[test]
fn test_battle_honey_team() {
    let pets = test_ant_team();
    let enemy_pets = test_ant_team();

    let mut team = Team::new("self", &pets).unwrap();
    let mut enemy_team = Team::new("enemy", &enemy_pets).unwrap();

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

    let mut team = Team::new("self", &pets).unwrap();
    let mut enemy_team = Team::new("enemy", &enemy_pets).unwrap();

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
        *team.get_next_pet().unwrap().borrow().stats.borrow(),
        Statistics {
            attack: 1,
            health: 1
        }
    );
    assert_eq!(
        *enemy_team.get_next_pet().unwrap().borrow().stats.borrow(),
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
        *team.get_next_pet().unwrap().borrow().stats.borrow(),
        Statistics {
            attack: 2,
            health: 1
        }
    );
    assert_eq!(
        *enemy_team.get_next_pet().unwrap().borrow().stats.borrow(),
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

    let mut team = Team::new("self", &pets).unwrap();
    let mut enemy_team = Team::new("enemy", &enemy_pets).unwrap();

    let winner = team.fight(&mut enemy_team, None).unwrap().clone();

    // Mosquitoes kill any team before game starts.
    assert_eq!(winner, team);

    for pet in team.get_all_pets().iter() {
        // Mosquitoes are unhurt
        assert_eq!(
            *pet.borrow().stats.borrow(),
            Statistics {
                attack: 2,
                health: 2,
            }
        )
    }
}
