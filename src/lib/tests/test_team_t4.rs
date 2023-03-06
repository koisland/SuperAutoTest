use std::rc::Rc;

use crate::{
    effects::{
        actions::{Action, CopyType, SummonType},
        effect::Entity,
        state::{Position, Target},
        stats::Statistics,
        trigger::*,
    },
    foods::names::FoodName,
    pets::names::PetName,
    teams::{combat::TeamCombat, effects::TeamEffects, team::TeamFightOutcome, viewer::TeamViewer},
    tests::common::{
        count_pets, test_ant_team, test_anteater_team, test_armadillo_team, test_bison_team,
        test_buffalo_team, test_caterpillar_team, test_crow_team, test_deer_team,
        test_doberman_highest_tier_team, test_doberman_team, test_donkey_team, test_dragonfly_team,
        test_eel_team, test_gorilla_team, test_hawk_team, test_hippo_team, test_jerboa_team,
        test_llama_team, test_lobster_team, test_lynx_team, test_mosq_team, test_orangutan_team,
        test_ox_team, test_parrot_team, test_pelican_team, test_penguin_team, test_platypus_team,
        test_porcupine_team, test_praying_mantis_team, test_rooster_team, test_skunk_team,
        test_snake_team, test_squirrel_team, test_turtle_team, test_whale_team, test_worm_team,
    },
    Effect, EntityName, ItemCondition, Pet, Shop, ShopItem, ShopItemViewer, ShopViewer, Team,
    TeamShopping,
};

#[test]
fn test_battle_deer_team() {
    let mut team = test_deer_team();
    let mut enemy_team = test_ox_team();

    // Only one deer.
    assert!(team.first().unwrap().borrow().name == PetName::Deer && team.all().len() == 1);
    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();

    // 1st attack kills dear and summons bus.
    // 2nd attack kills dog and ox before its effect triggers.
    // After completion, only bus remains with 2 health.
    let bus = team.any().unwrap();
    assert!(
        bus.borrow().name == PetName::Bus
            && bus.borrow().stats.health == 2
            && team.all().len() == 1
    );
    assert_eq!(enemy_team.fainted.len(), 2)
}

#[test]
fn test_battle_hippo_team() {
    let mut team = test_hippo_team();
    let mut enemy_team = test_ant_team();

    // Only one lvl.1 hippo.
    let hippo = team.first().unwrap();
    assert!(
        hippo.borrow().name == PetName::Hippo
            && hippo.borrow().lvl == 1
            && hippo.borrow().stats
                == Statistics {
                    attack: 4,
                    health: 5
                }
            && team.all().len() == 1,
    );
    // Versus three lvl.1 ants.
    assert!(enemy_team
        .all()
        .iter()
        .all(|pet| pet.borrow().name == PetName::Ant));
    let mut fight = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team).unwrap()
    }

    // Hippo takes 2 dmg (1st) + 8 dmg (2nd + 3rd) = 10 dmg
    // Hippo kills all three gaining (9,9) + base (4,5) - dmg (0,10) to (13,4)
    assert!(
        team.first().unwrap().borrow().stats
            == Statistics {
                attack: 13,
                health: 4
            }
    );
}

#[test]
fn test_shop_parrot_team() {
    let mut team = test_parrot_team();

    // Before end of turn, is def parrot effect.
    let parrot = team.nth(1).unwrap();
    assert_eq!(
        vec![Effect {
            owner: Some(Rc::downgrade(&parrot)),
            entity: Entity::Pet,
            trigger: TRIGGER_END_TURN.clone().set_affected(&parrot).to_owned(),
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Copy(
                CopyType::Effect(vec![], Some(1)),
                Target::Friend,
                Position::Nearest(1),
            ),
            uses: None,
            temp: true,
        }],
        team.nth(1).unwrap().borrow().effect
    );
    // Cricket is level 2.
    assert_eq!(team.first().unwrap().borrow().lvl, 2);

    // Open and then close shop. Creates end of turn trigger.
    team.open_shop().unwrap().close_shop().unwrap();

    // After the parrot's effects is a level one cricket.
    // Update id and affected pet to match.
    let mut updated_trigger = TRIGGER_SELF_FAINT;
    let mut zombie_cricket = Pet::try_from(PetName::ZombieCricket).unwrap();
    updated_trigger.set_affected(&team.nth(1).unwrap());
    zombie_cricket.id = None;

    assert_eq!(
        vec![Effect {
            owner: None,
            trigger: updated_trigger,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Summon(SummonType::StoredPet(Box::new(zombie_cricket))),
            uses: Some(1),
            entity: Entity::Pet,
            temp: false,
        }],
        team.nth(1).unwrap().borrow().effect
    );
}

