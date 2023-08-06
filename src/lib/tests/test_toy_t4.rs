use crate::{
    tests::common::{test_ant_team, test_scorpion_team},
    FoodName, Pet, PetName, TeamEffects, TeamShopping, TeamViewer, Toy, ToyName,
};

#[test]
fn test_toy_melon_helmet() {
    let mut team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::MelonHelmet).unwrap());

    let first_ant = team.first().unwrap();
    let second_ant = team.nth(1).unwrap();

    // First turn. Then second to break toy.
    team.open_shop().unwrap().close_shop().unwrap();
    team.open_shop().unwrap();

    assert_eq!(
        first_ant.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Melon
    );
    assert!(second_ant.read().unwrap().item.is_none());
}

#[test]
fn test_toy_foam_sword() {
    let mut team = test_ant_team();
    let mut enemy_team = test_scorpion_team();
    // Add a pet stronger than scorpion.
    enemy_team
        .add_pet(Pet::try_from(PetName::Dog).unwrap(), 0, None)
        .unwrap();

    let scorpion = enemy_team.nth(1).unwrap();
    assert_eq!(scorpion.read().unwrap().stats.health, 1);
    team.toys.push(Toy::new(ToyName::FoamSword, 1).unwrap());
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    assert_eq!(scorpion.read().unwrap().stats.health, 0);
}

#[test]
fn test_toy_toy_gun() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    let last_ant = enemy_team.last().unwrap();
    assert!(last_ant.read().unwrap().stats.health != 0);
    team.toys.push(Toy::new(ToyName::ToyGun, 1).unwrap());
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    assert_eq!(last_ant.read().unwrap().stats.health, 0);
}
