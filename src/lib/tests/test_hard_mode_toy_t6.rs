use crate::{tests::common::spawn_toy_test, PetName, ToyName};

#[test]
fn test_toy_stuffed_bear() {
    spawn_toy_test(ToyName::StuffedBear, PetName::Bear, 1)
}

// #[test]
// fn test_toy_toy_mouse() {
//     todo!()
// }
