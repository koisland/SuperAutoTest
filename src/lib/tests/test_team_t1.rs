use crate::{
    effects::{effect::EntityName, stats::Statistics, trigger::TRIGGER_START_BATTLE},
    foods::names::FoodName,
    pets::names::PetName,
    teams::{combat::TeamCombat, effects::TeamEffects, team::TeamFightOutcome, viewer::TeamViewer},
    tests::common::{
        count_pets, test_ant_team, test_beaver_team, test_beetle_team, test_bluebird_team,
        test_bulldog_team, test_chinchilla_team, test_chipmunk_team, test_cockroach_team,
        test_cone_snail_team, test_cricket_horse_team, test_duck_team, test_duckling_team,
        test_fish_team, test_frilled_dragon_team, test_frog_team, test_gecko_team, test_goose_team,
        test_groundhog_team, test_hummingbird_team, test_iguana_seahorse_team, test_kiwi_team,
        test_ladybug_team, test_magpie_team, test_marmoset_team, test_mosq_team, test_moth_team,
        test_mouse_team, test_opossum_team, test_pied_tamarin_team, test_pig_team,
        test_pillbug_team, test_silkmoth_team,
    },
    Entity, Food, ItemCondition, Pet, Position, ShopItemViewer, ShopViewer, Team, TeamShopping,
    Toy, ToyName,
};

#[test]
fn test_battle_ant_team() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    let all_2_2 = team
        .friends
        .iter()
        .flatten()
        .all(|pet| pet.read().unwrap().stats == Statistics::new(2, 2).unwrap());
    assert!(all_2_2);

    // One battle phase and one ant faints.
    team.fight(&mut enemy_team).unwrap();

    let any_gets_1_1 = team
        .friends
        .iter()
        .flatten()
        .any(|pet| pet.read().unwrap().stats == Statistics::new(3, 3).unwrap());

    // Another pet gets (1,1).
    assert!(any_gets_1_1)
}

#[test]
fn test_battle_cricket_horse_team() {
    let mut team = test_cricket_horse_team();
    let mut enemy_team = test_cricket_horse_team();

    // First pets are crickets
    // Horse is 3rd pet.
    assert_eq!(team.first().unwrap().read().unwrap().name, PetName::Cricket);
    assert_eq!(team.nth(2).unwrap().read().unwrap().name, PetName::Horse);
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
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
    assert_eq!(
        team.first().unwrap().read().unwrap().name,
        PetName::ZombieCricket
    );
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
}

#[test]
fn test_battle_mosquito_team() {
    let mut team = test_mosq_team();
    let mut enemy_team = Team::new(
        &[
            Some(
                Pet::new(
                    PetName::Ant,
                    Some(Statistics {
                        attack: 2,
                        health: 1,
                    }),
                    1,
                )
                .unwrap(),
            ),
            Some(
                Pet::new(
                    PetName::Ant,
                    Some(Statistics {
                        attack: 2,
                        health: 1,
                    }),
                    1,
                )
                .unwrap(),
            ),
            Some(
                Pet::new(
                    PetName::Ant,
                    Some(Statistics {
                        attack: 2,
                        health: 1,
                    }),
                    1,
                )
                .unwrap(),
            ),
        ],
        5,
    )
    .unwrap();
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
            pet.read().unwrap().stats,
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
        team.last().unwrap().read().unwrap().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );

    team.fight(&mut enemy_team).unwrap();

    // Team has two crickets with faint triggers. Gains (1,1) for each.
    let last_pet = team.all().into_iter().last();
    assert_eq!(
        last_pet.unwrap().read().unwrap().stats,
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

    let cricket = team.first();
    let frilled_dragon = team.nth(2);
    assert_eq!(
        cricket.unwrap().read().unwrap().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
    // Frilled dragon before activation of ability.
    assert_eq!(
        frilled_dragon.as_ref().unwrap().read().unwrap().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );

    // First, frog activates swapping stats of cricket and frilled dragon.
    // Then, Frilled dragon activates.
    // Cricket with 2/2 dies spawning zombie cricket.
    team.fight(&mut enemy_team).unwrap();

    // Frilled dragon gets cricket stats.
    assert_eq!(
        frilled_dragon.as_ref().unwrap().read().unwrap().stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );
}

#[test]
fn test_battle_moth_team() {
    let mut team = test_moth_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );
    // Ant deals 2 dmg. 2 moths gives (4,0).
    team.fight(&mut enemy_team).unwrap();

    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics {
            attack: 6,
            health: 1
        }
    );
}

