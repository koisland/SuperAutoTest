use itertools::Itertools;

use crate::{
    effects::{
        actions::{Action, StatChangeType},
        state::{EqualityCondition, Status},
    },
    Entity, EntityName, Food, FoodName, ItemCondition, Pet, PetName, Position, Shop, ShopItem,
    ShopItemViewer, ShopViewer, Statistics,
};

#[test]
fn test_create_shop_item() {
    let mut melon = Food::try_from(FoodName::Melon).unwrap();
    let item = ShopItem::new(melon.clone());
    assert!(
        item.name() == EntityName::Food(FoodName::Melon) && item.cost() == 3 && !item.is_frozen()
    );

    // Change cost.
    melon.cost = 2;
    let discounted_item = ShopItem::new(melon);
    assert!(
        discounted_item.name() == EntityName::Food(FoodName::Melon)
            && discounted_item.cost() == 2
            && !discounted_item.is_frozen()
    );
}

#[test]
fn test_add_shop_item() {
    // Create tier 5 shop with max size.
    let mut shop = Shop::new(5, Some(12)).unwrap();
    let pet = ShopItem::new(Pet::try_from(PetName::Ant).unwrap());
    let item = ShopItem::new(Food::try_from(FoodName::Coconut).unwrap());

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
    assert!(shop.add_item(pet).is_err())
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

#[test]
fn test_roll_no_gold() {
    let mut shop = Shop::new(1, Some(12)).unwrap();
    for _ in 0..10 {
        shop.roll().unwrap();
    }
    assert!(shop.roll().is_err())
}

#[test]
fn test_view_item_attr() {
    let mut shop = Shop::new(6, Some(122)).unwrap();
    shop.add_item(ShopItem::try_from(Food::try_from(FoodName::Garlic).unwrap()).unwrap())
        .unwrap()
        .add_item(ShopItem::try_from(Food::try_from(FoodName::Chocolate).unwrap()).unwrap())
        .unwrap();
    let pets = shop
        .get_shop_items_by_pos(&Position::All(ItemCondition::None), &Entity::Pet)
        .unwrap();
    let foods = shop
        .get_shop_items_by_pos(&Position::All(ItemCondition::None), &Entity::Food)
        .unwrap();
    let pizza = foods[0];
    let chili = foods[1];
    let garlic = foods[2];
    let choco = foods[3];
    let horse = pets[1];
    // Pet
    assert!(
        horse.tier() == 1
            && horse.attack_stat().unwrap() == 2
            && horse.health_stat().unwrap() == 1
            && horse.actions()[0]
                == Action::Add(StatChangeType::Static(Statistics {
                    attack: 1,
                    health: 0
                }))
    );
    // Foods
    assert!(
        pizza.tier() == 6
            && pizza.attack_stat().unwrap() == 2
            && pizza.health_stat().unwrap() == 2
            && pizza.actions()[0]
                == Action::Add(StatChangeType::Static(Statistics {
                    attack: 2,
                    health: 2
                }))
    );

    // Chili does 5 atk.
    assert!(
        chili.tier() == 5
            && chili.attack_stat().unwrap() == 5
            && chili.health_stat().unwrap() == 0
            && chili.actions()[0]
                == Action::Remove(StatChangeType::Static(Statistics {
                    attack: 5,
                    health: 0
                }))
    );

    // Garlic cannot negate health.
    assert!(
        garlic.tier() == 3 &&
        garlic.attack_stat().unwrap() == 2 &&
        garlic.health_stat().is_none() &&
        garlic.actions()[0] == Action::Negate(Statistics { attack: 2, health: 0 }) &&
        // Occurs before being hurt and not during attacking.
        garlic.triggers()[0] == Status::AnyDmgCalc
    );

    // Choco gives experience. Stats implicit.
    assert!(choco.attack_stat().is_none() && choco.health_stat().is_none())
}
#[test]
fn test_view_by_cond() {
    /*
    (Pets)
    (Normal) [Parrot: (4,2) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    (Normal) [Horse: (2,1) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    (Normal) [Seal: (3,8) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    (Normal) [Gorilla: (6,9) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    (Normal) [Giraffe: (1,3) (Level: 1 Exp: 0) (Pos: None) (Item: None)]

    (Foods)
    (Normal) [Pizza: [Effect (Uses: None): (Food) - Trigger: [Status: None, Position: None, Affected: None, From: None] - Action: Add(SetStatistics(Statistics { attack: 2, health: 2 })) on Friend (Any(None)) ]]
    (Normal) [Chili: [Effect (Uses: None): (Food) - Trigger: [Status: Attack, Position: OnSelf, Affected: None, From: None] - Action: Remove(SetStatistics(Statistics { attack: 5, health: 0 })) on Enemy (Relative(-1)) ]]
    */
    let shop = Shop::new(6, Some(122)).unwrap();
    let pets = shop
        .get_shop_items_by_pos(&Position::All(ItemCondition::None), &Entity::Pet)
        .unwrap();
    let foods = shop
        .get_shop_items_by_pos(&Position::All(ItemCondition::None), &Entity::Food)
        .unwrap();

    let (parrot, horse, seal, gorilla, giraffe) = pets
        .into_iter()
        .collect_tuple::<(&ShopItem, &ShopItem, &ShopItem, &ShopItem, &ShopItem)>()
        .unwrap();
    let (pizza, chili) = foods
        .into_iter()
        .collect_tuple::<(&ShopItem, &ShopItem)>()
        .unwrap();

    let healthiest = shop
        .get_shop_items_by_cond(&ItemCondition::Healthiest, &Entity::Pet)
        .unwrap()[0];
    assert_eq!(healthiest, gorilla);

    let illest = shop
        .get_shop_items_by_cond(&ItemCondition::Illest, &Entity::Pet)
        .unwrap()[0];
    assert_eq!(illest, horse);

    let strongest = shop
        .get_shop_items_by_cond(&ItemCondition::Strongest, &Entity::Pet)
        .unwrap()[0];
    assert_eq!(strongest, gorilla);

    let weakest = shop
        .get_shop_items_by_cond(&ItemCondition::Weakest, &Entity::Pet)
        .unwrap()[0];
    assert_eq!(weakest, giraffe);

    let highest_tier = shop
        .get_shop_items_by_cond(&ItemCondition::HighestTier, &Entity::Pet)
        .unwrap()[0];
    assert_eq!(highest_tier, gorilla);

    let lowest_tier = shop
        .get_shop_items_by_cond(&ItemCondition::LowestTier, &Entity::Pet)
        .unwrap()[0];
    assert_eq!(lowest_tier, horse);

    let name_is_parrot = shop
        .get_shop_items_by_cond(
            &ItemCondition::Equal(EqualityCondition::Name(EntityName::Pet(PetName::Parrot))),
            &Entity::Pet,
        )
        .unwrap()[0];
    assert_eq!(name_is_parrot, parrot);

    let tier_is_5 = shop
        .get_shop_items_by_cond(
            &ItemCondition::Equal(EqualityCondition::Tier(5)),
            &Entity::Pet,
        )
        .unwrap()[0];
    assert_eq!(tier_is_5, seal);

    // Can't check if self.
    assert!(shop
        .get_shop_items_by_cond(
            &ItemCondition::Equal(EqualityCondition::IsSelf),
            &Entity::Pet
        )
        .is_err());

    let trigger_is_hurt = shop
        .get_shop_items_by_cond(
            &ItemCondition::Equal(EqualityCondition::Trigger(Status::Hurt)),
            &Entity::Pet,
        )
        .unwrap()[0];
    assert_eq!(trigger_is_hurt, gorilla);

    // All other pets aside gorilla are included.
    let trigger_is_not_hurt = shop
        .get_shop_items_by_cond(
            &ItemCondition::NotEqual(EqualityCondition::Trigger(Status::Hurt)),
            &Entity::Pet,
        )
        .unwrap();
    assert!(trigger_is_not_hurt.len() == 4 && !trigger_is_not_hurt.contains(&gorilla));

    // Get multiple conditions.
    let tier_is_5_or_6 = shop
        .get_shop_items_by_cond(
            &ItemCondition::Multiple(vec![
                ItemCondition::Equal(EqualityCondition::Tier(5)),
                ItemCondition::Equal(EqualityCondition::Tier(6)),
            ]),
            &Entity::Pet,
        )
        .unwrap();

    assert!(
        tier_is_5_or_6.len() == 2
            && tier_is_5_or_6.contains(&gorilla)
            && tier_is_5_or_6.contains(&seal)
    );

    let tier_is_not_5_and_trigger_is_hurt = shop
        .get_shop_items_by_cond(
            &ItemCondition::MultipleAll(vec![
                ItemCondition::NotEqual(EqualityCondition::Tier(5)),
                ItemCondition::Equal(EqualityCondition::Trigger(Status::Hurt)),
            ]),
            &Entity::Pet,
        )
        .unwrap();
    assert!(
        tier_is_not_5_and_trigger_is_hurt.len() == 1
            && tier_is_not_5_and_trigger_is_hurt[0] == gorilla
    );

    // Also works for foods.
    // * All stat conditions will check effect stats. Chili has 5 attack dmg (3 more) compared to pizza.
    let strongest_food = shop
        .get_shop_items_by_cond(&ItemCondition::Strongest, &Entity::Food)
        .unwrap()[0];
    assert_eq!(strongest_food, chili);

    let gives_2_2_stats = shop
        .get_shop_items_by_cond(
            &ItemCondition::Equal(EqualityCondition::Action(Box::new(Action::Add(
                StatChangeType::Static(Statistics::new(2, 2).unwrap()),
            )))),
            &Entity::Food,
        )
        .unwrap();
    assert_eq!(gives_2_2_stats[0], pizza);
}

#[test]
fn test_view_by_pos() {
    /*
    (Pets)
    (Normal) [Parrot: (4,2) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    (Normal) [Horse: (2,1) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    (Normal) [Seal: (3,8) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    (Normal) [Gorilla: (6,9) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    (Normal) [Giraffe: (1,3) (Level: 1 Exp: 0) (Pos: None) (Item: None)]

    (Foods)
    (Normal) [Pizza: [Effect (Uses: None): (Food) - Trigger: [Status: None, Position: None, Affected: None, From: None] - Action: Add(SetStatistics(Statistics { attack: 2, health: 2 })) on Friend (Any(None)) ]]
    (Normal) [Chili: [Effect (Uses: None): (Food) - Trigger: [Status: Attack, Position: OnSelf, Affected: None, From: None] - Action: Remove(SetStatistics(Statistics { attack: 5, health: 0 })) on Enemy (Relative(-1)) ]]
    */
    let shop = Shop::new(6, Some(122)).unwrap();

    let all_pets = shop
        .get_shop_items_by_pos(&Position::All(ItemCondition::None), &Entity::Pet)
        .unwrap();
    let (parrot, horse, seal, _, _) = all_pets
        .clone()
        .into_iter()
        .collect_tuple::<(&ShopItem, &ShopItem, &ShopItem, &ShopItem, &ShopItem)>()
        .unwrap();

    let any_pet = shop
        .get_shop_items_by_pos(&Position::Any(ItemCondition::None), &Entity::Pet)
        .unwrap();
    let first_pet = shop
        .get_shop_items_by_pos(&Position::First, &Entity::Pet)
        .unwrap();
    let last_pet = shop
        .get_shop_items_by_pos(&Position::Last, &Entity::Pet)
        .unwrap();
    let rng_of_pet = shop
        .get_shop_items_by_pos(&Position::Range(0..=2), &Entity::Pet)
        .unwrap();
    let n_num_pets = shop
        .get_shop_items_by_pos(
            &Position::N {
                condition: ItemCondition::None,
                targets: 3,
                random: true,
            },
            &Entity::Pet,
        )
        .unwrap();

    let rel_idx_food = shop
        .get_shop_items_by_pos(&Position::Relative(-1), &Entity::Food)
        .unwrap();
    let rng_out_of_bounds_food = shop
        .get_shop_items_by_pos(&Position::Range(0..=2), &Entity::Food)
        .unwrap();

    // Relative idx from the first item in the shop category.
    assert_eq!(rel_idx_food[0].name(), EntityName::Food(FoodName::Chili));
    // Cannot get adjacent items because no starting pos.
    assert!(shop
        .get_shop_items_by_pos(&Position::Adjacent, &Entity::Food)
        .is_err());
    // Returns empty becase out of bounds.
    assert!(rng_out_of_bounds_food.is_empty());

    assert!(all_pets.len() == 5 && any_pet.len() == 1);
    assert!(
        first_pet[0].name() == EntityName::Pet(PetName::Parrot)
            && last_pet[0].name() == EntityName::Pet(PetName::Giraffe)
    );
    // Three pets because range is inclusive
    assert_eq!(rng_of_pet.len(), 3);
    assert!(
        rng_of_pet[0].name() == EntityName::Pet(PetName::Parrot)
            && rng_of_pet[1].name() == EntityName::Pet(PetName::Horse)
            && rng_of_pet[2].name() == EntityName::Pet(PetName::Seal)
    );

    // Position::N() gets 3 pets but because randomized, doesn't have all of the first 3 pets.
    assert_eq!(n_num_pets.len(), 3);
    let has_first_three_pets = [
        n_num_pets.contains(&parrot),
        n_num_pets.contains(&horse),
        n_num_pets.contains(&seal),
    ];
    assert!(!has_first_three_pets.into_iter().all(|cond| cond))
}
