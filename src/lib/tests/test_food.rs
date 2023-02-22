use crate::{
    effects::{
        actions::{Action, GainType, StatChangeType},
        effect::Entity,
        state::{Position, Target},
        stats::Statistics,
        trigger::{TRIGGER_NONE, TRIGGER_START_BATTLE},
    },
    foods::{food::Food, names::FoodName},
    pets::{combat::PetCombat, names::PetName, pet::Pet},
    teams::{
        combat::TeamCombat,
        team::{Team, TeamFightOutcome},
        viewer::TeamViewer,
    },
    tests::common::test_ant_team,
    Condition, Effect, EntityName, Shop, ShopItem, ShopItemViewer, ShopViewer, TeamEffects,
    TeamShopping,
};
// use crate::LOG_CONFIG;

#[test]
fn test_custom_food() {
    let custom_food = Food::new(
        &FoodName::Custom("Dung".to_string()),
        Some(Effect {
            entity: Entity::Food,
            owner: None,
            trigger: TRIGGER_START_BATTLE,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Gain(GainType::DefaultItem(FoodName::Weak)),
            uses: Some(1),
            temp: true,
        }),
    );
    assert!(custom_food.is_ok())
}

#[test]
fn test_food_override_effect() {
    let buffed_apple = Food::new(
        &FoodName::Apple,
        Some(Effect {
            entity: Entity::Food,
            owner: None,
            trigger: TRIGGER_NONE,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(StatChangeType::StaticValue(Statistics {
                attack: 5,
                health: 5,
            })),
            uses: Some(1),
            temp: true,
        }),
    );
    assert!(buffed_apple.is_ok())
}
#[test]
fn test_attack_meat() {
    let mut dog_w_meat = Pet::try_from(PetName::Dog).unwrap();
    dog_w_meat.item = Some(Food::try_from(FoodName::MeatBone).unwrap());

    let mut mammoth = Pet::try_from(PetName::Mammoth).unwrap();

    assert_eq!(
        dog_w_meat.stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    assert_eq!(
        mammoth.stats,
        Statistics {
            attack: 3,
            health: 10
        }
    );

    dog_w_meat.attack(&mut mammoth);

    // Dog deals 7 dmg (+4) with meat instead of 3 dmg .
    assert_eq!(
        dog_w_meat.stats,
        Statistics {
            attack: 3,
            health: 1
        }
    );
    assert_eq!(
        mammoth.stats,
        Statistics {
            attack: 3,
            health: 3
        }
    )
}

#[test]
fn test_attack_garlic() {
    let mut dog_w_garlic = Pet::try_from(PetName::Dog).unwrap();
    dog_w_garlic.item = Some(Food::try_from(FoodName::Garlic).unwrap());
    let mut dog = Pet::try_from(PetName::Dog).unwrap();

    assert_eq!(
        dog_w_garlic.stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    assert_eq!(
        dog.stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );

    dog.attack(&mut dog_w_garlic);

    // Garlic prevents 2 dmg from attack.
    assert_eq!(
        dog_w_garlic.stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
}

#[test]
fn test_attack_garlic_min_1() {
    let mut dog_w_garlic = Pet::try_from(PetName::Dog).unwrap();
    dog_w_garlic.item = Some(Food::try_from(FoodName::Garlic).unwrap());
    let mut ant = Pet::try_from(PetName::Ant).unwrap();

    assert_eq!(
        dog_w_garlic.stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    assert_eq!(
        ant.stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );

    ant.attack(&mut dog_w_garlic);

    // Dog still takes 1 dmg despite negating 2 dmg from 2 atk ant.
    assert_eq!(
        dog_w_garlic.stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
}

#[test]
fn test_attack_melon() {
    let mut dog_w_melon = Pet::try_from(PetName::Dog).unwrap();
    dog_w_melon.item = Some(Food::try_from(FoodName::Melon).unwrap());

    assert_eq!(dog_w_melon.item.as_ref().unwrap().ability.uses, Some(1));

    let original_stats = dog_w_melon.stats.clone();

    let mut big_ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 20,
            health: 20,
        }),
        1,
    )
    .unwrap();

    dog_w_melon.attack(&mut big_ant);

    // Dog takes no damage (20 - 20 = 0) due to melon armor.
    assert_eq!(dog_w_melon.stats, original_stats);
    assert_eq!(
        big_ant.stats,
        Statistics {
            health: 17,
            attack: 20
        }
    );

    // Negating one attack drops uses to 0.
    assert_eq!(dog_w_melon.item.as_ref().unwrap().ability.uses, Some(0));
}

#[test]
fn test_attack_steak() {
    let mut dog_w_steak = Pet::try_from(PetName::Dog).unwrap();
    dog_w_steak.item = Some(Food::try_from(FoodName::Steak).unwrap());

    assert_eq!(dog_w_steak.item.as_ref().unwrap().ability.uses, Some(1));

    let mut smol_ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 1,
            health: 23,
        }),
        1,
    )
    .unwrap();

    dog_w_steak.attack(&mut smol_ant);

    // Ant faints due to steak (3 + 20 = 23).
    assert_eq!(
        smol_ant.stats,
        Statistics {
            health: 0,
            attack: 1
        }
    );

    // Negating one attack drops uses to 0.
    assert_eq!(dog_w_steak.item.as_ref().unwrap().ability.uses, Some(0));
}

