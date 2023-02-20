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
        count_pets, test_alpaca_team, test_boar_team, test_cricket_horse_team, test_fly_team,
        test_gorilla_team, test_leopard_team, test_mammoth_team, test_octopus_team, test_orca_team,
        test_piranha_team, test_reindeer_team, test_sabertooth_team, test_scorpion_team,
        test_sheep_team, test_snake_team, test_spinosaurus_team, test_stegosaurus_team,
        test_tapir_team, test_tiger_team, test_velociraptor_team, test_walrus_team,
        test_white_tiger_team,
    },
    Pet, TeamEffects,
};

#[test]
fn test_battle_boar_team() {
    let mut team = test_boar_team();
    let mut enemy_team = test_sheep_team();

    let original_boar_stats = team.first().unwrap().borrow().stats;
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
        team.first().unwrap().borrow().stats,
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
    assert_eq!(team.first().unwrap().borrow().name, PetName::ZombieCricket);
    assert_eq!(team.nth(1).unwrap().borrow().name, PetName::ZombieFly);

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
    assert_eq!(team.first().unwrap().borrow().item, None);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 6,
            health: 9
        }
    );
    team.fight(&mut enemy_team).unwrap();

    // Gorilla is hurt and gains coconut.
    let mut coconut = Food::try_from(FoodName::Coconut).unwrap();
    coconut.ability.assign_owner(Some(&team.first().unwrap()));

    assert_eq!(team.first().unwrap().borrow().item, Some(coconut));
    assert_eq!(
        team.first().unwrap().borrow().stats,
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
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 10,
            health: 4
        }
    );
    // One gorilla on enemy team.
    assert_eq!(
        enemy_team.first().unwrap().borrow().stats,
        Statistics {
            attack: 6,
            health: 9
        }
    );

    // Leopard activates at start of battle and deals 50% of leopard attack.
    team.fight(&mut enemy_team).unwrap();

    assert_eq!(
        enemy_team.first().unwrap().borrow().stats,
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
        for pet in team.friends.get(1..).unwrap().iter() {
            assert_eq!(
                pet.borrow().stats,
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
        for pet in team.friends.iter() {
            assert_eq!(
                pet.borrow().stats,
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
            team.first().unwrap().borrow().stats,
            Statistics {
                attack: 1,
                health: 2
            }
        );
        let enemy_sheep = enemy_team.first().unwrap();
        assert_eq!(
            enemy_sheep.borrow().stats,
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
        assert_eq!(pet.borrow().name, PetName::Ram);
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
        assert_eq!(pets.get(0).unwrap().borrow().name, PetName::Leopard);
        assert_eq!(pets.get(1).unwrap().borrow().name, PetName::Tiger);
        assert_eq!(pets.len(), 2)
    }
    {
        // Enemy team of two scorpions.
        let enemy_pets = enemy_team.all();
        assert_eq!(enemy_pets.get(0).unwrap().borrow().name, PetName::Scorpion);
        assert_eq!(enemy_pets.get(1).unwrap().borrow().name, PetName::Scorpion);
        assert_eq!(enemy_pets.len(), 2)
    }
    // Start of battle triggers leopard effect twice (due to tiger behind it) against scorpion team.
    team.fight(&mut enemy_team).unwrap();

    // Frontline leopard lives because its effect triggers twice.
    let pets = team.all();
    assert_eq!(pets.get(0).unwrap().borrow().name, PetName::Leopard);
    assert_eq!(pets.get(1).unwrap().borrow().name, PetName::Tiger);
}

#[test]
fn test_battle_alpaca_team() {
    let mut team = test_alpaca_team();
    let mut enemy_team = test_gorilla_team();

    assert_eq!(count_pets(&team.friends, PetName::Alpaca), 2);
    // First alpaca has mushroom so will respawn as (1,1)
    assert_eq!(
        team.nth(1).unwrap().borrow().item.as_ref().unwrap().name,
        FoodName::Mushroom
    );

    for i in 0..4 {
        team.fight(&mut enemy_team).unwrap();
        // Cricket spawns and gets two exp leveling it to lvl 2.
        if i == 0 {
            let zombie_cricket = team.first().unwrap();
            assert_eq!(zombie_cricket.borrow().lvl, 2);
            assert_eq!(zombie_cricket.borrow().exp, 2);
        };
        // Alpaca respawns.
        if i == 3 {
            let respawned_alpaca = team.first().unwrap();
            // Alpaca summoned but doesn't get exp from remaining alpaca.
            assert_eq!(respawned_alpaca.borrow().exp, 0);
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
    assert!(spawned_pet.borrow().name == PetName::Tiger && spawned_pet.borrow().lvl == 1);

    team.restore();

    // Level tapir to lvl 2.
    team.set_level(&Position::First, 2).unwrap();

    team.fight(&mut enemy_team).unwrap();

    // Same lobster spawns but at lvl 2.
    let spawned_pet = team.first().unwrap();
    assert!(spawned_pet.borrow().name == PetName::Tiger && spawned_pet.borrow().lvl == 2);
}

#[test]
fn test_battle_walrus_team() {
    let mut team = test_walrus_team();
    team.set_seed(Some(25));
    let mut enemy_team = test_gorilla_team();

    team.fight(&mut enemy_team).unwrap();

    // First cricket after walrus faints gets peanuts.
    assert_eq!(
        team.first().unwrap().borrow().item.as_ref().unwrap().name,
        FoodName::Peanut
    );
}

#[test]
fn test_battle_white_tiger_team() {
    let mut team = test_white_tiger_team();
    let mut enemy_team = test_gorilla_team();

    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();

    // Deer behind gets 3 exp leveling to 2.
    let deer = team.nth(1).unwrap();
    assert!(deer.borrow().lvl == 2 && deer.borrow().exp == 3);
}

#[test]
fn test_battle_octopus_team() {
    let mut team = test_octopus_team();
    let mut enemy_team = test_cricket_horse_team();
    enemy_team.set_seed(Some(25));

    team.fight(&mut enemy_team).unwrap();

    // Octopus only takes one damage.
    assert_eq!(team.first().unwrap().borrow().stats.health, 7);
    // But kills two crickets with ability + attack.
    assert_eq!(enemy_team.fainted.len(), 2);
}

#[test]
fn test_battle_orca_team() {
    let mut team = test_orca_team();
    team.set_seed(Some(25));
    let mut enemy_team = test_gorilla_team();

    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();

    let summoned_pet = team.first().unwrap();
    let effect = &summoned_pet.borrow().effect[0];
    let effect_trigger_status = effect.trigger.status.clone();
    // Summoned pet is a pet with a faint trigger.
    assert_eq!(effect_trigger_status, Status::Faint);
}

#[test]
fn test_battle_piranha_team() {
    let mut team = test_piranha_team();
    let mut enemy_team = test_gorilla_team();

    for pet in team.all().get(1..).unwrap() {
        assert!(pet.borrow().stats.attack == 1)
    }

    // Piranha (lvl.1) faints.
    team.fight(&mut enemy_team).unwrap();

    // And all pets behind get (3,0).
    for pet in team.all() {
        assert!(pet.borrow().stats.attack == 4)
    }
}

#[test]
fn test_battle_reindeer_team() {
    let mut team = test_reindeer_team();
    let mut enemy_team = test_gorilla_team();

    // No item before fight.
    let reindeer = team.first().unwrap();
    assert_eq!(reindeer.borrow().item, None);

    team.fight(&mut enemy_team).unwrap();

    // After fight has melon but already used.
    assert_eq!(
        reindeer.borrow().item.as_ref().unwrap().name,
        FoodName::Melon
    );
    assert_eq!(
        reindeer.borrow().item.as_ref().unwrap().ability.uses,
        Some(0)
    );
}

#[test]
fn test_battle_sabertooth_team() {
    let mut team = test_sabertooth_team();
    let mut enemy_team = test_cricket_horse_team();

    let sabertooth_stats = team.first().unwrap().borrow().stats;

    team.fight(&mut enemy_team).unwrap();

    // Sabertooth hurt.
    assert_ne!(sabertooth_stats, team.nth(1).unwrap().borrow().stats);
    let summoned_pet = team.first().unwrap();
    // Tier 1, lvl 1, pet at fixed (8,8) summoned.
    assert!(
        summoned_pet.borrow().stats == Statistics::new(8, 8).unwrap()
            && summoned_pet.borrow().tier == 1
            && summoned_pet.borrow().lvl == 1
    );
}

#[test]
fn test_battle_spinosaurus_team() {
    let mut team = test_spinosaurus_team();
    team.set_seed(Some(52));
    let mut enemy_team = test_piranha_team();

    // Dog at pos 1 to get buff is (3,4).
    let dog = team.nth(1).unwrap();
    let dog_stats_original = dog.borrow().stats;
    assert_eq!(dog_stats_original, Statistics::new(3, 4).unwrap());

    // Dog at pos 0 faints
    team.fight(&mut enemy_team).unwrap();

    // Dog gains (3,2) from spinosaurus.
    assert_eq!(team.fainted.len(), 1);
    assert_eq!(
        dog_stats_original + Statistics::new(3, 2).unwrap(),
        dog.borrow().stats
    );
}

#[test]
fn test_battle_stegosaurus_team() {
    let mut team = test_stegosaurus_team();
    let mut enemy_team = test_gorilla_team();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(3, 4).unwrap()
    );

    // Current turn is 1 so stego should give (1/1 * 1) = (1/1)
    assert!(team.history.curr_turn == 1);

    // Activate start of battle effects.
    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(4, 5).unwrap()
    );

    // Reset team.
    team.restore();

    // Increase turns to 3. Stego should give (1/1 * 3) = (3/3)
    team.history.curr_turn += 2;

    // Re-activate start of battle effects.
    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();

    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(6, 7).unwrap()
    );
}

#[test]
fn test_battle_velociraptor_team() {
    let mut team = test_velociraptor_team();
    team.set_seed(Some(12));
    let mut enemy_team = test_gorilla_team();

    // Cricket at pos 1 has strawberry.
    assert_eq!(
        team.nth(1).unwrap().borrow().item.as_ref().unwrap().name,
        FoodName::Strawberry
    );
    // Trigger start of battle effects.
    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();

    // Cricket at pos 1 now has coconut.
    assert_eq!(
        team.nth(1).unwrap().borrow().item.as_ref().unwrap().name,
        FoodName::Coconut
    );
}
