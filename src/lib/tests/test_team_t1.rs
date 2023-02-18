use crate::{
    effects::{effect::EntityName, stats::Statistics, trigger::TRIGGER_START_BATTLE},
    foods::names::FoodName,
    pets::names::PetName,
    teams::{team::TeamFightOutcome, team_effect_apply::TeamEffects, team_viewer::TeamViewer},
    tests::common::{
        count_pets, test_ant_team, test_beaver_team, test_beetle_team, test_bluebird_team,
        test_chinchilla_team, test_cockroach_team, test_cricket_horse_team, test_duck_team,
        test_duckling_team, test_fish_team, test_frilled_dragon_team, test_frog_team,
        test_hummingbird_team, test_iguana_seahorse_team, test_ladybug_team, test_marmoset_team,
        test_mosq_team, test_moth_team, test_pig_team,
    },
    Condition, Entity, Position, ShopItemViewer, ShopViewer, TeamShopping,
};

#[test]
fn test_battle_ant_team() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    let all_2_1 = team
        .friends
        .iter()
        .all(|pet| pet.borrow().stats == Statistics::new(2, 1).unwrap());
    assert!(all_2_1);

    // One battle phase and one ant faints.
    team.fight(&mut enemy_team).unwrap();

    let any_gets_2_1 = team
        .friends
        .iter()
        .any(|pet| pet.borrow().stats == Statistics::new(4, 2).unwrap());
    // Another pet gets (2,1).
    assert!(any_gets_2_1)
}

#[test]
fn test_battle_cricket_horse_team() {
    let mut team = test_cricket_horse_team();
    let mut enemy_team = test_cricket_horse_team();

    // First pets are crickets
    // Horse is 3rd pet.
    assert_eq!(team.first().unwrap().borrow().name, PetName::Cricket);
    assert_eq!(team.nth(2).unwrap().borrow().name, PetName::Horse);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );

    // After one turn.
    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();

    // Cricket dies and zombie cricket is spawned.
    // Horse provides 1 attack.
    assert_eq!(team.first().unwrap().borrow().name, PetName::ZombieCricket);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
}

#[test]
fn test_battle_mosquito_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_mosq_team();
    let mut enemy_team = test_ant_team();
    enemy_team.set_seed(Some(0));

    let mut fight = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team).unwrap()
    }
    // Mosquitoes kill any team before game starts.
    assert_eq!(fight, TeamFightOutcome::Win);
    assert_eq!(team.friends.len(), 3);

    for pet in team.all().iter() {
        // Mosquitoes are unhurt
        assert_eq!(
            pet.borrow().stats,
            Statistics {
                attack: 2,
                health: 2,
            }
        )
    }
}

#[test]
fn test_battle_frilled_dragon_team() {
    let mut team = test_frilled_dragon_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.last().unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );

    team.fight(&mut enemy_team).unwrap();

    // Team has two crickets with faint triggers. Gains (1,1) for each.
    let last_pet = team.all().into_iter().last();
    assert_eq!(
        last_pet.unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    )
}

#[test]
fn test_battle_frog_team() {
    let mut team = test_frog_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.friends.get(0).unwrap().as_ref().borrow().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
    // Frilled dragon before activation of ability.
    assert_eq!(
        team.friends.get(2).unwrap().as_ref().borrow().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );

    // Frilled dragon activates.
    // Then frog activates swapping stats of cricket and frilled dragon.
    // Cricket with 2/2 dies spawning zombie cricket.
    team.fight(&mut enemy_team).unwrap();

    // Frilled dragon gets cricket stats.
    assert_eq!(
        team.friends.get(2).unwrap().as_ref().borrow().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
}

#[test]
fn test_battle_moth_team() {
    let mut team = test_moth_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.friends.first().unwrap().as_ref().borrow().stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );
    // Ant deals 2 dmg. 2 moths gives (6,0).
    team.fight(&mut enemy_team).unwrap();

    assert_eq!(
        team.friends.first().unwrap().as_ref().borrow().stats,
        Statistics {
            attack: 8,
            health: 1
        }
    );
}