#[test]
fn test_attack_coconut() {
    let mut dog_w_coconut = Pet::try_from(PetName::Dog).unwrap();
    dog_w_coconut.item = Some(Food::try_from(FoodName::Coconut).unwrap());
    let original_dog_w_coconut_stats = dog_w_coconut.stats.clone();

    assert_eq!(dog_w_coconut.item.as_ref().unwrap().ability.uses, Some(1));

    let mut big_ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();

    dog_w_coconut.attack(&mut big_ant);

    // Dog takes not damage due to invulnerability from coconut.
    assert_eq!(dog_w_coconut.stats, original_dog_w_coconut_stats);

    // Negating one attack drops uses to 0.
    assert_eq!(dog_w_coconut.item.as_ref().unwrap().ability.uses, Some(0));
}

#[test]
fn test_attack_peanuts() {
    let mut scorpion = Pet::try_from(PetName::Scorpion).unwrap();
    scorpion.item = Some(Food::try_from(FoodName::Peanut).unwrap());

    let mut big_ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();

    scorpion.attack(&mut big_ant);

    assert_eq!(
        big_ant.stats,
        Statistics {
            attack: 50,
            health: 0
        }
    );
}

#[test]
fn test_attack_peanuts_coconut() {
    let mut dog_w_coconut = Pet::try_from(PetName::Dog).unwrap();
    dog_w_coconut.item = Some(Food::try_from(FoodName::Coconut).unwrap());
    let original_dog_w_coconut_stats = dog_w_coconut.stats.clone();

    let mut scorpion = Pet::try_from(PetName::Scorpion).unwrap();
    scorpion.item = Some(Food::try_from(FoodName::Peanut).unwrap());

    // Dog survives attack with coconut and takes no damage.
    dog_w_coconut.attack(&mut scorpion);

    assert_eq!(dog_w_coconut.stats, original_dog_w_coconut_stats);
}

#[test]
fn test_attack_peanuts_melon() {
    let mut dog_w_melon = Pet::try_from(PetName::Dog).unwrap();
    dog_w_melon.item = Some(Food::try_from(FoodName::Melon).unwrap());
    let original_dog_w_melon_stats = dog_w_melon.stats.clone();

    let mut scorpion = Pet::try_from(PetName::Scorpion).unwrap();
    scorpion.item = Some(Food::try_from(FoodName::Peanut).unwrap());

    // Dog survives attack with melon and takes no damage.
    dog_w_melon.attack(&mut scorpion);

    assert_eq!(dog_w_melon.stats, original_dog_w_melon_stats);
}

#[test]
fn test_attack_peanuts_melon_overflow() {
    let mut dog_w_melon = Pet::new(
        PetName::Dog,
        Some("big_dog".to_string()),
        Some(Statistics {
            attack: 1,
            health: 50,
        }),
        1,
    )
    .unwrap();
    dog_w_melon.item = Some(Food::try_from(FoodName::Melon).unwrap());

    // Scorpion has just enough attack (> 20) to deliver death's touch.
    let mut scorpion = Pet::new(
        PetName::Scorpion,
        Some("scorpion".to_string()),
        Some(Statistics {
            attack: 21,
            health: 2,
        }),
        1,
    )
    .unwrap();
    // Note: Individually fighting pets doesn't trigger gaining peanuts.
    // Adding manually here.
    scorpion.item = Some(Food::try_from(FoodName::Peanut).unwrap());

    dog_w_melon.attack(&mut scorpion);

    // Dog dies.
    assert_eq!(
        dog_w_melon.stats,
        Statistics {
            health: 0,
            attack: 1
        }
    )
}

