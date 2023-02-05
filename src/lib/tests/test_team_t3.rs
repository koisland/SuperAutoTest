use crate::{
    battle::state::{Statistics, Status, TeamFightOutcome},
    foods::{food::Food, names::FoodName},
    pets::names::PetName,
    tests::common::{
        test_aardvark_team, test_ant_team, test_badger_team, test_bear_team, test_blobfish_team,
        test_blowfish_rally_team, test_blowfish_team, test_camel_team, test_clownfish_team,
        test_cricket_horse_team, test_dog_team, test_dolphin_team, test_filled_sheep_team,
        test_hummingbird_team, test_kangaroo_team, test_ox_team, test_seagull_team,
        test_sheep_team, test_toad_team, test_woodpecker_self_hurt_team, test_woodpecker_team,
    },
    EffectApply,
};

// use crate::LOG_CONFIG;

#[test]
fn test_battle_badger_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_badger_team();
    let mut enemy_team = test_dolphin_team();

    assert_eq!(team.nth(1).unwrap().stats.health, 5);
    // Dolphin immediately kills badger.
    // Badger's effect triggers dealing 3 dmg to both adjacent pets.
    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team)
    }

    assert_eq!(fight, TeamFightOutcome::Win);
    assert_eq!(team.first().unwrap().stats.health, 2)
}

#[test]
fn test_battle_blowfish_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_blowfish_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(team.nth(1).unwrap().stats.health, 5);

    team.fight(&mut enemy_team);

    // One pet dies to blowfish indirect attack.
    // Another dies to elephant attack.
    assert_eq!(enemy_team.all().len(), 1);
    // Blowfish takes 1 dmg.
    assert_eq!(team.nth(1).unwrap().stats.health, 4);
}

#[test]
fn test_battle_blowfish_rally_battle() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_blowfish_rally_team();
    let mut enemy_team = test_blowfish_rally_team();

    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team)
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
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_camel_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(team.nth(1).unwrap().stats.health, 6);
    // Ant has 1 health.
    assert_eq!(team.nth(2).unwrap().stats.health, 1);

    team.fight(&mut enemy_team);

    // Camel takes 1 dmg from elephant.
    assert_eq!(team.nth(1).unwrap().stats.health, 5);
    // And gives ant 2 hp.
    assert_eq!(team.nth(2).unwrap().stats.health, 3);
}

#[test]
fn test_battle_dog_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_dog_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(team.nth(0).unwrap().name, PetName::Cricket);
    assert_eq!(
        team.nth(1).unwrap().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    team.fight(&mut enemy_team);

    assert_eq!(team.nth(0).unwrap().name, PetName::ZombieCricket);
    // Dog gains (1,1) after Zombie Cricket spawns.
    assert_eq!(
        team.nth(1).unwrap().stats,
        Statistics {
            attack: 4,
            health: 5
        }
    );
}

#[test]
fn test_battle_kangaroo_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_kangaroo_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.nth(1).unwrap().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
    team.fight(&mut enemy_team);

    // Friend ahead attacks once increasing stats by (2,2)
    assert_eq!(
        team.nth(1).unwrap().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
}

#[test]
fn test_battle_ox_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_ox_team();
    let mut enemy_team = test_ant_team();

    {
        let ox = team.nth(1).unwrap();
        // No item on default lvl.1 ox.
        assert!(ox.item.is_none());
        assert_eq!(
            ox.stats,
            Statistics {
                attack: 1,
                health: 3
            }
        );
    };

    team.fight(&mut enemy_team);
    team.fight(&mut enemy_team);

    let ox = team.nth(0).unwrap();
    // Gets melon armor.
    assert_eq!(ox.item, Some(Food::new(&FoodName::Melon).unwrap()));
    // And an extra attack.
    assert_eq!(
        ox.stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );
}

#[test]
fn test_battle_sheep_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_sheep_team();
    let mut enemy_team = test_sheep_team();

    assert_eq!(team.all().len(), 1);
    // Sheep faint and summon two ram.
    team.fight(&mut enemy_team);

    for team in [team, enemy_team].iter_mut() {
        let pets = team.all();

        assert_eq!(pets.len(), 2);

        for pet in pets.iter() {
            assert_eq!(pet.name, PetName::Ram)
        }
    }
}

#[test]
fn test_battle_filled_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_filled_sheep_team();
    let mut enemy_team = test_filled_sheep_team();

    team.fight(&mut enemy_team);

    // Overflow in pets (ram in this case) gets added to team's dead.
    assert_eq!(2, team.fainted.len());
    assert_eq!(
        PetName::Ram,
        team.fainted.first().unwrap().as_ref().unwrap().name
    )
}

#[test]
fn test_battle_aardvark_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_aardvark_team();
    let mut enemy_team = test_cricket_horse_team();

    let aardvark_stats = team.first().unwrap().stats;
    assert_eq!(aardvark_stats, Statistics::new(2, 3).unwrap());

    // Fights first cricket.
    team.fight(&mut enemy_team);

    // Cricket faints and Zombie Cricket spawns
    assert_eq!(
        enemy_team.fainted.first().unwrap().as_ref().unwrap().name,
        PetName::Cricket
    );
    assert_eq!(enemy_team.first().unwrap().name, PetName::ZombieCricket);

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
        team.first().unwrap().stats
    );
}

