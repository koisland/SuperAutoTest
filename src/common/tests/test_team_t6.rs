use crate::common::{
    battle::state::{Statistics, TeamFightOutcome},
    foods::{food::Food, names::FoodName},
    pets::names::PetName,
    tests::common::{
        count_pets, test_boar_team, test_fly_team, test_gorilla_team, test_leopard_team,
        test_mammoth_team, test_scorpion_team, test_sheep_team, test_snake_team, test_tiger_team,
    },
};

use crate::LOG_CONFIG;

#[test]
fn test_battle_boar_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_boar_team("self");
    let mut enemy_team = test_sheep_team("enemy");

    let original_boar_stats = team.get_next_pet().unwrap().stats.clone();
    assert_eq!(
        original_boar_stats,
        Statistics {
            attack: 10,
            health: 6
        }
    );
    team.fight(&mut enemy_team);

    // After battle with first sheep (2,2) gains (4,2)
    assert_eq!(
        team.get_next_pet().unwrap().stats,
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
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_fly_team("self");
    let mut enemy_team = test_fly_team("enemy");

    team.fight(&mut enemy_team);
    team.fight(&mut enemy_team);

    // Zombie fly spawned after cricket dies.
    // Applies before cricket because fly has higher attack.
    assert_eq!(team.get_next_pet().unwrap().name, PetName::ZombieFly);
    assert_eq!(team.get_idx_pet(1).unwrap().name, PetName::ZombieCricket);

    // Zombie flies fight. But no flies are spawned when zombie flies die.
    team.fight(&mut enemy_team);

    assert_eq!(team.get_next_pet().unwrap().name, PetName::ZombieCricket);

    // Finish battle.
    let mut outcome = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = outcome {
        outcome = team.fight(&mut enemy_team)
    }

    assert_eq!(outcome, TeamFightOutcome::Draw);
    // Only three zombie flies spawned with a total of 4 valid faint triggers.
    let total_valid_faint_triggers = count_pets(&team.fainted, PetName::Cricket)
        + count_pets(&team.fainted, PetName::ZombieCricket);
    assert!(count_pets(&team.fainted, PetName::ZombieFly) == 3 && total_valid_faint_triggers == 4)
}

#[test]
fn test_battle_gorilla_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_gorilla_team("self");
    let mut enemy_team = test_gorilla_team("enemy");

    // Gorilla has no items before fight.
    assert_eq!(team.get_next_pet().unwrap().item, None);
    assert_eq!(
        team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 6,
            health: 9
        }
    );
    team.fight(&mut enemy_team);

    // Gorilla is hurt and gains coconut.
    assert_eq!(
        team.get_next_pet().unwrap().item,
        Some(Food::from(FoodName::Coconut))
    );
    assert_eq!(
        team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 6,
            health: 3
        }
    );
}

#[test]
fn test_battle_leopard_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_leopard_team("self");
    let mut enemy_team = test_gorilla_team("enemy");

    // One leopard on team.
    assert_eq!(
        team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 10,
            health: 4
        }
    );
    // One gorilla on enemy team.
    assert_eq!(
        enemy_team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 6,
            health: 9
        }
    );

    // Leopard activates at start of battle and deals 50% of leopard attack.
    team.fight(&mut enemy_team);

    assert_eq!(
        enemy_team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 6,
            health: 4
        }
    );
}

#[test]
fn test_battle_mammoth_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_mammoth_team("self");
    let mut enemy_team = test_mammoth_team("enemy");

    // Stats of every pet after mammoth.
    for team in [&team, &enemy_team].into_iter() {
        for pet in team.friends.get(1..).unwrap().iter() {
            assert_eq!(
                pet.as_ref().unwrap().stats,
                Statistics {
                    attack: 3,
                    health: 4
                }
            )
        }
    }

    // 4 attack phases to kill mammoth.
    for _ in 0..4 {
        team.fight(&mut enemy_team);
    }

    // All pets on team gained (2,2)
    for team in [&team, &enemy_team].into_iter() {
        for pet in team.friends.iter() {
            assert_eq!(
                pet.as_ref().unwrap().stats,
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
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_snake_team("self");
    let mut enemy_team = test_sheep_team("enemy");

    {
        // Frontline cricket won't kill enemy sheep in single turn.
        assert_eq!(
            team.get_next_pet().unwrap().stats,
            Statistics {
                attack: 1,
                health: 2
            }
        );
        let enemy_sheep = enemy_team.get_next_pet().unwrap();
        assert_eq!(
            enemy_sheep.stats,
            Statistics {
                attack: 2,
                health: 2
            }
        );
        assert_eq!(enemy_sheep.name, PetName::Sheep)
    }

    // One battle phase passes.
    // Cricket attacks and snake triggers killing sheep.
    team.fight(&mut enemy_team);

    // Two ram spawn as result.
    for pet in enemy_team.get_all_pets() {
        assert_eq!(pet.name, PetName::Ram);
    }
}

#[test]
fn test_battle_tiger_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_tiger_team("self");
    let mut enemy_team = test_scorpion_team("enemy");

    {
        // Team of leopard and tiger.
        let pets = team.get_all_pets();
        assert_eq!(pets.get(0).unwrap().name, PetName::Leopard);
        assert_eq!(pets.get(1).unwrap().name, PetName::Tiger);
        assert_eq!(pets.len(), 2)
    }
    {
        // Enemy team of two scorpions.
        let enemy_pets = enemy_team.get_all_pets();
        assert_eq!(enemy_pets.get(0).unwrap().name, PetName::Scorpion);
        assert_eq!(enemy_pets.get(1).unwrap().name, PetName::Scorpion);
        assert_eq!(enemy_pets.len(), 2)
    }
    // Start of battle triggers leopard effect twice (due to tiger behind it) against scorpion team.
    team.fight(&mut enemy_team);

    // Frontline leopard lives because its effect triggers twice.
    let pets = team.get_all_pets();
    assert_eq!(pets.get(0).unwrap().name, PetName::Leopard);
    assert_eq!(pets.get(1).unwrap().name, PetName::Tiger);
}