#[test]
fn test_attack_chili() {
    let mut team = Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
        ],
        5,
    )
    .unwrap();
    let mut enemy_team = Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
        ],
        5,
    )
    .unwrap();

    // Give first pet chili on first team.
    // Will kill entire team in first attack
    team.set_item(
        Position::First,
        Some(Food::try_from(&FoodName::Chili).unwrap()),
    )
    .unwrap();

    let outcome = team.fight(&mut enemy_team).unwrap();

    // Ant team wins instead of drawing due to chili's splash damage.
    assert_eq!(outcome, TeamFightOutcome::Win);
}

#[test]
fn test_attack_chili_w_front_space() {
    let mut team = Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
        ],
        5,
    )
    .unwrap();
    let mut enemy_team = Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            None,
            Some(Pet::try_from(PetName::Ant).unwrap()),
        ],
        5,
    )
    .unwrap();
    // Give first pet chili on first team.
    team.set_item(
        Position::First,
        Some(Food::try_from(&FoodName::Chili).unwrap()),
    )
    .unwrap();

    team.fight(&mut enemy_team).unwrap();

    // Only 1 pet on enemy team dies due to space at 1st position.
    assert_eq!(enemy_team.all().len(), 1);

    // Good positioning results in a draw instead of a loss.
    assert_eq!(team.fight(&mut enemy_team).unwrap(), TeamFightOutcome::Draw)
}

#[test]
fn test_battle_honey_team() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    // Give last pet honey on first team.
    team.set_item(
        Position::Last,
        Some(Food::try_from(&FoodName::Honey).unwrap()),
    )
    .unwrap();

    let mut fight = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team).unwrap();
    }

    // Ant team completes by team has honey so bee spawns.
    assert_eq!(fight, TeamFightOutcome::Win);
    assert_eq!(team.first().unwrap().borrow().name, PetName::Bee)
}

#[test]
fn test_battle_mushroom_team() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    // Give last pet mushroom on first team.
    team.set_item(
        Position::Last,
        Some(Food::try_from(&FoodName::Mushroom).unwrap()),
    )
    .unwrap();

    let mut fight = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team).unwrap();
    }

    // Team wins over enemy by summoning ant with (1,1).
    assert_eq!(fight, TeamFightOutcome::Win);
    assert_eq!(team.first().unwrap().borrow().name, PetName::Ant);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    )
}

#[test]
fn test_shop_sleeping_pill() {
    let mut team = test_ant_team();
    team.set_seed(Some(42))
        .set_shop_tier(4)
        .unwrap()
        .set_shop_seed(Some(332))
        .open_shop()
        .unwrap();

    let (pill_pos, item_type) = (Position::Last, Entity::Food);

    let found_items = team
        .shop
        .get_shop_items_by_pos(&pill_pos, &item_type)
        .unwrap();
    // Pill in shop.
    assert_eq!(
        found_items[0].name(),
        EntityName::Food(FoodName::SleepingPill)
    );
    // Three items on team.
    assert_eq!(team.all().len(), 3);
    assert_eq!(team.fainted.len(), 0);

    // Buy pill and put it on first pet on team.
    team.buy(&pill_pos, &item_type, &Position::First).unwrap();

    // Pet faints.
    assert_eq!(team.all().len(), 2);
    assert_eq!(team.fainted.len(), 1)
}

#[test]
fn test_shop_end_turn_foods() {
    let mut team = test_ant_team();
    let mut custom_shop = Shop::new(1, Some(12)).unwrap();
    custom_shop
        .add_item(ShopItem::from(Food::try_from(FoodName::Carrot).unwrap()))
        .unwrap()
        .add_item(ShopItem::from(Food::try_from(FoodName::Cucumber).unwrap()))
        .unwrap()
        .add_item(ShopItem::from(Food::try_from(FoodName::Croissant).unwrap()))
        .unwrap();

    team.replace_shop(custom_shop)
        .unwrap()
        .open_shop()
        .unwrap()
        .buy(&Position::Last, &Entity::Food, &Position::Relative(-2))
        .unwrap()
        .buy(&Position::Last, &Entity::Food, &Position::Relative(-1))
        .unwrap()
        .buy(&Position::Last, &Entity::Food, &Position::Relative(0))
        .unwrap();

    let first_ant = team.nth(0).unwrap();
    let second_ant = team.nth(1).unwrap();
    let third_ant = team.nth(2).unwrap();

    // All have normal stats.
    for ant in [&first_ant, &second_ant, &third_ant] {
        assert_eq!(
            ant.borrow().stats,
            Statistics {
                attack: 2,
                health: 1
            }
        );
    }

    // Close shop signaling the end of the turn.
    team.close_shop().unwrap();

    // First ant got carrot
    assert_eq!(
        first_ant.borrow().stats,
        Statistics {
            attack: 3,
            health: 2
        }
    );

    // Second ant got cucumber.
    assert_eq!(
        second_ant.borrow().stats,
        Statistics {
            attack: 2,
            health: 2
        }
    );

    // Third ant got croissant
    assert_eq!(
        third_ant.borrow().stats,
        Statistics {
            attack: 3,
            health: 1
        }
    );
}

