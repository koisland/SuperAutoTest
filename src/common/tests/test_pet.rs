use crate::common::{
    effect::{Outcome, Position, Status, Target},
    pet::{BattleOutcome, Combat},
    tests::common::ant,
};
use std::collections::VecDeque;

#[test]
fn test_attack_pet() {
    let mut ant_t1 = ant();
    let mut ant_t2 = ant();

    // Set 2nd ant health to survive attack.
    ant_t2.stats.borrow_mut().health = 3;

    let outcome = ant_t1.attack(&mut ant_t2);

    assert!(ant_t1.stats.borrow().health == 0 && ant_t2.stats.borrow().health == 1);
    assert_eq!(
        outcome,
        BattleOutcome {
            friends: VecDeque::from_iter([Outcome {
                status: Status::Faint,
                target: Target::Friend,
                position: Position::Specific(0)
            }]),
            opponents: VecDeque::from_iter([Outcome {
                status: Status::Hurt,
                target: Target::Friend,
                position: Position::Specific(0)
            }])
        }
    )
}

// #[test]
// fn test_attack_meat() {}

// #[test]
// fn test_attack_melon() {

// }
