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

pub fn test_cricket_horse_team(name: &str) -> Team {
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

pub fn test_badger_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Badger)),
            Some(Pet::from(PetName::Elephant)),
            None,
            None,
            None,
        ],
    )
}

pub fn test_blowfish_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Elephant)),
            Some(Pet::from(PetName::Blowfish)),
            None,
            None,
            None,
        ],
    )
}

pub fn test_camel_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Elephant)),
            Some(Pet::from(PetName::Camel)),
            Some(Pet::from(PetName::Ant)),
            None,
            None,
        ],
    )
}

pub fn test_dog_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Dog)),
            None,
            None,
            None,
        ],
    )
}

pub fn test_dolphin_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Dolphin)), None, None, None, None],
    )
}

pub fn test_kangaroo_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Kangaroo)),
            None,
            None,
            None,
        ],
    )
}

pub fn test_ox_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Ox)),
            None,
            None,
            None,
        ],
    )
}

pub fn test_sheep_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Sheep)), None, None, None, None],
    )
}
