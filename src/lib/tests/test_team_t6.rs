use std::ops::RangeFrom;

use itertools::Itertools;

use crate::{
    effects::{
        state::{Position, Status},
        stats::Statistics,
        trigger::TRIGGER_START_BATTLE,
    },
    foods::{food::Food, names::FoodName},
    pets::names::PetName,
    teams::{combat::TeamCombat, team::TeamFightOutcome, viewer::TeamViewer},
    tests::common::{
        count_pets, test_alpaca_team, test_boar_team, test_cat_team, test_chicken_team,
        test_cricket_horse_team, test_dragon_team, test_fly_team, test_gorilla_team,
        test_hammershark_team, test_komodo_team, test_leopard_team, test_lioness_team,
        test_lionfish_team, test_mammoth_team, test_octopus_team, test_orca_team,
        test_ostrich_team, test_piranha_team, test_reindeer_team, test_sabertooth_team,
        test_sauropod_team, test_scorpion_team, test_sheep_team, test_snake_team,
        test_spinosaurus_team, test_stegosaurus_team, test_tapir_team, test_tiger_team,
        test_tyrannosaurus_team, test_velociraptor_team, test_walrus_team, test_white_tiger_team,
    },
    Entity, ItemCondition, Pet, ShopItem, ShopItemViewer, ShopViewer, Team, TeamEffects,
    TeamShopping,
};

#[test]
fn test_battle_lionfish_team() {
    let mut team = test_lionfish_team();
    let mut enemy_team = test_mammoth_team();

    todo!()
}

#[test]
fn test_battle_boar_team() {
    let mut team = test_boar_team();
    let mut enemy_team = test_sheep_team();

    let original_boar_stats = team.first().unwrap().read().unwrap().stats;
    assert_eq!(
        original_boar_stats,
        Statistics {
            attack: 10,
            health: 6
        }
    );
    team.fight(&mut enemy_team).unwrap();

    // After battle with first sheep (2,2) gains (4,2)
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        original_boar_stats
            + Statistics {
                attack: 0,
                health: -2
            }
            + Statistics {
                health: 2,
                attack: 4
            }
    );
}

#[test]
fn test_battle_fly_team() {
    let mut team = test_fly_team();
    let mut enemy_team = test_fly_team();

    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();

    // Zombie fly spawned after cricket dies.
    assert_eq!(
        team.first().unwrap().read().unwrap().name,
        PetName::ZombieFly
    );
    assert_eq!(
        team.nth(1).unwrap().read().unwrap().name,
        PetName::ZombieCricket
    );

    // Finish battle.
    let mut outcome = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = outcome {
        outcome = team.fight(&mut enemy_team).unwrap()
    }

    assert_eq!(outcome, TeamFightOutcome::Draw);
    // Only three zombie flies spawned with a total of 4 valid faint triggers.
    let total_valid_faint_triggers = count_pets(&team.fainted, PetName::Cricket)
        + count_pets(&team.fainted, PetName::ZombieCricket);
    assert!(count_pets(&team.fainted, PetName::ZombieFly) == 3 && total_valid_faint_triggers == 4)
}

#[test]
fn test_battle_gorilla_team() {
    let mut team = test_gorilla_team();
    let mut enemy_team = test_gorilla_team();

    // Gorilla has no items before fight.
    assert_eq!(team.first().unwrap().read().unwrap().item, None);
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics {
            attack: 6,
            health: 9
        }
    );
    team.fight(&mut enemy_team).unwrap();

    // Gorilla is hurt and gains coconut.
    let mut coconut = Food::try_from(FoodName::Coconut).unwrap();
    coconut.ability.assign_owner(Some(&team.first().unwrap()));

    assert_eq!(team.first().unwrap().read().unwrap().item, Some(coconut));
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics {
            attack: 6,
            health: 3
        }
    );
}

