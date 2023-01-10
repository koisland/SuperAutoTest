use crate::common::{
    effect::Statistics,
    pet::{get_pet_effect, Pet},
    pets::names::PetName,
};
use std::{cell::RefCell, rc::Rc};

/// Manually specify ant pet. Done with DB call normally.
/// TODO: Use with mock object.
pub fn ant() -> Pet {
    Pet {
        name: PetName::Ant,
        tier: 1,
        stats: Rc::new(RefCell::new(Statistics {
            attack: 2,
            health: 1,
        })),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Ant,
            Statistics {
                attack: 2,
                health: 1,
            },
            1,
            1,
        ),
        item: None,
        pos: None,
    }
}

pub fn cricket() -> Pet {
    Pet {
        name: PetName::Cricket,
        tier: 1,
        stats: Rc::new(RefCell::new(Statistics {
            attack: 1,
            health: 1,
        })),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Cricket,
            Statistics {
                attack: 1,
                health: 1,
            },
            1,
            1,
        ),
        item: None,
        pos: None,
    }
}

pub fn horse() -> Pet {
    Pet {
        name: PetName::Horse,
        tier: 1,
        stats: Rc::new(RefCell::new(Statistics {
            attack: 2,
            health: 1,
        })),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Horse,
            Statistics {
                attack: 1,
                health: 0,
            },
            1,
            1,
        ),
        item: None,
        pos: None,
    }
}

pub fn test_team() -> [Option<Pet>; 5] {
    [Some(ant()), Some(ant()), Some(ant()), None, None]
}

pub fn test_summon_team() -> [Option<Pet>; 5] {
    [Some(cricket()), Some(cricket()), Some(horse()), None, None]
}
