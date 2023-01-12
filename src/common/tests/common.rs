use crate::common::{
    battle::state::Statistics,
    pets::{effects::get_pet_effect, names::PetName, pet::Pet},
};

/// Manually specify ant pet. Done with DB call normally.
/// TODO: Use with mock object.
pub fn ant() -> Pet {
    let stats = Statistics {
        attack: 2,
        health: 1,
    };
    Pet {
        name: PetName::Ant,
        tier: 1,
        stats: stats.clone(),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Ant,
            &stats,
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
    let stats = Statistics {
        attack: 1,
        health: 1,
    };
    Pet {
        name: PetName::Cricket,
        tier: 1,
        stats: stats.clone(),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Cricket,
            &stats,
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
    let stats = Statistics {
        attack: 2,
        health: 1,
    };
    Pet {
        name: PetName::Horse,
        tier: 1,
        stats: stats.clone(),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Horse,
            &stats,
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
    let stats = Statistics {
        attack: 2,
        health: 2,
    };
    Pet {
        name: PetName::Mosquito,
        tier: 1,
        stats: stats.clone(),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Mosquito,
            &stats,
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
    let stats = Statistics {
        attack: 3,
        health: 2,
    };
    Pet {
        name: PetName::Hedgehog,
        tier: 2,
        stats: stats.clone(),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Hedgehog,
            &stats,
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
    let stats = Statistics {
        attack: 2,
        health: 5,
    };
    Pet {
        name: PetName::Peacock,
        tier: 2,
        stats: stats.clone(),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Peacock,
            &stats,
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
    let stats = Statistics {
        attack: 3,
        health: 1,
    };
    Pet {
        name: PetName::Crab,
        tier: 2,
        stats: stats.clone(),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Crab,
            &stats,
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
    let stats = Statistics {
        attack: 3,
        health: 5,
    };
    Pet {
        name: PetName::Dodo,
        tier: 2,
        stats: stats.clone(),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Dodo,
            &stats,
            Statistics {
                attack: 33,
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
    let stats = Statistics {
        attack: 3,
        health: 5,
    };
    Pet {
        name: PetName::Elephant,
        tier: 2,
        stats: stats.clone(),
        lvl: 1,
        effect: get_pet_effect(
            &PetName::Elephant,
            &stats,
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
    [Some(dodo()), Some(dodo()), None, None, None]
}

pub fn test_crab_team() -> [Option<Pet>; 5] {
    let mut big_ant = ant();
    big_ant.stats.health = 50;
    [Some(crab()), Some(big_ant), Some(ant()), None, None]
}
