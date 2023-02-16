use itertools::Itertools;

use crate::{
    shop::store::ShopState, Condition, Entity, Pet, PetName, Position, ShopItemViewer, ShopViewer,
    Team, TeamShopping,
};

#[test]
fn test_team_shop_opening() {
    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Beaver).unwrap(),
            Pet::try_from(PetName::Mosquito).unwrap(),
        ],
        5,
    )
    .unwrap();

    assert!(team.open_shop().is_ok());
}

#[test]
fn test_team_shop_closing() {
    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Beaver).unwrap(),
            Pet::try_from(PetName::Mosquito).unwrap(),
        ],
        5,
    )
    .unwrap();

    team.open_shop().unwrap();
    assert!(team.close_shop().is_ok());
}

#[test]
fn test_team_shop_buy() {
    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Beaver).unwrap(),
            Pet::try_from(PetName::Mosquito).unwrap(),
        ],
        5,
    )
    .unwrap();

    team.set_shop_seed(Some(1212)).open_shop().unwrap();

    let (any_pos, pet_item_type, first_pos) =
        (Position::Any(Condition::None), Entity::Pet, Position::First);
    for _ in 0..2 {
        team.buy(&any_pos, &pet_item_type, &first_pos).unwrap();
    }
    team.buy(&first_pos, &Entity::Food, &first_pos).unwrap();
    // No more money.
    assert!(team.buy(&any_pos, &pet_item_type, &first_pos).is_err());
}

#[test]
fn test_team_shop_sell() {
    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Beaver).unwrap(),
            Pet::try_from(PetName::Mosquito).unwrap(),
        ],
        5,
    )
    .unwrap();

    assert_eq!(team.shop.coins, 10);
    team.open_shop().unwrap();
    team.sell(&Position::First).unwrap();

    assert_eq!(team.shop.coins, 11);
    assert_eq!(team.all().len(), 1)
}

#[test]
fn test_team_shop_state_battle() {
    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Beaver).unwrap(),
            Pet::try_from(PetName::Mosquito).unwrap(),
        ],
        5,
    )
    .unwrap();
    let mut opponent = team.clone();

    // Cannot fight while shop is open.
    team.shop.state = ShopState::Open;
    assert!(team.fight(&mut opponent).is_err());

    team.shop.state = ShopState::Closed;
    assert!(team.fight(&mut opponent).is_ok())
}

#[test]
fn test_shop_invalid_states() {
    let mut team = Team::default();
    let mut custom_team = Team::new(&[Pet::try_from(PetName::Ant).unwrap()], 5).unwrap();

    // Default state for both team is closed.
    assert!(custom_team.shop.state == ShopState::Closed && team.shop.state == ShopState::Closed);

    // Cannot open an open shop.
    assert!(team.open_shop().unwrap().open_shop().is_err());

    // Cannot close a closed shop.
    assert!(custom_team.close_shop().is_err());

    // Cannot buy from a closed shop.
    assert!(custom_team
        .buy(&Position::First, &Entity::Pet, &Position::Last)
        .is_err());

    // Cannot buy from a closed shop.
    assert!(custom_team.sell(&Position::First).is_err());

    // Cannot freeze items in a closed shop.
    assert!(custom_team
        .freeze_shop(Position::First, Entity::Food)
        .is_err());

    // Cannot roll items in a closed shop.
    assert!(custom_team.roll_shop().is_err());
}

#[test]
fn test_team_shop_freeze() {
    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Beaver).unwrap(),
            Pet::try_from(PetName::Mosquito).unwrap(),
        ],
        5,
    )
    .unwrap();
    let mut enemy_team = Team::default();

    let original_shop_foods = team
        .shop
        .get_shop_items_by_pos(&Position::All(Condition::None), &Entity::Food)
        .unwrap()
        .into_iter()
        .cloned()
        .collect_vec();
    let original_shop_pets = team
        .shop
        .get_shop_items_by_pos(&Position::All(Condition::None), &Entity::Pet)
        .unwrap()
        .into_iter()
        .cloned()
        .collect_vec();
    // Nothing frozen.
    assert!(original_shop_foods.iter().all(|food| !food.is_frozen()));
    assert!(original_shop_pets.iter().all(|food| !food.is_frozen()));

    team.open_shop()
        .unwrap()
        .freeze_shop(Position::All(Condition::None), Entity::Pet)
        .unwrap()
        .freeze_shop(Position::All(Condition::None), Entity::Food)
        .unwrap()
        .close_shop()
        .unwrap();

    // team.print_shop();

    // Everything frozen.
    let post_shop_foods = team
        .shop
        .get_shop_items_by_pos(&Position::All(Condition::None), &Entity::Food)
        .unwrap();
    let post_shop_pets = team
        .shop
        .get_shop_items_by_pos(&Position::All(Condition::None), &Entity::Pet)
        .unwrap();
    assert!(post_shop_foods.into_iter().all(|food| food.is_frozen()));
    assert!(post_shop_pets.into_iter().all(|food| food.is_frozen()));

    // Battle.
    team.fight(&mut enemy_team).unwrap();

    // Reopen shop.
    team.open_shop().unwrap();

    team.print_shop();
}
