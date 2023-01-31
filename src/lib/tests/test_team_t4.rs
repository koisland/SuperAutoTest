use itertools::Itertools;

use crate::{
    battle::{
        effect::Entity,
        state::{Action, CopyAttr, Position, Statistics, Status, Target, TeamFightOutcome},
        team_effect_apply::EffectApply,
        trigger::TRIGGER_SELF_FAINT,
    },
    foods::names::FoodName,
    pets::names::PetName,
    tests::common::{
        count_pets, test_ant_team, test_anteater_team, test_armadillo_team, test_caterpillar_team,
        test_deer_team, test_doberman_highest_tier_team, test_doberman_team, test_donkey_team,
        test_eel_team, test_gorilla_team, test_hawk_team, test_hippo_team, test_lynx_team,
        test_mosq_team, test_ox_team, test_parrot_team, test_pelican_team, test_porcupine_team,
        test_rooster_team, test_skunk_team, test_snake_team, test_turtle_team, test_whale_team,
    },
    Effect, Outcome, Pet,
};

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
            owner_target: None,
            owner_idx: Some(1),
            entity: Entity::Pet,
            trigger: Outcome {
                from_target: Target::None,
                status: Status::StartTurn,
                to_target: Target::None,
                position: Position::None,
                to_idx: Some(1),
                from_idx: None,
                stat_diff: None,
            },
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
    let mut zombie_cricket = Pet::try_from(PetName::ZombieCricket).unwrap();
    updated_trigger.to_idx = Some(1);
    zombie_cricket.id = None;

    assert_eq!(
        vec![Effect {
            owner_target: None,
            owner_idx: Some(1),
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
    // Copy cricket for comparison and set idx of trigger + owner of effect to None. (Reset on swallow)
    let mut cricket_copy = team.friends.first().unwrap().clone().unwrap();
    if let Some(effect) = cricket_copy.effect.first_mut() {
        effect.owner_idx = None;
        effect.trigger.to_idx = None
    }

    let mut outcome = team.fight(&mut enemy_team);

    // After start of battle, whale eats cricket and changes effect to summon cricket.
    let whale_effect = team.first().unwrap().effect.first_mut().unwrap();
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
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

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

#[test]
fn test_battle_porcupine_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_porcupine_team();
    let mut enemy_team = test_mosq_team();

    // Buff 1st mosquito so survives first returned attack.
    enemy_team.first().unwrap().stats.health += 8;

    // Trigger start of battle effects. Then clear fainted pets.
    enemy_team.trigger_effects(&mut team);
    team.trigger_effects(&mut enemy_team);
    enemy_team.clear_team();

    // 2 Mosquitoes faint from returned fire from porcupine
    assert_eq!(
        enemy_team
            .fainted
            .iter()
            .filter_map(|slot| slot.as_ref())
            .collect_vec()
            .len(),
        2
    );
    // 1 mosquito that was buffed survives.
    assert!(
        enemy_team.all().len() == 1
            && enemy_team.first().unwrap().stats == Statistics::new(2, 4).unwrap()
    );

    // Continue fight.
    team.fight(&mut enemy_team);

    // 1st mosquito faints from direct damage + returned porcupine damage.
    assert_eq!(
        enemy_team
            .fainted
            .iter()
            .filter_map(|slot| slot.as_ref())
            .collect_vec()
            .len(),
        3
    );
}

#[test]
fn test_battle_caterpillar_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_caterpillar_team();
    let mut enemy_team = test_hippo_team();
    {
        let caterpillar = team.first().unwrap();
        assert_eq!(caterpillar.stats, Statistics::new(2, 2).unwrap());
        assert_eq!(caterpillar.name, PetName::Caterpillar)
    }
    // Trigger start of battle effects.
    // Copy does not trigger yet.
    team.trigger_effects(&mut enemy_team);
    {
        let butterfly = team.first().unwrap();
        assert_eq!(butterfly.stats, Statistics::new(1, 1).unwrap());
        assert_eq!(butterfly.name, PetName::Butterfly)
    }
    // Right before battle phase, butterfly will copy effect.
    team.fight(&mut enemy_team);

    // Butterfly takes 4 dmg from hippo but copied (50/50) dog.
    {
        let butterfly = team.first().unwrap();
        assert_eq!(butterfly.stats, Statistics::new(50, 46).unwrap());
        assert_eq!(butterfly.name, PetName::Butterfly)
    }
}

#[test]
fn test_battle_sniped_caterpillar_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_caterpillar_team();
    let mut enemy_team = test_mosq_team();
    enemy_team.friends.remove(2);
    enemy_team.friends.remove(1);
    team.set_seed(42);

    let outcome = team.fight(&mut enemy_team);

    // The team wins.
    assert_eq!(outcome, TeamFightOutcome::Win);
    // But the butterfly faints due to snipe from single mosquito on enemy team.
    assert_eq!(
        team.fainted.first().unwrap().as_ref().unwrap().name,
        PetName::Butterfly
    )
}

