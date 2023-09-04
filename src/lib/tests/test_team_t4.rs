use std::sync::Arc;

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
        test_blowfish_team, test_buffalo_team, test_caterpillar_team, test_crow_team,
        test_deer_team, test_doberman_highest_tier_team, test_doberman_team, test_donkey_team,
        test_dragonfly_team, test_eel_team, test_gorilla_team, test_hawk_team, test_hippo_team,
        test_jerboa_team, test_llama_team, test_lobster_team, test_lynx_team, test_mosq_team,
        test_orangutan_team, test_ox_team, test_parrot_team, test_pelican_team, test_penguin_team,
        test_platypus_team, test_porcupine_team, test_praying_mantis_team, test_skunk_team,
        test_snake_team, test_squirrel_team, test_turtle_team, test_whale_team, test_worm_team,
    },
    Effect, EntityName, Food, ItemCondition, Pet, Shop, ShopItem, ShopItemViewer, ShopViewer, Team,
    TeamShopping,
};

#[test]
fn test_battle_blowfish_team() {
    let mut team = test_blowfish_team();
    let mut enemy_team = test_ant_team();

    let start_blowfish_health = team.nth(1).unwrap().read().unwrap().stats.health;

    team.fight(&mut enemy_team).unwrap();

    // One pet dies to blowfish indirect attack.
    // Another dies to elephant attack.
    assert_eq!(enemy_team.all().len(), 1);

    // Blowfish takes 1 dmg.
    assert_eq!(
        team.nth(1).unwrap().read().unwrap().stats.health,
        start_blowfish_health - 1
    );
}

#[test]
fn test_battle_deer_team() {
    let mut team = test_deer_team();
    let mut enemy_team = test_ox_team();

    // Only one deer.
    assert!(team.first().unwrap().read().unwrap().name == PetName::Deer && team.all().len() == 1);
    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();

    // 1st attack kills dear and summons bus.
    // 2nd attack kills dog and ox before its effect triggers.
    // After completion, only bus remains with 2 health.
    let bus = team.any().unwrap();
    assert!(
        bus.read().unwrap().name == PetName::Bus
            && bus.read().unwrap().stats.health == 3
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
        hippo.read().unwrap().name == PetName::Hippo
            && hippo.read().unwrap().lvl == 1
            && hippo.read().unwrap().stats
                == Statistics {
                    attack: 4,
                    health: 5
                }
            && team.all().len() == 1,
    );
    // Versus three lvl. 1 ants.
    assert!(enemy_team
        .all()
        .iter()
        .all(|pet| pet.read().unwrap().name == PetName::Ant));
    let mut fight = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team).unwrap()
    }

    // Hippo takes 2 dmg (1st) + 6 dmg (2nd + 3rd) = 8 dmg
    // Hippo kills all three gaining (9,9) + base (4,5) - dmg (0,8) to (13,6)
    assert!(
        team.first().unwrap().read().unwrap().stats
            == Statistics {
                attack: 13,
                health: 6
            }
    );
}

#[test]
fn test_battle_chili_hippo() {
    let mut team = Team::new(&[Some(Pet::new(PetName::Hippo, None, 1).unwrap())], 5).unwrap();
    team.set_item(
        &Position::First,
        Some(Food::try_from(FoodName::Chili).unwrap()),
    )
    .unwrap();
    let hippo = team.first().unwrap();
    let hippo_start_stats = hippo.read().unwrap().stats;
    const HIPPO_BUFF: Statistics = Statistics {
        attack: 3,
        health: 3,
    };

    let mut enemy_team = Team::new(
        &[
            Pet::new(PetName::Mammoth, None, 1).ok(),
            Pet::new(PetName::Ant, None, 1).ok(),
        ],
        5,
    )
    .unwrap();
    let mammoth_dmg = Statistics {
        attack: 0,
        health: -enemy_team.first().unwrap().read().unwrap().stats.attack,
    };

    team.fight(&mut enemy_team).unwrap();

    // Hippo effect trigger because ant behind mammoth faints.
    assert_eq!(
        hippo_start_stats + HIPPO_BUFF + mammoth_dmg,
        hippo.read().unwrap().stats
    );
}

#[test]
fn test_shop_parrot_team() {
    let mut team = test_parrot_team();

    // Before end of turn, is def parrot effect.
    let parrot = team.nth(1).unwrap();
    assert_eq!(
        vec![Effect {
            owner: Some(Arc::downgrade(&parrot)),
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
        team.nth(1).unwrap().read().unwrap().effect
    );
    // Cricket is level 2.
    assert_eq!(team.first().unwrap().read().unwrap().lvl, 2);

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
            temp: false,
        }],
        team.nth(1).unwrap().read().unwrap().effect
    );
}