#[test]
fn test_battle_rooster_lvl_1_team() {
    let mut team = test_rooster_team();
    let mut enemy_team = test_rooster_team();
    {
        let rooster = team.first().unwrap();
        assert!(
            rooster.borrow().name == PetName::Rooster
                && rooster.borrow().stats
                    == Statistics {
                        attack: 5,
                        health: 3
                    }
        )
    }

    team.fight(&mut enemy_team).unwrap();

    let chick = team.first().unwrap();
    // 50% of base lvl.1 rooster is 3 (2.5). Health is 1.
    assert!(
        chick.borrow().name == PetName::Chick
            && chick.borrow().stats
                == Statistics {
                    attack: 3,
                    health: 1
                }
    )
}

#[test]
fn test_battle_rooster_lvl_2_team() {
    let mut team = test_rooster_team();
    let mut enemy_team = test_rooster_team();
    {
        team.set_level(&Position::First, 2).unwrap();
        let rooster = team.first().unwrap();
        // Level 2 now. Will spawn 2 chicks.
        assert!(
            rooster.borrow().name == PetName::Rooster
                && rooster.borrow().stats
                    == Statistics {
                        attack: 5,
                        health: 3
                    }
                && rooster.borrow().lvl == 2
        )
    }

    team.fight(&mut enemy_team).unwrap();

    let chick = team.first().unwrap();
    let chick_2 = team.nth(1).unwrap();
    // 50% of base lvl.1 rooster is 3 (2.5). Health is 1.
    assert!(
        chick.borrow().name == PetName::Chick
            && chick.borrow().stats
                == Statistics {
                    attack: 3,
                    health: 1
                }
    );
    assert!(
        chick_2.borrow().name == PetName::Chick
            && chick_2.borrow().stats
                == Statistics {
                    attack: 3,
                    health: 1
                }
    )
}

#[test]
fn test_battle_skunk_team() {
    let mut team = test_skunk_team();
    let mut enemy_team = test_skunk_team();

    // Lvl. 1 skunks on both teams.
    assert!(
        team.first().unwrap().borrow().lvl == 1 && enemy_team.first().unwrap().borrow().lvl == 1,
    );
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 5
        }
    );
    assert_eq!(
        enemy_team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 5
        }
    );

    // Apply start of battle effects. No fighting.
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();
    enemy_team
        .trigger_effects(&TRIGGER_START_BATTLE, Some(&mut team))
        .unwrap();

    // Health reduced by 33% (2) from 5 -> 3.
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
    assert_eq!(
        enemy_team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
}

#[test]
fn test_battle_turtle_team() {
    let mut team = test_turtle_team();
    let mut enemy_team = test_turtle_team();

    assert_eq!(team.nth(1).unwrap().borrow().item, None);

    // Three attacks to kill both lvl.1 turtles.
    for _ in 0..3 {
        team.fight(&mut enemy_team).unwrap();
    }

    let pet_behind_turtle = team.first().unwrap();
    assert_eq!(
        pet_behind_turtle.borrow().item.as_ref().unwrap().name,
        FoodName::Melon
    );
}