#[test]
fn test_battle_hummingbird_team() {
    let mut team = test_hummingbird_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.friends.first().unwrap().as_ref().borrow().stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );
    // Duck has strawberry.
    let duck = team.friends.first().unwrap();
    assert_eq!(
        duck.borrow().item.as_ref().unwrap().name,
        FoodName::Strawberry
    );
    // Two hummingbirds on team.
    assert_eq!(count_pets(&team.friends, PetName::Hummingbird), 2);
    // Trigger start of battle effects.
    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(&mut enemy_team).unwrap();

    // Duck gets 2/1 for every hummingbird since only strawberry friend.
    assert_eq!(
        team.friends.first().unwrap().as_ref().borrow().stats,
        Statistics {
            attack: 6,
            health: 5
        }
    );
}

#[test]
fn test_battle_iguana_seahorse_team() {
    let mut team = test_iguana_seahorse_team();
    let mut enemy_team = test_cricket_horse_team();

    // Start of battle pushes horse to 2nd position and it gets hit by iguana.
    // Seahorse knockouts cricket leaving zombie cricket.
    // Zombie cricket hit by iguana.
    team.fight(&mut enemy_team).unwrap();

    // Only one pet remaining on enemy team.
    assert_eq!(enemy_team.first().unwrap().borrow().name, PetName::Cricket);
    assert_eq!(enemy_team.friends.len(), 1)
}

#[test]
fn test_shop_beaver_team() {
    let beaver_pos = Position::First;
    let mut team = test_beaver_team();
    for (i, pet) in team.all().into_iter().enumerate() {
        if i == 0 {
            continue;
        }
        // Two pets at (2,1)
        assert_eq!(pet.borrow().stats, Statistics::new(2, 1).unwrap())
    }

    // Init shop and set level of beaver at front to level 2.
    // Then sell it.
    team.set_shop_seed(Some(1212))
        .open_shop()
        .unwrap()
        .set_level(&beaver_pos, 2)
        .unwrap()
        .sell(&beaver_pos)
        .unwrap();

    // Ants get (0,2)
    for (_, pet) in team.all().into_iter().enumerate() {
        assert_eq!(pet.borrow().stats, Statistics::new(2, 3).unwrap())
    }
}

#[test]
fn test_shop_duck_team() {
    let mut team = test_duck_team();

    assert_eq!(team.first().unwrap().borrow().name, PetName::Duck);

    team.set_shop_seed(Some(11)).open_shop().unwrap();

    // Search pets.
    let (pet_pos, item_type) = (Position::All(Condition::None), Entity::Pet);
    let shop_pets_before = team
        .shop
        .get_shop_items_by_pos(&pet_pos, &item_type)
        .unwrap();

    for (i, item) in shop_pets_before.iter().enumerate() {
        if i == 2 {
            assert!(item.attack_stat() == Some(4) && item.health_stat() == Some(1))
        } else {
            assert!(item.attack_stat() == Some(2) && item.health_stat() == Some(1))
        }
    }

    // Sell duck
    team.sell(&Position::First).unwrap();

    // Pets in shop gain +1 health.
    let shop_pets_after = team
        .shop
        .get_shop_items_by_pos(&pet_pos, &item_type)
        .unwrap();
    for (i, item) in shop_pets_after.iter().enumerate() {
        if i == 2 {
            assert!(item.attack_stat() == Some(4) && item.health_stat() == Some(2))
        } else {
            assert!(item.attack_stat() == Some(2) && item.health_stat() == Some(2))
        }
    }
}

#[test]
fn test_shop_fish_team() {
    let mut team = test_fish_team();
    // Duck stats.
    for pet in team.all().get(1..).unwrap() {
        assert_eq!(pet.borrow().stats, Statistics::new(2, 3).unwrap())
    }
    let fish = team.first().unwrap();
    assert!(fish.borrow().lvl == 1 && fish.borrow().exp == 1);

    // Init seeded shop. Has fish.
    team.set_shop_seed(Some(22221211)).open_shop().unwrap();

    let (shop_fish_pos, item_type) = (Position::Relative(-1), Entity::Pet);
    let found_shop_pets = team
        .shop
        .get_shop_items_by_pos(&shop_fish_pos, &item_type)
        .unwrap();
    // Fish found.
    assert_eq!(
        found_shop_pets.first().unwrap().name(),
        EntityName::Pet(PetName::Fish)
    );
    team.buy(&shop_fish_pos, &item_type, &Position::First)
        .unwrap();

    // Pets gain (1, 1) from fish levelup.
    for pet in team.all().get(1..).unwrap() {
        assert_eq!(pet.borrow().stats, Statistics::new(3, 4).unwrap())
    }
}

