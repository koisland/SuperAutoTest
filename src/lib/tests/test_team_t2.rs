use crate::{
    battle::state::{Statistics, TeamFightOutcome},
    pets::names::PetName,
    tests::common::{
        test_ant_team, test_atlantic_puffin_team, test_bat_team, test_crab_team, test_dodo_team,
        test_dove_team, test_elephant_peacock_team, test_flamingo_team, test_hedgehog_team,
        test_koala_team, test_mammoth_team, test_panda_team, test_pug_team, test_racoon_team,
        test_rat_team, test_skunk_team, test_spider_team, test_stork_team, test_toucan_team,
        test_wombat_team,
    },
    EffectApply, Food, FoodName,
};

#[test]
fn test_battle_hedgehog_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_hedgehog_team();
    let mut enemy_team = test_ant_team();

    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team)
    }

    assert_eq!(fight, TeamFightOutcome::Draw);
}

#[test]
fn test_battle_elephant_peacock_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_elephant_peacock_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.nth(1).unwrap().stats,
        Statistics {
            attack: 2,
            health: 5
        }
    );
    team.fight(&mut enemy_team);

    // Lvl.1 elephant deals 1 dmg once to pet at back.
    // Lvl.1 peacock gains 4 atk.
    assert_eq!(
        team.nth(1).unwrap().stats,
        Statistics {
            attack: 6,
            health: 4
        }
    );
}

#[test]
fn test_battle_crab_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_crab_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.first().unwrap().stats,
        Statistics {
            attack: 3,
            health: 1
        }
    );
    assert_eq!(
        team.nth(1).unwrap().stats,
        Statistics {
            attack: 2,
            health: 50
        }
    );
    team.fight(&mut enemy_team);

    // Crab at lvl. 1 copies 25 from big ant at pos 2.
    // Gets hit for 2 dmg.
    assert_eq!(
        team.first().unwrap().stats,
        Statistics {
            attack: 3,
            health: 23
        }
    );
}

#[test]
fn test_battle_dodo_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_dodo_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.first().unwrap().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
    // Dodo atk at lvl. 1 is 3.
    // 3 * 0.33 = 1.
    assert_eq!(
        (team.nth(1).unwrap().stats.attack as f32 * 0.33).round(),
        1.0
    );
    team.fight(&mut enemy_team);

    assert_eq!(
        team.first().unwrap().stats,
        Statistics {
            attack: 4,
            health: 1
        }
    );
}

#[test]
fn test_battle_flamingo_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_flamingo_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.nth(1).unwrap().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    assert_eq!(
        team.nth(2).unwrap().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    team.fight(&mut enemy_team);

    // Flamingo faints giving two pets behind (1, 1).
    assert_eq!(
        team.nth(0).unwrap().stats,
        Statistics {
            attack: 3,
            health: 2
        }
    );
    assert_eq!(
        team.nth(1).unwrap().stats,
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

    team_lvl_1.fight(&mut enemy_team_lvl_1);
    team_lvl_1.fight(&mut enemy_team_lvl_1);

    assert_eq!(team_lvl_1.first().unwrap().name, PetName::DirtyRat);
    assert_eq!(enemy_team_lvl_1.first().unwrap().name, PetName::DirtyRat);
}

#[test]
fn test_battle_rat_lvl_2_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();s

    let mut team_lvl_2 = test_rat_team(2);
    let mut enemy_team_lvl_2 = test_rat_team(2);

    // Both rats are level 2.
    assert_eq!(team_lvl_2.first().unwrap().lvl, 2);
    assert_eq!(enemy_team_lvl_2.first().unwrap().lvl, 2);

    team_lvl_2.fight(&mut enemy_team_lvl_2);
    team_lvl_2.fight(&mut enemy_team_lvl_2);

    // Both rats die and summon two dirty rats.
    assert_eq!(team_lvl_2.all().len(), 2);
    assert_eq!(enemy_team_lvl_2.all().len(), 2);

    // All pets on both teams are dirty rats.
    for team in [team_lvl_2, enemy_team_lvl_2].iter_mut() {
        for pet_name in team.all().iter().map(|pet| pet.name.clone()) {
            assert_eq!(pet_name, PetName::DirtyRat)
        }
    }
}

#[test]
fn test_battle_spider_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_spider_team();
    let mut enemy_team = test_spider_team();

    team.fight(&mut enemy_team);

    // Spiders kill themselves and both spawn a random tier 3 pet from the Turtle pack.
    assert_eq!(team.first().unwrap().tier, 3);
    assert_eq!(enemy_team.first().unwrap().tier, 3);
}

#[test]
fn test_battle_bat_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_bat_team();
    let mut enemy_team = test_skunk_team();

    team.fight(&mut enemy_team);

    // Skunk takes additional 3 damage from weakness.
    assert_eq!(
        enemy_team.first().unwrap().stats,
        Statistics::new(3, 1).unwrap()
    );
    assert_eq!(
        enemy_team.first().unwrap().item.as_ref().unwrap().name,
        FoodName::Weak
    );
}

