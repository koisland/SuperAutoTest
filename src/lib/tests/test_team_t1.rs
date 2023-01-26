use itertools::Itertools;

use crate::{
    battle::{
        state::{Statistics, TeamFightOutcome},
        team_effect_apply::EffectApply,
    },
    foods::{food::Food, names::FoodName},
    pets::names::PetName,
    tests::common::{
        test_ant_team, test_cricket_horse_team, test_frilled_dragon_team, test_frog_team,
        test_hummingbird_team, test_iguana_seahorse_team, test_mosq_team, test_moth_team,
    },
};
// use crate::LOG_CONFIG;
// use petgraph::{dot::Dot, visit::Dfs};

#[test]
fn test_battle_ant_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    let all_2_1 = team.friends.iter().filter(|pet| pet.is_some()).all(|slot| {
        if let Some(pet) = slot {
            pet.stats
                == Statistics {
                    attack: 2,
                    health: 1,
                }
        } else {
            false
        }
    });
    assert!(all_2_1);

    // One battle phase and one ant faints.
    team.fight(&mut enemy_team);

    let any_gets_2_1 = team.friends.iter().any(|slot| {
        if let Some(pet) = slot {
            pet.stats
                == Statistics {
                    attack: 4,
                    health: 2,
                }
        } else {
            false
        }
    });
    // Another pet gets (2,1).
    assert!(any_gets_2_1)
}

#[test]
fn test_battle_cricket_horse_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();
    let mut team = test_cricket_horse_team();
    let mut enemy_team = test_cricket_horse_team();

    // First pets are crickets
    // Horse is 3rd pet.
    assert_eq!(team.first().unwrap().name, PetName::Cricket);
    assert_eq!(enemy_team.first().unwrap().name, PetName::Cricket);
    assert_eq!(team.nth(2).unwrap().name, PetName::Horse);
    assert_eq!(enemy_team.nth(2).unwrap().name, PetName::Horse);
    assert_eq!(
        team.first().unwrap().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
    assert_eq!(
        enemy_team.first().unwrap().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );

    // After one turn.
    team.fight(&mut enemy_team);
    team.fight(&mut enemy_team);

    // Cricket dies and zombie cricket is spawned.
    // Horse provides 1 attack.
    assert_eq!(team.first().unwrap().name, PetName::ZombieCricket);
    assert_eq!(enemy_team.first().unwrap().name, PetName::ZombieCricket);
    assert_eq!(
        team.first().unwrap().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
    assert_eq!(
        enemy_team.first().unwrap().stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );
}

#[test]
fn test_battle_mosquito_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_mosq_team();
    let mut enemy_team = test_ant_team();

    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team)
    }
    // Mosquitoes kill any team before game starts.
    assert_eq!(fight, TeamFightOutcome::Win);
    assert_eq!(team.friends.len(), 3);

    for pet in team.all().iter() {
        // Mosquitoes are unhurt
        assert_eq!(
            pet.stats,
            Statistics {
                attack: 2,
                health: 2,
            }
        )
    }
}

#[test]
fn test_battle_frilled_dragon_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_frilled_dragon_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.all().last().unwrap().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );

    team.fight(&mut enemy_team);

    // Team has two crickets with faint triggers. Gains (1,1) for each.
    assert_eq!(
        team.friends.last().unwrap().as_ref().unwrap().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    )
}

#[test]
fn test_battle_frog_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_frog_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.friends.get(0).unwrap().as_ref().unwrap().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
    // Frilled dragon before activation of ability.
    assert_eq!(
        team.friends.get(2).unwrap().as_ref().unwrap().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    );

    // Frilled dragon activates.
    // Then frog activates swapping stats of cricket and frilled dragon.
    // Cricket with 2/2 dies spawning zombie cricket.
    team.fight(&mut enemy_team);

    // Frilled dragon gets cricket stats.
    assert_eq!(
        team.friends.get(2).unwrap().as_ref().unwrap().stats,
        Statistics {
            attack: 1,
            health: 2
        }
    );
}

#[test]
fn test_battle_moth_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_moth_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.friends.first().unwrap().as_ref().unwrap().stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );
    // Ant deals 2 dmg. 2 moths gives (6,0).
    team.fight(&mut enemy_team);

    assert_eq!(
        team.friends.first().unwrap().as_ref().unwrap().stats,
        Statistics {
            attack: 8,
            health: 1
        }
    );
}

#[test]
fn test_battle_hummingbird_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_hummingbird_team();
    let mut enemy_team = test_ant_team();

    assert_eq!(
        team.friends.first().unwrap().as_ref().unwrap().stats,
        Statistics {
            attack: 2,
            health: 3
        }
    );
    // Duck has strawberry.
    assert_eq!(
        team.friends.first().unwrap().as_ref().unwrap().item,
        Some(Food::from(FoodName::Strawberry))
    );
    // Two hummingbirds on team.
    assert_eq!(
        team.friends
            .iter()
            .filter_map(|pet| if let Some(pet) = pet {
                (pet.name == PetName::Hummingbird).then_some(true)
            } else {
                None
            })
            .collect_vec()
            .len(),
        2
    );
    // Trigger start of battle effects.
    team.trigger_effects(&mut enemy_team);

    // Duck gets 2/1 for every hummingbird since only strawberry friend.
    assert_eq!(
        team.friends.first().unwrap().as_ref().unwrap().stats,
        Statistics {
            attack: 6,
            health: 5
        }
    );
}

#[test]
fn test_battle_iguana_seahorse_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_iguana_seahorse_team();
    let mut enemy_team = test_cricket_horse_team();

    // Start of battle pushes horse to 2nd position and it gets hit by iguana.
    // Seahorse knockouts cricket leaving zombie cricket.
    // Zombie cricket hit by iguana.
    team.fight(&mut enemy_team);

    // Only one pet remaining on enemy team.
    assert_eq!(enemy_team.first().unwrap().name, PetName::Cricket);
    assert_eq!(enemy_team.friends.len(), 1)
}