#[test]
fn test_battle_leopard_team() {
    let mut team = test_leopard_team();
    let mut enemy_team = test_gorilla_team();

    // One leopard on team.
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics {
            attack: 10,
            health: 4
        }
    );
    // One gorilla on enemy team.
    assert_eq!(
        enemy_team.first().unwrap().read().unwrap().stats,
        Statistics {
            attack: 6,
            health: 9
        }
    );

    // Leopard activates at start of battle and deals 50% of leopard attack.
    team.fight(&mut enemy_team).unwrap();

    assert_eq!(
        enemy_team.first().unwrap().read().unwrap().stats,
        Statistics {
            attack: 6,
            health: 4
        }
    );
}

#[test]
fn test_battle_mammoth_team() {
    let mut team = test_mammoth_team();
    let mut enemy_team = test_mammoth_team();

    // Stats of every pet after mammoth.
    for team in [&team, &enemy_team].into_iter() {
        for pet in team.friends.get(1..).unwrap().iter().flatten() {
            assert_eq!(
                pet.read().unwrap().stats,
                Statistics {
                    attack: 3,
                    health: 4
                }
            )
        }
    }

    // 4 attack phases to kill mammoth.
    for _ in 0..4 {
        team.fight(&mut enemy_team).unwrap();
    }

    // All pets on team gained (2,2)
    for team in [&team, &enemy_team].into_iter() {
        for pet in team.friends.iter().flatten() {
            assert_eq!(
                pet.read().unwrap().stats,
                Statistics {
                    attack: 5,
                    health: 6
                }
            )
        }
    }
}

#[test]
fn test_battle_snake_team() {
    let mut team = test_snake_team();
    let mut enemy_team = test_sheep_team();

    {
        // Frontline cricket won't kill enemy sheep in single turn.
        assert_eq!(
            team.first().unwrap().read().unwrap().stats,
            Statistics {
                attack: 1,
                health: 2
            }
        );
        let enemy_sheep = enemy_team.first().unwrap();
        assert_eq!(
            enemy_sheep.read().unwrap().stats,
            Statistics {
                attack: 2,
                health: 2
            }
        );
    }

    // One battle phase passes.
    // Cricket attacks and snake triggers killing sheep.
    team.fight(&mut enemy_team).unwrap();

    // Two ram spawn as result.
    for pet in enemy_team.all() {
        assert_eq!(pet.read().unwrap().name, PetName::Ram);
    }
}

#[test]
fn test_battle_tiger_team() {
    let mut team = test_tiger_team();
    let mut enemy_team = test_scorpion_team();
    // Add extra scorpion.
    enemy_team
        .add_pet(Pet::try_from(PetName::Scorpion).unwrap(), 1, None)
        .unwrap();

    {
        // Team of leopard and tiger.
        let pets = team.all();
        assert_eq!(pets.get(0).unwrap().read().unwrap().name, PetName::Leopard);
        assert_eq!(pets.get(1).unwrap().read().unwrap().name, PetName::Tiger);
        assert_eq!(pets.len(), 2)
    }
    {
        // Enemy team of two scorpions.
        let enemy_pets = enemy_team.all();
        assert_eq!(
            enemy_pets.get(0).unwrap().read().unwrap().name,
            PetName::Scorpion
        );
        assert_eq!(
            enemy_pets.get(1).unwrap().read().unwrap().name,
            PetName::Scorpion
        );
        assert_eq!(enemy_pets.len(), 2)
    }
    // Start of battle triggers leopard effect twice (due to tiger behind it) against scorpion team.
    team.fight(&mut enemy_team).unwrap();

    // Frontline leopard lives because its effect triggers twice.
    let pets = team.all();
    assert_eq!(pets.get(0).unwrap().read().unwrap().name, PetName::Leopard);
    assert_eq!(pets.get(1).unwrap().read().unwrap().name, PetName::Tiger);
}

