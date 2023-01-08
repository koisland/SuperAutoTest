use crate::common::{
    effect::Statistics,
    // effect::{Outcome, Position, Action},
    // food::Food,
    // foods::names::FoodName,
    pet::{Combat, Pet},
    pets::names::PetName,
};

#[test]
fn test_attack_pet() {
    let mut ant_t1 = Pet::new(
        PetName::Ant,
        Statistics {
            attack: 2,
            health: 1,
        },
        1,
        None,
    )
    .unwrap();
    let mut ant_t2 = Pet::new(
        PetName::Ant,
        Statistics {
            attack: 2,
            health: 3,
        },
        1,
        None,
    )
    .unwrap();

    ant_t1.attack(&mut ant_t2);

    assert!(ant_t1.stats.borrow().health == 0 && ant_t2.stats.borrow().health == 1);
    // TODO: Add triggers
}

// #[test]
// fn test_attack_meat() {}

// #[test]
// fn test_attack_melon() {

// }
