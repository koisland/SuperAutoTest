use std::rc::Rc;

use crate::{
    effects::{
        actions::{Action, CopyType, SummonType},
        effect::Entity,
        state::{Position, Target},
        stats::Statistics,
        trigger::*,
    },
    foods::names::FoodName,
    pets::names::PetName,
    teams::{
        combat::{ClearOption, TeamCombat},
        effects::TeamEffects,
        team::TeamFightOutcome,
        viewer::TeamViewer,
    },
    tests::common::{
        count_pets, test_ant_team, test_anteater_team, test_armadillo_team, test_caterpillar_team,
        test_deer_team, test_doberman_highest_tier_team, test_doberman_team, test_donkey_team,
        test_eel_team, test_gorilla_team, test_hawk_team, test_hippo_team, test_lynx_team,
        test_mosq_team, test_ox_team, test_parrot_team, test_pelican_team, test_porcupine_team,
        test_rooster_team, test_skunk_team, test_snake_team, test_turtle_team, test_whale_team,
    },
    Effect, Pet, TeamShopping,
};

// use crate::LOG_CONFIG;

#[test]
fn test_battle_deer_team() {
    let mut team = test_deer_team();
    let mut enemy_team = test_ox_team();

    // Only one deer.
    assert!(team.first().unwrap().borrow().name == PetName::Deer && team.all().len() == 1);
    let mut fight = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team).unwrap()
    }
    // 1st attack kills dear and summons bus.
    // 2nd attack kills dog and ox before its effect triggers.
    // After completion, only bus remains with 2 health.
    let bus = team.any().unwrap();
    assert!(
        bus.borrow().name == PetName::Bus
            && bus.borrow().stats.health == 2
            && team.all().len() == 1
    )
}

#[test]
fn test_battle_hippo_team() {
    let mut team = test_hippo_team();
    let mut enemy_team = test_ant_team();

    // Only one lvl.1 hippo.
    let hippo = team.first().unwrap();
    assert!(
        hippo.borrow().name == PetName::Hippo
            && hippo.borrow().lvl == 1
            && hippo.borrow().stats
                == Statistics {
                    attack: 4,
                    health: 5
                }
            && team.all().len() == 1,
    );
    // Versus three lvl.1 ants.
    assert!(enemy_team
        .all()
        .iter()
        .all(|pet| pet.borrow().name == PetName::Ant));
    let mut fight = team.fight(&mut enemy_team).unwrap();
    while let TeamFightOutcome::None = fight {
        fight = team.fight(&mut enemy_team).unwrap()
    }

    // Hippo takes 2 dmg (1st) + 8 dmg (2nd + 3rd) = 10 dmg
    // Hippo kills all three gaining (9,9) + base (4,5) - dmg (0,10) to (13,4)
    assert!(
        team.first().unwrap().borrow().stats
            == Statistics {
                attack: 13,
                health: 4
            }
    );
}

#[test]
fn test_shop_parrot_team() {
    let mut team = test_parrot_team();

    // Before end of turn, is def parrot effect.
    let parrot = team.nth(1).unwrap();
    assert_eq!(
        vec![Effect {
            owner: Some(Rc::downgrade(&parrot)),
            entity: Entity::Pet,
            trigger: TRIGGER_END_TURN.clone().set_affected(&parrot).to_owned(),
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Copy(
                CopyType::Effect(vec![], Some(1)),
                Target::Friend,
                Position::Relative(1),
            ),
            uses: None,
            temp: true,
        }],
        team.nth(1).unwrap().borrow().effect
    );
    // Cricket is level 2.
    assert_eq!(team.first().unwrap().borrow().lvl, 2);

    // Open and then close shop. Creates end of turn trigger.
    team.open_shop().unwrap().close_shop().unwrap();

    // After the parrot's effects is a level one cricket.
    // Update id and affected pet to match.
    let mut updated_trigger = TRIGGER_SELF_FAINT;
    let mut zombie_cricket = Pet::try_from(PetName::ZombieCricket).unwrap();
    updated_trigger.set_affected(&team.nth(1).unwrap());
    zombie_cricket.id = None;

    assert_eq!(
        vec![Effect {
            owner: None,
            trigger: updated_trigger,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Summon(SummonType::StoredPet(Box::new(zombie_cricket))),
            uses: Some(1),
            entity: Entity::Pet,
            temp: false,
        }],
        team.nth(1).unwrap().borrow().effect
    );
}

