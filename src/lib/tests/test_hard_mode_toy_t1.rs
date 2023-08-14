use crate::{
    teams::team::TeamFightOutcome, FoodName, Pet, PetName, Statistics, Team, TeamCombat,
    TeamEffects, TeamShopping, TeamViewer, Toy, ToyName,
};

use super::common::{test_ant_team, test_camel_team};

#[test]
fn test_toy_boomerang() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::Boomerang).unwrap());

    let first_ant = team.first().unwrap();
    first_ant.write().unwrap().stats.health = 32;
    let starting_ant_stats = first_ant.read().unwrap().stats;

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();
    // First ant takes 30 health.
    assert_eq!(
        first_ant.read().unwrap().stats,
        starting_ant_stats
            - Statistics {
                attack: 0,
                health: 30
            }
    );
}

#[test]
fn test_toy_dice_cup() {
    let mut team = test_camel_team();
    team.set_seed(Some(1234));

    let mut enemy_team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::DiceCup).unwrap());

    let pets = team.all();
    let original_ordering: Vec<PetName> = pets
        .iter()
        .map(|pet| pet.read().unwrap().name.clone())
        .collect();

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    let new_ordering: Vec<PetName> = pets
        .iter()
        .map(|pet| pet.read().unwrap().name.clone())
        .collect();

    // Pets reordered.
    assert_eq!(
        original_ordering,
        vec![PetName::Elephant, PetName::Camel, PetName::Ant]
    );
    assert_eq!(
        new_ordering,
        vec![PetName::Elephant, PetName::Ant, PetName::Camel]
    );
}

#[test]
fn test_toy_dodgeball() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::Dodgeball).unwrap());
    // println!("{team}");

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    // println!("{team}");

    todo!("Need to restore uses if kills first pet.")
}

#[test]
fn test_toy_handerkerchief() {
    let mut team = test_ant_team();
    // Set shop tier to two.
    team.set_shop_tier(2).unwrap();

    let mut enemy_team = test_ant_team();

    // Add handkerchief
    team.toys
        .push(Toy::try_from(ToyName::Handkerchief).unwrap());

    let pets = team.all();

    assert!(pets.iter().all(|pet| pet.read().unwrap().item.is_none()));
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    // At shop tier of two, only first two friends get weakness.
    assert!(pets.get(0..=1).unwrap().iter().all(|pet| pet
        .read()
        .unwrap()
        .item
        .as_ref()
        .unwrap()
        .name
        == FoodName::Weak));
    assert!(pets[2].read().unwrap().item.is_none())
}

#[test]
fn test_toy_pen() {
    let mut team = test_ant_team();
    // Set shop tier to two.
    team.set_shop_tier(2).unwrap();

    let mut enemy_team = test_ant_team();

    // Add handkerchief
    team.toys.push(Toy::try_from(ToyName::Pen).unwrap());

    let pets = team.all();

    assert!(pets.iter().all(|pet| pet.read().unwrap().item.is_none()));
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    // At shop tier of two, only first two friends get inked.
    assert!(pets.get(0..=1).unwrap().iter().all(|pet| pet
        .read()
        .unwrap()
        .item
        .as_ref()
        .unwrap()
        .name
        == FoodName::Ink));
    assert!(pets[2].read().unwrap().item.is_none())
}

#[test]
fn test_toy_pogo_stick() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    let first_ant = enemy_team.first().unwrap();
    first_ant.write().unwrap().stats.health = 1;
    let starting_stats = first_ant.read().unwrap().stats;

    // Add pogo stick
    team.toys.push(Toy::try_from(ToyName::PogoStick).unwrap());

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    // Weakest ant gets 4x in stats.
    assert_eq!(
        starting_stats.mult_perc(&Statistics {
            attack: 400,
            health: 400
        }),
        first_ant.read().unwrap().stats
    );
}

#[test]
fn test_toy_rock_bag() {
    let mut team = Team::new(
        &[
            Some(Pet::try_from(PetName::Beaver).unwrap()),
            Some(Pet::try_from(PetName::Gorilla).unwrap()),
        ],
        5,
    )
    .unwrap();
    let mut enemy_team = test_ant_team();

    // Set turn to 2.
    const CURR_TURN: usize = 2;
    team.history.curr_turn = CURR_TURN;

    team.toys.push(Toy::try_from(ToyName::RockBag).unwrap());

    let gorilla = team.nth(1).unwrap();
    let gorilla_stats = gorilla.read().unwrap().stats;

    team.fight(&mut enemy_team).unwrap();

    // Gorilla hit twice after pet faints.
    assert_eq!(
        gorilla.read().unwrap().stats,
        gorilla_stats
            - Statistics {
                attack: 0,
                health: (2 * CURR_TURN) as isize
            }
    );
}

#[test]
fn test_toy_scissors() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    let (first_ant, second_ant) = (team.first().unwrap(), team.nth(1).unwrap());
    team.toys.push(Toy::try_from(ToyName::Scissors).unwrap());

    assert!(
        first_ant.read().unwrap().stats.health != 1 && second_ant.read().unwrap().stats.health != 1
    );
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    // Health is set to 1.
    assert!(
        first_ant.read().unwrap().stats.health == 1 && second_ant.read().unwrap().stats.health == 1
    );
}

#[test]
fn test_toy_spinning_top() {
    let mut team = test_ant_team();
    team.set_seed(Some(123));

    let mut enemy_team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::SpinningTop).unwrap());

    const ANT_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };
    const TOP_DMG: Statistics = Statistics {
        attack: 0,
        health: 2,
    };
    let ant = team.nth(1).unwrap();
    let ant_stats = ant.read().unwrap().stats;

    team.fight(&mut enemy_team).unwrap();

    // Ant gets buff and then top dmg.
    assert_eq!(ant_stats + ANT_BUFF - TOP_DMG, ant.read().unwrap().stats);
}

#[test]
fn test_toy_unicycle() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    let enemies = enemy_team.all();

    team.toys.push(Toy::try_from(ToyName::Unicycle).unwrap());
    team.history.curr_turn = 2;

    // only first two enemies get buffed as turn is 2.
    const STAT_CHANGES: [Statistics; 3] = [
        Statistics {
            attack: 1,
            health: 1,
        },
        Statistics {
            attack: 1,
            health: 1,
        },
        Statistics {
            attack: 0,
            health: 0,
        },
    ];
    let exp_stats: Vec<Statistics> = enemies
        .iter()
        .zip(STAT_CHANGES)
        .map(|(pet, changes)| pet.read().unwrap().stats + changes)
        .collect();
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    assert_eq!(
        exp_stats,
        enemies
            .iter()
            .map(|pet| pet.read().unwrap().stats)
            .collect::<Vec<Statistics>>()
    );
}

#[test]
fn test_toy_yoyo() {
    let mut team = Team::new(
        &[Some(
            Pet::new(
                PetName::Gorilla,
                Some(Statistics {
                    attack: 50,
                    health: 50,
                }),
                3,
            )
            .unwrap(),
        )],
        5,
    )
    .unwrap();
    let mut enemy_team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::YoYo).unwrap());

    // Fight lvl 1 def ant team with lvl 3 maxed gorilla.
    let mut outcome = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = outcome {
        outcome = team.fight(&mut enemy_team).unwrap();
    }
    // Gorilla faints due to yoyo as all enemies fainted.
    assert!(team.friends.is_empty())
}