#[test]
fn test_battle_skunk_team() {
    let mut team = test_skunk_team();
    let mut enemy_team = test_skunk_team();

    // Lvl. 1 skunks on both teams.
    assert!(
        team.first().unwrap().read().unwrap().lvl == 1
            && enemy_team.first().unwrap().read().unwrap().lvl == 1,
    );
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics {
            attack: 3,
            health: 5
        }
    );
    assert_eq!(
        enemy_team.first().unwrap().read().unwrap().stats,
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
        team.first().unwrap().read().unwrap().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
    assert_eq!(
        enemy_team.first().unwrap().read().unwrap().stats,
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

    assert_eq!(team.nth(1).unwrap().read().unwrap().item, None);

    // Three attacks to kill both lvl.1 turtles.
    for _ in 0..3 {
        team.fight(&mut enemy_team).unwrap();
    }

    let pet_behind_turtle = team.first().unwrap();
    assert_eq!(
        pet_behind_turtle
            .read()
            .unwrap()
            .item
            .as_ref()
            .unwrap()
            .name,
        FoodName::Melon
    );
}

#[test]
fn test_battle_whale_team() {
    let mut team = test_whale_team();
    let mut enemy_team = test_hippo_team();

    // Only one cricket at start.
    assert_eq!(count_pets(&team.friends, PetName::Cricket), 1);
    // Copy at lvl 1.
    let cricket_copy = Pet::try_from(team.first().unwrap().read().unwrap().name.clone()).unwrap();

    team.fight(&mut enemy_team).unwrap();

    // After start of battle, whale eats cricket and changes effect to summon cricket.
    let whale = team.first().unwrap();
    let whale_effect = &whale.read().unwrap().effect;

    assert_eq!(
        whale_effect.first().unwrap().action,
        Action::Summon(SummonType::StoredPet(Box::new(cricket_copy)))
    );
    assert_eq!(count_pets(&team.friends, PetName::Cricket), 0);
}

#[test]
fn test_battle_front_whale_team() {
    let mut team = Team::new(&[Some(Pet::try_from(PetName::Whale).unwrap())], 5).unwrap();
    let mut enemy_team = test_hippo_team();

    let whale = team.first().unwrap();
    let hippo = enemy_team.first().unwrap();

    team.fight(&mut enemy_team).unwrap();

    // Whale has no target. Pets battle as normal.
    assert_eq!(
        whale.read().unwrap().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    assert_eq!(
        hippo.read().unwrap().stats,
        Statistics {
            attack: 4,
            health: 2
        }
    );
}

#[test]
fn test_battle_armadillo_team() {
    let mut team = test_armadillo_team();
    let mut enemy_team = test_hippo_team();

    // Armadillo at lvl. 1 provides 8 health
    const BUFF: Statistics = Statistics {
        attack: 0,
        health: 8,
    };
    let pets = team.all();
    let exp_stats: Vec<Statistics> = pets
        .iter()
        .enumerate()
        .map(|(i, pet)| {
            // Armadillo at position 0.
            if i == 0 {
                pet.read().unwrap().stats
            } else {
                pet.read().unwrap().stats + BUFF
            }
        })
        .collect();

    // Trigger start of battle.
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    assert_eq!(
        pets.iter()
            .map(|pet| pet.read().unwrap().stats)
            .collect::<Vec<Statistics>>(),
        exp_stats
    );
}

#[test]
fn test_battle_doberman_team() {
    let mut team = test_doberman_team();
    let mut enemy_team = test_hippo_team();

    // Doberman has no item.
    let doberman = team.first().unwrap();
    const BUFF: Statistics = Statistics {
        attack: 8,
        health: 0,
    };
    let start_stats = doberman.read().unwrap().stats;
    assert_eq!(doberman.read().unwrap().item, None);

    // Doberman is lowest tier.
    assert_eq!(
        team.all()
            .iter()
            .min_by(|pet_1, pet_2| pet_1.read().unwrap().tier.cmp(&pet_2.read().unwrap().tier))
            .unwrap()
            .read()
            .unwrap()
            .name,
        PetName::Doberman
    );
    team.fight(&mut enemy_team).unwrap();

    // Doberman gets coconut and gets (5,5)
    assert_eq!(
        doberman.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Coconut
    );
    assert_eq!(doberman.read().unwrap().stats, start_stats + BUFF);
}

#[test]
fn test_battle_doberman_highest_tier_team() {
    let mut team = test_doberman_highest_tier_team();
    let mut enemy_team = test_hippo_team();

    // Doberman has no item.
    assert_eq!(team.first().unwrap().read().unwrap().item, None);
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics::new(4, 5).unwrap()
    );
    // Doberman is not lowest tier.
    assert_ne!(
        team.all()
            .iter()
            .min_by(|pet_1, pet_2| pet_1.read().unwrap().tier.cmp(&pet_2.read().unwrap().tier))
            .unwrap()
            .read()
            .unwrap()
            .name,
        PetName::Doberman
    );
    team.fight(&mut enemy_team).unwrap();

    // Doberman doesn't get coconut or stats.
    assert_eq!(team.first().unwrap().read().unwrap().item, None);
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
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
        team.all()
            .iter()
            .map(|pet| pet.read().unwrap().lvl)
            .sum::<usize>(),
        5
    );

    team.fight(&mut enemy_team).unwrap();

    // Hippo faints at start of battle.
    assert_eq!(
        enemy_hippo.read().unwrap().stats,
        Statistics::new(4, 0).unwrap()
    );

    team.restore();
    enemy_team.restore();

    // Remove one level one pet.
    team.friends.pop();
    assert_eq!(
        team.all()
            .iter()
            .map(|pet| pet.read().unwrap().lvl)
            .sum::<usize>(),
        4
    );

    // Retrigger start of battle effects
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Hippo takes 4 dmg.
    assert_eq!(
        enemy_team.first().unwrap().read().unwrap().stats,
        Statistics::new(4, 1).unwrap()
    );
}

