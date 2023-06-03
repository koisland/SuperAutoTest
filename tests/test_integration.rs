use saptest::{
    effects::{
        actions::{Action, GainType, StatChangeType},
        effect::Entity,
        state::{Position, Status, Target},
        trigger::*,
    },
    teams::{combat::TeamCombat, team::TeamFightOutcome},
    Effect, Food, FoodName, Pet, PetName, SAPQuery, Shop, ShopItem, Statistics, Team, TeamEffects,
    TeamShopping, SAPDB,
};
use std::{str::FromStr, thread};

#[test]
fn test_query_db() {
    // Essentially "SELECT * FROM foods"
    let mut food_query = SAPQuery::builder();
    food_query.set_table(Entity::Food);
    assert!(SAPDB.execute_query(food_query).is_ok());

    // Essentially "SELECT * FROM pets"
    let mut pet_query = SAPQuery::builder();
    pet_query.set_table(Entity::Pet);
    assert!(SAPDB.execute_query(pet_query).is_ok());
}

#[test]
fn test_get_random_team_name() {
    let name = Team::get_random_name(5).unwrap();
    assert_eq!(&name, "The Submissive Stickers");
}

#[test]
fn test_create_known_food() {
    let food = Food::try_from(FoodName::Garlic);
    assert!(food.is_ok());
}

#[test]
fn test_create_custom_food() {
    // A custom Churro food item.
    // On any friend fainting, add (1,2) to the owner of the churro.
    // https://superautopets.fandom.com/f/p/4400000000000047398
    let food = Food::new(
        &FoodName::Custom("Churro".to_string()),
        Some(Effect::new(
            TRIGGER_ANY_FAINT,
            Target::Friend,
            Position::OnSelf,
            Action::Add(StatChangeType::Static(Statistics {
                attack: 1,
                health: 2,
            })),
            None,
            false,
        )),
    );
    assert!(food.is_ok());
}

#[test]
fn test_create_known_pet() {
    let pet_from_name = Pet::try_from(PetName::Ant).unwrap();
    let pet_from_constructor =
        Pet::new(PetName::Ant, Some(Statistics::new(2, 1).unwrap()), 1).unwrap();

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
        Statistics::new(50, 50).unwrap(),
        &[Effect::new(
            TRIGGER_START_BATTLE,
            Target::Friend,
            Position::Adjacent,
            Action::Gain(GainType::DefaultItem(FoodName::Melon)),
            Some(1),
            false,
        )],
    );
    let mut team = Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(custom_pet),
            Some(Pet::try_from(PetName::Ant).unwrap()),
        ],
        5,
    )
    .unwrap();
    let mut enemy_team = team.clone();

    // Trigger start of battle effects.
    team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team))
        .unwrap();

    let front_ant = team.friends[0].as_ref().unwrap();
    let behind_ant = team.friends[2].as_ref().unwrap();
    assert!(
        front_ant.read().unwrap().item.as_ref().unwrap().name == FoodName::Melon
            && behind_ant.read().unwrap().item.as_ref().unwrap().name == FoodName::Melon
    )
}

#[test]
fn test_create_team() {
    let mut team = Team::default();
    let dog = Pet::try_from(PetName::Dog).unwrap();
    // Add 5 dogs.
    team.add_pet(dog.clone(), 0, None)
        .unwrap()
        .add_pet(dog.clone(), 0, None)
        .unwrap()
        .add_pet(dog.clone(), 0, None)
        .unwrap()
        .add_pet(dog.clone(), 0, None)
        .unwrap()
        .add_pet(dog, 0, None)
        .unwrap()
        // Give the 2nd pet behind current pet Garlic.
        .set_item(
            &Position::Relative(-2),
            Food::try_from(FoodName::Garlic).ok(),
        )
        .unwrap();

    assert_eq!(team.friends.len(), 5)
}

