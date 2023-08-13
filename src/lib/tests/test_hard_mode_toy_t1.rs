use crate::{FoodName, PetName, Statistics, TeamEffects, TeamShopping, TeamViewer, Toy, ToyName};

use super::common::{test_ant_team, test_camel_team};

#[test]
fn test_toy_boomerang() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::Boomerang).unwrap());

    let first_ant = team.first().unwrap();
    first_ant.write().unwrap().stats.health = 32;
    let starting_ant_stats = first_ant.read().unwrap().stats;

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();
    // First ant takes 30 health.
    assert_eq!(
        first_ant.read().unwrap().stats,
        starting_ant_stats
            - Statistics {
                attack: 0,
                health: 30
            }
    );
}

#[test]
fn test_toy_dice_cup() {
    let mut team = test_camel_team();
    team.set_seed(Some(1234));

    let mut enemy_team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::DiceCup).unwrap());

    let pets = team.all();
    let original_ordering: Vec<PetName> = pets
        .iter()
        .map(|pet| pet.read().unwrap().name.clone())
        .collect();

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    let new_ordering: Vec<PetName> = pets
        .iter()
        .map(|pet| pet.read().unwrap().name.clone())
        .collect();

    // Pets reordered.
    assert_eq!(
        original_ordering,
        vec![PetName::Elephant, PetName::Camel, PetName::Ant]
    );
    assert_eq!(
        new_ordering,
        vec![PetName::Elephant, PetName::Ant, PetName::Camel]
    );
}

#[test]
fn test_toy_dodgeball() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::Dodgeball).unwrap());
    println!("{team}");

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    println!("{team}");

    todo!("Need to restore uses if kills first pet.")
}

#[test]
fn test_toy_handerkerchief() {
    let mut team = test_ant_team();
    // Set shop tier to two.
    team.set_shop_tier(2).unwrap();

    let mut enemy_team = test_ant_team();

    // Add handkerchief
    team.toys
        .push(Toy::try_from(ToyName::Handkerchief).unwrap());

    let pets = team.all();

    assert!(pets.iter().all(|pet| pet.read().unwrap().item.is_none()));
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    // At shop tier of two, only first two friends get weakness.
    assert!(pets.get(0..=1).unwrap().iter().all(|pet| pet
        .read()
        .unwrap()
        .item
        .as_ref()
        .unwrap()
        .name
        == FoodName::Weak));
    assert!(pets[2].read().unwrap().item.is_none())
}