#[test]
fn test_battle_hummingbird_team() {
    let mut team = test_hummingbird_team();
    let mut enemy_team = test_ant_team();

    let duck = team.first().unwrap();
    assert_eq!(
        duck.read().unwrap().stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );
    // Duck has strawberry.

    assert_eq!(
        duck.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Strawberry
    );
    // Two hummingbirds on team.
    assert_eq!(count_pets(&team.friends, PetName::Hummingbird), 2);
    // Trigger start of battle effects.
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Duck gets 2/1 for every hummingbird since only strawberry friend.
    assert_eq!(
        duck.read().unwrap().stats,
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
    assert_eq!(
        enemy_team.first().unwrap().read().unwrap().name,
        PetName::Cricket
    );
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
        assert_eq!(pet.read().unwrap().stats, Statistics::new(2, 2).unwrap())
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

    // Ants get (2, 0)
    for (_, pet) in team.all().into_iter().enumerate() {
        assert_eq!(pet.read().unwrap().stats, Statistics::new(4, 2).unwrap())
    }
}

#[test]
fn test_shop_duck_team() {
    let mut team = test_duck_team();

    assert_eq!(team.first().unwrap().read().unwrap().name, PetName::Duck);

    team.set_shop_seed(Some(11)).open_shop().unwrap();

    // Search pets.
    let (pet_pos, item_type) = (Position::All(ItemCondition::None), Entity::Pet);
    {
        // Check health before.
        let shop_pets_before = team
            .shop
            .get_shop_items_by_pos(&pet_pos, &item_type)
            .unwrap();

        assert!(
            shop_pets_before.get(0).unwrap().health_stat() == Some(1)
                && shop_pets_before.get(1).unwrap().health_stat() == Some(2)
                && shop_pets_before.get(2).unwrap().health_stat() == Some(3)
        );
    }

    // Sell duck
    team.sell(&Position::First).unwrap();

    // Pets in shop gain +1 health.
    let shop_pets_after = team
        .shop
        .get_shop_items_by_pos(&pet_pos, &item_type)
        .unwrap();

    assert!(
        shop_pets_after.get(0).unwrap().health_stat() == Some(2)
            && shop_pets_after.get(1).unwrap().health_stat() == Some(3)
            && shop_pets_after.get(2).unwrap().health_stat() == Some(4)
    );
}

#[test]
fn test_shop_fish_team() {
    let mut team = test_fish_team();
    // Duck stats.
    for pet in team.all().get(1..).unwrap() {
        assert_eq!(pet.read().unwrap().stats, Statistics::new(2, 3).unwrap())
    }
    let fish = team.first().unwrap();
    assert!(fish.read().unwrap().lvl == 1 && fish.read().unwrap().exp == 1);

    // Init seeded shop. Has fish.
    team.set_shop_seed(Some(109784368244)).open_shop().unwrap();

    let (shop_fish_pos, item_type) = (Position::First, Entity::Pet);
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
        assert_eq!(pet.read().unwrap().stats, Statistics::new(3, 4).unwrap())
    }
}

#[test]
fn test_shop_otter_team() {
    let mut team = test_duck_team();
    // Seed has otter.
    team.set_shop_seed(Some(432)).open_shop().unwrap();
    // Duck has (2,3)
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics::new(2, 3).unwrap()
    );

    let (otter_pos, item_type) = (Position::First, Entity::Pet);
    let pets = team
        .shop
        .get_shop_items_by_pos(&otter_pos, &item_type)
        .unwrap();
    assert_eq!(
        pets.first().unwrap().name(),
        EntityName::Pet(PetName::Otter)
    );
    // Buying otter buffs duck to (2,4)
    team.buy(&otter_pos, &item_type, &Position::Last).unwrap();
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics::new(2, 4).unwrap()
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

    let summoned_pet = team.first().unwrap();
    // Loyal chinchilla summoned.
    assert!(
        summoned_pet.read().unwrap().name == PetName::LoyalChinchilla
            && summoned_pet.read().unwrap().lvl == 1
            && summoned_pet.read().unwrap().stats
                == Statistics {
                    attack: 2,
                    health: 2
                }
    );
    team.restore();

    // Levelup chinchilla
    team.set_level(&chinchilla_pos, 2).unwrap();
    team.sell(&chinchilla_pos).unwrap();
    let summoned_pet = team.first().unwrap();
    assert!(
        summoned_pet.read().unwrap().name == PetName::LoyalChinchilla
            && summoned_pet.read().unwrap().lvl == 2
            && summoned_pet.read().unwrap().stats
                == Statistics {
                    attack: 4,
                    health: 4
                }
    )
}

