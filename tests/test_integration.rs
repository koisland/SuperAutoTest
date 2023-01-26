use sapt::{
    battle::{
        effect::Entity,
        state::{Action, Condition, Position, Status, Target},
        trigger::TRIGGER_SELF_FAINT,
    },
    Effect, EffectApply, Food, FoodName, Outcome, Pet, PetName, Statistics, Team,
};

#[test]
fn test_create_known_pet() {
    let pet_from_name = Pet::from(PetName::Ant);
    let pet_from_constructor = Pet::new(
        PetName::Ant,
        Some("Ant".to_string()),
        Some(Statistics::new(2, 1)),
        1,
    )
    .unwrap();
    let pet_from_struct_fields = Pet {
        id: Some("Ant".to_string()),
        name: PetName::Ant,
        tier: 1,
        stats: Statistics::new(2, 1),
        lvl: 1,
        exp: 0,
        effect: vec![Effect {
            trigger: TRIGGER_SELF_FAINT,
            target: Target::Friend,
            position: Position::Any(Condition::None),
            action: Action::Add(Statistics::new(2, 1)),
            uses: Some(1),
            entity: Entity::Pet,
            temp: false,
        }],
        item: None,
        pos: None,
        cost: 3,
        seed: 0,
    };
    assert!([
        pet_from_name.clone(),
        pet_from_constructor,
        pet_from_struct_fields
    ]
    .iter()
    .all(|pet| pet == &pet_from_name));
}

#[test]
fn test_create_custom_pet() {
    // Build your own pets.
    // This version of the Bear gives adjacent pets melon at the start of battle.
    let custom_pet = Pet {
        id: None,
        name: PetName::Custom("MelonBear".to_string()),
        tier: 1,
        stats: Statistics::new(50, 50),
        lvl: 1,
        exp: 0,
        effect: vec![Effect {
            entity: Entity::Pet,
            trigger: Outcome {
                status: Status::StartBattle,
                target: Target::None,
                position: Position::None,
                idx: None,
                stat_diff: None,
            },
            target: Target::Friend,
            position: Position::Adjacent,
            action: Action::Gain(Box::new(Food::from(FoodName::Melon))),
            uses: Some(1),
            temp: false,
        }],
        item: None,
        pos: None,
        cost: 3,
        seed: 0,
    };
    let mut team = Team::new(
        &[
            Some(Pet::from(PetName::Ant)),
            Some(custom_pet),
            Some(Pet::from(PetName::Ant)),
        ],
        5,
    )
    .unwrap();
    let mut enemy_team = team.clone();

    // Trigger start of battle effects.
    team.trigger_effects(&mut enemy_team);

    assert!(
        team.nth(0).unwrap().item.as_ref().unwrap().name == FoodName::Melon
            && team.nth(2).unwrap().item.as_ref().unwrap().name == FoodName::Melon
    )
}

#[test]
fn test_create_team() {
    let mut team = Team::default();
    team.add_pet(Pet::from(PetName::Dog), 0, None)
        .unwrap()
        .add_pet(Pet::from(PetName::Dog), 0, None)
        .unwrap()
        .add_pet(Pet::from(PetName::Dog), 0, None)
        .unwrap()
        .add_pet(Pet::from(PetName::Dog), 0, None)
        .unwrap()
        .add_pet(Pet::from(PetName::Dog), 0, None)
        .unwrap();

    assert_eq!(team.friends.len(), 5)
}

#[test]
fn test_pet_exp() {
    let mut pet = Pet::from(PetName::Ant);

    // Add single point.
    pet.add_experience(1).unwrap();
    assert!(pet.exp == 1 && pet.lvl == 1);
    assert!(pet.stats.attack == 3 && pet.stats.health == 2);

    // Add three points to reach level 2 and 4 total exp points.
    pet.add_experience(3).unwrap();
    assert!(pet.exp == 4 && pet.lvl == 2);
    assert!(pet.stats.attack == 6 && pet.stats.health == 5);

    // Add one point to reach level cap.
    pet.add_experience(1).unwrap();
    assert!(pet.exp == 5 && pet.lvl == 3);
    assert!(pet.stats.attack == 7 && pet.stats.health == 6);

    // Additional experience is not allowed.
    assert!(pet.add_experience(3).is_err())
}

#[test]
fn test_apply_effect() {
    use sapt::{EffectApply, Pet, PetName, Statistics, Team};

    // Get mosquito effect.
    let mosquito = Pet::from(PetName::Mosquito);
    let mosquito_effect = mosquito.effect.first().unwrap().clone();

    let mut team = Team::new(&vec![Some(mosquito); 5], 5).unwrap();
    let mut enemy_team = team.clone();
    enemy_team.set_seed(0);

    // Get start of battle trigger.
    let start_of_battle_trigger = team.triggers.pop_back().unwrap();

    // Apply effect of mosquito at position 0 to a pet on team to enemy team.
    team.apply_effect(
        0,
        &start_of_battle_trigger,
        &mosquito_effect,
        &mut enemy_team,
    )
    .unwrap();

    // Enemy mosquito takes one damage.
    assert_eq!(
        enemy_team.friends[4].as_ref().unwrap().stats,
        Statistics::new(2, 1)
    );
    assert!(enemy_team
        .triggers
        .iter()
        .find(|trigger| trigger.status == Status::Hurt)
        .is_some());
}