#[test]
fn test_custom_shop() {
    let mut team = Team::default();
    let mut tier_5_shop = Shop::new(3, Some(42)).unwrap();
    let new_shop_item = ShopItem::new(Food::try_from(FoodName::Coconut).unwrap());
    tier_5_shop.add_item(new_shop_item).unwrap();
    assert!(team.replace_shop(tier_5_shop).is_ok());
}

#[test]
fn test_team_shop() {
    // All teams are constructed with a shop at tier 1.
    let mut team = Team::default();

    // All shop functionality is supported.
    team.set_shop_seed(Some(1212))
        .open_shop()
        .unwrap()
        .buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap()
        .sell(&Position::First)
        .unwrap()
        .freeze_shop(&Position::Last, &Entity::Pet)
        .unwrap()
        .roll_shop()
        .unwrap()
        .close_shop()
        .unwrap();
}

#[test]
fn test_multithreaded_team_battle() {
    let n_threads = 12;
    let mut children = vec![];

    // Spawn 12 threads.
    for _ in 0..n_threads {
        // In each child thread, complete one battle.
        children.push(thread::spawn(|| {
            let mut team_1 = Team::new(
                &vec![
                    Some(Pet::try_from(PetName::Ant).unwrap()),
                    Some(Pet::try_from(PetName::Ant).unwrap()),
                    Some(Pet::try_from(PetName::Ant).unwrap()),
                    Some(Pet::try_from(PetName::Ant).unwrap()),
                    Some(Pet::try_from(PetName::Ant).unwrap()),
                ],
                5,
            )
            .unwrap();
            let mut team_2 = team_1.clone();

            let mut outcome = team_1.fight(&mut team_2).unwrap();
            while let TeamFightOutcome::None = outcome {
                outcome = team_1.fight(&mut team_2).unwrap();
            }
            outcome
        }))
    }

    let mut thread_team_1_outcomes = vec![];
    for child_thread in children.into_iter() {
        let outcome = child_thread.join();
        // Good outcome?
        assert!(outcome.is_ok());

        thread_team_1_outcomes.push(outcome.unwrap())
    }
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
    let mut team = Team::new(&vec![Some(mosquito.clone()); 5], 5).unwrap();
    let mut enemy_team = Team::new(&vec![Some(mosquito); 5], 5).unwrap();
    team.set_seed(Some(0));

    // Without a reference to the pet owning the effect, this will fail.
    assert!(team
        .apply_effect(
            &TRIGGER_START_BATTLE,
            &no_ref_mosquito_effect,
            Some(&mut enemy_team)
        )
        .is_err());

    // Get mosquito_effect with reference.
    // Apply effect of mosquito at position 0 to a pet on team to enemy team.
    let mosquito = team.friends[0].as_ref().unwrap();
    let mosquito_effect = mosquito.read().unwrap().effect[0].clone();
    team.apply_effect(
        &TRIGGER_START_BATTLE,
        &mosquito_effect,
        Some(&mut enemy_team),
    )
    .unwrap();

    // Last enemy mosquito takes one damage and opponent triggers gets updated.
    assert_eq!(
        enemy_team.friends[1]
            .as_ref()
            .unwrap()
            .read()
            .unwrap()
            .stats,
        Statistics::new(2, 1).unwrap()
    );
    assert!(enemy_team
        .triggers
        .iter()
        .any(|trigger| trigger.status == Status::Hurt));
}

#[test]
fn test_serialize_pet() {
    let mut pet = Pet::try_from(PetName::Ant).unwrap();
    pet.seed = Some(20);

    let json_pet = serde_json::to_string(&pet).unwrap();
    let exp_json = r#"{"id":null,"name":"Ant","tier":1,"stats":{"attack":2,"health":1},"effect":[{"trigger":{"status":"Faint","affected_team":"Friend","afflicting_team":"None","position":"OnSelf","stat_diff":null},"target":"Friend","position":{"Any":"None"},"action":{"Add":{"Static":{"attack":2,"health":1}}},"uses":1,"temp":false}],"item":null,"seed":20,"cost":3,"lvl":1,"exp":0,"pos":null,"team":null}"#;
    assert_eq!(json_pet, exp_json);

    // Restore pet from json string.
    let new_pet: Pet = serde_json::from_str(&json_pet).unwrap();
    assert_eq!(pet, new_pet)
}

