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

fn hedgehog() -> Pet {
    Pet {
        name: PetName::Hedgehog,
        tier: 2,
        stats: Statistics {
            attack: 3,
            health: 2,
        },
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Hedgehog,
            Statistics {
                attack: 2,
                health: 0,
            },
            1,
            1,
        ),
        item: None,
        pos: None,
    }
}

fn peacock() -> Pet {
    Pet {
        name: PetName::Peacock,
        tier: 2,
        stats: Statistics {
            attack: 2,
            health: 5,
        },
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Peacock,
            Statistics {
                attack: 4,
                health: 0,
            },
            1,
            1,
        ),
        item: None,
        pos: None,
    }
}
fn crab() -> Pet {
    Pet {
        name: PetName::Crab,
        tier: 2,
        stats: Statistics {
            attack: 3,
            health: 1,
        },
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Crab,
            // Instead of raw values, treat as percentage.
            Statistics {
                attack: 0,
                health: 50,
            },
            1,
            1,
        ),
        item: None,
        pos: None,
    }
}

fn dodo() -> Pet {
    let base_stats = Statistics {
        attack: 3,
        health: 5,
    };
    let effect_atk = base_stats.attack as f32;
    Pet {
        name: PetName::Dodo,
        tier: 2,
        stats: base_stats.clone(),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Dodo,
            Statistics {
                attack: (effect_atk * 0.33) as usize,
                health: 0,
            },
            1,
            1,
        ),
        item: None,
        pos: None,
    }
}

fn elephant() -> Pet {
    Pet {
        name: PetName::Elephant,
        tier: 2,
        stats: Statistics {
            attack: 3,
            health: 5,
        },
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Elephant,
            Statistics {
                attack: 0,
                health: 1,
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

pub fn test_solo_hedgehog_team() -> [Option<Pet>; 5] {
    [Some(hedgehog()), None, None, None, None]
}

pub fn test_elephant_peacock_team() -> [Option<Pet>; 5] {
    [Some(elephant()), Some(peacock()), None, None, None]
}

pub fn test_dodo_team() -> [Option<Pet>; 5] {
    [Some(ant()), Some(dodo()), None, None, None]
}
