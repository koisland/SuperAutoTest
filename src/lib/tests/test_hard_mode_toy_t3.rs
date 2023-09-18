use crate::{tests::common::test_ant_team, Toy, ToyName};

#[test]
fn test_toy_bowling_ball() {
    let mut team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::BowlingBall).unwrap());

    todo!()
}

#[test]
fn test_toy_glasses() {
    let mut team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::Glasses).unwrap());
    todo!()
}

#[test]
fn test_toy_lunchbox() {
    let mut team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::Lunchbox).unwrap());
    todo!()
}

#[test]
fn test_toy_paper_shredder() {
    let mut team = test_ant_team();

    team.toys
        .push(Toy::try_from(ToyName::PaperShredder).unwrap());
    todo!()
}

#[test]
fn test_toy_spring() {
    let mut team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::Spring).unwrap());
    todo!()
}
