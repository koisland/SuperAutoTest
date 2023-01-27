use crate::{
    battle::{
        state::{Statistics, TeamFightOutcome},
        team::Team,
    },
    foods::{food::Food, names::FoodName},
    pets::{combat::PetCombat, names::PetName, pet::Pet},
    tests::common::test_ant_team,
};
// use crate::LOG_CONFIG;

#[test]
fn test_attack_meat() {
    let mut dog_w_meat = Pet::from(PetName::Dog);
    dog_w_meat.item = Some(Food::from(FoodName::MeatBone));

    let mut mammoth = Pet::from(PetName::Mammoth);

    assert_eq!(
        dog_w_meat.stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    assert_eq!(
        mammoth.stats,
        Statistics {
            attack: 3,
            health: 10
        }
    );

    dog_w_meat.attack(&mut mammoth);

    // Dog deals 7 dmg (+4) with meat instead of 3 dmg .
    assert_eq!(
        dog_w_meat.stats,
        Statistics {
            attack: 3,
            health: 1
        }
    );
    assert_eq!(
        mammoth.stats,
        Statistics {
            attack: 3,
            health: 3
        }
    )
}

#[test]
fn test_attack_garlic() {
    let mut dog_w_garlic = Pet::from(PetName::Dog);
    dog_w_garlic.item = Some(Food::from(FoodName::Garlic));
    let mut dog = Pet::from(PetName::Dog);

    assert_eq!(
        dog_w_garlic.stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    assert_eq!(
        dog.stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );

    dog.attack(&mut dog_w_garlic);

    // Garlic prevents 2 dmg from attack.
    assert_eq!(
        dog_w_garlic.stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
}

#[test]
fn test_attack_garlic_min_1() {
    let mut dog_w_garlic = Pet::from(PetName::Dog);
    dog_w_garlic.item = Some(Food::from(FoodName::Garlic));
    let mut ant = Pet::from(PetName::Ant);

    assert_eq!(
        dog_w_garlic.stats,
        Statistics {
            attack: 3,
            health: 4
        }
    );
    assert_eq!(
        ant.stats,
        Statistics {
            attack: 2,
            health: 1
        }
    );

    ant.attack(&mut dog_w_garlic);

    // Dog still takes 1 dmg despite negating 2 dmg from 2 atk ant.
    assert_eq!(
        dog_w_garlic.stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
}

#[test]
fn test_attack_melon() {
    let mut dog_w_melon = Pet::from(PetName::Dog);
    dog_w_melon.item = Some(Food::from(FoodName::Melon));

    assert_eq!(dog_w_melon.item.as_ref().unwrap().ability.uses, Some(1));

    let original_stats = dog_w_melon.stats.clone();

    let mut big_ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 20,
            health: 20,
        }),
        1,
    )
    .unwrap();

    dog_w_melon.attack(&mut big_ant);

    // Dog takes no damage (20 - 20 = 0) due to melon armor.
    assert_eq!(dog_w_melon.stats, original_stats);
    assert_eq!(
        big_ant.stats,
        Statistics {
            health: 17,
            attack: 20
        }
    );

    // Negating one attack drops uses to 0.
    assert_eq!(dog_w_melon.item.as_ref().unwrap().ability.uses, Some(0));
}

#[test]
fn test_attack_steak() {
    let mut dog_w_steak = Pet::from(PetName::Dog);
    dog_w_steak.item = Some(Food::from(FoodName::Steak));

    assert_eq!(dog_w_steak.item.as_ref().unwrap().ability.uses, Some(1));

    let mut smol_ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 1,
            health: 23,
        }),
        1,
    )
    .unwrap();

    dog_w_steak.attack(&mut smol_ant);

    // Ant faints due to steak (3 + 20 = 23).
    assert_eq!(
        smol_ant.stats,
        Statistics {
            health: 0,
            attack: 1
        }
    );

    // Negating one attack drops uses to 0.
    assert_eq!(dog_w_steak.item.as_ref().unwrap().ability.uses, Some(0));
}

#[test]
fn test_attack_coconut() {
    let mut dog_w_coconut = Pet::from(PetName::Dog);
    dog_w_coconut.item = Some(Food::from(FoodName::Coconut));
    let original_dog_w_coconut_stats = dog_w_coconut.stats.clone();

    assert_eq!(dog_w_coconut.item.as_ref().unwrap().ability.uses, Some(1));

    let mut big_ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();

    dog_w_coconut.attack(&mut big_ant);

    // Dog takes not damage due to invulnerability from coconut.
    assert_eq!(dog_w_coconut.stats, original_dog_w_coconut_stats);

    // Negating one attack drops uses to 0.
    assert_eq!(dog_w_coconut.item.as_ref().unwrap().ability.uses, Some(0));
}

