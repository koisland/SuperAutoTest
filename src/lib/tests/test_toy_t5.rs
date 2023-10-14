use crate::{
    tests::common::{test_ant_team, test_gorilla_team},
    Statistics, TeamEffects, TeamShopping, TeamViewer, Toy, ToyName,
};

#[test]
fn test_toy_flashlight() {
    let mut team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::Flashlight).unwrap());

    let first_ant = team.first().unwrap();
    let first_ant_stats = first_ant.read().unwrap().stats;

    // First turn. Then second to break toy.
    team.open_shop().unwrap().close_shop().unwrap();
    team.open_shop().unwrap();

    assert_eq!(
        first_ant_stats
            + Statistics {
                attack: 6,
                health: 6
            },
        first_ant.read().unwrap().stats
    );
}

#[test]
fn test_toy_stinky_sock() {
    let mut team = test_ant_team();
    let mut enemy_team = test_gorilla_team();

    let gorilla = enemy_team.first().unwrap();
    let gorilla_stats = gorilla.read().unwrap().stats;

    let debuffed_gorilla_health =
        gorilla_stats.health - (gorilla_stats.health as f32 * 0.4) as isize;

    team.toys.push(Toy::new(ToyName::StinkySock, 1).unwrap());
    team.trigger_start_battle_effects(&mut enemy_team).unwrap();

    assert_eq!(
        gorilla.read().unwrap().stats.health,
        debuffed_gorilla_health
    );
}