#[test]
fn test_battle_rooster_lvl_1_team() {
    let mut team = test_rooster_team();
    let mut enemy_team = test_rooster_team();
    {
        let rooster = team.first().unwrap();
        assert!(
            rooster.borrow().name == PetName::Rooster
                && rooster.borrow().stats
                    == Statistics {
                        attack: 5,
                        health: 3
                    }
        )
    }

    team.fight(&mut enemy_team).unwrap();

    let chick = team.first().unwrap();
    // 50% of base lvl.1 rooster is 3 (2.5). Health is 1.
    assert!(
        chick.borrow().name == PetName::Chick
            && chick.borrow().stats
                == Statistics {
                    attack: 3,
                    health: 1
                }
    )
}

#[test]
fn test_battle_rooster_lvl_2_team() {
    let mut team = test_rooster_team();
    let mut enemy_team = test_rooster_team();
    {
        team.set_level(&Position::First, 2).unwrap();
        let rooster = team.first().unwrap();
        // Level 2 now. Will spawn 2 chicks.
        assert!(
            rooster.borrow().name == PetName::Rooster
                && rooster.borrow().stats
                    == Statistics {
                        attack: 5,
                        health: 3
                    }
                && rooster.borrow().lvl == 2
        )
    }

    team.fight(&mut enemy_team).unwrap();

    let chick = team.first().unwrap();
    let chick_2 = team.nth(1).unwrap();
    // 50% of base lvl.1 rooster is 3 (2.5). Health is 1.
    assert!(
        chick.borrow().name == PetName::Chick
            && chick.borrow().stats
                == Statistics {
                    attack: 3,
                    health: 1
                }
    );
    assert!(
        chick_2.borrow().name == PetName::Chick
            && chick_2.borrow().stats
                == Statistics {
                    attack: 3,
                    health: 1
                }
    )
}

#[test]
fn test_battle_skunk_team() {
    let mut team = test_skunk_team();
    let mut enemy_team = test_skunk_team();

    // Lvl. 1 skunks on both teams.
    assert!(
        team.first().unwrap().borrow().lvl == 1 && enemy_team.first().unwrap().borrow().lvl == 1,
    );
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 5
        }
    );
    assert_eq!(
        enemy_team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 5
        }
    );

    // Apply start of battle effects. No fighting.
    team.triggers.push_front(TRIGGER_START_BATTLE);
    enemy_team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();
    enemy_team.trigger_effects(Some(&mut team)).unwrap();

    // Health reduced by 33% (2) from 5 -> 3.
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
    assert_eq!(
        enemy_team.first().unwrap().borrow().stats,
        Statistics {
            attack: 3,
            health: 3
        }
    );
}

#[test]
fn test_battle_turtle_team() {
    let mut team = test_turtle_team();
    let mut enemy_team = test_turtle_team();

    assert_eq!(team.nth(1).unwrap().borrow().item, None);

    // Three attacks to kill both lvl.1 turtles.
    for _ in 0..3 {
        team.fight(&mut enemy_team).unwrap();
    }

    let pet_behind_turtle = team.first().unwrap();
    assert_eq!(
        pet_behind_turtle.borrow().item.as_ref().unwrap().name,
        FoodName::Melon
    );
}

#[test]
fn test_battle_whale_team() {
    let mut team = test_whale_team();
    let mut enemy_team = test_hippo_team();

    // Only one cricket at start.
    assert_eq!(count_pets(&team.friends, PetName::Cricket), 1);
    // Copy cricket for comparison and set idx of trigger + owner of effect to None. (Reset on swallow)
    for effect in team.first().unwrap().borrow_mut().effect.iter_mut() {
        effect.assign_owner(None);
    }
    let cricket_copy = team.first().unwrap().borrow().clone();

    team.fight(&mut enemy_team).unwrap();

    // After start of battle, whale eats cricket and changes effect to summon cricket.
    let whale = team.first().unwrap();
    let whale_effect = &whale.borrow().effect;

    assert_eq!(
        whale_effect.first().unwrap().action,
        Action::Summon(SummonType::StoredPet(Box::new(cricket_copy)))
    );
}

#[test]
fn test_battle_armadillo_team() {
    let mut team = test_armadillo_team();
    let mut enemy_team = test_hippo_team();

    team.fight(&mut enemy_team).unwrap();

    for (i, pet) in team.all().into_iter().enumerate() {
        // First pet is armadillo, it takes (2,6)-(0,4).
        // It doesn't gain (0,1) but all dogs do.
        if i == 0 {
            assert_eq!(pet.borrow().stats, Statistics::new(2, 2).unwrap())
        } else {
            assert_eq!(pet.borrow().stats, Statistics::new(3, 5).unwrap())
        }
    }
}