#[test]
fn test_battle_whale_team() {
    let mut team = test_whale_team();
    let mut enemy_team = test_hippo_team();

    // Only one cricket at start.
    assert_eq!(count_pets(&team.friends, PetName::Cricket), 1);
    // Copy cricket for comparison and set idx of trigger + owner of effect to None. (Reset on swallow)
    for effect in team.first().unwrap().borrow_mut().effect.iter_mut() {
        effect.assign_owner(None);
    }
    let cricket_copy = team.first().unwrap().borrow().clone();

    team.fight(&mut enemy_team).unwrap();

    // After start of battle, whale eats cricket and changes effect to summon cricket.
    let whale = team.first().unwrap();
    let whale_effect = &whale.borrow().effect;

    assert_eq!(
        whale_effect.first().unwrap().action,
        Action::Summon(SummonType::StoredPet(Box::new(cricket_copy)))
    );
}

#[test]
fn test_battle_armadillo_team() {
    let mut team = test_armadillo_team();
    let mut enemy_team = test_hippo_team();

    team.fight(&mut enemy_team).unwrap();

    for (i, pet) in team.all().into_iter().enumerate() {
        // First pet is armadillo, it takes (2,6)-(0,4).
        // It doesn't gain (0,1) but all dogs do.
        if i == 0 {
            assert_eq!(pet.borrow().stats, Statistics::new(2, 2).unwrap())
        } else {
            assert_eq!(pet.borrow().stats, Statistics::new(3, 5).unwrap())
        }
    }
}

#[test]
fn test_battle_doberman_team() {
    let mut team = test_doberman_team();
    let mut enemy_team = test_hippo_team();

    // Doberman has no item.
    assert_eq!(team.first().unwrap().borrow().item, None);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(4, 5).unwrap()
    );
    // Doberman is lowest tier.
    assert_eq!(
        team.all()
            .iter()
            .min_by(|pet_1, pet_2| pet_1.borrow().tier.cmp(&pet_2.borrow().tier))
            .unwrap()
            .borrow()
            .name,
        PetName::Doberman
    );
    team.fight(&mut enemy_team).unwrap();

    // Doberman gets coconut and gets (5,5)
    let doberman = team.first().unwrap();
    assert_eq!(
        doberman.borrow().item.as_ref().unwrap().name,
        FoodName::Coconut
    );
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(9, 10).unwrap()
    );
}

#[test]
fn test_battle_doberman_highest_tier_team() {
    let mut team = test_doberman_highest_tier_team();
    let mut enemy_team = test_hippo_team();

    // Doberman has no item.
    assert_eq!(team.first().unwrap().borrow().item, None);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(4, 5).unwrap()
    );
    // Doberman is not lowest tier.
    assert_ne!(
        team.all()
            .iter()
            .min_by(|pet_1, pet_2| pet_1.borrow().tier.cmp(&pet_2.borrow().tier))
            .unwrap()
            .borrow()
            .name,
        PetName::Doberman
    );
    team.fight(&mut enemy_team).unwrap();

    // Doberman doesn't get coconut or stats.
    assert_eq!(team.first().unwrap().borrow().item, None);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(4, 1).unwrap()
    );
}

#[test]
fn test_battle_lynx_team() {
    let mut team = test_lynx_team();
    let mut enemy_team = test_hippo_team();

    let enemy_hippo = enemy_team.first().unwrap();
    // 5 levels on team. So 5 dmg.
    assert_eq!(
        team.all().iter().map(|pet| pet.borrow().lvl).sum::<usize>(),
        5
    );

    team.fight(&mut enemy_team).unwrap();

    // Hippo faints at start of battle.
    assert_eq!(enemy_hippo.borrow().stats, Statistics::new(4, 0).unwrap());

    team.restore();
    enemy_team.restore();

    // Remove one level one pet.
    team.friends.pop();
    assert_eq!(
        team.all().iter().map(|pet| pet.borrow().lvl).sum::<usize>(),
        4
    );

    // Retrigger start of battle effects
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Hippo takes 4 dmg.
    assert_eq!(
        enemy_team.first().unwrap().borrow().stats,
        Statistics::new(4, 1).unwrap()
    );
}

