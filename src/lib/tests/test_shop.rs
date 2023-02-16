use crate::{
    shop::store::ItemSlot, Entity, EntityName, FoodName, PetName, Position, Shop, ShopItem,
    ShopItemViewer, ShopViewer,
};

#[test]
fn test_create_shop_item_slot() {
    assert!(ItemSlot::new(EntityName::Food(FoodName::Apple)).is_ok());
    assert!(ItemSlot::new(EntityName::Pet(PetName::Ant)).is_ok());
}

#[test]
fn test_create_shop_item() {
    assert!(ShopItem::new(EntityName::Food(FoodName::Coconut), 5).is_ok())
}

#[test]
fn test_add_shop_item() {
    // Create tier 5 shop with max size.
    let mut shop = Shop::new(5, Some(12)).unwrap();
    let pet = ShopItem::new(EntityName::Pet(PetName::Ant), 3).unwrap();
    let item = ShopItem::new(EntityName::Food(FoodName::Coconut), 5).unwrap();

    // This fails because the maximum number of pets has been reached.
    assert!(shop.add_item(pet.clone()).is_err());
    assert_eq!(shop.pets.len(), 5);
    // But this succeeds because space is made by removing a pet.
    assert!(shop.add_item(item).is_ok());
    assert_eq!(shop.pets.len(), 4);

    // By clearing the shop foods, we can free space for additional pets.
    // Up until the food capacity is filled with pets.
    shop.foods.clear();
    for _ in 0..3 {
        shop.add_item(pet.clone()).unwrap();
    }
    assert_eq!(
        shop.pets.len(),
        shop.max_pet_slots() + shop.max_food_slots()
    );
    assert!(shop.add_item(pet.clone()).is_err())
}

#[test]
fn test_max_food_slots() {
    let mut shop = Shop::default();
    shop.set_tier(1).unwrap();
    assert_eq!(shop.max_food_slots(), 1);

    shop.set_tier(4).unwrap();
    assert_eq!(shop.max_food_slots(), 2);

    // Fill foods for shop at tier 4.
    shop.fill_foods().unwrap();
    // Remove 1.
    shop.foods.pop();
    // Now one food slot open.
    assert_eq!(shop.available_food_slots(), 1)
}

#[test]
fn test_max_pet_slots() {
    let mut shop = Shop::default();
    shop.set_tier(2).unwrap();
    assert_eq!(shop.max_pet_slots(), 3);

    shop.set_tier(4).unwrap();
    assert_eq!(shop.max_pet_slots(), 4);

    shop.set_tier(6).unwrap();
    assert_eq!(shop.max_pet_slots(), 5);

    // Fill pets for shop at tier 4.
    shop.fill_pets().unwrap();
    // Remove 1.
    shop.pets.pop();
    // Now one pet slot open.
    assert_eq!(shop.available_pet_slots(), 1)
}

#[test]
fn test_set_tier() {
    let mut shop = Shop::default();
    assert_eq!(shop.tier(), 1);

    shop.set_tier(2).unwrap();
    assert_eq!(shop.tier(), 2);

    assert!(shop.set_tier(7).is_err());
}

#[test]
fn test_fill_pets() {
    let mut shop = Shop::default();
    assert!(shop.pets.is_empty());

    shop.fill_pets().unwrap();
    assert_eq!(shop.pets.len(), 3);
}

#[test]
fn test_fill_foods() {
    let mut shop = Shop::default();
    assert!(shop.pets.is_empty());

    shop.fill_foods().unwrap();
    assert_eq!(shop.foods.len(), 1);
}

#[test]
fn test_add_levelup_pet() {
    let mut shop = Shop::default();
    shop.add_levelup_pet().unwrap();

    let curr_tier = shop.tier();
    let levelup_pet = shop.pets.first().unwrap();
    assert_eq!(levelup_pet.tier(), curr_tier + 1)
}

#[test]
fn test_roll() {
    let mut shop = Shop::new(1, Some(12)).unwrap();
    // Freeze first pet.
    let (pos, item_type) = (Position::First, Entity::Pet);
    shop.freeze(&pos, &item_type).unwrap();

    // Check first item is frozen.
    let found_items = shop.get_shop_items_by_pos(&pos, &item_type).unwrap();
    let first_item_no_roll = found_items.first().cloned().unwrap().clone();
    assert!(first_item_no_roll.is_frozen());

    // Roll the shop.
    shop.roll().unwrap();

    // Check items again.
    let found_items_rolled = shop.get_shop_items_by_pos(&pos, &item_type).unwrap();
    let first_item_rolled = found_items_rolled.first().unwrap();

    // First item was retained.
    assert_eq!(&&first_item_no_roll, first_item_rolled)
}