#[test]
fn test_attack_peanuts() {
    let mut scorpion = Pet::from(PetName::Scorpion);
    scorpion.item = Some(Food::from(FoodName::Peanut));

    let mut big_ant = Pet::new(
        PetName::Ant,
        None,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();

    scorpion.attack(&mut big_ant);

    assert_eq!(
        big_ant.stats,
        Statistics {
            attack: 50,
            health: 0
        }
    );
}

#[test]
fn test_attack_peanuts_coconut() {
    let mut dog_w_coconut = Pet::from(PetName::Dog);
    dog_w_coconut.item = Some(Food::from(FoodName::Coconut));
    let original_dog_w_coconut_stats = dog_w_coconut.stats.clone();

    let mut scorpion = Pet::from(PetName::Scorpion);
    scorpion.item = Some(Food::from(FoodName::Peanut));

    // Dog survives attack with coconut and takes no damage.
    dog_w_coconut.attack(&mut scorpion);

    assert_eq!(dog_w_coconut.stats, original_dog_w_coconut_stats);
}

#[test]
fn test_attack_peanuts_melon() {
    let mut dog_w_melon = Pet::from(PetName::Dog);
    dog_w_melon.item = Some(Food::from(FoodName::Melon));
    let original_dog_w_melon_stats = dog_w_melon.stats.clone();

    let mut scorpion = Pet::from(PetName::Scorpion);
    scorpion.item = Some(Food::from(FoodName::Peanut));

    // Dog survives attack with melon and takes no damage.
    dog_w_melon.attack(&mut scorpion);

    assert_eq!(dog_w_melon.stats, original_dog_w_melon_stats);
}

#[test]
fn test_attack_peanuts_melon_overflow() {
    let mut dog_w_melon = Pet::new(
        PetName::Dog,
        Some("big_dog".to_string()),
        Some(Statistics {
            attack: 1,
            health: 50,
        }),
        1,
    )
    .unwrap();
    dog_w_melon.item = Some(Food::from(FoodName::Melon));

    // Scorpion has just enough attack (> 20) to deliver death's touch.
    let mut scorpion = Pet::new(
        PetName::Scorpion,
        Some("scorpion".to_string()),
        Some(Statistics {
            attack: 21,
            health: 2,
        }),
        1,
    )
    .unwrap();
    // Note: Individually fighting pets doesn't trigger gaining peanuts.
    // Adding manually here.
    scorpion.item = Some(Food::from(FoodName::Peanut));

    dog_w_melon.attack(&mut scorpion);

    // Dog dies.
    assert_eq!(
        dog_w_melon.stats,
        Statistics {
            health: 0,
            attack: 1
        }
    )
}

#[test]
fn test_attack_chili() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = Team::new(
        &[Some(Pet::from(PetName::Ant)), Some(Pet::from(PetName::Ant))],
        5,
    )
    .unwrap();
    let mut enemy_team = Team::new(
        &[Some(Pet::from(PetName::Ant)), Some(Pet::from(PetName::Ant))],
        5,
    )
    .unwrap();

    // Give first pet chili on first team.
    // Will kill entire team in first attack
    let first_pet = team.friends.first_mut().unwrap().as_mut().unwrap();
    first_pet.item = Some(Food::new(&FoodName::Chili).unwrap());

    let outcome = team.fight(&mut enemy_team);

    // Ant team wins instead of drawing due to chili's splash damage.
    assert_eq!(outcome, TeamFightOutcome::Win);
}

#[test]
fn test_battle_honey_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    // Give last pet honey on first team.
    let last_pet = team.friends.get_mut(2).unwrap().as_mut().unwrap();
    last_pet.item = Some(Food::new(&FoodName::Honey).unwrap());

    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team);
    }

    // Ant team completes by team has honey so bee spawns.
    assert_eq!(fight, TeamFightOutcome::Win);
    assert_eq!(team.first().unwrap().name, PetName::Bee)
}

#[test]
fn test_battle_mushroom_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    // Give last pet mushroom on first team.
    let last_pet = team.friends.get_mut(2).unwrap().as_mut().unwrap();
    last_pet.item = Some(Food::new(&FoodName::Mushroom).unwrap());

    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team);
    }

    // Team wins over enemy by summoning ant with (1,1).
    assert_eq!(fight, TeamFightOutcome::Win);
    assert_eq!(team.first().unwrap().name, PetName::Ant);
    assert_eq!(
        team.first().unwrap().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    )
}