#[test]
fn test_battle_porcupine_team() {
    let mut team = test_porcupine_team();
    let mut enemy_team = test_mosq_team();

    // Buff 1st mosquito so survives first returned attack.
    enemy_team.first().unwrap().write().unwrap().stats.health += 3;

    // Trigger start of battle effects. Then clear fainted pets.
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();
    enemy_team.clear_team();

    // 2 Mosquitoes faint from returned fire from porcupine
    assert_eq!(enemy_team.fainted.len(), 2);
    // 1 mosquito that was buffed survives.
    assert!(
        enemy_team.all().len() == 1
            && enemy_team.first().unwrap().read().unwrap().stats == Statistics::new(2, 2).unwrap()
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
    assert_eq!(
        caterpillar.read().unwrap().stats,
        Statistics::new(1, 1).unwrap()
    );
    assert_eq!(caterpillar.read().unwrap().name, PetName::Caterpillar);

    // Right before battle phase, butterfly will copy effect.
    team.fight(&mut enemy_team).unwrap();

    // Butterfly takes 4 dmg from hippo but copied hippo's stats leaving it with 1 health.
    let butterfly = team.first().unwrap();
    assert_eq!(
        butterfly.read().unwrap().stats,
        Statistics::new(4, 1).unwrap()
    );
    assert_eq!(butterfly.read().unwrap().name, PetName::Butterfly);
}

#[test]
fn test_shop_caterpillar_team() {
    let mut team = Team::new(&[Some(Pet::try_from(PetName::Caterpillar).unwrap())], 5).unwrap();

    let caterpillar = team.first().unwrap();
    // Starts with lvl 1 and no exp.
    assert!(caterpillar.read().unwrap().exp == 0 && caterpillar.read().unwrap().lvl == 1);

    team.open_shop().unwrap();

    let caterpillar = team.first().unwrap();
    // Gains 1 exp on start of turn.
    assert!(caterpillar.read().unwrap().exp == 1 && caterpillar.read().unwrap().lvl == 1)
}

#[test]
fn test_battle_anteater_team() {
    let mut team = test_anteater_team();
    let mut enemy_team = test_hippo_team();

    // Single anteater.
    let anteater = team.first().unwrap();
    assert_eq!(team.all().len(), 1);
    assert_eq!(anteater.read().unwrap().name, PetName::Anteater);
    team.fight(&mut enemy_team).unwrap();

    // After faint, two anteaters spawn.
    let fainted_pet = team.fainted.first().unwrap().as_ref().unwrap();
    assert!(Arc::ptr_eq(fainted_pet, &anteater));
    let all_friends = team.all();
    assert_eq!(all_friends.len(), 2);
    assert!(all_friends
        .iter()
        .all(|pet| pet.read().unwrap().name == PetName::Ant))
}

#[test]
fn test_battle_donkey_team() {
    let mut team = test_donkey_team();
    let mut enemy_team = test_snake_team();
    team.set_seed(Some(2));
    enemy_team.set_seed(Some(2));

    assert_eq!(
        enemy_team.nth(1).unwrap().read().unwrap().name,
        PetName::Snake
    );

    team.fight(&mut enemy_team).unwrap();

    // Cricket faints and donkey ability triggers.
    assert_eq!(enemy_team.fainted.len(), 1);
    // Snake pushed to front.
    assert_eq!(
        enemy_team.first().unwrap().read().unwrap().name,
        PetName::Snake
    );
    // And zombie cricket now in back.
    assert_eq!(
        enemy_team.nth(1).unwrap().read().unwrap().name,
        PetName::ZombieCricket
    )
}

#[test]
fn test_battle_eel_team() {
    let mut team = test_eel_team();
    let mut enemy_team = test_hippo_team();

    let eel_stats = team.first().unwrap().read().unwrap().stats;

    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Eel at lvl.1 gains 50% of original health.
    assert_eq!(
        eel_stats + Statistics::new(0, eel_stats.health / 2).unwrap(),
        team.first().unwrap().read().unwrap().stats
    )
}

#[test]
fn test_battle_hawk_team() {
    let mut team = test_hawk_team();
    let mut enemy_team = test_gorilla_team();

    // Hawk on 1st position.
    assert_eq!(team.first().unwrap().read().unwrap().name, PetName::Hawk);
    let gorilla_on_1st = enemy_team.first().unwrap();
    let gorilla_on_first_start_stats = gorilla_on_1st.read().unwrap().stats;

    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Gorilla takes 7 dmg.
    assert_eq!(
        gorilla_on_1st.read().unwrap().stats,
        gorilla_on_first_start_stats
            - Statistics {
                attack: 0,
                health: 7
            }
    );
}

#[test]
fn test_battle_pelican_team() {
    let mut team = test_pelican_team();
    let mut enemy_team = test_hippo_team();

    {
        // Ant has strawberry.
        let ant = team.nth(1).unwrap();

        assert_eq!(ant.read().unwrap().stats, Statistics::new(2, 2).unwrap());
        let ant_item = &ant.read().unwrap().item;
        assert_eq!(ant_item.as_ref().unwrap().name, FoodName::Strawberry)
    }

    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Pelican at lvl.1 give strawberry ant (2,1)
    assert_eq!(
        team.nth(1).unwrap().read().unwrap().stats,
        Statistics::new(4, 3).unwrap()
    );
}

#[test]
fn test_shop_bison_team() {
    let mut team = test_bison_team();
    // Lvl 3 duck on team.
    assert_eq!(team.first().unwrap().read().unwrap().lvl, 3);
    assert_eq!(
        team.last().unwrap().read().unwrap().stats,
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
    assert_eq!(team.last().unwrap().read().unwrap().stats, exp_bison_stats);

    // Sell lvl 3 friend
    team.open_shop()
        .unwrap()
        .sell(&Position::First)
        .unwrap()
        .close_shop()
        .unwrap();

    // Stats don't change at end of turn anymore.
    assert_eq!(team.last().unwrap().read().unwrap().stats, exp_bison_stats);
}

#[test]
fn test_shop_penguin_team() {
    let mut team = test_penguin_team();

    let lvl_3_duck = team.first().unwrap();
    // Base stats and level of duck.
    assert_eq!(lvl_3_duck.read().unwrap().lvl, 3);
    assert_eq!(
        lvl_3_duck.read().unwrap().stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );

    team.open_shop().unwrap().close_shop().unwrap();

    // Duck gets (1,1)
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
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
        team.first().unwrap().read().unwrap().stats,
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
        worm.read().unwrap().stats == Statistics::new(4, 4).unwrap()
            && worm.read().unwrap().item.as_ref().unwrap().name == FoodName::Honey
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
        duck.read().unwrap().lvl == 3
            && duck.read().unwrap().stats == starting_duck_stats
            && dog.read().unwrap().lvl == 1
            && dog.read().unwrap().stats == starting_dog_stats
    );

    team.close_shop().unwrap();

    let duck_stats_after_end_turn = starting_duck_stats + DRAGONFLY_BUFF;
    let dog_stats_after_end_turn = starting_dog_stats + DRAGONFLY_BUFF;
    assert!(
        duck.read().unwrap().stats == duck_stats_after_end_turn
            && dog.read().unwrap().stats == dog_stats_after_end_turn
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
        duck.read().unwrap().stats == duck_stats_after_end_turn
            && dog.read().unwrap().stats == dog_stats_after_end_turn + DRAGONFLY_BUFF
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
        duck.read().unwrap().stats,
        jerboa.read().unwrap().stats,
        dog.read().unwrap().stats,
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
            jerboa.read().unwrap().stats
        );
        // Jerboa buffs (1,1) all pets when apple eaten.
        assert_eq!(duck_start_stats + JERBOA_BUFF, duck.read().unwrap().stats);
        assert_eq!(dog_start_stats + JERBOA_BUFF, dog.read().unwrap().stats);
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
        ant_1.read().unwrap().stats,
        ant_2.read().unwrap().stats,
        ant_3.read().unwrap().stats,
    );
    const MOLE_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };
    team.buy(&Position::First, &Entity::Pet, &Position::Relative(-1))
        .unwrap();

    // Ants adjacent to mole get stats.
    assert_eq!(ant_1_start_stats + MOLE_BUFF, ant_1.read().unwrap().stats);
    assert_eq!(ant_2_start_stats + MOLE_BUFF, ant_2.read().unwrap().stats);
    // Ant not adjacent gets nothing.
    assert_eq!(ant_3_start_stats, ant_3.read().unwrap().stats);
}

