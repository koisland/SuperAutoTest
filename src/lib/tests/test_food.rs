use crate::{
    effects::{
        actions::{Action, GainType, StatChangeType},
        effect::Entity,
        state::{Position, Target},
        stats::Statistics,
        trigger::{TRIGGER_NONE, TRIGGER_START_BATTLE},
    },
    foods::{food::Food, names::FoodName},
    pets::{combat::PetCombat, names::PetName, pet::Pet},
    teams::{
        team::{Team, TeamFightOutcome},
        team_viewer::TeamViewer,
    },
    tests::common::test_ant_team,
    Effect,
};
// use crate::LOG_CONFIG;

#[test]
fn test_custom_food() {
    let custom_food = Food::new(
        &FoodName::Custom("Dung".to_string()),
        Some(Effect {
            entity: Entity::Food,
            owner: None,
            trigger: TRIGGER_START_BATTLE,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Gain(GainType::DefaultItem(FoodName::Weak)),
            uses: Some(1),
            temp: true,
        }),
    );
    assert!(custom_food.is_ok())
}

#[test]
fn test_food_override_effect() {
    let buffed_apple = Food::new(
        &FoodName::Apple,
        Some(Effect {
            entity: Entity::Food,
            owner: None,
            trigger: TRIGGER_NONE,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Add(StatChangeType::StaticValue(Statistics {
                attack: 5,
                health: 5,
            })),
            uses: Some(1),
            temp: true,
        }),
    );
    assert!(buffed_apple.is_ok())
}
#[test]
fn test_attack_meat() {
    let mut dog_w_meat = Pet::try_from(PetName::Dog).unwrap();
    dog_w_meat.item = Some(Food::try_from(FoodName::MeatBone).unwrap());

    let mut mammoth = Pet::try_from(PetName::Mammoth).unwrap();

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
    let mut dog_w_garlic = Pet::try_from(PetName::Dog).unwrap();
    dog_w_garlic.item = Some(Food::try_from(FoodName::Garlic).unwrap());
    let mut dog = Pet::try_from(PetName::Dog).unwrap();

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
    let mut dog_w_garlic = Pet::try_from(PetName::Dog).unwrap();
    dog_w_garlic.item = Some(Food::try_from(FoodName::Garlic).unwrap());
    let mut ant = Pet::try_from(PetName::Ant).unwrap();

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
    let mut dog_w_melon = Pet::try_from(PetName::Dog).unwrap();
    dog_w_melon.item = Some(Food::try_from(FoodName::Melon).unwrap());

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
    let mut dog_w_steak = Pet::try_from(PetName::Dog).unwrap();
    dog_w_steak.item = Some(Food::try_from(FoodName::Steak).unwrap());

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
    let mut dog_w_coconut = Pet::try_from(PetName::Dog).unwrap();
    dog_w_coconut.item = Some(Food::try_from(FoodName::Coconut).unwrap());
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
    let mut scorpion = Pet::try_from(PetName::Scorpion).unwrap();
    scorpion.item = Some(Food::try_from(FoodName::Peanut).unwrap());

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
    let mut dog_w_coconut = Pet::try_from(PetName::Dog).unwrap();
    dog_w_coconut.item = Some(Food::try_from(FoodName::Coconut).unwrap());
    let original_dog_w_coconut_stats = dog_w_coconut.stats.clone();

    let mut scorpion = Pet::try_from(PetName::Scorpion).unwrap();
    scorpion.item = Some(Food::try_from(FoodName::Peanut).unwrap());

    // Dog survives attack with coconut and takes no damage.
    dog_w_coconut.attack(&mut scorpion);

    assert_eq!(dog_w_coconut.stats, original_dog_w_coconut_stats);
}

#[test]
fn test_attack_peanuts_melon() {
    let mut dog_w_melon = Pet::try_from(PetName::Dog).unwrap();
    dog_w_melon.item = Some(Food::try_from(FoodName::Melon).unwrap());
    let original_dog_w_melon_stats = dog_w_melon.stats.clone();

    let mut scorpion = Pet::try_from(PetName::Scorpion).unwrap();
    scorpion.item = Some(Food::try_from(FoodName::Peanut).unwrap());

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
    dog_w_melon.item = Some(Food::try_from(FoodName::Melon).unwrap());

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
    scorpion.item = Some(Food::try_from(FoodName::Peanut).unwrap());

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
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        5,
    )
    .unwrap();
    let mut enemy_team = Team::new(
        &[
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        5,
    )
    .unwrap();

    // Give first pet chili on first team.
    // Will kill entire team in first attack
    team.set_item(
        Position::First,
        Some(Food::try_from(&FoodName::Chili).unwrap()),
    )
    .unwrap();

    let outcome = team.fight(&mut enemy_team).unwrap();

    // Ant team wins instead of drawing due to chili's splash damage.
    assert_eq!(outcome, TeamFightOutcome::Win);
}

#[test]
fn test_battle_honey_team() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    // Give last pet honey on first team.
    team.set_item(
        Position::Last,
        Some(Food::try_from(&FoodName::Honey).unwrap()),
    )
    .unwrap();

    let mut fight = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team).unwrap();
    }

    // Ant team completes by team has honey so bee spawns.
    assert_eq!(fight, TeamFightOutcome::Win);
    assert_eq!(team.first().unwrap().borrow().name, PetName::Bee)
}

#[test]
fn test_battle_mushroom_team() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    // Give last pet mushroom on first team.
    team.set_item(
        Position::Last,
        Some(Food::try_from(&FoodName::Mushroom).unwrap()),
    )
    .unwrap();

    let mut fight = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team).unwrap();
    }

    // Team wins over enemy by summoning ant with (1,1).
    assert_eq!(fight, TeamFightOutcome::Win);
    assert_eq!(team.first().unwrap().borrow().name, PetName::Ant);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 1,
            health: 1
        }
    )
}
