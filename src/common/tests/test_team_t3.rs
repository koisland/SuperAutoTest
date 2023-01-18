use crate::common::{
    battle::state::{Statistics, Status, TeamFightOutcome},
    foods::{food::Food, names::FoodName},
    pets::names::PetName,
    tests::common::{
        test_ant_team, test_badger_team, test_blowfish_rally_team, test_blowfish_team,
        test_camel_team, test_dog_team, test_dolphin_team, test_filled_sheep_team,
        test_kangaroo_team, test_ox_team, test_sheep_team,
    },
};

use crate::LOG_CONFIG;

#[test]
fn test_battle_badger_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_badger_team("self");
    let mut enemy_team = test_dolphin_team("enemy");

    assert_eq!(team.get_idx_pet(1).unwrap().stats.health, 5);
    // Dolphin immediately kills badger.
    // Badger's effect triggers dealing 3 dmg to both adjacent pets.
    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team)
    }

    assert_eq!(fight, TeamFightOutcome::Win);
    assert_eq!(team.get_next_pet().unwrap().stats.health, 2)
}

#[test]
fn test_battle_blowfish_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_blowfish_team("self");
    let mut enemy_team = test_ant_team("enemy");

    assert_eq!(team.get_idx_pet(1).unwrap().stats.health, 5);

    team.fight(&mut enemy_team);

    // One pet dies to blowfish indirect attack.
    // Another dies to elephant attack.
    assert_eq!(enemy_team.get_all_pets().len(), 1);
    // Blowfish takes 1 dmg.
    assert_eq!(team.get_idx_pet(1).unwrap().stats.health, 4);
}

#[test]
fn test_battle_blowfish_rally_battle() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_blowfish_rally_team("self");
    let mut enemy_team = test_blowfish_rally_team("enemy");

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

    let mut team = test_camel_team("self");
    let mut enemy_team = test_ant_team("enemy");

    assert_eq!(team.get_idx_pet(1).unwrap().stats.health, 6);
    // Ant has 1 health.
    assert_eq!(team.get_idx_pet(2).unwrap().stats.health, 1);

    team.fight(&mut enemy_team);

    // Camel takes 1 dmg from elephant.
    assert_eq!(team.get_idx_pet(1).unwrap().stats.health, 5);
    // And gives ant 2 hp.
    assert_eq!(team.get_idx_pet(2).unwrap().stats.health, 3);
}

#[test]
fn test_battle_dog_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_dog_team("self");
    let mut enemy_team = test_ant_team("enemy");

    assert_eq!(team.get_idx_pet(0).unwrap().name, PetName::Cricket);
    assert_eq!(
        team.get_idx_pet(1).unwrap().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    team.fight(&mut enemy_team);

    assert_eq!(team.get_idx_pet(0).unwrap().name, PetName::ZombieCricket);
    // Dog gains (1,1) after Zombie Cricket spawns.
    assert_eq!(
        team.get_idx_pet(1).unwrap().stats,
        Statistics {
            attack: 4,
            health: 5
        }
    );
}

#[test]
fn test_battle_kangaroo_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_kangaroo_team("self");
    let mut enemy_team = test_ant_team("enemy");

    assert_eq!(
        team.get_idx_pet(1).unwrap().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
    team.fight(&mut enemy_team);

    // Friend ahead attacks once increasing stats by (2,2)
    assert_eq!(
        team.get_idx_pet(1).unwrap().stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
}

#[test]
fn test_battle_ox_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_ox_team("self");
    let mut enemy_team = test_ant_team("enemy");

    {
        let ox = team.get_idx_pet(1).unwrap();
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

    let ox = team.get_idx_pet(0).unwrap();
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

    let mut team = test_sheep_team("self");
    let mut enemy_team = test_sheep_team("enemy");

    assert_eq!(team.get_all_pets().len(), 1);
    // Sheep faint and summon two ram.
    team.fight(&mut enemy_team);

    for team in [team, enemy_team].iter_mut() {
        let pets = team.get_all_pets();

        assert_eq!(pets.len(), 2);

        for pet in pets.iter() {
            assert_eq!(pet.name, PetName::Ram)
        }
    }
}

#[test]
fn test_battle_filled_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_filled_sheep_team("self");
    let mut enemy_team = test_filled_sheep_team("enemy");

    team.fight(&mut enemy_team);

    // Overflow in pets (ram in this case) gets added to team's dead.
    assert_eq!(2, team.fainted.len());
    assert_eq!(
        PetName::Ram,
        team.fainted.first().unwrap().as_ref().unwrap().name
    )
}