#[test]
fn test_battle_atlantic_puffin_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_atlantic_puffin_team();
    let mut enemy_team = test_mammoth_team();
    enemy_team.set_seed(0);

    // Dog at 4th position is 4.
    assert_eq!(enemy_team.nth(4).unwrap().stats.health, 4);
    // Two strawberries on team.
    assert_eq!(
        team.all()
            .iter()
            .map(|pet| pet
                .item
                .as_ref()
                .map_or(0, |food| if food.name == FoodName::Strawberry {
                    1
                } else {
                    0
                }))
            .sum::<usize>(),
        2
    );
    // Activate start of battle effects.
    team.trigger_effects(&mut enemy_team);
    // Dog took 4 damage from puffin. 2 dmg x 2 strawberries.
    assert_eq!(
        enemy_team
            .friends
            .get(4)
            .unwrap()
            .as_ref()
            .unwrap()
            .stats
            .health,
        0
    )
}

#[test]
fn test_battle_dove_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_dove_team();
    let mut enemy_team = test_mammoth_team();

    team.fight(&mut enemy_team);

    // Lvl 1 dove faints.
    assert_eq!(
        team.fainted.get(0).unwrap().as_ref().unwrap().name,
        PetName::Dove
    );
    for i in 0..2 {
        // First two strawberry friends get (2,2)
        assert_eq!(team.nth(i).unwrap().stats, Statistics::new(4, 3).unwrap());
        assert_eq!(
            team.nth(i).unwrap().item.as_ref().unwrap().name,
            FoodName::Strawberry
        )
    }
}

#[test]
fn test_battle_koala_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_koala_team();
    let mut enemy_team = test_mammoth_team();

    // Original koala stats.
    assert_eq!(team.nth(1).unwrap().stats, Statistics::new(1, 2).unwrap());

    // Fight and mammoth hurt.
    team.fight(&mut enemy_team);

    let buffed_stats = Statistics::new(2, 3).unwrap();
    assert_eq!(team.nth(1).unwrap().stats, buffed_stats);

    // Fight again and mammoth hurt.
    team.fight(&mut enemy_team);

    // No change since single use.
    assert_eq!(team.nth(1).unwrap().stats, buffed_stats);
}

#[test]
fn test_battle_panda_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_panda_team();
    let mut enemy_team = test_mammoth_team();

    // Adds 50% of attack (1,0).
    let add_stats = team.nth(1).unwrap().stats
        * Statistics {
            attack: 50,
            health: 50,
        };
    assert_eq!(add_stats, Statistics::new(1, 2).unwrap());
    // Initial dog stats.
    let original_stats = team.first().unwrap().stats;

    team.trigger_effects(&mut enemy_team);

    assert_eq!(team.first().unwrap().stats, original_stats + add_stats);
    team.clear_team();
    // Panda died.
    assert_eq!(
        team.fainted.first().unwrap().as_ref().unwrap().name,
        PetName::Panda
    )
}

#[test]
fn test_battle_pug_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_pug_team();
    let mut enemy_team = test_mammoth_team();

    // Pug has lvl. 1 with 1 exp.
    assert_eq!(team.first().as_ref().unwrap().exp, 1);
    assert_eq!(team.first().as_ref().unwrap().lvl, 1);
    assert_eq!(
        team.first().as_ref().unwrap().stats,
        Statistics::new(3, 2).unwrap()
    );
    // Activate start of battle effect of pug.
    team.trigger_effects(&mut enemy_team);

    // Ant levels up.
    assert_eq!(team.first().as_ref().unwrap().exp, 2);
    assert_eq!(team.first().as_ref().unwrap().lvl, 2);
    assert_eq!(
        team.first().as_ref().unwrap().stats,
        Statistics::new(4, 3).unwrap()
    );
}

#[test]
fn test_battle_stork_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_stork_team();
    let mut enemy_team = test_mammoth_team();

    team.fight(&mut enemy_team);

    // TODO: Currently, has no tier information so uses tier 1 ( (stork tier) 2 - 1) by default.
    assert_eq!(team.first().as_ref().unwrap().tier, 1);
    assert_eq!(
        team.fainted
            .first()
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .name,
        PetName::Stork
    )
}

#[test]
fn test_battle_racoon_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_racoon_team();
    let mut enemy_team = test_mammoth_team();
    // Give melon to first pet.
    enemy_team.first().unwrap().item = Some(Food::from(FoodName::Melon));

    // Not item for racoon.
    assert_eq!(team.first().unwrap().item, None);

    // Trigger attack.
    team.fight(&mut enemy_team);

    // Racoon got mammoth's melon.
    assert_eq!(
        team.first().unwrap().item.as_ref().unwrap().name,
        FoodName::Melon
    );
}

#[test]
fn test_battle_toucan_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_toucan_team();
    let mut enemy_team = test_mammoth_team();

    // Toucan has honey.
    assert_eq!(
        team.first().unwrap().item.as_ref().unwrap().name,
        FoodName::Honey
    );
    // Dog behind toucan has no item.
    assert_eq!(team.nth(1).unwrap().item, None);
    team.fight(&mut enemy_team);

    // Dog behind bee now has honey.
    assert_eq!(
        team.nth(1).unwrap().item.as_ref().unwrap().name,
        FoodName::Honey
    );
}

#[test]
fn test_battle_wombat_team() {
    log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_wombat_team();
    let mut enemy_team = test_mammoth_team();
    // Mammoth faint effect.
    let mammoth_effect = enemy_team.first().unwrap().get_effect(1).unwrap();

    // Activate start of battle.
    team.trigger_effects(&mut enemy_team);

    // Wombat gains mammoth's effect.
    assert_eq!(
        team.first().unwrap().effect.first().unwrap(),
        mammoth_effect.first().unwrap()
    )
}