#[test]
fn test_battle_porcupine_team() {
    let mut team = test_porcupine_team();
    let mut enemy_team = test_mosq_team();

    // Buff 1st mosquito so survives first returned attack.
    enemy_team.first().unwrap().borrow_mut().stats.health += 8;

    // Trigger start of battle effects. Then clear fainted pets.
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();
    enemy_team.clear_team();

    // 2 Mosquitoes faint from returned fire from porcupine
    assert_eq!(enemy_team.fainted.len(), 2);
    // 1 mosquito that was buffed survives.
    assert!(
        enemy_team.all().len() == 1
            && enemy_team.first().unwrap().borrow().stats == Statistics::new(2, 4).unwrap()
    );

    // Continue fight.
    team.fight(&mut enemy_team).unwrap();

    // 1st mosquito faints from direct damage + returned porcupine damage.
    assert_eq!(enemy_team.fainted.len(), 3);
}

#[test]
fn test_battle_caterpillar_team() {
    let mut team = test_caterpillar_team();
    let mut enemy_team = test_hippo_team();
    let caterpillar = team.first().unwrap();
    assert_eq!(caterpillar.borrow().stats, Statistics::new(2, 2).unwrap());
    assert_eq!(caterpillar.borrow().name, PetName::Caterpillar);

    // Right before battle phase, butterfly will copy effect.
    team.fight(&mut enemy_team).unwrap();

    // Butterfly takes 4 dmg from hippo but copied (50/50) dog.
    let butterfly = team.first().unwrap();
    assert_eq!(butterfly.borrow().stats, Statistics::new(50, 46).unwrap());
    assert_eq!(butterfly.borrow().name, PetName::Butterfly);
}

#[test]
fn test_shop_caterpillar_team() {
    let mut team = Team::new(&[Some(Pet::try_from(PetName::Caterpillar).unwrap())], 5).unwrap();

    let caterpillar = team.first().unwrap();
    // Starts with lvl 1 and no exp.
    assert!(caterpillar.borrow().exp == 0 && caterpillar.borrow().lvl == 1);

    team.open_shop().unwrap();

    let caterpillar = team.first().unwrap();
    // Gains 1 exp on start of turn.
    assert!(caterpillar.borrow().exp == 1 && caterpillar.borrow().lvl == 1)
}

#[test]
fn test_battle_anteater_team() {
    let mut team = test_anteater_team();
    let mut enemy_team = test_hippo_team();

    // Single anteater.
    let anteater = team.first();
    assert_eq!(team.all().len(), 1);
    assert_eq!(anteater.as_ref().unwrap().borrow().name, PetName::Anteater);
    team.fight(&mut enemy_team).unwrap();

    // After faint, two anteaters spawn.
    assert_eq!(team.fainted.first().unwrap(), &anteater);
    let all_friends = team.all();
    assert_eq!(all_friends.len(), 2);
    assert!(all_friends
        .iter()
        .all(|pet| pet.borrow().name == PetName::Ant))
}

#[test]
fn test_battle_donkey_team() {
    let mut team = test_donkey_team();
    let mut enemy_team = test_snake_team();
    team.set_seed(Some(2));

    assert_eq!(enemy_team.nth(1).unwrap().borrow().name, PetName::Snake);

    team.fight(&mut enemy_team).unwrap();

    // Cricket faints and donkey ability triggers.
    assert_eq!(enemy_team.fainted.len(), 1);
    // Snake pushed to front.
    assert_eq!(enemy_team.first().unwrap().borrow().name, PetName::Snake);
    // And zombie cricket now in back.
    assert_eq!(
        enemy_team.nth(1).unwrap().borrow().name,
        PetName::ZombieCricket
    )
}