#[test]
fn test_battle_doberman_team() {
    let mut team = test_doberman_team();
    let mut enemy_team = test_hippo_team();

    // Doberman has no item.
    assert_eq!(team.first().unwrap().borrow().item, None);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(4, 5).unwrap()
    );
    // Doberman is lowest tier.
    assert_eq!(
        team.all()
            .iter()
            .min_by(|pet_1, pet_2| pet_1.borrow().tier.cmp(&pet_2.borrow().tier))
            .unwrap()
            .borrow()
            .name,
        PetName::Doberman
    );
    team.fight(&mut enemy_team).unwrap();

    // Doberman gets coconut and gets (5,5)
    let doberman = team.first().unwrap();
    assert_eq!(
        doberman.borrow().item.as_ref().unwrap().name,
        FoodName::Coconut
    );
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(9, 10).unwrap()
    );
}

#[test]
fn test_battle_doberman_highest_tier_team() {
    let mut team = test_doberman_highest_tier_team();
    let mut enemy_team = test_hippo_team();

    // Doberman has no item.
    assert_eq!(team.first().unwrap().borrow().item, None);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(4, 5).unwrap()
    );
    // Doberman is not lowest tier.
    assert_ne!(
        team.all()
            .iter()
            .min_by(|pet_1, pet_2| pet_1.borrow().tier.cmp(&pet_2.borrow().tier))
            .unwrap()
            .borrow()
            .name,
        PetName::Doberman
    );
    team.fight(&mut enemy_team).unwrap();

    // Doberman doesn't get coconut or stats.
    assert_eq!(team.first().unwrap().borrow().item, None);
    assert_eq!(
        team.first().unwrap().borrow().stats,
        Statistics::new(4, 1).unwrap()
    );
}

#[test]
fn test_battle_lynx_team() {
    let mut team = test_lynx_team();
    let mut enemy_team = test_hippo_team();

    let enemy_hippo = enemy_team.first().unwrap();
    // 5 levels on team. So 5 dmg.
    assert_eq!(
        team.all().iter().map(|pet| pet.borrow().lvl).sum::<usize>(),
        5
    );

    team.fight(&mut enemy_team).unwrap();

    // Hippo faints at start of battle.
    assert_eq!(enemy_hippo.borrow().stats, Statistics::new(4, 0).unwrap());

    team.restore();
    enemy_team.restore();

    // Remove one level one pet.
    team.friends.pop();
    assert_eq!(
        team.all().iter().map(|pet| pet.borrow().lvl).sum::<usize>(),
        4
    );

    // Retrigger start of battle effects
    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();

    // Hippo takes 4 dmg.
    assert_eq!(
        enemy_team.first().unwrap().borrow().stats,
        Statistics::new(4, 1).unwrap()
    );
}

#[test]
fn test_battle_porcupine_team() {
    let mut team = test_porcupine_team();
    let mut enemy_team = test_mosq_team();

    // Buff 1st mosquito so survives first returned attack.
    enemy_team.first().unwrap().borrow_mut().stats.health += 8;

    // Trigger start of battle effects. Then clear fainted pets.
    team.triggers.push_front(TRIGGER_START_BATTLE);
    enemy_team.triggers.push_front(TRIGGER_START_BATTLE);
    enemy_team.trigger_effects(Some(&mut team)).unwrap();
    team.trigger_effects(Some(&mut enemy_team)).unwrap();
    enemy_team.clear_team(ClearOption::RemoveSlots);

    // 2 Mosquitoes faint from returned fire from porcupine
    assert_eq!(enemy_team.fainted.len(), 2);
    // 1 mosquito that was buffed survives.
    assert!(
        enemy_team.all().len() == 1
            && enemy_team.first().unwrap().borrow().stats == Statistics::new(2, 4).unwrap()
    );

    // Continue fight.
    team.fight(&mut enemy_team).unwrap();

    // 1st mosquito faints from direct damage + returned porcupine damage.
    assert_eq!(enemy_team.fainted.len(), 3);
}

#[test]
fn test_battle_caterpillar_team() {
    let mut team = test_caterpillar_team();
    let mut enemy_team = test_hippo_team();
    let caterpillar = team.first().unwrap();
    assert_eq!(caterpillar.borrow().stats, Statistics::new(2, 2).unwrap());
    assert_eq!(caterpillar.borrow().name, PetName::Caterpillar);

    // Trigger start of battle effects.
    // Copy does not trigger yet.
    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();

    let butterfly = team.first().unwrap();
    assert_eq!(butterfly.borrow().stats, Statistics::new(1, 1).unwrap());
    assert_eq!(butterfly.borrow().name, PetName::Butterfly);

    // Right before battle phase, butterfly will copy effect.
    team.fight(&mut enemy_team).unwrap();

    // Butterfly takes 4 dmg from hippo but copied (50/50) dog.
    let butterfly = team.first().unwrap();
    assert_eq!(butterfly.borrow().stats, Statistics::new(50, 46).unwrap());
    assert_eq!(butterfly.borrow().name, PetName::Butterfly);
}

