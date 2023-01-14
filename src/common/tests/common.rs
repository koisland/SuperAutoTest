use crate::common::{
    battle::team::Team,
    pets::{names::PetName, pet::Pet},
};

pub fn test_ant_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            None,
            None,
        ],
    )
}

pub fn test_summon_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Horse)),
            None,
            None,
        ],
    )
}

pub fn test_mosq_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Mosquito)),
            Some(Pet::from(PetName::Mosquito)),
            Some(Pet::from(PetName::Mosquito)),
            None,
            None,
        ],
    )
}

pub fn test_hedgehog_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Hedgehog)),
            Some(Pet::from(PetName::Ant)),
            None,
            None,
            None,
        ],
    )
}

pub fn test_elephant_peacock_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Elephant)),
            Some(Pet::from(PetName::Peacock)),
            None,
            None,
            None,
        ],
    )
}

pub fn test_dodo_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Dodo)),
            Some(Pet::from(PetName::Dodo)),
            None,
            None,
            None,
        ],
    )
}

pub fn test_flamingo_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Flamingo)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            None,
            None,
        ],
    )
}

pub fn test_rat_team(name: &str, lvl: usize) -> Team {
    let mut rat = Pet::from(PetName::Rat);
    for _ in 1..lvl {
        if let Err(error) = rat.levelup() {
            println!("{:?}", error)
        };
    }

    let pets = [Some(rat), None, None, None, None];
    Team::new(name, &pets)
}

pub fn test_spider_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Spider)), None, None, None, None],
    )
}

pub fn test_crab_team(name: &str) -> Team {
    let mut big_ant = Pet::from(PetName::Ant);
    big_ant.stats.health = 50;
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Crab)),
            Some(big_ant),
            Some(Pet::from(PetName::Ant)),
            None,
            None,
        ],
    )
}
