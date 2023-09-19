use crate::{tests::common::spawn_toy_test, PetName, ToyName};

#[test]
fn test_toy_cardboard_box() {
    spawn_toy_test(ToyName::CardboardBox, PetName::Scorpion, 2);
}

#[test]
fn test_toy_trampoline() {
    spawn_toy_test(ToyName::Trampoline, PetName::Kangaroo, 1);
}