#[test]
fn test_battle_alpaca_team() {
    let mut team = test_alpaca_team();
    let mut enemy_team = test_gorilla_team();

    assert_eq!(count_pets(&team.friends, PetName::Alpaca), 2);
    // First alpaca has mushroom so will respawn as (1,1)
    assert_eq!(
        team.nth(1)
            .unwrap()
            .read()
            .unwrap()
            .item
            .as_ref()
            .unwrap()
            .name,
        FoodName::Mushroom
    );

    for i in 0..4 {
        team.fight(&mut enemy_team).unwrap();
        // Cricket spawns and gets two exp leveling it to lvl 2.
        if i == 0 {
            let zombie_cricket = team.first().unwrap();
            assert_eq!(zombie_cricket.read().unwrap().lvl, 2);
            assert_eq!(zombie_cricket.read().unwrap().exp, 2);
        };
        // Alpaca respawns.
        if i == 3 {
            let respawned_alpaca = team.first().unwrap();
            // Alpaca summoned but doesn't get exp from remaining alpaca.
            assert_eq!(respawned_alpaca.read().unwrap().exp, 0);
        }
    }
}

#[test]
fn test_battle_tapir_team() {
    let mut team = test_tapir_team();
    let mut enemy_team = test_gorilla_team();

    team.fight(&mut enemy_team).unwrap();

    // Tapir faints and a tiger spawns at lvl.1.
    assert_eq!(team.fainted.len(), 1);
    let spawned_pet = team.first().unwrap();
    assert!(
        spawned_pet.read().unwrap().name == PetName::Tiger && spawned_pet.read().unwrap().lvl == 1
    );

    team.restore();

    // Level tapir to lvl 2.
    team.set_level(&Position::First, 2).unwrap();

    team.fight(&mut enemy_team).unwrap();

    // Same tiger spawns but at lvl 2.
    let spawned_pet = team.first().unwrap();
    assert!(
        spawned_pet.read().unwrap().name == PetName::Tiger && spawned_pet.read().unwrap().lvl == 2
    );
}

#[test]
fn test_battle_walrus_team() {
    let mut team = test_walrus_team();
    team.set_seed(Some(25));
    let mut enemy_team = test_gorilla_team();

    team.fight(&mut enemy_team).unwrap();

    // First cricket after walrus faints gets peanuts.
    assert_eq!(
        team.first()
            .unwrap()
            .read()
            .unwrap()
            .item
            .as_ref()
            .unwrap()
            .name,
        FoodName::Peanut
    );
}

#[test]
fn test_battle_white_tiger_team() {
    let mut team = test_white_tiger_team();
    let mut enemy_team = test_gorilla_team();

    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Deer behind gets 3 exp leveling to 2.
    let deer = team.nth(1).unwrap();
    assert!(deer.read().unwrap().lvl == 2 && deer.read().unwrap().exp == 3);
}

#[test]
fn test_battle_octopus_team() {
    let mut team = test_octopus_team();
    let mut enemy_team = test_cricket_horse_team();
    team.set_seed(Some(10));

    team.fight(&mut enemy_team).unwrap();

    // Octopus only takes one damage.
    const OCTOPUS_HEALTH: Statistics = Statistics {
        attack: 8,
        health: 8,
    };
    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        OCTOPUS_HEALTH - Statistics::new(0, 1).unwrap()
    );
    // Octopus snipes horse. And octopus direct attacks first cricket.
    assert_eq!(
        enemy_team
            .fainted
            .iter()
            .flatten()
            .map(|pet| pet.read().unwrap().name.clone())
            .collect_vec(),
        vec![PetName::Cricket, PetName::Horse]
    );
    // Horse killed first by snipe as seen by zombie cricket stats being the default.
    assert_eq!(
        enemy_team.first().unwrap().read().unwrap().stats,
        Pet::try_from(PetName::ZombieCricket).unwrap().stats
    )
}

