use sapt::{
    battle::{
        effect::Entity,
        state::{Action, Position, Status, Target},
        trigger::*,
    },
    Effect, EffectApply, Food, FoodName, Pet, PetName, Statistics, Team,
};

#[test]
fn test_create_known_pet() {
    let pet_from_name = Pet::try_from(PetName::Ant).unwrap();
    let pet_from_constructor = Pet::new(
        PetName::Ant,
        Some("Ant".to_string()),
        Some(Statistics::new(2, 1).unwrap()),
        1,
    )
    .unwrap();

    assert!([pet_from_name.clone(), pet_from_constructor]
        .iter()
        .all(|pet| pet == &pet_from_name));
}

#[test]
fn test_create_custom_pet() {
    // Build your own pets.
    // This version of the Bear gives adjacent pets melon at the start of battle.
    let custom_pet = Pet::custom(
        "MelonBear",
        None,
        Statistics::new(50, 50).unwrap(),
        &[Effect::new(
            Entity::Pet,
            TRIGGER_START_BATTLE,
            Target::Friend,
            Position::Adjacent,
            Action::Gain(Some(Box::new(Food::try_from(FoodName::Melon).unwrap()))),
            Some(1),
            false,
        )],
    );
    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Ant).unwrap(),
            custom_pet,
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        5,
    )
    .unwrap();
    let mut enemy_team = team.clone();

    // Trigger start of battle effects.
    team.trigger_effects(&mut enemy_team);

    assert!(
        team.friends[0].borrow().item.as_ref().unwrap().name == FoodName::Melon
            && team.friends[2].borrow().item.as_ref().unwrap().name == FoodName::Melon
    )
}

#[test]
fn test_create_team() {
    let mut team = Team::default();
    team.add_pet(Pet::try_from(PetName::Dog).unwrap(), 0, None)
        .unwrap()
        .add_pet(Pet::try_from(PetName::Dog).unwrap(), 0, None)
        .unwrap()
        .add_pet(Pet::try_from(PetName::Dog).unwrap(), 0, None)
        .unwrap()
        .add_pet(Pet::try_from(PetName::Dog).unwrap(), 0, None)
        .unwrap()
        .add_pet(Pet::try_from(PetName::Dog).unwrap(), 0, None)
        .unwrap();

    assert_eq!(team.friends.len(), 5)
}

#[test]
fn test_pet_exp() {
    let mut pet = Pet::try_from(PetName::Ant).unwrap();

    // Add single point.
    pet.add_experience(1).unwrap();
    assert!(pet.get_experience() == 1 && pet.get_level() == 1);
    assert!(pet.stats.attack == 3 && pet.stats.health == 2);

    // Add three points to reach level 2 and 4 total exp points.
    pet.add_experience(3).unwrap();
    assert!(pet.get_experience() == 4 && pet.get_level() == 2);
    assert!(pet.stats.attack == 6 && pet.stats.health == 5);

    // Add one point to reach level cap.
    pet.add_experience(1).unwrap();
    assert!(pet.get_experience() == 5 && pet.get_level() == 3);
    assert!(pet.stats.attack == 7 && pet.stats.health == 6);

    // Additional experience is not allowed.
    assert!(pet.add_experience(3).is_err())
}

#[test]
fn test_apply_effect() {
    // Get mosquito effect.
    let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    // Get effect with no reference.
    let no_ref_mosquito_effect = mosquito.effect.first().cloned().unwrap();

    // Init teams.
    let mut team = Team::new(&vec![mosquito.clone(); 5], 5).unwrap();
    let mut enemy_team = Team::new(&vec![mosquito; 5], 5).unwrap();
    enemy_team.set_seed(0);

    // Without a reference to the pet owning the effect, this will fail.
    assert!(team
        .apply_effect(
            &TRIGGER_START_BATTLE,
            &no_ref_mosquito_effect,
            &mut enemy_team
        )
        .is_err());

    // Get mosquito_effect with reference.
    // Apply effect of mosquito at position 0 to a pet on team to enemy team.
    let mosquito_effect = team.friends[0].borrow().effect[0].clone();
    team.apply_effect(&TRIGGER_START_BATTLE, &mosquito_effect, &mut enemy_team)
        .unwrap();

    // Last enemy mosquito takes one damage and opponent triggers gets updated.
    assert_eq!(
        enemy_team.friends[4].borrow().stats,
        Statistics::new(2, 1).unwrap()
    );
    assert!(enemy_team
        .triggers
        .iter()
        .find(|trigger| trigger.status == Status::Hurt)
        .is_some());
}