#[test]
fn test_shop_marmoset_team() {
    let mut team = test_marmoset_team();

    assert_eq!(team.shop.free_rolls, 0);
    team.open_shop().unwrap();
    team.sell(&Position::First).unwrap();

    assert_eq!(team.shop.free_rolls, 1);

    team.roll_shop().unwrap();

    assert_eq!(team.shop.free_rolls, 0);
}

#[test]
fn test_battle_beetle_team() {
    let mut team = test_beetle_team();
    let mut enemy_team = test_ant_team();

    let beetle = team.first().unwrap();
    // Beetle has no item.
    assert!(beetle.read().unwrap().item.is_none());

    team.fight(&mut enemy_team).unwrap();

    // Beetle gains honey at start of battle.
    assert_eq!(
        beetle.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Honey
    );
}

#[test]
fn test_shop_bluebird_team() {
    let mut team = test_bluebird_team();
    team.set_seed(Some(42));

    // All base stat birds.
    assert!(team
        .all()
        .into_iter()
        .all(|pet| pet.read().unwrap().stats == Statistics::new(2, 1).unwrap()));

    // Set seed so reproducible
    // Open shop and close.
    team.open_shop().unwrap().close_shop().unwrap();

    // Three random (1,0) buffs.
    for (i, pet) in team.all().into_iter().enumerate() {
        let pet_stats = pet.read().unwrap().stats;
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

    let ladybug = team.first().unwrap();
    let original_stats = ladybug.read().unwrap().stats;

    team.set_shop_seed(Some(432))
        .open_shop()
        .unwrap()
        .buy(&Position::First, &Entity::Food, &Position::First)
        .unwrap();

    // Ladybug gains (2,0) after eats honey and gains food perk.
    assert_eq!(
        ladybug.read().unwrap().stats,
        original_stats
            + Statistics {
                attack: 2,
                health: 0
            }
    );

    // Close and reenter shop.
    team.close_shop().unwrap().open_shop().unwrap();

    // Reverted to original stats.
    assert_eq!(ladybug.read().unwrap().stats, original_stats);
}

#[test]
fn test_shop_cockroach_team() {
    let mut team = test_cockroach_team();

    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics::new(1, 4).unwrap()
    );
    // Open shop for start of turn.
    assert_eq!(team.shop.tier(), 1);
    team.open_shop().unwrap();
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics::new(2, 4).unwrap()
    );
    team.close_shop().unwrap();

    // Set turn to reach tier 2 shop.
    team.history.curr_turn = 3;

    team.open_shop().unwrap();
    assert_eq!(team.shop.tier(), 2);
    // Attack of roach is 2 + 1.
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
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

    let old_leftmost_pet = pets.first().unwrap();
    let old_leftmost_pet_stats = Statistics {
        attack: old_leftmost_pet.attack_stat().unwrap(),
        health: old_leftmost_pet.health_stat().unwrap(),
    };
    // Sell the duckling.
    team.sell(&Position::First).unwrap();

    // Get leftmost pet stats again. Now has two more health.
    let pets = team
        .shop
        .get_shop_items_by_pos(&leftmost_pos, &item_type)
        .unwrap();
    let curr_leftmost_pet = pets.first().unwrap();
    assert_eq!(
        Statistics {
            attack: curr_leftmost_pet.attack_stat().unwrap(),
            health: curr_leftmost_pet.health_stat().unwrap()
        },
        old_leftmost_pet_stats
            + Statistics {
                attack: 0,
                health: 2
            }
    );
}

