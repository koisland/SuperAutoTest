use crate::common::pets::{names::PetName, pet::Pet};

/// Manually specify ant pet. Done with DB call normally.
/// TODO: Use with mock object.
pub fn ant() -> Pet {
    Pet::new(PetName::Ant, None, 1, None, None).unwrap()
}

pub fn cricket() -> Pet {
    Pet::new(PetName::Cricket, None, 1, None, None).unwrap()
}

pub fn horse() -> Pet {
    Pet::new(PetName::Horse, None, 1, None, None).unwrap()
}

pub fn mosquito() -> Pet {
    Pet::new(PetName::Mosquito, None, 1, None, None).unwrap()
}

fn hedgehog() -> Pet {
    Pet::new(PetName::Hedgehog, None, 1, None, None).unwrap()
}

fn peacock() -> Pet {
    Pet::new(PetName::Peacock, None, 1, None, None).unwrap()
}
fn crab() -> Pet {
    Pet::new(PetName::Crab, None, 1, None, None).unwrap()
}

fn dodo() -> Pet {
    Pet::new(PetName::Dodo, None, 1, None, None).unwrap()
}

fn elephant() -> Pet {
    Pet::new(PetName::Elephant, None, 1, None, None).unwrap()
}

fn flamingo() -> Pet {
    Pet::new(PetName::Flamingo, None, 1, None, None).unwrap()
}

fn rat() -> Pet {
    Pet::new(PetName::Rat, None, 1, None, None).unwrap()
}

fn spider() -> Pet {
    Pet::new(PetName::Spider, None, 1, None, None).unwrap()
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

pub fn test_flamingo_team() -> [Option<Pet>; 5] {
    [Some(flamingo()), Some(ant()), Some(ant()), None, None]
}

pub fn test_rat_team() -> [Option<Pet>; 5] {
    [Some(rat()), None, None, None, None]
}

pub fn test_spider_team() -> [Option<Pet>; 5] {
    [Some(spider()), None, None, None, None]
}

pub fn test_crab_team() -> [Option<Pet>; 5] {
    let mut big_ant = ant();
    big_ant.stats.health = 50;
    [Some(crab()), Some(big_ant), Some(ant()), None, None]
}
