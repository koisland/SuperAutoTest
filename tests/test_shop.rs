use saptest::{shop::viewer::ShopItemViewer, Entity, Shop, Position};

#[test]
fn test_shop() {
    let mut shop = Shop::new(1, Some(1212)).unwrap();

    shop.freeze(&Position::First, &Entity::Food).unwrap();

    println!("{shop}")
}
