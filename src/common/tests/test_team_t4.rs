use crate::common::{
    battle::{
        effect::{Effect, EffectType},
        state::{Action, Position, Statistics, Target, TeamFightOutcome},
        team_effect_apply::EffectApply,
        trigger::TRIGGER_SELF_FAINT,
    },
    foods::names::FoodName,
    pets::{effects::get_pet_effect, names::PetName, pet::Pet},
    tests::common::{
        test_ant_team, test_deer_team, test_hippo_team, test_ox_team, test_parrot_team,
        test_rooster_team, test_skunk_team, test_turtle_team, test_whale_team,
    },
};

use crate::LOG_CONFIG;

#[test]
fn test_battle_deer_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_deer_team("self");
    let mut enemy_team = test_ox_team("enemy");

    // Only one deer.
    assert!(team.get_next_pet().unwrap().name == PetName::Deer && team.get_all_pets().len() == 1,);
    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team)
    }
    // 1st attack kills dear and summons bus.
    // 2nd attack kills dog and ox before its effect triggers.
    // After completion, only bus remains with 2 health.
    let bus = team.get_any_pet().unwrap();
    assert!(bus.name == PetName::Bus && bus.stats.health == 2 && team.get_all_pets().len() == 1)
}

#[test]
fn test_battle_hippo_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_hippo_team("self");
    let mut enemy_team = test_ant_team("enemy");

    // Only one lvl.1 hippo.
    let hippo = team.get_next_pet().unwrap();
    assert!(
        hippo.name == PetName::Hippo
            && hippo.lvl == 1
            && hippo.stats
                == Statistics {
                    attack: 4,
                    health: 5
                }
            && team.get_all_pets().len() == 1,
    );
    // Versus three lvl.1 ants.
    assert!(enemy_team
        .get_all_pets()
        .iter()
        .all(|pet| pet.name == PetName::Ant));
    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team)
    }

    // Hippo takes 2 dmg (1st) + 8 dmg (2nd + 3rd) = 10 dmg
    // Hippo kills all three gaining (9,9) + base (4,5) - dmg (0,10) to (13,4)
    assert!(
        team.get_next_pet().unwrap().stats
            == Statistics {
                attack: 13,
                health: 4
            }
    );
}

#[test]
fn test_battle_parrot_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_parrot_team("self");
    let mut enemy_team = test_ant_team("enemy");

    // Before start of turn, is def parrot effect.
    assert_eq!(
        get_pet_effect(
            &PetName::Parrot,
            // Pet stats field. Not used.
            &Statistics::default(),
            // Effect stats
            Statistics::default(),
            // lvl
            1,
            // n_triggers
            1
        ),
        team.get_idx_pet(1).unwrap().effect
    );
    // Cricket is level 2.
    assert_eq!(team.get_next_pet().unwrap().lvl, 2);
    team.fight(&mut enemy_team);

    // After the parrot's effects is a level one cricket.
    assert_eq!(
        get_pet_effect(
            &PetName::Cricket,
            // Pet stats field. Not used.
            &Statistics::default(),
            // Effect stats
            Statistics {
                attack: 1,
                health: 1
            },
            // lvl
            1,
            // n_triggers
            1
        ),
        team.get_idx_pet(1).unwrap().effect
    );
}

#[test]
fn test_battle_rooster_lvl_1_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_rooster_team("self");
    let mut enemy_team = test_rooster_team("enemy");
    {
        let rooster = team.get_next_pet().unwrap();
        assert!(
            rooster.name == PetName::Rooster
                && rooster.stats
                    == Statistics {
                        attack: 5,
                        health: 3
                    }
        )
    }

    team.fight(&mut enemy_team);

    let chick = team.get_next_pet().unwrap();
    // 50% of base lvl.1 rooster is 3 (2.5). Health is 1.
    assert!(
        chick.name == PetName::Chick
            && chick.stats
                == Statistics {
                    attack: 3,
                    health: 1
                }
    )
}