#[test]
fn test_battle_anteater_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_anteater_team();
    let mut enemy_team = test_hippo_team();

    // Single anteater.
    assert_eq!(team.all().len(), 1);
    team.fight(&mut enemy_team);

    // After faint, two anteaters spawn.
    assert_eq!(
        team.fainted.first().unwrap().as_ref().unwrap().name,
        PetName::Anteater
    );
    let all_friends = team.all();
    assert_eq!(all_friends.len(), 2);
    assert!(all_friends.iter().all(|pet| pet.name == PetName::Ant))
}

#[test]
fn test_battle_donkey_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_donkey_team();
    let mut enemy_team = test_snake_team();
    team.set_seed(2);

    assert_eq!(enemy_team.nth(1).unwrap().name, PetName::Snake);

    team.fight(&mut enemy_team);

    // Cricket faints and donkey ability triggers.
    assert_eq!(enemy_team.fainted.len(), 1);
    // Snake pushed to front.
    assert_eq!(enemy_team.first().unwrap().name, PetName::Snake);
    // And zombie cricket now in back.
    assert_eq!(enemy_team.nth(1).unwrap().name, PetName::ZombieCricket)
}

#[test]
fn test_battle_eel_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_eel_team();
    let mut enemy_team = test_hippo_team();

    let eel_stats = team.first().unwrap().stats;

    team.trigger_effects(&mut enemy_team);

    // Eel at lvl.1 gains 50% of original health.
    assert_eq!(
        eel_stats + Statistics::new(0, eel_stats.health / 2).unwrap(),
        team.first().unwrap().stats
    )
}

#[test]
fn test_battle_hawk_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_hawk_team();
    let mut enemy_team = test_gorilla_team();

    // Hawk on 1st position.
    assert_eq!(team.first().unwrap().name, PetName::Hawk);
    {
        let gorilla_on_1st = enemy_team.first().unwrap();
        assert_eq!(
            gorilla_on_1st.stats,
            Statistics {
                attack: 6,
                health: 9
            }
        );
    }

    team.trigger_effects(&mut enemy_team);

    {
        // Gorilla takes 7 dmg.
        let gorilla_on_1st = enemy_team.first().unwrap();
        assert_eq!(
            gorilla_on_1st.stats,
            Statistics {
                attack: 6,
                health: 2
            }
        );
    }
}

#[test]
fn test_battle_pelican_team() {
    // log4rs::init_file("./config/log_config.yaml", Default::default()).unwrap();

    let mut team = test_pelican_team();
    let mut enemy_team = test_hippo_team();

    {
        // Ant has strawberry.
        let strawberry_ant = team.nth(1).unwrap();

        assert_eq!(strawberry_ant.stats, Statistics::new(2, 1).unwrap());
        assert_eq!(
            strawberry_ant.item.as_ref().unwrap().name,
            FoodName::Strawberry
        )
    }

    team.trigger_effects(&mut enemy_team);

    // Pelican at lvl.1 give strawberry ant (2,1)
    assert_eq!(team.nth(1).unwrap().stats, Statistics::new(4, 2).unwrap());
}
