use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use itertools::Itertools;

use crate::{
    shop::store::ShopState,
    teams::{combat::TeamCombat, viewer::TeamViewer},
    Condition, Entity, Pet, PetName, Position, ShopItemViewer, ShopViewer, Team, TeamShopping,
};

use super::common::test_jellyfish_team;

#[test]
fn test_team_shop_opening() {
    let mut team = Team::new(
        &[
            Some(Pet::try_from(PetName::Beaver).unwrap()),
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
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
            Some(Pet::try_from(PetName::Beaver).unwrap()),
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
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
            Some(Pet::try_from(PetName::Beaver).unwrap()),
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
        ],
        5,
    )
    .unwrap();

    team.set_shop_seed(Some(1212)).open_shop().unwrap();

    let (any_pos, pet_item_type, first_pos) =
        (Position::Any(Condition::None), Entity::Pet, Position::First);
    // Invalid position to buy.
    assert!(team
        .buy(&Position::Relative(-12), &pet_item_type, &first_pos)
        .is_err());
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
            Some(Pet::try_from(PetName::Beaver).unwrap()),
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
        ],
        5,
    )
    .unwrap();

    assert_eq!(team.shop.coins, 10);
    team.open_shop().unwrap();
    team.sell(&Position::First).unwrap();

    assert_eq!(team.shop.coins, 11);
    assert_eq!(team.all().len(), 1);

    // Invalid position to sell.
    assert!(team.sell(&Position::Relative(-12)).is_err())
}

#[test]
fn test_team_shop_state_battle() {
    let mut team = Team::new(
        &[
            Some(Pet::try_from(PetName::Beaver).unwrap()),
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
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
    let mut custom_team = Team::new(&[Some(Pet::try_from(PetName::Ant).unwrap())], 5).unwrap();

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
            Some(Pet::try_from(PetName::Beaver).unwrap()),
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
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

#[test]
fn test_shop_move() {
    let mut team = test_jellyfish_team();
    /*
    JFish_0, Ant_2, Ant_1, Ant_0
    */
    let (ant_0, ant_1, _ant_2, _jfish_0) = team
        .friends
        .iter()
        .flatten()
        .map(Rc::downgrade)
        .collect_tuple::<(
            Weak<RefCell<Pet>>,
            Weak<RefCell<Pet>>,
            Weak<RefCell<Pet>>,
            Weak<RefCell<Pet>>,
        )>()
        .unwrap();

    /*
    Ant_0, JFish_0, Ant_2, Ant_1,
    */
    team.move_pets(&Position::First, &Position::Last, false)
        .unwrap();

    assert!(ant_0.upgrade().unwrap().borrow().pos == Some(3));
    /*
    JFish_0, Ant_2, Ant_1, Ant_0
    */
    // Setting to 3rd arg to false moved the pet without merging it into the first pet.
    team.move_pets(&Position::Last, &Position::First, false)
        .unwrap();

    assert!(ant_0.upgrade().unwrap().borrow().pos == Some(0));

    /*
    JFish_0, Ant_2, Ant_1 (+1)
    */
    team.move_pets(&Position::First, &Position::Relative(-1), true)
        .unwrap();

    // Pet has merged (+1 exp) and has been dropped.
    assert!(ant_0.upgrade().is_none());
    assert!(ant_1.upgrade().unwrap().borrow().exp == 1);

    // Position::Any is okay.
    assert!(team
        .move_pets(&Position::First, &Position::Any(Condition::None), true)
        .is_ok());
    // Positions that require an opponent or target multiple pets are not supported.
    assert!(team
        .move_pets(&Position::Opposite, &Position::All(Condition::None), true)
        .is_err())
}