#[test]
fn test_battle_eel_team() {
    let mut team = test_eel_team();
    let mut enemy_team = test_hippo_team();

    let eel_stats = team.first().unwrap().borrow().stats;

    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Eel at lvl.1 gains 50% of original health.
    assert_eq!(
        eel_stats + Statistics::new(0, eel_stats.health / 2).unwrap(),
        team.first().unwrap().borrow().stats
    )
}

#[test]
fn test_battle_hawk_team() {
    let mut team = test_hawk_team();
    let mut enemy_team = test_gorilla_team();

    // Hawk on 1st position.
    assert_eq!(team.first().unwrap().borrow().name, PetName::Hawk);
    {
        let gorilla_on_1st = enemy_team.first().unwrap();
        assert_eq!(
            gorilla_on_1st.borrow().stats,
            Statistics {
                attack: 6,
                health: 9
            }
        );
    }

    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    {
        // Gorilla takes 7 dmg.
        let gorilla_on_1st = enemy_team.first().unwrap();
        assert_eq!(
            gorilla_on_1st.borrow().stats,
            Statistics {
                attack: 6,
                health: 2
            }
        );
    }
}

#[test]
fn test_battle_pelican_team() {
    let mut team = test_pelican_team();
    let mut enemy_team = test_hippo_team();

    {
        // Ant has strawberry.
        let ant = team.nth(1).unwrap();

        assert_eq!(ant.borrow().stats, Statistics::new(2, 1).unwrap());
        let ant_item = &ant.borrow().item;
        assert_eq!(ant_item.as_ref().unwrap().name, FoodName::Strawberry)
    }

    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Pelican at lvl.1 give strawberry ant (2,1)
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics::new(4, 2).unwrap()
    );
}

#[test]
fn test_shop_bison_team() {
    let mut team = test_bison_team();
    // Lvl 3 duck on team.
    assert_eq!(team.first().unwrap().borrow().lvl, 3);
    assert_eq!(
        team.last().unwrap().borrow().stats,
        Statistics {
            attack: 5,
            health: 3
        }
    );
    team.open_shop().unwrap().close_shop().unwrap();

    let exp_bison_stats = Statistics {
        attack: 7,
        health: 5,
    };
    // Bison gains (2,2)
    assert_eq!(team.last().unwrap().borrow().stats, exp_bison_stats);

    // Sell lvl 3 friend
    team.open_shop()
        .unwrap()
        .sell(&Position::First)
        .unwrap()
        .close_shop()
        .unwrap();

    // Stats don't change at end of turn anymore.
    assert_eq!(team.last().unwrap().borrow().stats, exp_bison_stats);
}

#[test]
fn test_shop_penguin_team() {
    let mut team = test_penguin_team();

    let lvl_3_duck = team.first().unwrap();
    // Base stats and level of duck.
    assert_eq!(lvl_3_duck.borrow().lvl, 3);
    assert_eq!(
        lvl_3_duck.borrow().stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );

    team.open_shop().unwrap().close_shop().unwrap();

    // Duck gets (1,1)
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
}

#[test]
fn test_shop_squirrel_team() {
    let mut team = test_squirrel_team();

    // Default tier 1 shop has 1 max food slots.
    assert_eq!(team.shop.max_food_slots(), 1);
    // Open shop and squirrel effect activates.
    team.open_shop().unwrap();

    // Two items in shop despite max food slot of 1. Both are discounted.
    assert_eq!(team.len_shop_foods(), 2);
    assert!(team
        .shop
        .get_shop_items_by_pos(&Position::All(ItemCondition::None), &Entity::Food)
        .unwrap()
        .iter()
        .all(|item| item.cost == 2))
}

#[test]
fn test_shop_worm_team() {
    let mut team = test_worm_team();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(3, 3).unwrap()
    );
    team.set_shop_seed(Some(12))
        .open_shop()
        .unwrap()
        .buy(&Position::First, &Entity::Food, &Position::First)
        .unwrap();

    let worm = team.first().unwrap();
    // Worm gets (1,1) from eating honey.
    assert!(
        worm.borrow().stats == Statistics::new(4, 4).unwrap()
            && worm.borrow().item.as_ref().unwrap().name == FoodName::Honey
    );
}