#[test]
fn test_shop_start_turn_foods() {
    let mut ant = Pet::try_from(PetName::Ant).unwrap();
    ant.item = Some(Food::try_from(FoodName::Grapes).unwrap());

    let mut team = Team::new(&[Some(ant)], 5).unwrap();

    // Start with 10.
    assert_eq!(team.gold(), 10);
    // Open shop triggering start of turn.
    team.open_shop().unwrap();
    // Now has 11.
    assert_eq!(team.gold(), 11);
}

#[test]
fn test_direct_attack_pepper() {
    let mut ant = Pet::try_from(PetName::Ant).unwrap();

    ant.item = Some(Food::try_from(FoodName::Pepper).unwrap());

    let mut mammoth = Pet::try_from(PetName::Mammoth).unwrap();

    // At start Ant has 1 health.
    assert!(ant.stats.health == 1);

    ant.attack(&mut mammoth);

    // Survives single hit. Pepper uses depleted.
    assert!(ant.stats.health == 1);
    assert!(ant.item.as_ref().unwrap().ability.uses == Some(0));

    // Second attack.
    ant.attack(&mut mammoth);

    // Ant faints.
    assert!(ant.stats.health == 0);
}

#[test]
fn test_indirect_attack_pepper() {
    let mut ant = Pet::try_from(PetName::Ant).unwrap();
    ant.item = Some(Food::try_from(FoodName::Pepper).unwrap());

    // At start Ant has 1 health.
    assert!(ant.stats.health == 1);

    let dmg = Statistics {
        attack: 2,
        health: 0,
    };
    ant.indirect_attack(&dmg);

    // Survives single hit. Pepper uses depleted.
    assert!(ant.stats.health == 1);
    assert!(ant.item.as_ref().unwrap().ability.uses == Some(0));

    ant.indirect_attack(&dmg);

    // Ant faints.
    assert!(ant.stats.health == 0);
}

#[test]
fn test_direct_attack_pepper_peanut_1hp() {
    let mut ant = Pet::try_from(PetName::Ant).unwrap();
    ant.item = Some(Food::try_from(FoodName::Pepper).unwrap());

    let mut mammoth = Pet::try_from(PetName::Mammoth).unwrap();
    mammoth.item = Some(Food::try_from(FoodName::Peanut).unwrap());

    assert!(ant.stats.health == 1);

    ant.attack(&mut mammoth);

    // Ant resists fainting surviving with 1 health despite peanut.
    assert!(ant.stats.health == 1);
    assert!(ant.item.as_ref().unwrap().ability.uses == Some(0));
}

#[test]
fn test_direct_attack_pepper_peanut() {
    let mut big_ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();
    big_ant.item = Some(Food::try_from(FoodName::Pepper).unwrap());

    let mut mammoth = Pet::try_from(PetName::Mammoth).unwrap();
    mammoth.item = Some(Food::try_from(FoodName::Peanut).unwrap());

    big_ant.attack(&mut mammoth);

    // Big ant faints.
    // https://superautopets.fandom.com/wiki/Pepper
    assert!(big_ant.stats.health == 0);
    assert!(big_ant.item.as_ref().unwrap().ability.uses == Some(0));
}

#[test]
fn test_direct_attack_cheese() {
    let mut ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 5,
            health: 10,
        }),
        1,
    )
    .unwrap();
    ant.item = Some(Food::try_from(FoodName::Cheese).unwrap());

    let mut mammoth = Pet::try_from(PetName::Mammoth).unwrap();

    // Single use.
    assert!(ant.item.as_ref().unwrap().ability.uses == Some(1));
    // Mammoth has 10 health
    assert!(mammoth.stats.health == 10);

    ant.attack(&mut mammoth);

    assert!(ant.item.as_ref().unwrap().ability.uses == Some(0));
    // Mammoth has 0 health
    assert!(mammoth.stats.health == 0);
}

