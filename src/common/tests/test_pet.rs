use crate::common::{
    battle::trigger::*,
    pets::pet::{BattleOutcome, Combat},
    tests::common::ant,
};
use std::collections::VecDeque;

#[test]
fn test_attack_pet() {
    let mut ant_t1 = ant();
    let mut ant_t2 = ant();

    // Set 2nd ant health to survive attack.
    ant_t2.stats.health = 3;

    let outcome = ant_t1.attack(&mut ant_t2);

    assert!(ant_t1.stats.health == 0 && ant_t2.stats.health == 1);
    assert_eq!(
        outcome,
        BattleOutcome {
            friends: VecDeque::from_iter([TRIGGER_SELF_FAINT, TRIGGER_ANY_FAINT]),
            opponents: VecDeque::from_iter([TRIGGER_SELF_HURT])
        }
    )
}

// #[test]
// fn test_attack_meat() {}

// #[test]
// fn test_attack_melon() {

// }
