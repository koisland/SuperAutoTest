use crate::{
    effects::{
        state::{Position, Status},
        trigger::TRIGGER_START_BATTLE,
    },
    pets::names::PetName,
    teams::{combat::TeamCombat, team::TeamFightOutcome, viewer::TeamViewer},
    tests::common::{
        test_ant_team, test_atlantic_puffin_team, test_bat_team, test_crab_team, test_dodo_team,
        test_dove_team, test_dromedary_team, test_elephant_peacock_team, test_flamingo_team,
        test_goldfish_team, test_hedgehog_team, test_jellyfish_team, test_koala_team,
        test_mammoth_team, test_panda_team, test_pug_team, test_racoon_team, test_rat_team,
        test_salamander_team, test_shrimp_team, test_skunk_team, test_spider_team, test_stork_team,
        test_swan_team, test_tabby_cat_team, test_toucan_team, test_wombat_team, test_yak_team,
    },
    Entity, Food, FoodName, Pet, Shop, ShopItem, ShopItemViewer, ShopViewer, Statistics, Team,
    TeamEffects, TeamShopping,
};

#[test]
fn test_battle_hedgehog_team() {
    let mut team = test_hedgehog_team();
    let mut enemy_team = test_ant_team();

    let mut fight = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team).unwrap()
    }

    assert_eq!(fight, TeamFightOutcome::Draw);
}

#[test]
fn test_battle_elephant_peacock_team() {
    let mut team = test_elephant_peacock_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 2,
            health: 5
        }
    );
    team.fight(&mut enemy_team).unwrap();

    // Lvl.1 elephant deals 1 dmg once to pet at back.
    // Lvl.1 peacock gains 4 atk.
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 6,
            health: 4
        }
    );
}

#[test]
fn test_battle_crab_team() {
    let mut team = test_crab_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 1
        }
    );
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 2,
            health: 50
        }
    );
    team.fight(&mut enemy_team).unwrap();

    // Crab at lvl. 1 copies 25 from big ant at pos 2.
    // Gets hit for 2 dmg.
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 23
        }
    );
}

#[test]
fn test_battle_dodo_team() {
    let mut team = test_dodo_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
    // Dodo atk at lvl. 1 is 3.
    // 3 * 0.33 = 1.
    assert_eq!(
        (team.nth(1).unwrap().borrow().stats.attack as f32 * 0.33).round(),
        1.0
    );
    team.fight(&mut enemy_team).unwrap();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 4,
            health: 1
        }
    );
}

#[test]
fn test_battle_flamingo_team() {
    let mut team = test_flamingo_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    assert_eq!(
        team.nth(2).unwrap().borrow().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    team.fight(&mut enemy_team).unwrap();

    // Flamingo faints giving two pets behind (1, 1).
    assert_eq!(
        team.nth(0).unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 2
        }
    );
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 2
        }
    );
}

#[test]
fn test_battle_rat_lvl_1_team() {
    let mut team_lvl_1 = test_rat_team(1);
    let mut enemy_team_lvl_1 = test_rat_team(1);

    team_lvl_1.fight(&mut enemy_team_lvl_1).unwrap();
    team_lvl_1.fight(&mut enemy_team_lvl_1).unwrap();

    assert_eq!(team_lvl_1.first().unwrap().borrow().name, PetName::DirtyRat);
    assert_eq!(
        enemy_team_lvl_1.first().unwrap().borrow().name,
        PetName::DirtyRat
    );
}

#[test]
fn test_battle_rat_lvl_2_team() {
    let mut team_lvl_2 = test_rat_team(2);
    let mut enemy_team_lvl_2 = test_rat_team(2);

    // Both rats are level 2.
    assert_eq!(team_lvl_2.first().unwrap().borrow().lvl, 2);
    assert_eq!(enemy_team_lvl_2.first().unwrap().borrow().lvl, 2);

    team_lvl_2.fight(&mut enemy_team_lvl_2).unwrap();
    team_lvl_2.fight(&mut enemy_team_lvl_2).unwrap();

    // Both rats die and summon two dirty rats.
    assert_eq!(team_lvl_2.all().len(), 2);
    assert_eq!(enemy_team_lvl_2.all().len(), 2);

    // All pets on both teams are dirty rats.
    for team in [team_lvl_2, enemy_team_lvl_2].iter_mut() {
        for pet_name in team.all().iter() {
            assert_eq!(pet_name.borrow().name, PetName::DirtyRat)
        }
    }
}

#[test]
fn test_battle_spider_team() {
    let mut team = test_spider_team();
    let mut enemy_team = test_spider_team();

    team.fight(&mut enemy_team).unwrap();

    // Spiders kill themselves and both spawn a random tier 3 pet from the Turtle pack.
    assert_eq!(team.first().unwrap().borrow().tier, 3);
    assert_eq!(enemy_team.first().unwrap().borrow().tier, 3);
}

