use itertools::Itertools;

use crate::{
    battle::{
        actions::{Action, GainType, StatChangeType},
        state::{Position, TeamFightOutcome},
        stats::Statistics,
    },
    foods::names::FoodName,
    pets::names::PetName,
    tests::common::{
        count_pets, test_cricket_horse_team, test_crocodile_team, test_eagle_team, test_hyena_team,
        test_lion_highest_tier_team, test_lion_lowest_tier_team, test_lionfish_team,
        test_mammoth_team, test_microbe_team, test_rhino_team, test_scorpion_team, test_shark_team,
        test_skunk_team, test_swordfish_team, test_triceratops_team, test_turkey_team,
        test_vulture_team,
    },
    Food, TeamEffects,
};

// use crate::LOG_CONFIG;

#[test]
fn test_battle_croc_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_crocodile_team();
    let mut enemy_team = test_crocodile_team();

    let last_pet = team.friends.last().unwrap();
    let last_enemy_pet = team.friends.last().unwrap();
    assert_eq!(last_pet.borrow().name, PetName::Cricket);
    assert_eq!(last_enemy_pet.borrow().name, PetName::Cricket);

    // After start of battle, both crickets at end are sniped.
    // Two zombie crickets are spawned in their place.
    team.fight(&mut enemy_team).unwrap();

    let last_pet = team.friends.last().unwrap().borrow();
    let last_enemy_pet = team.friends.last().unwrap().borrow();

    assert_eq!(team.friends.len(), 4);
    assert_eq!(enemy_team.friends.len(), 4);
    assert_eq!(last_pet.name, PetName::ZombieCricket);
    assert_eq!(last_enemy_pet.name, PetName::ZombieCricket);
}

#[test]
fn test_battle_rhino_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_rhino_team();
    let mut enemy_team = test_cricket_horse_team();

    let outcome = team.fight(&mut enemy_team).unwrap();

    assert_eq!(outcome, TeamFightOutcome::None);
    // Only one damage from first cricket to trigger chain of faint triggers.
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 5,
            health: 7
        }
    );
    // All pets mowed down by rhino. After horse faints, zombie crickets spawn.
    assert!(enemy_team
        .all()
        .iter()
        .all(|pet| pet.borrow().name == PetName::ZombieCricket))
}

#[test]
fn test_battle_scorpion_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_scorpion_team();
    let mut enemy_team = test_skunk_team();
    // At start of turn, scorpion doesn't have peanuts. Then gains it.
    assert_eq!(team.first().unwrap().borrow().item, None);
    let outcome = team.fight(&mut enemy_team).unwrap();

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
        &Action::Gain(GainType::DefaultItem(FoodName::Peanut))
    )
}

#[test]
fn test_battle_shark_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_shark_team();
    let mut enemy_team = test_shark_team();

    // Removed
    enemy_team.friends.remove(0);

    let n_team_crickets = count_pets(&team.friends, PetName::Cricket);
    let n_enemy_team_crickets = count_pets(&enemy_team.friends, PetName::Cricket);

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

    let mut outcome = team.fight(&mut enemy_team).unwrap();

    while let TeamFightOutcome::None = outcome {
        outcome = team.fight(&mut enemy_team).unwrap();
    }

    let mut added_shark_stats = Statistics::default();
    let mut added_enemy_shark_stats = Statistics::default();
    for node in team.history.effect_graph.raw_edges() {
        let (_, _, action, _) = &node.weight;
        if let Action::Add(StatChangeType::StaticValue(stats)) = action {
            added_shark_stats += stats.clone()
        }
    }
    for node in enemy_team.history.effect_graph.raw_edges() {
        let (_, _, action, _) = &node.weight;
        if let Action::Add(StatChangeType::StaticValue(stats)) = action {
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

    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();

    // Cricket faints, zombie version spawned, and it gains (3,3) (lvl.1 turkey)
    let zombie_cricket = team.first().unwrap();
    assert_eq!(
        zombie_cricket.borrow().stats,
        Statistics {
            attack: 4,
            health: 4
        }
    );
}

#[test]
fn test_battle_hyena_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_hyena_team();
    let mut enemy_team = test_cricket_horse_team();
    team.set_seed(20);
    enemy_team.set_seed(20);

    // Original positions
    assert_eq!(team.nth(1).unwrap().borrow().name, PetName::Gorilla);
    assert_eq!(enemy_team.nth(2).unwrap().borrow().name, PetName::Horse);

    let team_stats = team
        .friends
        .iter()
        .map(|pet| pet.borrow().stats)
        .collect_vec();
    let enemy_stats = enemy_team
        .friends
        .iter()
        .map(|pet| pet.borrow().stats)
        .collect_vec();

    team.trigger_effects(&mut enemy_team).unwrap();

    // At lvl. 1 hyena swaps stats of all pets.
    for (mut og, new) in team
        .friends
        .iter()
        .map(|pet| pet.borrow().stats)
        .zip_eq(team_stats)
    {
        assert_eq!(og.invert().to_owned(), new)
    }
    for (mut og, new) in enemy_team
        .friends
        .iter()
        .map(|pet| pet.borrow().stats)
        .zip_eq(enemy_stats)
    {
        assert_eq!(og.invert().to_owned(), new)
    }

    // Reset teams.
    team.restore();
    enemy_team.restore();

    // Level up hyena.
    team.set_level(Position::First, 2).unwrap();
    team.trigger_effects(&mut enemy_team).unwrap();

    // Hyena at lvl. 2 swaps positions of pets.
    assert_eq!(team.first().unwrap().borrow().name, PetName::Gorilla);
    assert_eq!(enemy_team.first().unwrap().borrow().name, PetName::Horse);

    // Reset teams.
    team.restore();
    enemy_team.restore();

    // Level up hyena.
    team.set_level(Position::First, 3).unwrap();
    team.trigger_effects(&mut enemy_team).unwrap();

    // Hyena at lvl. 3 swaps positions and stats of pets.
    let gorilla = team.first().unwrap();
    let horse = enemy_team.first().unwrap();
    assert!(
        gorilla.borrow().name == PetName::Gorilla
            && gorilla.borrow().stats == Statistics::new(9, 6).unwrap()
    );
    assert!(
        horse.borrow().name == PetName::Horse
            && horse.borrow().stats == Statistics::new(1, 2).unwrap()
    );
}

