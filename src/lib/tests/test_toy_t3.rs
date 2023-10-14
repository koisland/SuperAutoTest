use crate::{
    tests::common::test_ant_team, Entity, EntityName, FoodName, ItemCondition, ShopItemViewer,
    ShopViewer, TeamEffects, TeamShopping, TeamViewer, Toy, ToyName,
};

#[test]
fn test_toy_oven_mitts() {
    let mut team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::OvenMitts).unwrap());

    // First turn. Then second to break toy.
    team.open_shop().unwrap().close_shop().unwrap();
    team.open_shop().unwrap();

    let foods = team
        .shop
        .get_shop_items_by_cond(&ItemCondition::None, &Entity::Food)
        .unwrap();
    let last_food = foods.last().unwrap();
    assert_eq!(last_food.name(), EntityName::Food(FoodName::Lasagna))
}

#[test]
fn test_toy_toilet_paper() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    let enemy_pets = enemy_team.all();
    assert!(enemy_pets
        .iter()
        .all(|pet| pet.read().unwrap().item.is_none()));

    team.toys.push(Toy::new(ToyName::ToiletPaper, 2).unwrap());
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    for pet in enemy_pets.get(0..2).unwrap() {
        assert_eq!(
            pet.read().unwrap().item.as_ref().unwrap().name,
            FoodName::Weak
        )
    }
}