#[test]
fn test_battle_bat_team() {
    let mut team = test_bat_team();
    let mut enemy_team = test_skunk_team();

    team.fight(&mut enemy_team).unwrap();

    // Skunk takes additional 3 damage from weakness.
    assert_eq!(
        enemy_team.first().unwrap().borrow().stats,
        Statistics::new(3, 1).unwrap()
    );
    assert_eq!(
        enemy_team
            .first()
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
fn test_battle_atlantic_puffin_team() {
    let mut team = test_atlantic_puffin_team();
    let mut enemy_team = test_mammoth_team();
    team.set_seed(Some(0));

    // Dog at 4th position is 4.
    assert_eq!(enemy_team.nth(4).unwrap().borrow().stats.health, 4);
    // Two strawberries on team.
    assert_eq!(
        team.all()
            .iter()
            .map(|pet| pet.borrow().item.as_ref().map_or(0, |item| {
                if item.name == FoodName::Strawberry {
                    1
                } else {
                    0
                }
            }))
            .sum::<usize>(),
        2
    );
    // Activate start of battle effects.
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();
    // Dog took 4 damage from puffin. 2 dmg x 2 strawberries.
    let targeted_dog = enemy_team.friends.get(4).unwrap();
    let dog_health = targeted_dog.as_ref().unwrap().borrow().stats.health;
    assert_eq!(dog_health, 0);
}

#[test]
fn test_battle_dove_team() {
    let mut team = test_dove_team();
    team.set_seed(Some(11));
    let mut enemy_team = test_mammoth_team();

    team.fight(&mut enemy_team).unwrap();

    // Lvl 1 dove faints.
    let dove = team.fainted.get(0).unwrap();
    assert_eq!(dove.as_ref().unwrap().borrow().name, PetName::Dove);
    for i in 0..2 {
        // First two strawberry friends get (2,2)
        assert_eq!(
            team.nth(i).unwrap().borrow().stats,
            Statistics::new(4, 3).unwrap()
        );
        assert_eq!(
            team.nth(i).unwrap().borrow().item.as_ref().unwrap().name,
            FoodName::Strawberry
        )
    }
}

#[test]
fn test_battle_koala_team() {
    let mut team = test_koala_team();
    let mut enemy_team = test_mammoth_team();

    // Original koala stats.
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics::new(1, 2).unwrap()
    );

    // Fight and mammoth hurt.
    team.fight(&mut enemy_team).unwrap();

    let buffed_stats = Statistics::new(2, 3).unwrap();
    assert_eq!(team.nth(1).unwrap().borrow().stats, buffed_stats);

    // Fight again and mammoth hurt.
    team.fight(&mut enemy_team).unwrap();

    // No change since single use.
    assert_eq!(team.nth(1).unwrap().borrow().stats, buffed_stats);
}

#[test]
fn test_battle_panda_team() {
    let mut team = test_panda_team();
    let mut enemy_team = test_mammoth_team();

    // Adds 50% of attack (1,0).
    let add_stats = team.nth(1).unwrap().borrow().stats.mult_perc(&Statistics {
        attack: 50,
        health: 50,
    });
    assert_eq!(add_stats, Statistics::new(1, 2).unwrap());
    // Initial dog stats.
    let original_stats = team.first().unwrap().borrow().stats;

    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        original_stats + add_stats
    );
    team.clear_team();

    // Panda died.
    let first_fainted_pet = team.fainted[0].as_ref().unwrap();
    assert_eq!(first_fainted_pet.borrow().name, PetName::Panda);
}

#[test]
fn test_battle_pug_team() {
    let mut team = test_pug_team();
    let mut enemy_team = test_mammoth_team();

    // Pug has lvl. 1 with 1 exp.
    assert_eq!(team.first().unwrap().borrow().exp, 1);
    assert_eq!(team.first().unwrap().borrow().lvl, 1);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(3, 2).unwrap()
    );
    // Activate start of battle effect of pug.
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Ant levels up.
    assert_eq!(team.first().unwrap().borrow().exp, 2);
    assert_eq!(team.first().unwrap().borrow().lvl, 2);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(4, 3).unwrap()
    );
}

#[test]
fn test_battle_stork_team() {
    let mut team = test_stork_team();
    let mut enemy_team = test_mammoth_team();

    team.fight(&mut enemy_team).unwrap();

    // TODO: Currently, has no tier information so uses tier 1 ( (stork tier) 2 - 1) by default.
    assert_eq!(team.first().unwrap().borrow().tier, 1);
    let first_fainted_pet = team.fainted.first().unwrap();
    assert_eq!(
        first_fainted_pet.as_ref().unwrap().borrow().name,
        PetName::Stork
    );
}