#[test]
fn test_battle_lionfish_team() {
    // log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_lionfish_team();
    let mut enemy_team = test_mammoth_team();

    // Enemy team's mammoth has no item.
    let mammoth = enemy_team.first().unwrap();
    assert_eq!(mammoth.borrow().item, None);
    assert_eq!(mammoth.borrow().stats, Statistics::new(3, 10).unwrap());

    team.fight(&mut enemy_team).unwrap();

    // Prior to Dog at position 0 of team attacking, lionfish ability activates giving weakness to frontline mammoth.
    // Mammoth takes additional damage as a result.
    let mut example_weakness = Food::try_from(FoodName::Weak).unwrap();
    example_weakness.ability.assign_owner(Some(&mammoth));

    assert_eq!(mammoth.borrow().item, Some(example_weakness));
    assert_eq!(mammoth.borrow().stats, Statistics::new(3, 4).unwrap());
}

#[test]
fn test_battle_eagle_team() {
    // log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_eagle_team();
    let mut enemy_team = test_eagle_team();

    team.fight(&mut enemy_team).unwrap();

    let summoned_pet = team.first().unwrap();
    assert_eq!(summoned_pet.borrow().tier, 6);
}

#[test]
fn test_battle_microbe_team() {
    // log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_microbe_team();
    let mut enemy_team = test_eagle_team();

    team.fight(&mut enemy_team).unwrap();

    // All pets have weakness after microbe faints.
    for pet in team.friends.iter().chain(enemy_team.friends.iter()) {
        assert_eq!(pet.borrow().item.as_ref().unwrap().name, FoodName::Weak);
    }
}

#[test]
fn test_battle_lion_lowest_tier_team() {
    // log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_lion_lowest_tier_team();
    let mut enemy_team = test_eagle_team();

    let highest_tier_pet = team
        .all()
        .into_iter()
        .max_by(|pet_1, pet_2| pet_1.borrow().tier.cmp(&pet_2.borrow().tier))
        .unwrap();
    // Highest tier pet not lion.
    assert_ne!(highest_tier_pet.borrow().name, PetName::Lion);
    let lion = team.first().unwrap();
    let lion_original_stats = lion.borrow().stats;

    // Activate start of battle effects.
    team.trigger_effects(&mut enemy_team).unwrap();

    // Stats are unchanged.
    assert_eq!(lion_original_stats, team.first().unwrap().borrow().stats)
}

#[test]
fn test_battle_lion_highest_tier_team() {
    // log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_lion_highest_tier_team();
    let mut enemy_team = test_eagle_team();

    let highest_tier_pet = team
        .all()
        .into_iter()
        .max_by(|pet_1, pet_2| pet_1.borrow().tier.cmp(&pet_2.borrow().tier))
        .unwrap();
    // Highest tier pet is lion.
    assert_eq!(highest_tier_pet.borrow().name, PetName::Lion);
    let lion = team.first().unwrap();
    let lion_original_stats = lion.borrow().stats;

    // Activate start of battle effects.
    team.trigger_effects(&mut enemy_team).unwrap();

    // Adds 50% of attack and health to original stats.
    assert_eq!(
        lion_original_stats + (lion_original_stats * Statistics::new(50, 50).unwrap()),
        team.first().unwrap().borrow().stats
    )
}

#[test]
fn test_battle_swordfish_team() {
    // log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_swordfish_team();
    let mut enemy_team = test_eagle_team();

    let swordfish = team.first().unwrap();
    let eagle = enemy_team.first().unwrap();

    assert!(eagle.borrow().stats.health == 5);
    assert!(swordfish.borrow().stats.health == 25);

    // Activate start of battle effect.
    team.trigger_effects(&mut enemy_team).unwrap();

    // Both swordfish and enemy eagle are hit and take 25 dmg.
    assert!(swordfish.borrow().stats.health == 0);
    assert!(eagle.borrow().stats.health == 0);
}

#[test]
fn test_battle_triceratops_team() {
    // log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_triceratops_team();
    let mut enemy_team = test_cricket_horse_team();

    let triceratops = team.first().unwrap();
    let gorilla = team.nth(1).unwrap();

    assert_eq!(triceratops.borrow().stats, Statistics::new(5, 6).unwrap());
    assert_eq!(gorilla.borrow().stats, Statistics::new(6, 9).unwrap());

    team.fight(&mut enemy_team).unwrap();

    // Triceratops takes 1 dmg. Gorilla behind gets (3,3) buff.
    assert_eq!(triceratops.borrow().stats, Statistics::new(5, 5).unwrap());
    assert_eq!(gorilla.borrow().stats, Statistics::new(9, 12).unwrap());
}

#[test]
fn test_battle_vulture_team() {
    // log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_vulture_team();
    let mut enemy_team = test_cricket_horse_team();
    enemy_team.set_seed(25);

    // Three attack phases to reach two fainted pets.
    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();

    // Two fainted pets.
    assert_eq!(team.fainted.len(), 2);
    // Enemy team has an additional fainted pet because vulture effect triggers.
    assert_eq!(enemy_team.fainted.len(), 3);
}