#[test]
fn test_battle_bear_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_bear_team();
    let mut enemy_team = test_hummingbird_team();

    // Dog at position behind bear has no item.
    assert_eq!(team.nth(1).unwrap().item, None);
    // Enemy team first pet (duck) has strawberry.
    assert_eq!(
        enemy_team.first().unwrap().item.as_ref().unwrap().name,
        FoodName::Strawberry
    );
    team.fight(&mut enemy_team);

    // Bear fainted.
    assert_eq!(
        team.fainted.first().unwrap().as_ref().unwrap().name,
        PetName::Bear
    );
    // Duck now has honey.
    assert_eq!(
        enemy_team.first().unwrap().item.as_ref().unwrap().name,
        FoodName::Honey
    );
    // Dog now has honey.
    assert_eq!(
        team.first().unwrap().item.as_ref().unwrap().name,
        FoodName::Honey
    );
}

#[test]
fn test_battle_seagull_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_seagull_team();
    let mut enemy_team = test_ant_team();

    // First pet summons zombie cricket.
    assert_eq!(team.first().unwrap().name, PetName::Cricket);
    // Seagull has honey.
    assert_eq!(
        team.nth(1).unwrap().item.as_ref().unwrap().name,
        FoodName::Honey
    );
    team.fight(&mut enemy_team);

    // Zombie cricket gets honey from seagull.
    {
        let zombie_cricket = team.first().unwrap();
        assert!(
            zombie_cricket.name == PetName::ZombieCricket
                && zombie_cricket.item.as_ref().unwrap().name == FoodName::Honey
        );
    }

    // Fight again to kill zombie cricket with honey.
    team.fight(&mut enemy_team);

    // Seagull ability only activates once. Bee does not get honey.
    assert!(team.first().unwrap().name == PetName::Bee && team.first().unwrap().item == None);
}

#[test]
fn test_battle_blobfish_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_blobfish_team();
    let mut enemy_team = test_hummingbird_team();

    // Dog behind has no experience.
    assert_eq!(team.nth(1).unwrap().exp, 0);
    team.fight(&mut enemy_team);

    // Blobfish dies.
    assert_eq!(
        team.fainted.first().unwrap().as_ref().unwrap().name,
        PetName::Blobfish
    );
    // Dog in front now has 1 experience.
    assert_eq!(team.first().unwrap().exp, 1);
}

#[test]
fn test_battle_clownfish_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_clownfish_team();
    let mut enemy_team = test_hummingbird_team();

    // Dog behind blobfish is level 1 and has 1 exp.
    let dog_stats = {
        let dog = team.nth(1).unwrap();
        assert!(dog.exp == 1 && dog.lvl == 1);
        assert_eq!(Statistics::new(4, 5).unwrap(), dog.stats);
        dog.stats
    };
    // Blobfish dies during fight and levels dog to 2.
    team.fight(&mut enemy_team);

    {
        let dog = team.first().unwrap();
        assert!(dog.exp == 2 && dog.lvl == 2);
        // Dog gains (1,1) from blobfish experience and (2,2) from clownfish on level.
        assert_eq!(
            dog_stats + Statistics::new(1, 1).unwrap() + Statistics::new(2, 2).unwrap(),
            dog.stats
        )
    }
}

#[test]
fn test_battle_toad_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_toad_team();
    let mut enemy_team = test_cricket_horse_team();

    // Seed ensures that target always cricket at pos 1.
    enemy_team.set_seed(2);
    assert_eq!(
        enemy_team.nth(1).unwrap().stats,
        Statistics::new(1, 2).unwrap()
    );
    // Trigger start of battle effects.
    team.trigger_effects(&mut enemy_team);

    // Cricket hit by mosquito and takes 1 dmg
    assert_eq!(
        enemy_team.nth(1).unwrap().stats,
        Statistics::new(1, 1).unwrap()
    );
    // Frog triggers and cricket now has weakness.
    assert_eq!(
        enemy_team.nth(1).unwrap().item.as_ref().unwrap().name,
        FoodName::Weak
    );
}

#[test]
fn test_battle_woodpecker_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_woodpecker_team();
    let mut enemy_team = test_cricket_horse_team();

    assert_eq!(
        enemy_team.nth(0).unwrap().stats,
        Statistics::new(1, 2).unwrap()
    );
    assert_eq!(
        enemy_team.nth(1).unwrap().stats,
        Statistics::new(1, 2).unwrap()
    );
    // Trigger start of battle effects.
    team.trigger_effects(&mut enemy_team);

    // Two crickets at front on enemy team die.
    assert_eq!(
        enemy_team.friends.first().unwrap().as_ref().unwrap().stats,
        Statistics::new(1, 0).unwrap()
    );
    assert_eq!(
        enemy_team.friends.get(1).unwrap().as_ref().unwrap().stats,
        Statistics::new(1, 0).unwrap()
    );
}

#[test]
fn test_battle_woodpecker_self_hurt_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();
    let mut team = test_woodpecker_self_hurt_team();
    let mut enemy_team = test_cricket_horse_team();

    assert_eq!(team.nth(0).unwrap().stats, Statistics::new(1, 2).unwrap());
    assert_eq!(team.nth(1).unwrap().stats, Statistics::new(1, 2).unwrap());

    // Trigger start of battle effects and clear dead pets.
    team.trigger_effects(&mut enemy_team);
    team.clear_team();

    // Two crickets at front of woodpecker on same team faint.
    assert_eq!(
        team.fainted.first().unwrap().as_ref().unwrap().stats,
        Statistics::new(1, 0).unwrap()
    );
    assert_eq!(
        team.fainted.get(1).unwrap().as_ref().unwrap().stats,
        Statistics::new(1, 0).unwrap()
    );
}