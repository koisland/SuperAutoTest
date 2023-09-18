use crate::{
    teams::team::TeamFightOutcome, tests::common::test_scorpion_team, PetName, TeamCombat,
    TeamEffects, TeamViewer, Toy, ToyName,
};

use super::common::test_ant_team;

#[test]
fn test_toy_action_figure() {
    let mut team = test_ant_team();
    let mut enemy_team = test_ant_team();

    team.toys
        .push(Toy::try_from(ToyName::ActionFigure).unwrap());

    team.trigger_start_battle_effects(&mut enemy_team).unwrap();
    todo!()
}

#[test]
fn test_toy_dice() {
    let mut team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::Dice).unwrap());
    todo!()
}

#[test]
fn test_toy_open_piggy_bank() {
    let mut team = test_ant_team();

    team.toys
        .push(Toy::try_from(ToyName::OpenPiggyBank).unwrap());
    todo!()
}

#[test]
fn test_toy_rubber_duck() {
    let mut team = test_scorpion_team();
    let mut enemy_team = test_scorpion_team();
    team.toys.push(Toy::try_from(ToyName::RubberDuck).unwrap());

    let outcome = team.fight(&mut enemy_team).unwrap();

    // Lose fight because of rubber duck.
    assert_eq!(outcome, TeamFightOutcome::Loss);
    let enemy_first_pet = enemy_team.first().unwrap();
    assert_eq!(enemy_first_pet.read().unwrap().name, PetName::Duck);
}