#[test]
fn test_shop_otter_team() {
    let mut team = test_duck_team();
    // Seed has otter.
    team.set_shop_seed(Some(432)).open_shop().unwrap();
    // Duck has (2,3)
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(2, 3).unwrap()
    );

    let (otter_pos, item_type) = (Position::Relative(-1), Entity::Pet);
    let pets = team
        .shop
        .get_shop_items_by_pos(&otter_pos, &item_type)
        .unwrap();
    assert_eq!(
        pets.first().unwrap().name(),
        EntityName::Pet(PetName::Otter)
    );
    // Buying otter buffs duck to (3,4)
    team.buy(&otter_pos, &item_type, &Position::Last).unwrap();
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(3, 4).unwrap()
    )
}

#[test]
fn test_shop_pig_team() {
    let mut team = test_pig_team();
    assert_eq!(team.gold(), 10);

    team.open_shop().unwrap();
    team.sell(&Position::First).unwrap();
    assert_eq!(team.gold(), 12);
}

#[test]
fn test_shop_chinchilla_team() {
    let chinchilla_pos = Position::First;
    let mut team = test_chinchilla_team();

    team.open_shop().unwrap();
    team.sell(&Position::First).unwrap();

    // Loyal chinchilla summoned.
    assert_eq!(
        team.first().unwrap().borrow().name,
        PetName::LoyalChinchilla
    );

    team.restore();

    // Levelup chinchilla
    team.set_level(&chinchilla_pos, 2).unwrap();

    team.sell(&chinchilla_pos).unwrap();

    let all_pets = team.all();
    assert!(
        all_pets
            .iter()
            .all(|pet| pet.borrow().name == PetName::LoyalChinchilla)
            && all_pets.len() == 2
    )
}

#[test]
fn test_shop_marmoset_team() {
    let mut team = test_marmoset_team();

    assert_eq!(team.shop.free_rolls, 0);
    team.open_shop().unwrap();
    team.sell(&Position::First).unwrap();

    assert_eq!(team.shop.free_rolls, 1);
}

#[test]
fn test_shop_beetle_team() {
    let mut team = test_beetle_team();

    team.set_shop_seed(Some(432)).open_shop().unwrap();

    let (leftmost_pos, item_type) = (Position::First, Entity::Pet);

    // Get leftmost pet stats.
    let pets = team
        .shop
        .get_shop_items_by_pos(&leftmost_pos, &item_type)
        .unwrap();
    let leftmost_pet = pets.first().unwrap();
    assert!(leftmost_pet.attack_stat() == Some(2) && leftmost_pet.health_stat() == Some(1));
    // Buy the item for the beetle.
    team.buy(&leftmost_pos, &Entity::Food, &leftmost_pos)
        .unwrap();

    // Get leftmost pet stats again. Now has one more attack.
    let pets = team
        .shop
        .get_shop_items_by_pos(&leftmost_pos, &item_type)
        .unwrap();
    let leftmost_pet = pets.first().unwrap();
    assert!(leftmost_pet.attack_stat() == Some(3) && leftmost_pet.health_stat() == Some(1));
}

