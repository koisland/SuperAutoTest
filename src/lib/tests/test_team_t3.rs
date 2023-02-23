use itertools::Itertools;

use crate::{
    effects::{state::Status, stats::Statistics, trigger::TRIGGER_START_BATTLE},
    foods::{food::Food, names::FoodName},
    pets::names::PetName,
    teams::{
        combat::{ClearOption, TeamCombat},
        team::TeamFightOutcome,
        viewer::TeamViewer,
    },
    tests::common::{
        test_aardvark_team, test_ant_team, test_badger_team, test_bear_team, test_blobfish_team,
        test_blowfish_rally_team, test_blowfish_team, test_camel_team, test_capybara_team,
        test_cassowary_team, test_clownfish_team, test_cricket_horse_team, test_dog_team,
        test_dolphin_team, test_emperor_tamarin_team, test_filled_sheep_team, test_giraffe_team,
        test_hatching_chick_team, test_hippo_team, test_hummingbird_team, test_kangaroo_team,
        test_leech_team, test_mouse_team, test_okapi_team, test_owl_team, test_ox_team,
        test_puppy_team, test_rabbit_team, test_seagull_team, test_sheep_team, test_starfish_team,
        test_toad_team, test_tropicalfish_team, test_wasp_team, test_woodpecker_self_hurt_team,
        test_woodpecker_team,
    },
    Condition, Entity, Pet, Position, Shop, ShopItem, ShopItemViewer, ShopViewer, TeamEffects,
    TeamShopping,
};

#[test]
fn test_battle_badger_team() {
    let mut team = test_badger_team();
    let mut enemy_team = test_dolphin_team();

    assert_eq!(team.nth(1).unwrap().borrow().stats.health, 5);
    // Dolphin immediately kills badger.
    // Badger's effect triggers dealing 3 dmg to both adjacent pets.
    let mut fight = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team).unwrap()
    }

    assert_eq!(fight, TeamFightOutcome::Win);
    assert_eq!(team.first().unwrap().borrow().stats.health, 2)
}

#[test]
fn test_battle_blowfish_team() {
    let mut team = test_blowfish_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(team.nth(1).unwrap().borrow().stats.health, 5);

    team.fight(&mut enemy_team).unwrap();

    // One pet dies to blowfish indirect attack.
    // Another dies to elephant attack.
    assert_eq!(enemy_team.all().len(), 1);
    // Blowfish takes 1 dmg.
    assert_eq!(team.nth(1).unwrap().borrow().stats.health, 4);
}

#[test]
fn test_battle_blowfish_rally_battle() {
    let mut team = test_blowfish_rally_team();
    let mut enemy_team = test_blowfish_rally_team();

    let mut fight = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team).unwrap()
    }

    // Only one attack occurs in fight.
    let n_atks: usize = team
        .history
        .effect_graph
        .raw_nodes()
        .iter()
        .filter_map(|node| (node.weight.status == Status::Attack).then_some(1))
        .sum();
    assert_eq!(1, n_atks);
    // 25 atks occur 1 + 50 = 51 dmg.
    assert_eq!(25, team.history.effect_graph.raw_edges().len())
}
#[test]
fn test_battle_camel_team() {
    let mut team = test_camel_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(team.nth(1).unwrap().borrow().stats.health, 6);
    // Ant has 1 health.
    assert_eq!(team.nth(2).unwrap().borrow().stats.health, 1);

    team.fight(&mut enemy_team).unwrap();

    // Camel takes 1 dmg from elephant.
    assert_eq!(team.nth(1).unwrap().borrow().stats.health, 5);
    // And gives ant 2 hp.
    assert_eq!(team.nth(2).unwrap().borrow().stats.health, 3);
}

#[test]
fn test_battle_dog_team() {
    let mut team = test_dog_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(team.nth(0).unwrap().borrow().name, PetName::Cricket);
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    team.fight(&mut enemy_team).unwrap();

    assert_eq!(team.nth(0).unwrap().borrow().name, PetName::ZombieCricket);
    // Dog gains (1,1) after Zombie Cricket spawns.
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 4,
            health: 5
        }
    );
}