#[test]
fn test_serialize_team() {
    let seed = 20;
    let name = Team::get_random_name(seed).unwrap();
    // Create a team.
    let mut team = Team::new(
        &[
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
        ],
        5,
    )
    .unwrap();
    team.set_seed(Some(seed)).set_name(&name).unwrap();

    let json_team: String = (&team).try_into().unwrap();
    let exp_json = r#"{"seed":20,"name":"The Wavy Monks","friends":[{"id":"Mosquito_0","name":"Mosquito","tier":1,"stats":{"attack":2,"health":2},"effect":[{"trigger":{"status":"StartOfBattle","affected_team":"None","afflicting_team":"None","position":"None","stat_diff":null},"target":"Enemy","position":{"N":{"condition":"None","targets":1,"random":true}},"action":{"Remove":{"Static":{"attack":1,"health":0}}},"uses":1,"temp":false}],"item":null,"seed":20,"cost":3,"lvl":1,"exp":0,"pos":0,"team":"The Wavy Monks"},{"id":"Mosquito_1","name":"Mosquito","tier":1,"stats":{"attack":2,"health":2},"effect":[{"trigger":{"status":"StartOfBattle","affected_team":"None","afflicting_team":"None","position":"None","stat_diff":null},"target":"Enemy","position":{"N":{"condition":"None","targets":1,"random":true}},"action":{"Remove":{"Static":{"attack":1,"health":0}}},"uses":1,"temp":false}],"item":null,"seed":20,"cost":3,"lvl":1,"exp":0,"pos":1,"team":"The Wavy Monks"},{"id":"Mosquito_2","name":"Mosquito","tier":1,"stats":{"attack":2,"health":2},"effect":[{"trigger":{"status":"StartOfBattle","affected_team":"None","afflicting_team":"None","position":"None","stat_diff":null},"target":"Enemy","position":{"N":{"condition":"None","targets":1,"random":true}},"action":{"Remove":{"Static":{"attack":1,"health":0}}},"uses":1,"temp":false}],"item":null,"seed":20,"cost":3,"lvl":1,"exp":0,"pos":2,"team":"The Wavy Monks"},{"id":"Mosquito_3","name":"Mosquito","tier":1,"stats":{"attack":2,"health":2},"effect":[{"trigger":{"status":"StartOfBattle","affected_team":"None","afflicting_team":"None","position":"None","stat_diff":null},"target":"Enemy","position":{"N":{"condition":"None","targets":1,"random":true}},"action":{"Remove":{"Static":{"attack":1,"health":0}}},"uses":1,"temp":false}],"item":null,"seed":20,"cost":3,"lvl":1,"exp":0,"pos":3,"team":"The Wavy Monks"}],"fainted":[],"sold":[],"max_size":5,"triggers":[],"stored_friends":[{"id":"Mosquito_0","name":"Mosquito","tier":1,"stats":{"attack":2,"health":2},"effect":[{"trigger":{"status":"StartOfBattle","affected_team":"None","afflicting_team":"None","position":"None","stat_diff":null},"target":"Enemy","position":{"N":{"condition":"None","targets":1,"random":true}},"action":{"Remove":{"Static":{"attack":1,"health":0}}},"uses":1,"temp":false}],"item":null,"seed":20,"cost":3,"lvl":1,"exp":0,"pos":0,"team":"The Wavy Monks"},{"id":"Mosquito_1","name":"Mosquito","tier":1,"stats":{"attack":2,"health":2},"effect":[{"trigger":{"status":"StartOfBattle","affected_team":"None","afflicting_team":"None","position":"None","stat_diff":null},"target":"Enemy","position":{"N":{"condition":"None","targets":1,"random":true}},"action":{"Remove":{"Static":{"attack":1,"health":0}}},"uses":1,"temp":false}],"item":null,"seed":20,"cost":3,"lvl":1,"exp":0,"pos":1,"team":"The Wavy Monks"},{"id":"Mosquito_2","name":"Mosquito","tier":1,"stats":{"attack":2,"health":2},"effect":[{"trigger":{"status":"StartOfBattle","affected_team":"None","afflicting_team":"None","position":"None","stat_diff":null},"target":"Enemy","position":{"N":{"condition":"None","targets":1,"random":true}},"action":{"Remove":{"Static":{"attack":1,"health":0}}},"uses":1,"temp":false}],"item":null,"seed":20,"cost":3,"lvl":1,"exp":0,"pos":2,"team":"The Wavy Monks"},{"id":"Mosquito_3","name":"Mosquito","tier":1,"stats":{"attack":2,"health":2},"effect":[{"trigger":{"status":"StartOfBattle","affected_team":"None","afflicting_team":"None","position":"None","stat_diff":null},"target":"Enemy","position":{"N":{"condition":"None","targets":1,"random":true}},"action":{"Remove":{"Static":{"attack":1,"health":0}}},"uses":1,"temp":false}],"item":null,"seed":20,"cost":3,"lvl":1,"exp":0,"pos":3,"team":"The Wavy Monks"}],"persistent_effects":[{"trigger":{"status":{"IsTeam":{"NumberPetsEqual":0}},"affected_team":"Friend","afflicting_team":"Enemy","position":"None","stat_diff":null},"target":"Friend","position":"First","action":{"Conditional":[{"If":{"Shop":{"InState":"Closed"}}},{"Conditional":[{"IfNot":{"Team":["Friend",{"CounterEqual":["Trumpets",0]}]}},{"Multiple":[{"Summon":{"CustomPet":["GoldenRetriever",{"TeamCounter":"Trumpets"},1]}},{"AddToCounter":["Trumpets",-50]}]},"None"]},"None"]},"uses":1,"temp":true},{"trigger":{"status":{"IsTeam":{"NumberPetsEqual":1}},"affected_team":"Friend","afflicting_team":"Enemy","position":"None","stat_diff":null},"target":"Friend","position":"First","action":{"Conditional":[{"If":{"Shop":{"InState":"Closed"}}},{"Conditional":[{"IfNot":{"Team":["Friend",{"CounterEqual":["Trumpets",0]}]}},{"Multiple":[{"Summon":{"CustomPet":["GoldenRetriever",{"TeamCounter":"Trumpets"},1]}},{"AddToCounter":["Trumpets",-50]}]},"None"]},"None"]},"uses":1,"temp":true}],"counters":{"Trumpets":0}}"#;
    assert_eq!(exp_json, json_team);

    let new_team = Team::from_str(&json_team).unwrap();

    // Note this creates a clone of the pets in a new rc ptr. They aren't equivalent.
    assert_ne!(new_team, team)
}

#[test]
fn test_battle_to_tsv() {
    let mosquito = Pet::try_from(PetName::Mosquito).unwrap();

    // Init teams.
    let mut team = Team::new(&vec![Some(mosquito.clone()); 5], 5).unwrap();
    let mut enemy_team = Team::new(&vec![Some(mosquito); 5], 5).unwrap();
    team.set_seed(Some(99));
    enemy_team.set_seed(Some(100));

    team.fight(&mut enemy_team).unwrap();

    let tsv = saptest::create_battle_df(&team);

    let out = std::path::Path::new("test_battle_mosq.tsv");
    std::fs::write(out, tsv).unwrap();

    // File created.
    assert!(out.try_exists().unwrap());
    // Cleanup.
    std::fs::remove_file(out).unwrap();
}
