use crate::{
    battle::{
        effect::Entity,
        state::{Action, CopyAttr, Position, Statistics, Target, TeamFightOutcome},
        team_effect_apply::EffectApply,
        trigger::{TRIGGER_SELF_FAINT, TRIGGER_START_TURN},
    },
    foods::names::FoodName,
    pets::names::PetName,
    tests::common::{
        count_pets, test_ant_team, test_deer_team, test_doberman_highest_tier_team,
        test_hippo_team, test_ox_team, test_parrot_team, test_rooster_team, test_skunk_team,
        test_turtle_team, test_whale_team,
    },
    Effect, Pet,
};

use super::common::{test_armadillo_team, test_doberman_team, test_lynx_team};

// use crate::LOG_CONFIG;

#[test]
fn test_battle_deer_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_deer_team();
    let mut enemy_team = test_ox_team();

    // Only one deer.
    assert!(team.first().unwrap().name == PetName::Deer && team.all().len() == 1,);
    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team)
    }
    // 1st attack kills dear and summons bus.
    // 2nd attack kills dog and ox before its effect triggers.
    // After completion, only bus remains with 2 health.
    let bus = team.any().unwrap();
    assert!(bus.name == PetName::Bus && bus.stats.health == 2 && team.all().len() == 1)
}

#[test]
fn test_battle_hippo_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_hippo_team();
    let mut enemy_team = test_ant_team();

    // Only one lvl.1 hippo.
    let hippo = team.first().unwrap();
    assert!(
        hippo.name == PetName::Hippo
            && hippo.lvl == 1
            && hippo.stats
                == Statistics {
                    attack: 4,
                    health: 5
                }
            && team.all().len() == 1,
    );
    // Versus three lvl.1 ants.
    assert!(enemy_team.all().iter().all(|pet| pet.name == PetName::Ant));
    let mut fight = team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team)
    }

    // Hippo takes 2 dmg (1st) + 8 dmg (2nd + 3rd) = 10 dmg
    // Hippo kills all three gaining (9,9) + base (4,5) - dmg (0,10) to (13,4)
    assert!(
        team.first().unwrap().stats
            == Statistics {
                attack: 13,
                health: 4
            }
    );
}

#[test]
fn test_battle_parrot_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_parrot_team();
    let mut enemy_team = test_ant_team();

    // Before start of turn, is def parrot effect.
    assert_eq!(
        vec![Effect {
            owner_idx: None,
            entity: Entity::Pet,
            trigger: TRIGGER_START_TURN,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Copy(
                CopyAttr::Effect(vec![], Some(1)),
                Target::Friend,
                Position::Relative(1),
            ),
            uses: None,
            temp: true,
        }],
        team.nth(1).unwrap().effect
    );
    // Cricket is level 2.
    assert_eq!(team.first().unwrap().lvl, 2);
    team.fight(&mut enemy_team);

    // After the parrot's effects is a level one cricket.
    // Update id and idx to match.
    let mut updated_trigger = TRIGGER_SELF_FAINT;
    let mut zombie_cricket = Pet::from(PetName::ZombieCricket);
    updated_trigger.to_idx = Some(1);
    zombie_cricket.id = None;

    assert_eq!(
        vec![Effect {
            owner_idx: None,
            trigger: updated_trigger,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Summon(Some(Box::new(zombie_cricket)), None),
            uses: Some(1),
            entity: Entity::Pet,
            temp: false,
        }],
        team.nth(1).unwrap().effect
    );
}

