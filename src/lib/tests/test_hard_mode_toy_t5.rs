use crate::{
    tests::common::{spawn_toy_test, test_ant_team},
    Pet, PetName, Statistics, TeamCombat, TeamEffects, TeamViewer, Toy, ToyName,
};

#[test]
fn test_toy_boot() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::Boot).unwrap());

    // Three pets at start.
    assert_eq!(team.all().len(), 3);

    team.fight(&mut enemy_team).unwrap();

    // Only one pet survives after last pet is booted.
    assert_eq!(team.all().len(), 1);
}

#[test]
fn test_toy_pill_bottle() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::PillBottle).unwrap());
    // First friend faints at start of battle.
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    let Some(Some(pet)) = &team.friends.first() else { panic!("No first friend.") };
    assert_eq!(pet.read().unwrap().stats.health, 0);
}

#[test]
fn test_toy_ring_pyramid() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    const RING_PYRAMID_DMG: isize = 2;
    const ANT_BUFF: Statistics = Statistics {
        attack: 1,
        health: 1,
    };
    let ant_orig_stats = Pet::try_from(PetName::Ant).unwrap().stats;

    team.toys.push(Toy::try_from(ToyName::RingPyramid).unwrap());

    assert_eq!(team.all().len(), 3);

    team.fight(&mut enemy_team).unwrap();

    let surviving_pets = team.all();
    assert_eq!(surviving_pets.len(), 1);

    // After first ant faints, ant buff triggers first allowing single ant to survive ring pyramid dmg.
    // Then gains remaining ant buffs.
    assert_eq!(
        ant_orig_stats + ANT_BUFF + ANT_BUFF
            - Statistics {
                attack: 0,
                health: RING_PYRAMID_DMG
            },
        surviving_pets[0].read().unwrap().stats
    );
}

#[test]
fn test_toy_rocking_horse() {
    spawn_toy_test(ToyName::RockingHorse, PetName::Horse, 3)
}