#[test]
fn test_battle_orca_team() {
    let mut team = test_orca_team();
    team.set_seed(Some(25));
    let mut enemy_team = test_gorilla_team();

    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();

    let summoned_pet = team.first().unwrap();
    let effect = &summoned_pet.read().unwrap().effect[0];
    let effect_trigger_status = effect.trigger.status.clone();
    // Summoned pet is a pet with a faint trigger.
    assert_eq!(effect_trigger_status, Status::Faint);
}

#[test]
fn test_battle_piranha_team() {
    let mut team = test_piranha_team();
    let mut enemy_team = test_gorilla_team();

    for pet in team.all().get(1..).unwrap() {
        assert!(pet.read().unwrap().stats.attack == 1)
    }

    // Piranha (lvl.1) faints.
    team.fight(&mut enemy_team).unwrap();

    // And all pets behind get (3,0).
    for pet in team.all() {
        assert!(pet.read().unwrap().stats.attack == 4)
    }
}

#[test]
fn test_battle_reindeer_team() {
    let mut team = test_reindeer_team();
    let mut enemy_team = test_gorilla_team();

    // No item before fight.
    let reindeer = team.first().unwrap();
    assert_eq!(reindeer.read().unwrap().item, None);

    team.fight(&mut enemy_team).unwrap();

    // After fight has melon but already used.
    assert_eq!(
        reindeer.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Melon
    );
    assert_eq!(
        reindeer.read().unwrap().item.as_ref().unwrap().ability.uses,
        Some(0)
    );
}

#[test]
fn test_battle_sabertooth_team() {
    let mut team = test_sabertooth_team();
    let mut enemy_team = test_cricket_horse_team();

    let sabertooth_stats = team.first().unwrap().read().unwrap().stats;

    team.fight(&mut enemy_team).unwrap();

    // Sabertooth hurt.
    assert_ne!(sabertooth_stats, team.nth(1).unwrap().read().unwrap().stats);
    let summoned_pet = team.first().unwrap();
    // Tier 1, lvl 1, pet at fixed (8,8) summoned.
    assert!(
        summoned_pet.read().unwrap().stats == Statistics::new(8, 8).unwrap()
            && summoned_pet.read().unwrap().tier == 1
            && summoned_pet.read().unwrap().lvl == 1
    );
}

#[test]
fn test_battle_spinosaurus_team() {
    let mut team = test_spinosaurus_team();
    team.set_seed(Some(52));
    let mut enemy_team = test_piranha_team();

    // Dog at pos 1 to get buff is (3,4).
    let dog = team.nth(1).unwrap();
    let dog_stats_original = dog.read().unwrap().stats;
    assert_eq!(dog_stats_original, Statistics::new(3, 4).unwrap());

    // Dog at pos 0 faints
    team.fight(&mut enemy_team).unwrap();

    // Dog gains (3,2) from spinosaurus.
    assert_eq!(team.fainted.len(), 1);
    assert_eq!(
        dog_stats_original + Statistics::new(3, 2).unwrap(),
        dog.read().unwrap().stats
    );
}

#[test]
fn test_battle_stegosaurus_team() {
    let mut team = test_stegosaurus_team();
    let mut enemy_team = test_gorilla_team();

    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics::new(3, 4).unwrap()
    );

    // Current turn is 1 so stego should give (1/1 * 1) = (1/1)
    assert!(team.history.curr_turn == 1);

    // Activate start of battle effects.
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics::new(4, 5).unwrap()
    );

    // Reset team.
    team.restore();

    // Increase turns to 3. Stego should give (1/1 * 3) = (3/3)
    team.history.curr_turn += 2;

    // Re-activate start of battle effects.
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    assert_eq!(
        team.first().unwrap().read().unwrap().stats,
        Statistics::new(6, 7).unwrap()
    );
}

