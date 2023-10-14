use crate::{
    teams::team::TeamFightOutcome,
    tests::common::{spawn_toy_test, test_ant_team, test_cricket_horse_team},
    Entity, PetName, Position, TeamCombat, TeamEffects, TeamShopping, TeamViewer, Toy, ToyName,
};

#[test]
fn test_toy_bowling_ball() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    // Remove one pet from both teams.
    team.friends.pop();
    enemy_team.friends.pop();

    team.toys.push(Toy::try_from(ToyName::BowlingBall).unwrap());

    // Fight ant teams.
    let outcome = team.fight(&mut enemy_team).unwrap();

    // Squad loses because of bowling ball causing an additional ant to faint.
    assert_eq!(enemy_team.fainted.len(), 1);
    assert_eq!(team.fainted.len(), 2);
    assert_eq!(outcome, TeamFightOutcome::Loss)
}

#[test]
fn test_toy_glasses() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();
    let pets = team.all();

    team.toys.push(Toy::try_from(ToyName::Glasses).unwrap());

    // Pets don't have health stat of 5.
    assert!(pets.iter().all(|pet| pet.read().unwrap().stats.health != 5));

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    // Now they do.
    assert!(pets.iter().all(|pet| pet.read().unwrap().stats.health == 5))
}

#[test]
fn test_toy_lunchbox() {
    spawn_toy_test(ToyName::Lunchbox, PetName::Ant, 2);
}

#[test]
fn test_toy_paper_shredder_battle() {
    let mut team = test_cricket_horse_team();
    let mut enemy_team = test_cricket_horse_team();

    team.toys
        .push(Toy::try_from(ToyName::PaperShredder).unwrap());

    assert!(team.fainted.is_empty());
    assert!(enemy_team.fainted.is_empty());

    // Fight to get cricket to spawn zombie cricket.
    team.fight(&mut enemy_team).unwrap();
    team.fight(&mut enemy_team).unwrap();

    // Second zombie cricket faints.
    assert_eq!(team.fainted.len(), 2);
    assert_eq!(enemy_team.fainted.len(), 1)
}

#[test]
fn test_toy_paper_shredder_shop() {
    let mut team = test_cricket_horse_team();

    team.toys
        .push(Toy::try_from(ToyName::PaperShredder).unwrap());

    team.set_shop_seed(Some(123)).open_shop().unwrap();

    // Shop isn't empty
    assert!(!team.get_shop().pets.is_empty());
    assert!(team.fainted.is_empty());

    // Buy a pet.
    team.buy(&Position::First, &Entity::Pet, &Position::First)
        .unwrap();

    // One pet fainted once purchased
    assert_eq!(team.fainted.len(), 1)
}

#[test]
fn test_toy_spring() {
    spawn_toy_test(ToyName::Spring, PetName::Dog, 1);
}
