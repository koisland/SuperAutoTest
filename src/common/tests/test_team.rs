use crate::common::{
    food::Food,
    foods::names::FoodName,
    pet::Pet,
    team::{Battle, Team},
    tests::common::{ant, test_summon_team, test_team},
};

// use crate::LOG_CONFIG;

#[test]
fn test_build_team() {
    let pets: [Option<Pet>; 5] = test_team();
    let team = Team::new("test", &pets);

    assert!(team.is_ok())
}

#[test]
fn test_build_invalid_team() {
    let mut pets: Vec<Option<Pet>> = test_team().into_iter().collect();

    // Make a invalid team of six pets.
    pets.push(Some(ant()));

    assert!(Team::new("test", &pets).is_err());
}

#[test]
fn test_battle_team() {
    // // Logger for debugging.
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let pets = test_team();
    let enemy_pets = test_team();

    let mut team = Team::new("self", &pets).unwrap();
    let mut enemy_team = Team::new("enemy", &enemy_pets).unwrap();

    // Give last pet honey.
    if let Some(last_pet) = &team.friends.borrow_mut()[2] {
        last_pet.borrow_mut().item = Some(Food::new(&FoodName::Honey))
    }

    let winner = team.fight(&mut enemy_team, None).unwrap().clone();

    assert_eq!(winner, team);
}

// #[test]
// fn test_battle_summon_team() {
//     // // Logger for debugging.
//     // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

//     let pets = test_summon_team();
//     let enemy_pets = test_summon_team();

//     let mut team = Team::new("self", &pets).unwrap();
//     let mut enemy_team = Team::new("enemy", &enemy_pets).unwrap();

//     let winner = team.fight(&mut enemy_team, None).unwrap().clone();

//     println!("{:?}", winner)
// }