#[test]
fn test_battle_velociraptor_team() {
    let mut team = test_velociraptor_team();
    team.set_seed(Some(12));
    let mut enemy_team = test_gorilla_team();

    // Cricket at front has strawberry.
    assert_eq!(
        team.first()
            .unwrap()
            .read()
            .unwrap()
            .item
            .as_ref()
            .unwrap()
            .name,
        FoodName::Strawberry
    );
    // Trigger start of battle effects.
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    // Cricket at front now has coconut.
    assert_eq!(
        team.first()
            .unwrap()
            .read()
            .unwrap()
            .item
            .as_ref()
            .unwrap()
            .name,
        FoodName::Coconut
    );
}

#[test]
fn test_shop_cat_team() {
    let mut team = test_cat_team();

    team.set_shop_seed(Some(19)).open_shop().unwrap();

    let (pos, item_type) = (Position::First, Entity::Food);
    let items = team
        .get_shop()
        .get_shop_items_by_pos(&pos, &item_type)
        .unwrap();
    // Apple has (1,1) buff.
    assert!(
        items.first().unwrap().attack_stat() == Some(1)
            && items.first().unwrap().health_stat() == Some(1)
    );

    let cat = team.first().unwrap();
    let cat_start_stats = cat.read().unwrap().stats;
    const APPLE_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };
    const CAT_MULTIPLIER: Statistics = Statistics {
        attack: 2,
        health: 2,
    };

    // Buy apple.
    team.buy(&pos, &item_type, &Position::First).unwrap();

    // Apple buff doubled.
    assert_eq!(
        cat.read().unwrap().stats,
        cat_start_stats + APPLE_BUFF * CAT_MULTIPLIER
    );
}

#[test]
fn test_shop_dragon_team() {
    let mut team = test_dragon_team();

    team.set_shop_seed(Some(19))
        .shop
        .add_item(ShopItem::new(Pet::try_from(PetName::Badger).unwrap()))
        .unwrap();

    team.open_shop().unwrap();
    let pets = team
        .shop
        .get_shop_items_by_pos(&Position::All(ItemCondition::None), &Entity::Pet)
        .unwrap();
    let (badger, beaver) = (pets.first().unwrap(), pets.get(1).unwrap());
    let beaver_shop_stats =
        Statistics::new(beaver.attack_stat().unwrap(), beaver.health_stat().unwrap()).unwrap();
    // Dog won't proc dragon but beaver will.
    assert!(badger.tier() == 3 && beaver.tier() == 1);
    let team_pets = team.all();
    let (ant, blowfish, dragon) = (
        team_pets.first().unwrap(),
        team_pets.get(1).unwrap(),
        team_pets.last().unwrap(),
    );
    let (ant_start_stats, blowfish_start_stats, dragon_start_stats) = (
        ant.read().unwrap().stats,
        blowfish.read().unwrap().stats,
        dragon.read().unwrap().stats,
    );

    const DRAGON_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };

    // Buy badger.
    team.buy(&Position::Relative(-1), &Entity::Pet, &Position::First)
        .unwrap();

    // Every pet on team gets buffed except dragon.
    let beaver = team.first().unwrap();
    assert!(
        beaver.read().unwrap().stats == beaver_shop_stats + DRAGON_BUFF
            && ant.read().unwrap().stats == ant_start_stats + DRAGON_BUFF
            && blowfish.read().unwrap().stats == blowfish_start_stats + DRAGON_BUFF
            && dragon.read().unwrap().stats == dragon_start_stats
    );

    team.buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap();

    // Buying a non-tier 1 pet does nothing.
    assert!(
        beaver.read().unwrap().stats == beaver_shop_stats + DRAGON_BUFF
            && ant.read().unwrap().stats == ant_start_stats + DRAGON_BUFF
            && blowfish.read().unwrap().stats == blowfish_start_stats + DRAGON_BUFF
            && dragon.read().unwrap().stats == dragon_start_stats
    );
}

