use crate::common::{
    battle::state::Statistics,
    pets::{effects::get_pet_effect, names::PetName, pet::Pet},
};

/// Manually specify ant pet. Done with DB call normally.
/// TODO: Use with mock object.
pub fn ant() -> Pet {
    Pet {
        name: PetName::Ant,
        tier: 1,
        stats: Statistics {
            attack: 2,
            health: 1,
        },
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
        stats: Statistics {
            attack: 1,
            health: 1,
        },
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
        stats: Statistics {
            attack: 2,
            health: 1,
        },
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

pub fn mosquito() -> Pet {
    Pet {
        name: PetName::Mosquito,
        tier: 1,
        stats: Statistics {
            attack: 2,
            health: 2,
        },
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Mosquito,
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
pub fn test_ant_team() -> [Option<Pet>; 5] {
    [Some(ant()), Some(ant()), Some(ant()), None, None]
}

pub fn test_summon_team() -> [Option<Pet>; 5] {
    [Some(cricket()), Some(cricket()), Some(horse()), None, None]
}

pub fn test_mosq_team() -> [Option<Pet>; 5] {
    [
        Some(mosquito()),
        Some(mosquito()),
        Some(mosquito()),
        None,
        None,
    ]
}