#[test]
fn test_shop_buffalo_team() {
    let mut team = test_buffalo_team();
    team.set_shop_seed(Some(11)).open_shop().unwrap();

    let buffalo = team.first().unwrap();
    let buffalo_start_stats = buffalo.read().unwrap().stats;
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
        assert_eq!(
            buffalo_start_stats + stat_increase,
            buffalo.read().unwrap().stats
        );
    }
    let final_buffalo_stats = buffalo.read().unwrap().stats;

    // Add some gold to allow buying a 4th pet.
    team.shop.coins += 3;
    team.roll_shop()
        .unwrap()
        .buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap();

    // No additional stats on buy as limit reached.
    assert_eq!(final_buffalo_stats, buffalo.read().unwrap().stats);
}

#[test]
fn test_shop_llama_team() {
    let mut team = test_llama_team();

    team.open_shop().unwrap();

    let llama = team.first().unwrap();

    let llama_start_stats = llama.read().unwrap().stats;
    const LLAMA_BUFF: Statistics = Statistics {
        attack: 2,
        health: 2,
    };

    team.close_shop().unwrap();

    // Llama gets (2,2)
    assert_eq!(llama_start_stats + LLAMA_BUFF, llama.read().unwrap().stats);

    team.open_shop().unwrap();

    // Drop invalid reference created on opening shop/restoring team.
    std::mem::drop(llama);

    let llama = team.first().unwrap();
    let llama_new_start_stats = llama.read().unwrap().stats;
    // Add four pets to fill the team.
    for _ in 0..4 {
        team.add_pet(Pet::try_from(PetName::Ant).unwrap(), 0, None)
            .unwrap();
    }
    // Filled team.
    assert_eq!(team.friends.len(), team.max_size);

    team.close_shop().unwrap();
    // No change in stats as team is filled and no empty space.
    assert_eq!(llama.read().unwrap().stats, llama_new_start_stats);
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
    assert_eq!(mosq.read().unwrap().stats, base_mosq.stats + LOBSTER_BUFF);
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
    assert_eq!(
        zombie_cricket.read().unwrap().stats,
        base_zombie_cricket.stats
    );
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
    assert!(ant.read().unwrap().lvl == 1 && ant.read().unwrap().exp == 0);

    // Buy chocolate for it.
    team.buy(&Position::First, &Entity::Food, &Position::First)
        .unwrap();
    // Ant gains 2 exp and levels up because crow is lvl 2.
    assert!(ant.read().unwrap().lvl == 2 && ant.read().unwrap().exp == 2);
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
        duck.read().unwrap().name == PetName::Duck
            && duck.read().unwrap().lvl == 1
            && beaver.read().unwrap().name == PetName::Beaver
            && beaver.read().unwrap().lvl == 1
    );
}

#[test]
fn test_shop_praying_mantis_team() {
    let mut team = test_praying_mantis_team();
    // Three pets on team. One in middle is mantis
    assert_eq!(team.friends.len(), 3);
    let mantis_start_stats = team.nth(1).unwrap().read().unwrap().stats;
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
        pets.first().unwrap().read().unwrap().stats,
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
    assert_eq!(lowest_health_pet.read().unwrap().name, PetName::Ant);

    let ant_start_stats = lowest_health_pet.read().unwrap().stats;
    const ORANGUTAN_BUFF: Statistics = Statistics {
        attack: 0,
        health: 4,
    };

    team.close_shop().unwrap();
    // Ant gains buff because lowest health.
    assert_eq!(
        lowest_health_pet.read().unwrap().stats,
        ant_start_stats + ORANGUTAN_BUFF
    );
}