#[test]
fn test_shop_lioness_team() {
    let mut team = test_lioness_team();
    const LIONESS_BUFF: Statistics = Statistics {
        attack: 2,
        health: 2,
    };
    team.open_shop().unwrap();

    fn get_shop_pet_stats(team: &Team) -> Vec<Statistics> {
        let pets = team
            .get_shop()
            .get_shop_items_by_pos(&Position::All(ItemCondition::None), &Entity::Pet)
            .unwrap();
        pets.iter()
            .map(|shop_pet| {
                Statistics::new(
                    shop_pet.attack_stat().unwrap(),
                    shop_pet.health_stat().unwrap(),
                )
                .unwrap()
            })
            .collect()
    }
    let shop_pet_stats = get_shop_pet_stats(&team);
    team.freeze_shop(&Position::All(ItemCondition::None), &Entity::Pet)
        .unwrap();

    // End turn.
    team.close_shop().unwrap();

    let new_shop_pet_stats = get_shop_pet_stats(&team);

    // Current shop pets get buff.
    for (prev, new) in shop_pet_stats.iter().zip_eq(new_shop_pet_stats) {
        assert_eq!(*prev + LIONESS_BUFF, new)
    }
    // And future pets get stats.
    assert_eq!(team.shop.perm_stats, LIONESS_BUFF)
}

#[test]
fn test_shop_chicken() {
    let mut team = test_chicken_team();

    team.set_shop_seed(Some(12)).open_shop().unwrap();
    team.print_shop();

    fn get_shop_stats(team: &Team, range: RangeFrom<usize>) -> (Statistics, Statistics) {
        let shop_pets = team
            .shop
            .get_shop_items_by_pos(&Position::All(ItemCondition::None), &Entity::Pet)
            .unwrap();
        let [pet_1, pet_2] = shop_pets.get(range).unwrap() else {
            panic!()
        };
        (
            Statistics::new(pet_1.attack_stat().unwrap(), pet_1.health_stat().unwrap()).unwrap(),
            Statistics::new(pet_2.attack_stat().unwrap(), pet_2.health_stat().unwrap()).unwrap(),
        )
    }
    const CHICKEN_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };
    let (pig_start_stats, beaver_start_stats) = get_shop_stats(&team, 1..);

    team.buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap();

    let (pig_curr_stats, beaver_curr_stats) = get_shop_stats(&team, 0..);

    // Current shop pets get buffed.
    assert_eq!(pig_start_stats + CHICKEN_BUFF, pig_curr_stats);
    assert_eq!(beaver_start_stats + CHICKEN_BUFF, beaver_curr_stats);
    // Shop has perm buff.
    assert_eq!(team.shop.perm_stats, CHICKEN_BUFF)
}

#[test]
fn test_shop_sauropod() {
    let mut team = test_sauropod_team();

    team.open_shop().unwrap();
    assert_eq!(team.gold(), 10);

    team.buy(&Position::First, &Entity::Food, &Position::First)
        .unwrap();

    // Food purchase costs 3 and 1 gold refunded.
    assert_eq!(team.gold(), 8)
}

#[test]
fn test_shop_trex() {
    let mut team = test_tyrannosaurus_team();

    team.open_shop().unwrap();

    const TREX_GOLD_REQ: usize = 3;
    const TREX_BUFF: Statistics = Statistics {
        attack: 2,
        health: 1,
    };
    let pet_start_stats = team
        .all()
        .into_iter()
        .map(|pet| pet.read().unwrap().stats)
        .collect_vec();

    // Gold is sufficient for effect trigger.
    assert!(team.gold() >= TREX_GOLD_REQ);
    team.close_shop().unwrap();

    fn check_stats(team: &Team, starting_stats: &[Statistics]) {
        for (pet, starting_stats) in team.all().into_iter().zip_eq(starting_stats) {
            // Trex gets no buff
            if pet.read().unwrap().name == PetName::Tyrannosaurus {
                assert_eq!(pet.read().unwrap().stats, *starting_stats)
            } else {
                assert_eq!(pet.read().unwrap().stats, *starting_stats + TREX_BUFF)
            }
        }
    }

    check_stats(&team, &pet_start_stats);

    team.open_shop().unwrap();

    // No gold
    team.shop.coins = 0;

    // Start are unchanged.
    check_stats(&team, &pet_start_stats)
}

