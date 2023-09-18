use crate::{tests::common::test_ant_team, Toy, ToyName};

#[test]
fn test_toy_stuffed_bear() {
    let mut team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::StuffedBear).unwrap());
    todo!()
}

#[test]
fn test_toy_toy_mouse() {
    let mut team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::ToyMouse).unwrap());
    todo!()
}
