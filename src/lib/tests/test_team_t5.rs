use crate::{
    battle::state::{Action, Statistics, TeamFightOutcome},
    foods::{food::Food, names::FoodName},
    pets::{names::PetName, pet::Pet},
    tests::common::{
        test_cricket_horse_team, test_crocodile_team, test_rhino_team, test_scorpion_team,
        test_shark_team, test_skunk_team, test_turkey_team,
    },
};

// use crate::LOG_CONFIG;

#[test]
fn test_battle_croc_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_crocodile_team();
    let mut enemy_team = test_crocodile_team();

    let last_pet = team.friends.last().unwrap().as_ref().unwrap();
    let last_enemy_pet = team.friends.last().unwrap().as_ref().unwrap();
    assert_eq!(last_pet.name, PetName::Cricket);
    assert_eq!(last_enemy_pet.name, PetName::Cricket);

    // After start of battle, both crickets at end are sniped.
    // Two zombie crickets are spawned in their place.
    team.fight(&mut enemy_team);

    let last_pet = team.friends.last().unwrap().as_ref().unwrap();
    let last_enemy_pet = team.friends.last().unwrap().as_ref().unwrap();

    assert_eq!(team.friends.len(), 4);
    assert_eq!(enemy_team.friends.len(), 4);
    assert_eq!(last_pet.name, PetName::ZombieCricket);
    assert_eq!(last_enemy_pet.name, PetName::ZombieCricket);
}

#[test]
fn test_battle_rhino_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_rhino_team();
    let mut enemy_team = test_cricket_horse_team();

    let outcome = team.fight(&mut enemy_team);

    assert_eq!(outcome, TeamFightOutcome::None);
    // Only one damage from first cricket to trigger chain of faint triggers.
    assert_eq!(
        team.first().unwrap().stats,
        Statistics {
            attack: 5,
            health: 7
        }
    );
    // All pets mowed down by rhino. After horse faints, zombie crickets spawn.
    assert!(enemy_team
        .all()
        .iter()
        .all(|pet| pet.name == PetName::ZombieCricket))
}

#[test]
fn test_battle_scorpion_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_scorpion_team();
    let mut enemy_team = test_skunk_team();
    // At start of turn, scorpion doesn't have peanuts. Then gains it.
    assert_eq!(team.first().unwrap().item, None);
    let outcome = team.fight(&mut enemy_team);

    // Win after single turn due to peanuts.
    assert_eq!(outcome, TeamFightOutcome::Win);

    // Scropion gained peanuts.
    let (_, _, action, _) = &team
        .history
        .effect_graph
        .raw_edges()
        .first()
        .unwrap()
        .weight;
    assert_eq!(
        action,
        &Action::Gain(Box::new(Food::from(FoodName::Peanuts)))
    )
}

#[test]
fn test_battle_shark_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_shark_team();
    let mut enemy_team = test_shark_team();

    // Removed
    enemy_team.friends.remove(0);

    let count_crickets = |friends: &[Option<Pet>]| {
        friends
            .iter()
            .filter_map(|pet| {
                if let Some(pet) = pet {
                    (pet.name == PetName::Cricket).then_some(1)
                } else {
                    None
                }
            })
            .sum::<usize>()
    };

    let n_team_crickets = count_crickets(&team.friends);
    let n_enemy_team_crickets = count_crickets(&enemy_team.friends);

    // Lvl. 1 shark gains (1,2) on any faint.
    // (self) 4 crickets so 8 total faint triggers.
    // 8 attack and 16 health gained.
    // (enemy) 3 crickets so 6 total faint triggers.
    // 6 attack and 12 health gained.
    let exp_shark_gained_stats = Statistics {
        attack: (1 * n_team_crickets * 2).try_into().unwrap(),
        health: (2 * n_team_crickets * 2).try_into().unwrap(),
    };
    let exp_enemy_shark_gained_stats = Statistics {
        attack: (1 * n_enemy_team_crickets * 2).try_into().unwrap(),
        health: (2 * n_enemy_team_crickets * 2).try_into().unwrap(),
    };

    let mut outcome = team.fight(&mut enemy_team);

    while let TeamFightOutcome::None = outcome {
        outcome = team.fight(&mut enemy_team);
    }

    let mut added_shark_stats = Statistics::default();
    let mut added_enemy_shark_stats = Statistics::default();
    for node in team.history.effect_graph.raw_edges() {
        let (_, _, action, _) = &node.weight;
        if let Action::Add(stats) = action {
            added_shark_stats += stats.clone()
        }
    }
    for node in enemy_team.history.effect_graph.raw_edges() {
        let (_, _, action, _) = &node.weight;
        if let Action::Add(stats) = action {
            added_enemy_shark_stats += stats.clone()
        }
    }

    assert_eq!(added_shark_stats, exp_shark_gained_stats);
    assert_eq!(added_enemy_shark_stats, exp_enemy_shark_gained_stats);
}

#[test]
fn test_battle_turkey_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_turkey_team();
    let mut enemy_team = test_turkey_team();

    team.fight(&mut enemy_team);
    team.fight(&mut enemy_team);

    // Cricket faints, zombie version spawned, and it gains (3,3) (lvl.1 turkey)
    let zombie_cricket = team.first().unwrap();
    assert_eq!(
        zombie_cricket.stats,
        Statistics {
            attack: 4,
            health: 4
        }
    );
    assert_eq!(zombie_cricket.name, PetName::ZombieCricket)
}