#[test]
fn test_shop_hammershark() {
    let mut team = test_hammershark_team();

    // Pet on team is level 3.
    assert!(team
        .friends
        .iter()
        .flatten()
        .any(|pet| pet.read().unwrap().lvl == 3));
    // Start turn.
    team.open_shop().unwrap();
    assert_eq!(team.gold(), 13);

    // Remove level 3.
    team.sell(&Position::First).unwrap();
    team.close_shop().unwrap();
    // No level 3.
    assert!(!team
        .friends
        .iter()
        .flatten()
        .any(|pet| pet.read().unwrap().lvl == 3));

    // Start turn again.
    team.open_shop().unwrap();
    // No gold gained.
    assert_eq!(team.gold(), 10);
}

#[test]
fn test_shop_komodo() {
    let mut team = test_komodo_team();

    team.set_seed(Some(12)).open_shop().unwrap();

    let team_pets = team.all();
    let [ant, dog, komodo, tiger] = &team_pets[..] else {
        panic!()
    };

    // Ant and dog starting positions.
    assert!(ant.read().unwrap().pos == Some(0) && ant.read().unwrap().name == PetName::Ant);
    assert!(dog.read().unwrap().pos == Some(1) && dog.read().unwrap().name == PetName::Dog);

    const KOMODO_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };
    let (ant_start_stats, dog_start_stats, komodo_start_stats, tiger_start_stats) = team_pets
        .iter()
        .map(|pet| pet.read().unwrap().stats)
        .collect_tuple()
        .unwrap();

    team.close_shop().unwrap();

    // Dog and ant get swapped in process and existing references reversed.
    // Ant and dog in front of komodo swap positions.
    assert!(ant.read().unwrap().pos == Some(0) && ant.read().unwrap().name == PetName::Dog);
    assert!(dog.read().unwrap().pos == Some(1) && dog.read().unwrap().name == PetName::Ant);

    // Tiger not buffed as not ahead of komodo.
    // Komodo does not get a buff.
    assert!(
        dog.read().unwrap().stats == ant_start_stats + KOMODO_BUFF
            && ant.read().unwrap().stats == dog_start_stats + KOMODO_BUFF
            && komodo.read().unwrap().stats == komodo_start_stats
            && tiger.read().unwrap().stats == tiger_start_stats
    );
}

#[test]
fn test_shop_ostrich_team() {
    let mut team = test_ostrich_team();

    team.set_shop_seed(Some(12)).open_shop().unwrap();

    let ostrich = team.first().unwrap();
    let ostrich_start_stats = ostrich.read().unwrap().stats;
    const OSTRICH_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };

    let is_tier_5_6 = |item: &ShopItem| {
        let pet_tier = item.tier();
        pet_tier == 5 || pet_tier == 6
    };
    // No tier 5 or 6.
    assert!(!team.get_shop().pets.iter().any(is_tier_5_6));

    team.close_shop().unwrap();
    // No change in stats.
    assert_eq!(ostrich.read().unwrap().stats, ostrich_start_stats);

    team.set_shop_tier(6).unwrap().open_shop().unwrap();

    // Create new ostrich reference.
    let ostrich = team.first().unwrap();
    // Check shop for tier 5+.
    let shop = team.get_shop();
    let num_tier_5_6 = shop.pets.iter().filter(|item| is_tier_5_6(item)).count();
    // One tier 5 or above pet.
    assert_eq!(num_tier_5_6, 1);

    team.close_shop().unwrap();
    // Ostrich gains (1,1) * (1,1) = (1,1)
    assert_eq!(
        ostrich.read().unwrap().stats,
        ostrich_start_stats + OSTRICH_BUFF * Statistics::new(num_tier_5_6, num_tier_5_6).unwrap()
    );
}