#[test]
fn test_battle_kangaroo_team() {
    let mut team = test_kangaroo_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
    team.fight(&mut enemy_team).unwrap();

    // Friend ahead attacks once increasing stats by (2,2)
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
}

#[test]
fn test_battle_ox_team() {
    let mut team = test_ox_team();
    let mut enemy_team = test_ant_team();

    {
        let ox = team.nth(1).unwrap();
        // No item on default lvl.1 ox.
        assert!(ox.borrow().item.is_none());
        assert_eq!(
            ox.borrow().stats,
            Statistics {
                attack: 1,
                health: 3
            }
        );
    };

    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();

    let ox = team.nth(0).unwrap();
    // Gets melon armor.
    let mut melon_armor = Food::try_from(&FoodName::Melon).unwrap();
    melon_armor.ability.assign_owner(Some(&ox));
    assert_eq!(ox.borrow().item, Some(melon_armor));
    // And an extra attack.
    assert_eq!(
        ox.borrow().stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );
}

#[test]
fn test_battle_sheep_team() {
    let mut team = test_sheep_team();
    let mut enemy_team = test_sheep_team();

    assert_eq!(team.all().len(), 1);
    // Sheep faint and summon two ram.
    team.fight(&mut enemy_team).unwrap();

    for team in [team, enemy_team].iter_mut() {
        let pets = team.all();

        assert_eq!(pets.len(), 2);

        for pet in pets.iter() {
            assert_eq!(pet.borrow().name, PetName::Ram)
        }
    }
}

#[test]
fn test_battle_filled_team() {
    let mut team = test_filled_sheep_team();
    let mut enemy_team = test_filled_sheep_team();

    team.fight(&mut enemy_team).unwrap();

    // Overflow in pets (ram in this case) gets added to team's dead.
    let first_pet = team.fainted.first();
    assert_eq!(2, team.fainted.len());
    assert_eq!(
        PetName::Ram,
        first_pet.as_ref().unwrap().as_ref().unwrap().borrow().name
    );
}

#[test]
fn test_battle_aardvark_team() {
    let mut team = test_aardvark_team();
    let mut enemy_team = test_cricket_horse_team();

    let aardvark_stats = team.first().unwrap().borrow().stats;
    assert_eq!(aardvark_stats, Statistics::new(2, 3).unwrap());

    // Fights first cricket.
    team.fight(&mut enemy_team).unwrap();

    // Cricket faints and Zombie Cricket spawns
    let fainted_pet = enemy_team.fainted.first().unwrap();
    assert_eq!(
        fainted_pet.as_ref().unwrap().borrow().name,
        PetName::Cricket
    );
    assert_eq!(
        enemy_team.first().unwrap().borrow().name,
        PetName::ZombieCricket
    );

    // One dmg from cricket, zombie cricket spawns and (2,2) given to aardvark.
    assert_eq!(
        aardvark_stats
            + Statistics {
                attack: 0,
                health: -1
            }
            + Statistics {
                attack: 2,
                health: 2
            },
        team.first().unwrap().borrow().stats
    );
}

#[test]
fn test_battle_bear_team() {
    let mut team = test_bear_team();
    let mut enemy_team = test_hummingbird_team();

    // Dog at position behind bear has no item.
    assert_eq!(team.nth(1).unwrap().borrow().item, None);
    // Enemy team first pet (duck) has strawberry.
    let enemy_duck_item = enemy_team.first().unwrap().borrow().item.clone();
    assert_eq!(enemy_duck_item.unwrap().name, FoodName::Strawberry);
    team.fight(&mut enemy_team).unwrap();

    // Bear fainted.
    let fainted_bear = team.fainted.first().unwrap();
    assert_eq!(fainted_bear.as_ref().unwrap().borrow().name, PetName::Bear);
    // Duck now has honey.
    assert_eq!(
        enemy_team
            .first()
            .unwrap()
            .borrow()
            .item
            .as_ref()
            .unwrap()
            .name,
        FoodName::Honey
    );
    // Dog now has honey.
    assert_eq!(
        team.first().unwrap().borrow().item.as_ref().unwrap().name,
        FoodName::Honey
    );
}