#[test]
fn test_shop_dragonfly_team() {
    let mut team = test_dragonfly_team();
    team.set_seed(Some(12));
    team.open_shop().unwrap();

    let starting_duck_stats = Statistics {
        attack: 2,
        health: 3,
    };
    let starting_dog_stats = Statistics {
        attack: 3,
        health: 4,
    };
    const DRAGONFLY_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };
    let (duck, dog) = (team.first().unwrap(), team.last().unwrap());
    assert!(
        duck.borrow().lvl == 3
            && duck.borrow().stats == starting_duck_stats
            && dog.borrow().lvl == 1
            && dog.borrow().stats == starting_dog_stats
    );

    team.close_shop().unwrap();

    let duck_stats_after_end_turn = starting_duck_stats + DRAGONFLY_BUFF;
    let dog_stats_after_end_turn = starting_dog_stats + DRAGONFLY_BUFF;
    assert!(
        duck.borrow().stats == duck_stats_after_end_turn
            && dog.borrow().stats == dog_stats_after_end_turn
    );

    team.open_shop().unwrap();

    let (duck, dog) = (team.first().unwrap(), team.last().unwrap());
    // Set duck level to 1.
    team.set_level(&Position::First, 1)
        .unwrap()
        .close_shop()
        .unwrap();

    // Only dog gets buff.
    assert!(
        duck.borrow().stats == duck_stats_after_end_turn
            && dog.borrow().stats == dog_stats_after_end_turn + DRAGONFLY_BUFF
    );
}

#[test]
fn test_shop_jerboa_team() {
    let mut team = test_jerboa_team();

    team.set_shop_seed(Some(121)).open_shop().unwrap();

    let pets = team.all();
    let (duck, jerboa, dog) = (
        pets.first().unwrap(),
        pets.get(1).unwrap(),
        pets.get(2).unwrap(),
    );
    let (duck_start_stats, jerboa_start_stats, dog_start_stats) = (
        duck.borrow().stats,
        jerboa.borrow().stats,
        dog.borrow().stats,
    );
    const JERBOA_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };

    // Apple in shop at first position.
    assert_eq!(
        team.shop.foods.first().unwrap().name(),
        EntityName::Food(FoodName::Apple)
    );
    team.buy(&Position::First, &Entity::Food, &Position::Relative(-1))
        .unwrap()
        // Set seed so different items.
        .set_shop_seed(Some(12))
        .roll_shop()
        .unwrap();

    // Check stats after purchase.
    let check_stats = move || {
        // Jerboa gets (1,1) from apple but not from own ability.
        assert_eq!(
            jerboa_start_stats
                + Statistics {
                    attack: 1,
                    health: 1
                },
            jerboa.borrow().stats
        );
        // Jerboa buffs (1,1) all pets when apple eaten.
        assert_eq!(duck_start_stats + JERBOA_BUFF, duck.borrow().stats);
        assert_eq!(dog_start_stats + JERBOA_BUFF, dog.borrow().stats);
    };

    check_stats();

    // Now honey in first.
    assert_eq!(
        team.shop.foods.first().unwrap().name(),
        EntityName::Food(FoodName::Honey)
    );
    team.buy(&Position::First, &Entity::Food, &Position::Relative(-1))
        .unwrap();

    // No stat changes when placed on jerboa.
    check_stats();
}