#[test]
fn test_battle_racoon_team() {
    let mut team = test_racoon_team();
    let mut enemy_team = test_mammoth_team();

    let racoon = team.first().unwrap();
    let mammoth = enemy_team.first().unwrap();
    // Give melon to first pet.
    enemy_team
        .set_item(
            &Position::First,
            Some(Food::try_from(FoodName::Garlic).unwrap()),
        )
        .unwrap();

    // No item for racoon. Mammoth has garlic.
    assert_eq!(racoon.borrow().item, None);
    let mammoth_item = mammoth.borrow().item.as_ref().unwrap().name.clone();
    assert_eq!(mammoth_item, FoodName::Garlic);

    // Trigger attack.
    team.fight(&mut enemy_team).unwrap();

    // Racoon got mammoth's melon. Mammoth loses garlic.
    let racoon_item = racoon.borrow().item.as_ref().unwrap().name.clone();
    assert_eq!(racoon_item, FoodName::Garlic);
    assert!(mammoth.borrow().item.is_none());
}

#[test]
fn test_battle_toucan_team() {
    let mut team = test_toucan_team();
    let mut enemy_team = test_mammoth_team();

    // Toucan has honey.
    let toucan = team.first().unwrap();
    assert_eq!(
        toucan.borrow().item.as_ref().unwrap().name.clone(),
        FoodName::Honey
    );
    // Dog behind toucan has no item.
    let dog = team.nth(1).unwrap();
    assert_eq!(dog.borrow().item, None);
    team.fight(&mut enemy_team).unwrap();

    // Dog behind bee now has honey.
    assert_eq!(dog.borrow().item.as_ref().unwrap().name, FoodName::Honey);
}

#[test]
fn test_battle_wombat_team() {
    let mut team = test_wombat_team();
    let mut enemy_team = test_mammoth_team();
    // Mammoth faint effect.
    // Note: No owners are attached to this effect.
    let mammoth_effect = enemy_team.first().unwrap().borrow().get_effect(1).unwrap();

    // Activate start of battle.
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    // Wombat gains mammoth's effect.
    let wombat_effect = team
        .first()
        .unwrap()
        .borrow_mut()
        .effect
        .first_mut()
        .unwrap()
        .assign_owner(None)
        .to_owned();
    let mammoth_effect = mammoth_effect.first().unwrap();

    assert_eq!(&wombat_effect, mammoth_effect)
}

