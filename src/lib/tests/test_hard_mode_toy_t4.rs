use crate::{tests::common::test_ant_team, Toy, ToyName};

#[test]
fn test_toy_cardboard_box() {
    let mut team = test_ant_team();

    team.toys
        .push(Toy::try_from(ToyName::CardboardBox).unwrap());

    todo!()
}

#[test]
fn test_toy_trampoline() {
    let mut team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::Trampoline).unwrap());
    todo!()
}