#[test]
fn test_battle_rooster_lvl_1_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_rooster_team();
    let mut enemy_team = test_rooster_team();
    {
        let rooster = team.first().unwrap();
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

    let chick = team.first().unwrap();
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

    let mut team = test_rooster_team();
    let mut enemy_team = test_rooster_team();
    {
        let rooster = team.first().unwrap();
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

    let mut team = test_skunk_team();
    let mut enemy_team = test_skunk_team();

    // Lvl. 1 skunks on both teams.
    assert!(team.first().unwrap().lvl == 1 && enemy_team.first().unwrap().lvl == 1,);
    assert_eq!(
        team.first().unwrap().stats,
        Statistics {
            attack: 3,
            health: 5
        }
    );
    assert_eq!(
        enemy_team.first().unwrap().stats,
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
        team.first().unwrap().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
    assert_eq!(
        enemy_team.first().unwrap().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
}

#[test]
fn test_battle_turtle_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_turtle_team();
    let mut enemy_team = test_turtle_team();

    assert_eq!(team.nth(1).unwrap().item, None);
    assert_eq!(enemy_team.nth(1).unwrap().item, None);

    // Three attacks to kill both lvl.1 turtles.
    for _ in 0..3 {
        team.fight(&mut enemy_team);
    }

    assert_eq!(
        team.first().unwrap().item.as_ref().unwrap().name,
        FoodName::Melon
    );
    assert_eq!(
        enemy_team.first().unwrap().item.as_ref().unwrap().name,
        FoodName::Melon
    )
}

#[test]
fn test_battle_whale_team() {
    // log4rs::init_file(LOG_CONFIG, Default::default()).unwrap();

    let mut team = test_whale_team();
    let mut enemy_team = test_hippo_team();

    // Only one cricket at start.
    assert_eq!(count_pets(&team.friends, PetName::Cricket), 1);
    // Copy cricket for comparison
    let cricket_copy = team.friends.first().unwrap().clone().unwrap();

    let mut outcome = team.fight(&mut enemy_team);

    // After start of battle, what eats cricket and changes effect to summon cricket.
    let whale_effect = team.first().unwrap().effect.first().unwrap();
    assert_eq!(
        whale_effect.action,
        Action::Summon(Some(Box::new(cricket_copy)), None)
    );

    // Finish fight.
    while let TeamFightOutcome::None = outcome {
        outcome = team.fight(&mut enemy_team);
    }

    // Two dead crickets on team.
    let n_crickets: usize = count_pets(&team.fainted, PetName::Cricket);

    assert_eq!(2, n_crickets)
}

#[test]
fn test_battle_armadillo_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_armadillo_team();
    let mut enemy_team = test_hippo_team();

    team.fight(&mut enemy_team);

    for (i, pet) in team.all().into_iter().enumerate() {
        // First pet is armadillo, it takes (2,6)-(0,4).
        // It doesn't gain (0,1) but all dogs do.
        if i == 0 {
            assert_eq!(pet.stats, Statistics::new(2, 2).unwrap())
        } else {
            assert_eq!(pet.stats, Statistics::new(3, 5).unwrap())
        }
    }
}

#[test]
fn test_battle_doberman_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_doberman_team();
    let mut enemy_team = test_hippo_team();

    // Doberman has no item.
    assert_eq!(team.first().unwrap().item, None);
    assert_eq!(team.first().unwrap().stats, Statistics::new(4, 5).unwrap());
    // Doberman is lowest tier.
    assert_eq!(
        team.all()
            .iter()
            .min_by(|pet_1, pet_2| pet_1.tier.cmp(&pet_2.tier))
            .unwrap()
            .name,
        PetName::Doberman
    );
    team.fight(&mut enemy_team);

    // Doberman gets coconut and gets (5,5)
    assert_eq!(
        team.first().unwrap().item.as_ref().unwrap().name,
        FoodName::Coconut
    );
    assert_eq!(team.first().unwrap().stats, Statistics::new(9, 10).unwrap());
}

#[test]
fn test_battle_doberman_highest_tier_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_doberman_highest_tier_team();
    let mut enemy_team = test_hippo_team();

    // Doberman has no item.
    assert_eq!(team.first().unwrap().item, None);
    assert_eq!(team.first().unwrap().stats, Statistics::new(4, 5).unwrap());
    // Doberman is not lowest tier.
    assert_ne!(
        team.all()
            .iter()
            .min_by(|pet_1, pet_2| pet_1.tier.cmp(&pet_2.tier))
            .unwrap()
            .name,
        PetName::Doberman
    );
    team.fight(&mut enemy_team);

    // Doberman doesn't get coconut or stats.
    assert_eq!(team.first().unwrap().item, None);
    assert_eq!(team.first().unwrap().stats, Statistics::new(4, 1).unwrap());
}

#[test]
fn test_battle_lynx_team() {
    log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_lynx_team();
    let mut enemy_team = test_hippo_team();

    // 5 levels on team. So 5 dmg.
    assert_eq!(team.all().iter().map(|pet| pet.lvl).sum::<usize>(), 5);

    team.fight(&mut enemy_team);

    // Hippo faints at start of battle.
    assert_eq!(
        enemy_team.fainted.first().unwrap().as_ref().unwrap().stats,
        Statistics::new(4, 0).unwrap()
    );

    team.restore();
    enemy_team.restore();

    // Remove one level one pet.
    team.friends.pop();
    assert_eq!(team.all().iter().map(|pet| pet.lvl).sum::<usize>(), 4);

    // Retrigger start of battle effects
    team.trigger_effects(&mut enemy_team);

    // Hippo takes 4 dmg.
    assert_eq!(
        enemy_team.first().unwrap().stats,
        Statistics::new(4, 1).unwrap()
    );
}
