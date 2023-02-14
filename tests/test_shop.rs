use saptest::{shop::viewer::ShopItemViewer, Entity, Position, Shop, Shopping, Team};

#[test]
fn test_shop() {
    let mut shop = Shop::new(1, Some(1212)).unwrap();

    shop.freeze(&Position::First, &Entity::Food).unwrap();

    println!("{shop}")
}

#[test]
fn test_team_shop() {
    let mut team = Team::default();
}