#[test]
fn test_battle_sniped_caterpillar_team() {
    let mut team = test_caterpillar_team();
    let mut enemy_team = test_mosq_team();
    enemy_team.friends.remove(2);
    enemy_team.friends.remove(1);
    team.set_seed(Some(42));

    let outcome = team.fight(&mut enemy_team).unwrap();

    // The team wins.
    assert_eq!(outcome, TeamFightOutcome::Win);
    // But the butterfly faints due to snipe from single mosquito on enemy team.
    let first_fainted_pet = &team.fainted.first().unwrap();
    assert_eq!(
        first_fainted_pet.as_ref().unwrap().borrow().name,
        PetName::Butterfly
    );
}

#[test]
fn test_battle_anteater_team() {
    let mut team = test_anteater_team();
    let mut enemy_team = test_hippo_team();

    // Single anteater.
    let anteater = team.first();
    assert_eq!(team.all().len(), 1);
    assert_eq!(anteater.as_ref().unwrap().borrow().name, PetName::Anteater);
    team.fight(&mut enemy_team).unwrap();

    // After faint, two anteaters spawn.
    assert_eq!(team.fainted.first().unwrap(), &anteater);
    let all_friends = team.all();
    assert_eq!(all_friends.len(), 2);
    assert!(all_friends
        .iter()
        .all(|pet| pet.borrow().name == PetName::Ant))
}

#[test]
fn test_battle_donkey_team() {
    let mut team = test_donkey_team();
    let mut enemy_team = test_snake_team();
    team.set_seed(Some(2));

    assert_eq!(enemy_team.nth(1).unwrap().borrow().name, PetName::Snake);

    team.fight(&mut enemy_team).unwrap();

    // Cricket faints and donkey ability triggers.
    assert_eq!(enemy_team.fainted.len(), 1);
    // Snake pushed to front.
    assert_eq!(enemy_team.first().unwrap().borrow().name, PetName::Snake);
    // And zombie cricket now in back.
    assert_eq!(
        enemy_team.nth(1).unwrap().borrow().name,
        PetName::ZombieCricket
    )
}

#[test]
fn test_battle_eel_team() {
    let mut team = test_eel_team();
    let mut enemy_team = test_hippo_team();

    let eel_stats = team.first().unwrap().borrow().stats;

    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();

    // Eel at lvl.1 gains 50% of original health.
    assert_eq!(
        eel_stats + Statistics::new(0, eel_stats.health / 2).unwrap(),
        team.first().unwrap().borrow().stats
    )
}

#[test]
fn test_battle_hawk_team() {
    let mut team = test_hawk_team();
    let mut enemy_team = test_gorilla_team();

    // Hawk on 1st position.
    assert_eq!(team.first().unwrap().borrow().name, PetName::Hawk);
    {
        let gorilla_on_1st = enemy_team.first().unwrap();
        assert_eq!(
            gorilla_on_1st.borrow().stats,
            Statistics {
                attack: 6,
                health: 9
            }
        );
    }

    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();

    {
        // Gorilla takes 7 dmg.
        let gorilla_on_1st = enemy_team.first().unwrap();
        assert_eq!(
            gorilla_on_1st.borrow().stats,
            Statistics {
                attack: 6,
                health: 2
            }
        );
    }
}

#[test]
fn test_battle_pelican_team() {
    let mut team = test_pelican_team();
    let mut enemy_team = test_hippo_team();

    {
        // Ant has strawberry.
        let ant = team.nth(1).unwrap();

        assert_eq!(ant.borrow().stats, Statistics::new(2, 1).unwrap());
        let ant_item = &ant.borrow().item;
        assert_eq!(ant_item.as_ref().unwrap().name, FoodName::Strawberry)
    }

    team.triggers.push_front(TRIGGER_START_BATTLE);
    team.trigger_effects(Some(&mut enemy_team)).unwrap();

    // Pelican at lvl.1 give strawberry ant (2,1)
    assert_eq!(
        team.nth(1).unwrap().borrow().stats,
        Statistics::new(4, 2).unwrap()
    );
}