#[test]
fn test_direct_attack_fortune_cookie() {
    let mut ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 5,
            health: 10,
        }),
        1,
    )
    .unwrap();
    ant.item = Some(Food::try_from(FoodName::FortuneCookie).unwrap());
    ant.seed = Some(12);

    let mut mammoth = Pet::try_from(PetName::Mammoth).unwrap();

    ant.attack(&mut mammoth);

    // Mammoth has 0 health
    assert!(mammoth.stats.health == 0);

    // Reset stats and set seed to scenario where cookie fails.
    ant.seed = Some(25);
    mammoth.stats.health = 10;

    ant.attack(&mut mammoth);

    assert!(mammoth.stats.health == 5);
}

#[test]
fn test_battle_pineapple() {
    let mut mosq = Pet::try_from(PetName::Mosquito).unwrap();
    mosq.item = Some(Food::try_from(FoodName::Pineapple).unwrap());

    let mut team = Team::new(&[Some(mosq)], 5).unwrap();
    let mut enemy_team = Team::new(&[Some(Pet::try_from(PetName::Mammoth).unwrap())], 5).unwrap();

    let mammoth = enemy_team.first().unwrap();
    // Starting mammoth stats.
    assert_eq!(
        mammoth.borrow().stats,
        Statistics {
            attack: 3,
            health: 10
        }
    );

    team.triggers.push_back(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();

    // Mammoth takes 2 additional damage than normal thanks to pineapple.
    assert_eq!(
        mammoth.borrow().stats,
        Statistics {
            attack: 3,
            health: 7
        }
    );
}

#[test]
fn test_shop_canned_food() {
    let mut team = test_ant_team();
    let mut custom_shop = Shop::new(1, Some(12)).unwrap();
    custom_shop
        .add_item(ShopItem::from(
            Food::try_from(FoodName::CannedFood).unwrap(),
        ))
        .unwrap();

    team.replace_shop(custom_shop).unwrap().open_shop().unwrap();

    fn first_shop_pet_query<'a>(team: &'a Team) -> &'a ShopItem {
        let shop_pets = team
            .shop
            .get_shop_items_by_pos(&Position::All(Condition::None), &Entity::Pet)
            .unwrap();
        let pet_1 = shop_pets.get(0).unwrap();
        pet_1
    }

    let mosq = first_shop_pet_query(&team);

    // Starting pets in shop.
    assert!(mosq.attack_stat() == Some(2) && mosq.health_stat() == Some(2));
    team.buy(&Position::Last, &Entity::Food, &Position::None)
        .unwrap();

    let mosq = first_shop_pet_query(&team);
    // Pets in shop receive (1,1).
    assert!(mosq.attack_stat() == Some(3) && mosq.health_stat() == Some(3));

    // Roll shop.
    team.set_shop_seed(Some(13)).roll_shop().unwrap();

    let horse = first_shop_pet_query(&team);
    // Future pets get buff as permanent stats added to shop.
    assert!(horse.attack_stat() == Some(3) && horse.health_stat() == Some(2));
    assert_eq!(
        team.shop.perm_stats,
        Statistics {
            attack: 1,
            health: 1
        }
    )
}

#[test]
fn test_shop_lollipop() {
    let mut team = test_ant_team();
    let mut custom_shop = Shop::new(1, Some(12)).unwrap();
    custom_shop
        .add_item(ShopItem::from(Food::try_from(FoodName::Lollipop).unwrap()))
        .unwrap();

    team.replace_shop(custom_shop).unwrap().open_shop().unwrap();

    let ant = team.first().unwrap();

    assert_eq!(
        ant.borrow().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    // Buy lollipop for ant.
    team.buy(&Position::Last, &Entity::Food, &Position::First)
        .unwrap();

    // Stats are swapped.
    assert_eq!(
        ant.borrow().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
}

#[test]
fn test_battle_popcorns() {
    let mut team = test_ant_team();
    let mut enemy_team = team.clone();

    let mut custom_shop = Shop::new(1, Some(12)).unwrap();
    custom_shop
        .add_item(ShopItem::from(Food::try_from(FoodName::Popcorns).unwrap()))
        .unwrap();

    team.replace_shop(custom_shop).unwrap().open_shop().unwrap();

    // Buy popcorns for ant.
    team.buy(&Position::Last, &Entity::Food, &Position::First)
        .unwrap();
    team.close_shop().unwrap();

    // Fight to get first ant to faint.
    let first_ant = team.first().unwrap();
    team.fight(&mut enemy_team).unwrap();

    assert_eq!(team.fainted.first().unwrap(), &Some(first_ant.clone()));
    // Summoned pet is same tier as ant.
    let summoned_pet = team.first().unwrap();
    assert_eq!(summoned_pet.borrow().tier, first_ant.borrow().tier);
}