#[test]
fn test_battle_seagull_team() {
    let mut team = test_seagull_team();
    let mut enemy_team = test_ant_team();

    // First pet summons zombie cricket.
    assert_eq!(team.first().unwrap().borrow().name, PetName::Cricket);
    // Seagull has honey.
    assert_eq!(
        team.nth(1).unwrap().borrow().item.as_ref().unwrap().name,
        FoodName::Honey
    );
    team.fight(&mut enemy_team).unwrap();

    // Zombie cricket gets honey from seagull.
    {
        let zombie_cricket = team.first().unwrap();
        assert!(
            zombie_cricket.borrow().name == PetName::ZombieCricket
                && zombie_cricket.borrow().item.as_ref().unwrap().name == FoodName::Honey
        );
    }

    // Fight again to kill zombie cricket with honey.
    team.fight(&mut enemy_team).unwrap();

    // Seagull ability only activates once. Bee does not get honey.
    assert!(
        team.first().unwrap().borrow().name == PetName::Bee
            && team.first().unwrap().borrow().item == None
    );
}

#[test]
fn test_battle_blobfish_team() {
    let mut team = test_blobfish_team();
    let mut enemy_team = test_hummingbird_team();

    // Dog behind has no experience.
    assert_eq!(team.nth(1).unwrap().borrow().exp, 0);
    team.fight(&mut enemy_team).unwrap();

    // Blobfish dies.
    let fainted_blobfish = team.fainted.first().unwrap();
    assert_eq!(
        fainted_blobfish.as_ref().unwrap().borrow().name,
        PetName::Blobfish
    );
    // Dog in front now has 1 experience.
    assert_eq!(team.first().unwrap().borrow().exp, 1);
}

#[test]
fn test_battle_clownfish_team() {
    let mut team = test_clownfish_team();
    let mut enemy_team = test_hummingbird_team();

    // Dog behind blobfish is level 1 and has 1 exp.
    let dog_stats = {
        let dog = team.nth(1).unwrap();
        assert!(dog.borrow().exp == 1 && dog.borrow().lvl == 1);
        assert_eq!(Statistics::new(4, 5).unwrap(), dog.borrow().stats);
        let stats = dog.borrow().stats;
        stats
    };
    // Blobfish dies during fight and levels dog to 2.
    team.fight(&mut enemy_team).unwrap();

    {
        let dog = team.first().unwrap();
        let new_dog_stats = dog.borrow().stats;
        assert!(dog.borrow().exp == 2 && dog.borrow().lvl == 2);
        // Dog gains (1,1) from blobfish experience and (2,2) from clownfish on level.
        assert_eq!(
            dog_stats + Statistics::new(1, 1).unwrap() + Statistics::new(2, 2).unwrap(),
            new_dog_stats
        )
    }
}

#[test]
fn test_battle_toad_team() {
    let mut team = test_toad_team();
    let mut enemy_team = test_cricket_horse_team();

    // Seed ensures that target always cricket at pos 1.
    enemy_team.set_seed(Some(2));
    assert_eq!(
        enemy_team.nth(1).unwrap().borrow().stats,
        Statistics::new(1, 2).unwrap()
    );
    // Trigger start of battle effects.
    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();

    // Cricket hit by mosquito and takes 1 dmg
    assert_eq!(
        enemy_team.nth(0).unwrap().borrow().stats,
        Statistics::new(1, 1).unwrap()
    );
    // Frog triggers and cricket now has weakness.
    assert_eq!(
        enemy_team
            .nth(0)
            .unwrap()
            .borrow()
            .item
            .as_ref()
            .unwrap()
            .name,
        FoodName::Weak
    );
}