#[test]
fn test_shop_shrimp_team() {
    let mut team = test_shrimp_team();
    team.set_seed(Some(12)).open_shop().unwrap();

    let (ant_1, ant_2, shrimp) = (
        team.nth(0).unwrap(),
        team.nth(1).unwrap(),
        team.nth(2).unwrap(),
    );

    assert_eq!(
        ant_1.borrow().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    assert_eq!(
        ant_2.borrow().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    assert_eq!(
        shrimp.borrow().stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );

    team.sell(&Position::First).unwrap();

    // Shrimp got (0,1) from sell.
    assert_eq!(
        shrimp.borrow().stats,
        Statistics {
            attack: 2,
            health: 4
        }
    );

    // Sell shrimp
    team.sell(&Position::Last).unwrap();

    // Shrimp doesn't activate on self, ant at same stats.
    assert_eq!(
        ant_2.borrow().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
}

#[test]
fn test_shop_swan_team() {
    let mut team = test_swan_team();

    assert_eq!(team.gold(), 10);

    team.open_shop().unwrap();

    assert_eq!(team.gold(), 11);
}

#[test]
fn test_shop_frigate_bird_team() {
    let mut team = test_elephant_peacock_team();
    let mut shop = Shop::default();
    // Add frigatebird
    shop.add_item(ShopItem::from(Pet::try_from(PetName::Frigatebird).unwrap()))
        .unwrap();

    // Peacock has hurt ability.
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics::new(2, 5).unwrap()
    );
    // Replace shop.
    team.replace_shop(shop)
        .unwrap()
        .open_shop()
        .unwrap()
        .buy(&Position::First, &Entity::Pet, &Position::Last)
        .unwrap();

    // Peacock gains two health after purchase.
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics::new(2, 7).unwrap()
    );
}

#[test]
fn test_shop_goldfish_team() {
    let mut team = test_goldfish_team();
    team.open_shop().unwrap();

    let affected_pos = Position::Multiple(vec![Position::First, Position::Relative(-1)]);
    let affected_shop_pets = team
        .get_shop()
        .get_shop_items_by_pos(&affected_pos, &Entity::Pet)
        .unwrap();
    // Pets are discounted by 1 gold from 3 gold to 2 gold.
    assert!(affected_shop_pets.iter().all(|pet| pet.cost == 2));
}

#[test]
fn test_shop_dromedary_team() {
    let mut team = test_dromedary_team();
    team.set_shop_seed(Some(12)).open_shop().unwrap();

    let affected_pos = Position::Multiple(vec![Position::First, Position::Relative(-1)]);
    let affected_shop_pets = team
        .get_shop()
        .get_shop_items_by_pos(&affected_pos, &Entity::Pet)
        .unwrap();
    let (def_mosq, def_pig) = (
        Pet::try_from(PetName::Mosquito).unwrap(),
        Pet::try_from(PetName::Pig).unwrap(),
    );
    let (mosq, pig) = (
        affected_shop_pets.first().unwrap(),
        affected_shop_pets.get(1).unwrap(),
    );

    // Mosquito and pig are (1,1) higher than default.
    assert!(
        mosq.attack_stat().unwrap() == def_mosq.stats.attack + 1
            && mosq.health_stat().unwrap() == def_mosq.stats.health + 1
    );
    assert!(
        pig.attack_stat().unwrap() == def_pig.stats.attack + 1
            && pig.health_stat().unwrap() == def_pig.stats.health + 1
    );
}

#[test]
fn test_shop_tabbycat_team() {
    let mut team = test_tabby_cat_team();
    team.set_shop_seed(Some(12)).open_shop().unwrap();

    assert!(team.last().unwrap().borrow().item.is_none());

    let first_pet = team.first().unwrap();
    let second_pet = team.nth(1).unwrap();

    assert_eq!(first_pet.borrow().stats, Statistics::new(2, 1).unwrap());
    assert_eq!(second_pet.borrow().stats, Statistics::new(2, 1).unwrap());
    // Buy food on tabby.
    team.buy(&Position::First, &Entity::Food, &Position::Last)
        .unwrap();

    assert!(team.last().unwrap().borrow().item.is_some());

    assert_eq!(first_pet.borrow().stats, Statistics::new(3, 1).unwrap());
    assert_eq!(second_pet.borrow().stats, Statistics::new(3, 1).unwrap());
}

#[test]
fn test_shop_guinea_pig_team() {
    let mut team = Team::default();
    let mut shop = Shop::default();
    // Add guinea pig.
    shop.add_item(ShopItem::from(Pet::try_from(PetName::GuineaPig).unwrap()))
        .unwrap();

    assert!(team.all().is_empty());
    // Replace shop.
    team.replace_shop(shop)
        .unwrap()
        .open_shop()
        .unwrap()
        .buy(&Position::First, &Entity::Pet, &Position::Last)
        .unwrap();

    // Two guinea pigs after purchase of one guinea pig.
    let pets = team.all();
    assert!(
        pets.len() == 2
            && pets
                .iter()
                .all(|pet| pet.borrow().name == PetName::GuineaPig)
    )
}

#[test]
fn test_shop_jellyfish_team() {
    let mut team = test_jellyfish_team();

    let leveled_pet = team.nth(2).unwrap();
    let jellyfish = team.last().unwrap();
    assert!(leveled_pet.borrow().lvl == 1);
    assert_eq!(jellyfish.borrow().stats, Statistics::new(2, 3).unwrap());
    // Merge pets to reach level 2.
    team.move_pets(&Position::First, &Position::Relative(-2), true)
        .unwrap();
    team.move_pets(&Position::First, &Position::Relative(-1), true)
        .unwrap();

    // Pet reached level 2.
    assert!(leveled_pet.borrow().lvl == 2);
    // Jellyfish gets (1,1)
    assert_eq!(jellyfish.borrow().stats, Statistics::new(3, 4).unwrap());
}

#[test]
fn test_shop_salamander_team() {
    let mut team = test_salamander_team();
    team.set_shop_seed(Some(12)).open_shop().unwrap();

    let salamander = team.first().unwrap();
    assert_eq!(
        salamander.borrow().stats,
        Statistics {
            attack: 2,
            health: 4
        }
    );

    team.buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap();

    // Pet bought has start of battle trigger.
    assert_eq!(
        team.first().unwrap().borrow().effect[0].trigger.status,
        Status::StartOfBattle
    );
    // Salamander gains +2 atk as result.
    assert_eq!(
        salamander.borrow().stats,
        Statistics {
            attack: 4,
            health: 4
        }
    );
}

#[test]
fn test_shop_yak_team() {
    let mut team = test_yak_team();
    team.open_shop().unwrap();

    let yak = team.first().unwrap();

    assert_eq!(
        yak.borrow().stats,
        Statistics {
            attack: 3,
            health: 5
        }
    );

    // End turn.
    team.close_shop().unwrap();

    // Yak gains (1, -1).
    assert_eq!(
        yak.borrow().stats,
        Statistics {
            attack: 4,
            health: 4
        }
    );
}