#[test]
fn test_shop_frog_team() {
    let frog_pos = Position::Relative(-1);
    let mut team_frog_sell = test_frog_team();
    let mut team_frog_end_turn = test_frog_team();

    assert_eq!(
        team_frog_sell.nth(0).unwrap().read().unwrap().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
    // Frilled dragon before activation of ability.
    assert_eq!(
        team_frog_sell.nth(2).unwrap().read().unwrap().stats,
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
        team_frog_sell.nth(0).unwrap().read().unwrap().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );
    // Frilled dragon now has cricket stats.
    assert_eq!(
        team_frog_sell.nth(2).unwrap().read().unwrap().stats,
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
        team_frog_end_turn.nth(0).unwrap().read().unwrap().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );
    // Frilled dragon now has cricket stats.
    assert_eq!(
        team_frog_end_turn.nth(2).unwrap().read().unwrap().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
}

#[test]
fn test_shop_kiwi_team() {
    let mut team = test_kiwi_team();
    team.open_shop().unwrap();
    team.set_item(
        &Position::Last,
        Some(Food::try_from(FoodName::Strawberry).unwrap()),
    )
    .unwrap();

    let original_stats = team.last().unwrap().read().unwrap().stats;

    team.sell(&Position::First).unwrap();

    // Gains (1,2) from sell.
    assert_eq!(
        team.last().unwrap().read().unwrap().stats,
        original_stats
            + Statistics {
                attack: 1,
                health: 2
            }
    );
}

#[test]
fn test_shop_mouse_team() {
    let mut team = test_mouse_team();
    team.set_shop_seed(Some(121))
        .set_shop_tier(6)
        .unwrap()
        .open_shop()
        .unwrap();

    // Starting shop has two foods.
    assert_eq!(team.len_shop_foods(), 2);

    // Sell mouse.
    team.sell(&Position::First).unwrap();

    // Get one apple but clears shop.
    assert_eq!(team.len_shop_foods(), 1);
    let shop_items = team
        .shop
        .get_shop_items_by_pos(&Position::First, &Entity::Food)
        .unwrap();
    let apple = shop_items[0];
    // Apple is free.
    assert_eq!(apple.cost, 0);
}

#[test]
fn test_shop_pillbug_team() {
    let mut team = test_pillbug_team();
    let pets = team.all();

    // Check health of two pets behind pillbug.
    for pet in pets.get(1..).unwrap() {
        assert_eq!(
            pet.read().unwrap().stats,
            Statistics {
                attack: 2,
                health: 1
            }
        )
    }
    // Upgrade shop.
    team.set_shop_tier(2).unwrap().open_shop().unwrap();
    // Two pets behind pillbug get (0,1) on shop tier upgrade.
    for pet in pets.get(1..).unwrap() {
        assert_eq!(
            pet.read().unwrap().stats,
            Statistics {
                attack: 2,
                health: 2
            }
        )
    }
}

#[test]
fn test_battle_bulldog_team() {
    let mut team = test_bulldog_team();
    let mut enemy_team = test_cockroach_team();

    let bulldog = team.first().unwrap();

    team.fight(&mut enemy_team).unwrap();

    // Bulldog attack now = new health + 1 (lvl)
    assert_eq!(
        bulldog.read().unwrap().stats.attack,
        bulldog.read().unwrap().stats.health + bulldog.read().unwrap().lvl as isize
    );

    team.fight(&mut enemy_team).unwrap();

    // Second attack drops health to 1.
    assert_eq!(
        bulldog.read().unwrap().stats.attack,
        bulldog.read().unwrap().stats.health + bulldog.read().unwrap().lvl as isize
    );
}

#[test]
fn test_shop_chipmunk_team() {
    let mut team = test_chipmunk_team();

    team.set_shop_seed(Some(21))
        .open_shop()
        .unwrap()
        // Give cocnut to chipmunk
        .set_item(
            &Position::First,
            Some(Food::try_from(FoodName::Coconut).unwrap()),
        )
        .unwrap();

    // Shop originally has apple.
    {
        let shop = team.get_shop();
        let first_item = shop.foods.first().unwrap();
        assert_eq!(first_item.name(), EntityName::Food(FoodName::Apple));
        assert_eq!(first_item.cost(), 3)
    }

    team.sell(&Position::First).unwrap();

    // Now has free coconut.
    {
        let shop = team.get_shop();
        let first_item = shop.foods.first().unwrap();
        assert_eq!(first_item.name(), EntityName::Food(FoodName::Coconut));
        assert_eq!(first_item.cost(), 0)
    }
}

#[test]
fn test_battle_groundhog_team() {
    let mut team = test_groundhog_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(Some(0), team.counters.get("Trumpets").copied());
    // One groundhog.
    assert_eq!(team.all().len(), 1);

    team.fight(&mut enemy_team).unwrap();

    // Groundhog faints providing one trumpet which is consumed when it faints.
    assert_eq!(Some(0), team.counters.get("Trumpets").copied());
    let golden_retriever = team.first().unwrap();
    assert_eq!(
        golden_retriever.read().unwrap().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );
}

#[test]
fn test_battle_cone_snail_team() {
    let mut team = test_cone_snail_team();
    let mut enemy_team = test_ant_team();

    let pet_behind_cone_snail = team.nth(1).unwrap();
    let prev_stats = pet_behind_cone_snail.read().unwrap().stats;

    // Trigger start of battle effects.
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    let new_stats = pet_behind_cone_snail.read().unwrap().stats;
    let stat_change = Statistics {
        attack: 0,
        health: 2,
    };
    // Pet behind cone snail gains 2 health.
    assert_eq!(new_stats, prev_stats + stat_change)
}

#[test]
fn test_battle_goose_team() {
    let mut team = test_goose_team();
    let mut enemy_team = test_ant_team();

    let first_enemy = enemy_team.first().unwrap();
    let prev_stats = first_enemy.read().unwrap().stats;

    // Trigger start of battle effects.
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    let new_stats = first_enemy.read().unwrap().stats;
    let stat_change = Statistics {
        attack: 1,
        health: 0,
    };
    // First enemy pet loses 1 atk.
    assert_eq!(new_stats, prev_stats - stat_change)
}

#[test]
fn test_battle_pied_tamarin_team() {
    let mut team = test_pied_tamarin_team();
    let mut enemy_team = Team::new(
        &vec![
            Some(Pet::try_from(PetName::Duck).unwrap()),
            Some(Pet::try_from(PetName::Duck).unwrap()),
        ],
        5,
    )
    .unwrap();

    // First attack kills groundhog providing one trumpet.
    team.fight(&mut enemy_team).unwrap();

    assert_eq!(team.counters.get("Trumpets").copied(), Some(1));
    // Two ducks remaining.
    assert_eq!(enemy_team.all().len(), 2);

    // Tamarin and duck fight.
    team.fight(&mut enemy_team).unwrap();
    // Pied tamarin faints, consumed one trumpet and range attacks remaining duck.
    assert!(enemy_team.all().is_empty())
}

#[test]
fn test_shop_opossum_team() {
    let mut team = test_opossum_team();

    team.set_shop_seed(Some(131)).open_shop().unwrap();

    // Ant is only faint pet in shop.
    let original_stats = {
        let shop = team.get_shop();
        let faint_pet = shop.pets.first().unwrap();
        Statistics {
            attack: faint_pet.attack_stat().unwrap(),
            health: faint_pet.health_stat().unwrap(),
        }
    };

    // Sell opossum.
    team.sell(&Position::First).unwrap();

    // Ant gains (1,1)
    let shop = team.get_shop();
    let faint_pet = shop.pets.first().unwrap();
    assert_eq!(
        Statistics {
            attack: faint_pet.attack_stat().unwrap(),
            health: faint_pet.health_stat().unwrap()
        },
        original_stats
            + Statistics {
                attack: 1,
                health: 1
            }
    )
}

#[test]
fn test_battle_silkmoth_team() {
    let mut team = test_silkmoth_team();
    let mut enemy_team = test_ant_team();

    let pet_ahead_silkmoth = team.first().unwrap();
    let original_stats = pet_ahead_silkmoth.read().unwrap().stats;

    team.fight(&mut enemy_team).unwrap();

    // Stats identical because silkmoth restores pets health lost from ant attack.
    assert_eq!(original_stats, pet_ahead_silkmoth.read().unwrap().stats);
}

#[test]
fn test_shop_magpie_team() {
    let mut team = test_magpie_team();

    let roll_n_times = |n: usize, team: &mut Team| {
        for _ in 0..n {
            team.roll_shop().unwrap();
        }
    };

    // First round.
    team.open_shop().unwrap();
    assert_eq!(team.gold(), 10);

    // One gold left.
    roll_n_times(9, &mut team);
    assert_eq!(team.gold(), 1);
    team.close_shop().unwrap();

    // Next round.
    // Save one gold from previous round.
    team.open_shop().unwrap();
    assert_eq!(team.gold(), 11);

    // Roll 9 times leaving 2 gold.
    roll_n_times(9, &mut team);
    assert_eq!(team.gold(), 2);
    team.close_shop().unwrap();

    // Only save up to 1 gold as magpie lvl 1.
    team.open_shop().unwrap();
    assert_eq!(team.gold(), 11);
}

#[test]
fn test_battle_gecko_team() {
    let mut team = test_gecko_team();
    let mut enemy_team = test_ant_team();

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    let gecko = team.first().unwrap();
    let gecko_start_stats = gecko.read().unwrap().stats;
    const GECKO_BUFF: Statistics = Statistics {
        attack: 0,
        health: 2,
    };

    // Gecko gains no buff as no toy.
    assert_eq!(gecko.read().unwrap().stats, gecko_start_stats);

    team.restore();

    // Add a toy.
    team.toys.push(Toy::try_from(ToyName::Dice).unwrap());

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    // Gecko gains buff w/toy.
    assert_eq!(gecko.read().unwrap().stats, gecko_start_stats + GECKO_BUFF);
}