#[test]
fn test_battle_woodpecker_team() {
    let mut team = test_woodpecker_team();
    let mut enemy_team = test_cricket_horse_team();

    let (first_enemy, second_enemy) = (enemy_team.first().unwrap(), enemy_team.nth(1).unwrap());
    assert_eq!(first_enemy.borrow().stats, Statistics::new(1, 2).unwrap());
    assert_eq!(second_enemy.borrow().stats, Statistics::new(1, 2).unwrap());
    // Trigger start of battle effects.
    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();

    // Two crickets at front on enemy team die.
    assert_eq!(first_enemy.borrow().stats, Statistics::new(1, 0).unwrap());
    assert_eq!(second_enemy.borrow().stats, Statistics::new(1, 0).unwrap());
}

#[test]
fn test_battle_woodpecker_self_hurt_team() {
    let mut team = test_woodpecker_self_hurt_team();
    let mut enemy_team = test_cricket_horse_team();

    assert_eq!(
        team.nth(0).unwrap().borrow().stats,
        Statistics::new(1, 2).unwrap()
    );
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics::new(1, 2).unwrap()
    );

    // Trigger start of battle effects and clear dead pets.
    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();
    team.clear_team(ClearOption::RemoveSlots);

    // Two crickets at front of woodpecker on same team faint.
    assert_eq!(team.fainted.len(), 2);
}

#[test]
fn test_shop_giraffe_team() {
    let mut team = test_giraffe_team();

    team.open_shop().unwrap();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    team.close_shop().unwrap();
    // Gain (1,1) after ending turn.
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 4,
            health: 5
        }
    );
}

#[test]
fn test_shop_rabbit_team() {
    let mut team = test_rabbit_team();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    team.set_shop_seed(Some(12))
        .open_shop()
        .unwrap()
        .buy(&Position::First, &Entity::Food, &Position::First)
        .unwrap();

    // Pet gains (0,1) after item bought and eaten by pet.
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 5
        }
    );
}

#[test]
fn test_shop_snail_team() {
    let mut team = test_mouse_team();
    let mut enemy_team = test_hippo_team();

    let mut outcome = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = outcome {
        outcome = team.fight(&mut enemy_team).unwrap()
    }

    // Team loses.
    assert!(outcome == TeamFightOutcome::Loss);

    let mut shop = Shop::default();
    shop.add_item(ShopItem::from(Pet::try_from(PetName::Snail).unwrap()))
        .unwrap();

    team.replace_shop(shop).unwrap();
    team.open_shop().unwrap();

    let pets = team.all();
    let (mouse, ant) = (pets.first().unwrap(), pets.last().unwrap());

    assert_eq!(
        mouse.borrow().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    assert_eq!(
        ant.borrow().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    team.buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap();

    let pets = team.all();
    let (snail, mouse, ant) = (
        pets.first().unwrap(),
        pets.get(1).unwrap(),
        pets.last().unwrap(),
    );
    // Pets get (1,1)
    // Snail gets no stats. Same as default.
    assert_eq!(
        snail.borrow().stats,
        Pet::try_from(PetName::Snail).unwrap().stats
    );
    assert_eq!(
        mouse.borrow().stats,
        Statistics {
            attack: 3,
            health: 2
        }
    );
    assert_eq!(
        ant.borrow().stats,
        Statistics {
            attack: 3,
            health: 2
        }
    );
}

#[test]
fn test_shop_emperor_tamarin_team() {
    let mut team = test_emperor_tamarin_team();

    team.set_shop_seed(Some(12)).open_shop().unwrap();

    let shop_pets = team
        .shop
        .get_shop_items_by_pos(&Position::First, &Entity::Pet)
        .unwrap();
    let first_shop_pet_slot = shop_pets.first().unwrap();
    assert!(
        first_shop_pet_slot.attack_stat() == Some(2)
            && first_shop_pet_slot.health_stat() == Some(2)
    );

    // Sell emperor tamarin.
    team.sell(&Position::First).unwrap();

    // First shop pet gains (1,2).
    let shop_pets = team
        .shop
        .get_shop_items_by_pos(&Position::First, &Entity::Pet)
        .unwrap();
    let first_shop_pet_slot = shop_pets.first().unwrap();
    assert!(
        first_shop_pet_slot.attack_stat() == Some(3)
            && first_shop_pet_slot.health_stat() == Some(4)
    );
}

#[test]
fn test_shop_wasp_team() {
    let mut team = test_wasp_team();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 2,
            health: 2
        }
    );
    // Open shop and set shop tier to tier 2.
    team.set_shop_seed(Some(12))
        .open_shop()
        .unwrap()
        .set_shop_tier(2)
        .unwrap();

    // Wasp gains (1,0) which is 50% of base attack.
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 2
        }
    );
}

