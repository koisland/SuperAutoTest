use crate::common::{
    effect::Statistics, food::Food, foods::names::FoodName, pet::Pet, pets::names::PetName,
    team::Battle, team::Team,
};

use crate::LOG_CONFIG;

fn test_team() -> [Option<Pet>; 5] {
    let pet_1 = Pet::new(
        PetName::Ant,
        Statistics {
            attack: 2,
            health: 1,
        },
        1,
        None,
    )
    .unwrap();
    let pet_2 = Pet::new(
        PetName::Ant,
        Statistics {
            attack: 2,
            health: 1,
        },
        1,
        None,
    )
    .unwrap();
    let pet_3 = Pet::new(
        PetName::Ant,
        Statistics {
            attack: 2,
            health: 1,
        },
        1,
        None,
    )
    .unwrap();

    [Some(pet_1), Some(pet_2), Some(pet_3), None, None]
}

#[test]
fn test_build_team() {
    let pets = test_team();
    let team = Team::new("test", &pets);
    println!("{:?}", team)
}

#[test]
fn test_battle_team() {
    log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let pets = test_team();
    let enemy_pets = test_team();

    let mut team = Team::new("self", &pets).unwrap();
    let mut enemy_team = Team::new("enemy", &enemy_pets).unwrap();
    // Give last pet honey.
    if let Some(last_pet) = &team.friends.borrow_mut()[2] {
        last_pet.borrow_mut().item = Some(Food::new(&FoodName::Honey))
    }

    team.fight(&mut enemy_team);

    println!("{:?}", team);
    println!("{:?}", enemy_team);
}
