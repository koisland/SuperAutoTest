use crate::{tests::common::test_ant_team, Toy, ToyName};

#[test]
fn test_toy_boot() {
    let mut team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::Boot).unwrap());

    todo!()
}

#[test]
fn test_toy_pill_bottle() {
    let mut team = test_ant_team();
    team.toys.push(Toy::try_from(ToyName::PillBottle).unwrap());
    todo!()
}

#[test]
fn test_toy_ring_pyramid() {
    let mut team = test_ant_team();

    team.toys.push(Toy::try_from(ToyName::RingPyramid).unwrap());
    todo!()
}

#[test]
fn test_toy_rocking_horse() {
    let mut team = test_ant_team();

    team.toys
        .push(Toy::try_from(ToyName::RockingHorse).unwrap());

    todo!()
}