#[test]
fn test_shop_hatching_chick_lvl_1_team() {
    let mut team = test_hatching_chick_team();

    let original_dog_stats = Statistics {
        attack: 3,
        health: 4,
    };
    assert_eq!(team.first().unwrap().borrow().stats, original_dog_stats);

    // Open shop and set shop tier to tier 2.
    team.open_shop().unwrap().close_shop().unwrap();

    // Gain (4,4)
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 7,
            health: 8
        }
    );

    team.open_shop().unwrap();

    // Stats are temporary.
    assert_eq!(original_dog_stats, team.first().unwrap().borrow().stats,);
}

#[test]
fn test_shop_hatching_chick_lvl_2_team() {
    let mut team = test_hatching_chick_team();
    let original_dog_stats = Statistics {
        attack: 3,
        health: 4,
    };
    let new_dog_stats = Statistics {
        attack: 5,
        health: 6,
    };
    assert_eq!(team.first().unwrap().borrow().stats, original_dog_stats);

    team.open_shop().unwrap();
    // Upgrade chick to level 2.
    team.last().unwrap().borrow_mut().set_level(2).unwrap();
    team.close_shop().unwrap();

    // Gain (2, 2)
    assert_eq!(team.first().unwrap().borrow().stats, new_dog_stats);

    team.open_shop().unwrap();
    // Stats are not temporary.
    assert_eq!(team.first().unwrap().borrow().stats, new_dog_stats);
}

#[test]
fn test_shop_hatching_chick_lvl_3_team() {
    let mut team = test_hatching_chick_team();
    // Dog in front of chick has no exp.
    assert_eq!(team.first().unwrap().borrow().exp, 0);

    team.open_shop().unwrap();
    // Upgrade chick to level 3 during shop phase so stats/exp retained.
    team.last().unwrap().borrow_mut().set_level(3).unwrap();
    team.close_shop().unwrap();

    // Reopen shop. Dog now has 1 exp.
    team.open_shop().unwrap();
    assert_eq!(team.first().unwrap().borrow().exp, 1);
}

#[test]
fn test_shop_owl_team() {
    let mut team = test_owl_team();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );

    // Sell at last position.
    team.open_shop().unwrap().sell(&Position::Last).unwrap();

    // Dog gets (2,2).
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 5,
            health: 6
        }
    );
}

#[test]
fn test_shop_puppy_team() {
    let mut team = test_puppy_team();

    assert_eq!(team.gold(), 10);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
    // End turn.
    team.open_shop().unwrap().close_shop().unwrap();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
}

#[test]
fn test_shop_tropical_fish_team() {
    let mut team = test_tropicalfish_team();

    let pets = team.all();
    let (first_pet, last_pet) = (pets.first().unwrap(), pets.last().unwrap());

    assert!(
        first_pet.borrow().stats
            == Statistics {
                attack: 3,
                health: 4
            }
            && last_pet.borrow().stats
                == Statistics {
                    attack: 3,
                    health: 4
                }
    );
    // End turn.
    team.open_shop().unwrap().close_shop().unwrap();

    // Pets adjacent get (0, 1).
    let pets = team.all();
    let (first_pet, last_pet) = (pets.first().unwrap(), pets.last().unwrap());
    assert!(
        first_pet.borrow().stats
            == Statistics {
                attack: 3,
                health: 5
            }
            && last_pet.borrow().stats
                == Statistics {
                    attack: 3,
                    health: 5
                }
    );
}