#[test]
fn test_shop_mole_team() {
    let pets = [
        Some(Pet::try_from(PetName::Ant).unwrap()),
        None,
        Some(Pet::try_from(PetName::Ant).unwrap()),
        Some(Pet::try_from(PetName::Ant).unwrap()),
    ];
    let mut team = Team::new(&pets, 5).unwrap();

    // Create shop with mole inside.
    let mut shop = Shop::default();
    shop.add_item(ShopItem::from(Pet::try_from(PetName::Mole).unwrap()))
        .unwrap();

    // replace shop.
    team.replace_shop(shop).unwrap().open_shop().unwrap();

    // Ants at 0 and 2 position.
    let (ant_1, ant_2, ant_3) = (
        team.nth(0).unwrap(),
        team.nth(2).unwrap(),
        team.nth(3).unwrap(),
    );
    let (ant_1_start_stats, ant_2_start_stats, ant_3_start_stats) = (
        ant_1.borrow().stats,
        ant_2.borrow().stats,
        ant_3.borrow().stats,
    );
    const MOLE_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };
    team.buy(&Position::First, &Entity::Pet, &Position::Relative(-1))
        .unwrap();

    // Ants adjacent to mole get stats.
    assert_eq!(ant_1_start_stats + MOLE_BUFF, ant_1.borrow().stats);
    assert_eq!(ant_2_start_stats + MOLE_BUFF, ant_2.borrow().stats);
    // Ant not adjacent gets nothing.
    assert_eq!(ant_3_start_stats, ant_3.borrow().stats);
}

#[test]
fn test_shop_buffalo_team() {
    let mut team = test_buffalo_team();
    team.set_shop_seed(Some(11)).open_shop().unwrap();

    let buffalo = team.first().unwrap();
    let buffalo_start_stats = buffalo.borrow().stats;
    const BUFF_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };

    // Buy three pets.
    for i in 1..=3 {
        team.buy(&Position::First, &Entity::Pet, &Position::First)
            .unwrap();
        // Get (1,1) for every pet purchased.
        let stat_increase = BUFF_BUFF * Statistics::new(i, i).unwrap();
        assert_eq!(buffalo_start_stats + stat_increase, buffalo.borrow().stats);
    }
    let final_buffalo_stats = buffalo.borrow().stats;

    // Add some gold to allow buying a 4th pet.
    team.shop.coins += 3;
    team.roll_shop()
        .unwrap()
        .buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap();

    // No additional stats on buy as limit reached.
    assert_eq!(final_buffalo_stats, buffalo.borrow().stats);
}

#[test]
fn test_shop_llama_team() {
    let mut team = test_llama_team();

    team.open_shop().unwrap();

    let llama = team.first().unwrap();

    let llama_start_stats = llama.borrow().stats;
    const LLAMA_BUFF: Statistics = Statistics {
        attack: 2,
        health: 2,
    };

    team.close_shop().unwrap();

    // Llama gets (2,2)
    assert_eq!(llama_start_stats + LLAMA_BUFF, llama.borrow().stats);

    team.open_shop().unwrap();

    // Drop invalid reference created on opening shop/restoring team.
    std::mem::drop(llama);

    let llama = team.first().unwrap();
    let llama_new_start_stats = llama.borrow().stats;
    // Add four pets to fill the team.
    for _ in 0..4 {
        team.add_pet(Pet::try_from(PetName::Ant).unwrap(), 0, None)
            .unwrap();
    }
    // Filled team.
    assert_eq!(team.friends.len(), team.max_size);

    team.close_shop().unwrap();
    // No change in stats as team is filled and no empty space.
    assert_eq!(llama.borrow().stats, llama_new_start_stats);
}

#[test]
fn test_shop_lobster_team() {
    let mut team = test_lobster_team();

    team.set_shop_seed(Some(12)).open_shop().unwrap();
    // Buy the first pet in the shop. A mosquito.
    team.buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap();

    const LOBSTER_BUFF: Statistics = Statistics {
        attack: 2,
        health: 2,
    };
    let base_mosq = Pet::try_from(PetName::Mosquito).unwrap();
    let mosq = team.first().unwrap();

    // Mosquito summoned to team and buffed by lobster.
    assert_eq!(mosq.borrow().stats, base_mosq.stats + LOBSTER_BUFF);
}