#[test]
fn test_battle_rooster_lvl_2_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_rooster_team("self");
    let mut enemy_team = test_rooster_team("enemy");
    {
        let rooster = team.get_next_pet().unwrap();
        rooster.set_level(2).unwrap();
        // Level 2 now. Will spawn 2 chicks.
        assert!(
            rooster.name == PetName::Rooster
                && rooster.stats
                    == Statistics {
                        attack: 5,
                        health: 3
                    }
                && rooster.lvl == 2
        )
    }

    team.fight(&mut enemy_team);

    let chick = team.friends.first().unwrap().as_ref().unwrap();
    let chick_2 = team.friends.get(1).unwrap().as_ref().unwrap();
    // 50% of base lvl.1 rooster is 3 (2.5). Health is 1.
    assert!(
        chick.name == PetName::Chick
            && chick.stats
                == Statistics {
                    attack: 3,
                    health: 1
                }
    );
    assert!(
        chick_2.name == PetName::Chick
            && chick_2.stats
                == Statistics {
                    attack: 3,
                    health: 1
                }
    )
}

#[test]
fn test_battle_skunk_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_skunk_team("self");
    let mut enemy_team = test_skunk_team("enemy");

    // Lvl. 1 skunks on both teams.
    assert!(team.get_next_pet().unwrap().lvl == 1 && enemy_team.get_next_pet().unwrap().lvl == 1,);
    assert_eq!(
        team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 3,
            health: 5
        }
    );
    assert_eq!(
        enemy_team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 3,
            health: 5
        }
    );

    // Apply start of battle effects. No fighting.
    team.trigger_effects(&mut enemy_team);
    enemy_team.trigger_effects(&mut team);

    // Health reduced by 33% (2) from 5 -> 3.
    assert_eq!(
        team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
    assert_eq!(
        enemy_team.get_next_pet().unwrap().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
}

#[test]
fn test_battle_turtle_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_turtle_team("self");
    let mut enemy_team = test_turtle_team("enemy");

    assert_eq!(team.get_idx_pet(1).unwrap().item, None);
    assert_eq!(enemy_team.get_idx_pet(1).unwrap().item, None);

    // Three attacks to kill both lvl.1 turtles.
    for _ in 0..3 {
        team.fight(&mut enemy_team);
    }

    assert_eq!(
        team.get_next_pet().unwrap().item.as_ref().unwrap().name,
        FoodName::Melon
    );
    assert_eq!(
        enemy_team
            .get_next_pet()
            .unwrap()
            .item
            .as_ref()
            .unwrap()
            .name,
        FoodName::Melon
    )
}

#[test]
fn test_battle_whale_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_whale_team("self");
    let mut enemy_team = test_hippo_team("enemy");

    let count_crickets = |friends: &[Option<Pet>]| {
        friends
            .iter()
            .filter_map(|pet| {
                if let Some(pet) = pet {
                    (pet.name == PetName::Cricket).then_some(1)
                } else {
                    None
                }
            })
            .sum::<usize>()
    };

    // Only one cricket at start.
    assert_eq!(count_crickets(&team.friends), 1);

    let mut outcome = team.fight(&mut enemy_team);

    // Recreate effect of whale after swallowing cricket.
    let mut trigger_faint = TRIGGER_SELF_FAINT;
    let mut cricket = Pet::from(PetName::Cricket);
    cricket.pos = Some(0);
    trigger_faint.idx = Some(0);

    let new_whale_effect = Effect {
        effect_type: EffectType::Pet,
        trigger: trigger_faint,
        target: Target::Friend,
        position: Position::OnSelf,
        action: Action::Summon(Some(Box::new(cricket))),
        uses: Some(1),
    };

    assert_eq!(team.get_next_pet().unwrap().effect, Some(new_whale_effect));

    // Finish fight.
    while let TeamFightOutcome::None = outcome {
        outcome = team.fight(&mut enemy_team);
    }

    // Two dead crickets on team.
    let n_crickets: usize = count_crickets(&team.fainted);

    assert_eq!(2, n_crickets)
}