#[test]
fn test_shop_capybara_team() {
    let mut team = test_capybara_team();

    team.set_shop_seed(Some(12))
        .open_shop()
        .unwrap()
        .roll_shop()
        .unwrap();

    let add_stats = Statistics {
        attack: 2,
        health: 1,
    };
    let shop_pets = team
        .shop
        .get_shop_items_by_pos(&Position::All(Condition::None), &Entity::Pet)
        .unwrap();

    // Item stats.
    let (mosq_stats, pig_stats, beaver_stats) = (0..3)
        .map(|idx| {
            Statistics::new(
                shop_pets[idx].attack_stat().unwrap(),
                shop_pets[idx].health_stat().unwrap(),
            )
            .unwrap()
        })
        .collect_tuple::<(Statistics, Statistics, Statistics)>()
        .unwrap();

    // Items stats are added on roll.
    assert_eq!(
        mosq_stats,
        Pet::try_from(PetName::Mosquito).unwrap().stats + add_stats
    );
    assert_eq!(
        pig_stats,
        Pet::try_from(PetName::Pig).unwrap().stats + add_stats
    );
    assert_eq!(
        beaver_stats,
        Pet::try_from(PetName::Beaver).unwrap().stats + add_stats
    );
}

#[test]
fn test_shop_cassowary_team() {
    let mut team = test_cassowary_team();
    team.open_shop().unwrap();

    let cassowary = team.first().unwrap();
    assert_eq!(
        cassowary.borrow().stats,
        Statistics {
            attack: 2,
            health: 4
        }
    );
    // Cassowary has strawberry.
    assert_eq!(
        cassowary.borrow().item.as_ref().unwrap().name,
        FoodName::Strawberry
    );
    team.close_shop().unwrap();

    // Gains (2, 1)
    assert_eq!(
        cassowary.borrow().stats,
        Statistics {
            attack: 3,
            health: 5
        }
    );
}

#[test]
fn test_shop_leech_team() {
    let mut team = test_leech_team();

    let pets = team.all();
    let peacock = pets.first().unwrap();
    let leech = pets.last().unwrap();

    // Starting stats.
    assert!(
        peacock.borrow().stats == Statistics::new(2, 5).unwrap()
            && leech.borrow().stats == Statistics::new(4, 2).unwrap()
    );

    team.open_shop().unwrap().close_shop().unwrap();

    let pets = team.all();
    let peacock = pets.first().unwrap();
    let leech = pets.last().unwrap();

    // Leech damages peacock by (0,1) causing it to gain (4,0).
    // Then it gains (0, 1)
    assert!(
        peacock.borrow().stats == Statistics::new(6, 4).unwrap()
            && leech.borrow().stats == Statistics::new(4, 3).unwrap()
    );
}

#[test]
fn test_shop_okapi_team() {
    let mut team = test_okapi_team();

    team.open_shop().unwrap();

    let okapi = team.first().unwrap();
    let starting_stats = okapi.borrow().stats;
    const STATS_PER_ROLL: Statistics = Statistics {
        attack: 1,
        health: 1,
    };
    const NUM_ROLLS: usize = 5;

    // Roll 5 times.
    for i in 1..=NUM_ROLLS {
        let added_stats = STATS_PER_ROLL * Statistics::new(i, i).unwrap();

        team.roll_shop().unwrap();

        // Every rolls adds (1,1)
        assert_eq!(
            starting_stats + added_stats,
            team.first().unwrap().borrow().stats
        )
    }
    let final_stats = okapi.borrow().stats;
    // Try rolling an additional time.
    team.roll_shop().unwrap();

    // Stats don't increase as max number of uses per turn reached.
    assert_eq!(final_stats, okapi.borrow().stats);
}

#[test]
fn test_shop_starfish_team() {
    let mut team = test_starfish_team();

    // Duck has Sell trigger.
    let duck = team.first().unwrap();
    assert!(
        duck.borrow().name == PetName::Duck
            && duck
                .borrow()
                .effect
                .iter()
                .any(|effect| effect.trigger.status == Status::Sell)
    );
    // Dog will be targeted as startfish cannot target itself.
    assert_eq!(
        team.last().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    team.open_shop().unwrap().sell(&Position::First).unwrap();

    // Gains (1,1)
    assert_eq!(
        team.last().unwrap().borrow().stats,
        Statistics {
            attack: 4,
            health: 5
        }
    );
}