#[test]
fn test_shop_bluebird_team() {
    let mut team = test_bluebird_team();
    team.set_seed(Some(42));

    // All base stat birds.
    assert!(team
        .all()
        .into_iter()
        .all(|pet| pet.borrow().stats == Statistics::new(2, 1).unwrap()));

    // Set seed so reproducible
    // Open shop and close.
    team.open_shop().unwrap().close_shop().unwrap();

    // Three random (1,0) buffs.
    for (i, pet) in team.all().into_iter().enumerate() {
        let pet_stats = pet.borrow().stats;
        if i == 0 {
            assert_eq!(pet_stats, Statistics::new(4, 1).unwrap());
        } else if i == 1 {
            assert_eq!(pet_stats, Statistics::new(3, 1).unwrap());
        } else {
            assert_eq!(pet_stats, Statistics::new(2, 1).unwrap());
        }
    }
}

#[test]
fn test_shop_ladybug_team() {
    let mut team = test_ladybug_team();

    let original_stats = Statistics::new(1, 3).unwrap();
    for pet in team.all().into_iter() {
        assert_eq!(pet.borrow().stats, original_stats)
    }

    team.set_shop_seed(Some(432))
        .open_shop()
        .unwrap()
        .buy(&Position::First, &Entity::Food, &Position::First)
        .unwrap();

    // Ladybugs gain (2,0) after first ladybug eats honey.
    for pet in team.all().into_iter() {
        assert_eq!(pet.borrow().stats, Statistics::new(3, 3).unwrap())
    }

    // Close and reenter shop.
    team.close_shop().unwrap().open_shop().unwrap();
    // Reverted to original stats.
    for pet in team.all().into_iter() {
        assert_eq!(pet.borrow().stats, original_stats)
    }
}

#[test]
fn test_shop_cockroach_team() {
    let mut team = test_cockroach_team();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(1, 4).unwrap()
    );
    // Open shop for start of turn.
    assert_eq!(team.shop.tier(), 1);
    team.open_shop().unwrap();
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(2, 4).unwrap()
    );
    team.close_shop().unwrap();

    // Set turn to reach tier 2 shop.
    team.history.curr_turn = 3;

    team.open_shop().unwrap();
    assert_eq!(team.shop.tier(), 2);
    // Attack of roach is 2 + 1.
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(3, 4).unwrap()
    );
}

#[test]
fn test_shop_duckling_team() {
    let mut team = test_duckling_team();

    team.set_shop_seed(Some(432)).open_shop().unwrap();

    let (leftmost_pos, item_type) = (Position::First, Entity::Pet);

    // Get leftmost pet stats.
    let pets = team
        .shop
        .get_shop_items_by_pos(&leftmost_pos, &item_type)
        .unwrap();
    let leftmost_pet = pets.first().unwrap();
    assert!(leftmost_pet.attack_stat() == Some(2) && leftmost_pet.health_stat() == Some(1));
    // Sell the duckling.
    team.sell(&Position::First).unwrap();

    // Get leftmost pet stats again. Now has two more health.
    let pets = team
        .shop
        .get_shop_items_by_pos(&leftmost_pos, &item_type)
        .unwrap();
    let leftmost_pet = pets.first().unwrap();
    assert!(leftmost_pet.attack_stat() == Some(2) && leftmost_pet.health_stat() == Some(3));
}

#[test]
fn test_shop_frog_team() {
    let frog_pos = Position::Relative(-1);
    let mut team_frog_sell = test_frog_team();
    let mut team_frog_end_turn = test_frog_team();

    assert_eq!(
        team_frog_sell.nth(0).unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
    // Frilled dragon before activation of ability.
    assert_eq!(
        team_frog_sell.nth(2).unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );

    // Sell
    team_frog_sell
        .set_shop_seed(Some(432))
        .open_shop()
        .unwrap()
        .set_level(&frog_pos, 2)
        .unwrap();
    team_frog_sell.sell(&frog_pos).unwrap();

    assert_eq!(
        team_frog_sell.nth(0).unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );
    // Frilled dragon now has cricket stats.
    assert_eq!(
        team_frog_sell.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );

    // End turn
    team_frog_end_turn
        .open_shop()
        .unwrap()
        .set_level(&frog_pos, 3)
        .unwrap()
        .close_shop()
        .unwrap();

    assert_eq!(
        team_frog_end_turn.nth(0).unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );
    // Frilled dragon now has cricket stats.
    assert_eq!(
        team_frog_end_turn.nth(2).unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
}
