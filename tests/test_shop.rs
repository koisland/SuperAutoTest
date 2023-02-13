use saptest::{shop::store::CheckShopState, Shop, Team};

#[test]
fn test_shop() {
    let mut team = Team::default();
    let mut shop = Shop::new(&mut team, Some(1212)).unwrap();

    shop.freeze_food(0);

    // let food = shop.get_food(0).unwrap();
    // assert!(food.is_frozen());
}