#[test]
fn test_battle_lobster_team() {
    let mut team = test_lobster_team();
    // Create enemy team.
    let mut enemy_team = Team::new(
        &[Some(
            Pet::new(PetName::Hippo, Some(Statistics::new(50, 50).unwrap()), 3).unwrap(),
        )],
        5,
    )
    .unwrap();

    team.fight(&mut enemy_team).unwrap();

    let zombie_cricket = team.first().unwrap();
    let base_zombie_cricket = Pet::try_from(PetName::ZombieCricket).unwrap();
    // Zombie cricket summoned has base stats despite summon. Lobster effect only procs in shop.
    assert_eq!(zombie_cricket.borrow().stats, base_zombie_cricket.stats);
}

#[test]
fn test_shop_crow_team() {
    let mut team = test_crow_team();

    team.set_shop_tier(6).unwrap().open_shop().unwrap();
    // Two items in shop
    assert_eq!(team.shop.len_foods(), 2);

    // Set crow level to 2. Then sell it.
    team.set_level(&Position::First, 2)
        .unwrap()
        .sell(&Position::First)
        .unwrap();

    let new_items = team
        .shop
        .get_shop_items_by_pos(&Position::First, &Entity::Food)
        .unwrap();
    // New item in shop is only chocolate.
    assert_eq!(team.shop.len_foods(), 1);
    assert_eq!(
        new_items.first().unwrap().name(),
        EntityName::Food(FoodName::Chocolate)
    );

    // Ant on team is base exp and lvl.
    team.clear_team();
    let ant = team.nth(1).unwrap();
    assert!(ant.borrow().lvl == 1 && ant.borrow().exp == 0);

    // Buy chocolate for it.
    team.buy(&Position::First, &Entity::Food, &Position::First)
        .unwrap();
    // Ant gains 2 exp and levels up because crow is lvl 2.
    assert!(ant.borrow().lvl == 2 && ant.borrow().exp == 2);
}

#[test]
fn test_shop_platypus_team() {
    let mut team = test_platypus_team();
    // Only platypus on team.
    assert_eq!(team.friends.len(), 1);

    team.open_shop().unwrap().sell(&Position::First).unwrap();
    team.clear_team();
    // Two pets after selling platypus: duck and a beaver are spawned.
    assert_eq!(team.friends.len(), 2);
    let (duck, beaver) = (team.first().unwrap(), team.last().unwrap());
    // Both are level 1.
    assert!(
        duck.borrow().name == PetName::Duck
            && duck.borrow().lvl == 1
            && beaver.borrow().name == PetName::Beaver
            && beaver.borrow().lvl == 1
    );
}

#[test]
fn test_shop_praying_mantis_team() {
    let mut team = test_praying_mantis_team();
    // Three pets on team. One in middle is mantis
    assert_eq!(team.friends.len(), 3);
    let mantis_start_stats = team.nth(1).unwrap().borrow().stats;
    const MANTIS_BUFF: Statistics = Statistics {
        attack: 2,
        health: 2,
    };
    team.open_shop().unwrap();
    // Two adjacent friends now dead.
    let pets = team.all();
    assert_eq!(pets.len(), 1);
    // Mantis gains (2,2)
    assert_eq!(
        pets.first().unwrap().borrow().stats,
        mantis_start_stats + MANTIS_BUFF
    );
}

#[test]
fn test_shop_orangutan_team() {
    let mut team = test_orangutan_team();

    team.open_shop().unwrap();

    // Lowest health pet is ant.
    let found_pets = team.get_pets_by_cond(&ItemCondition::Illest);
    let lowest_health_pet = found_pets.first().unwrap();
    assert_eq!(lowest_health_pet.borrow().name, PetName::Ant);

    let ant_start_stats = lowest_health_pet.borrow().stats;
    const ORANGUTAN_BUFF: Statistics = Statistics {
        attack: 0,
        health: 4,
    };

    team.close_shop().unwrap();
    // Ant gains buff because lowest health.
    assert_eq!(
        lowest_health_pet.borrow().stats,
        ant_start_stats + ORANGUTAN_BUFF
    );
}
