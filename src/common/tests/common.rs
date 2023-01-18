use crate::common::{
    battle::{state::Statistics, team::Team},
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
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
}

pub fn test_rat_team(name: &str, lvl: usize) -> Team {
    let mut rat = Pet::from(PetName::Rat);
    if let Err(error) = rat.set_level(lvl) {
        println!("{:?}", error)
    };

    let pets = [Some(rat), None, None, None, None];
    Team::new(name, &pets, 5).unwrap()
}

pub fn test_spider_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Spider)), None, None, None, None],
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
}

pub fn test_dolphin_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Dolphin)), None, None, None, None],
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
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
        5,
    )
    .unwrap()
}

pub fn test_sheep_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Sheep)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_filled_sheep_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Sheep)),
            Some(Pet::from(PetName::Sheep)),
            Some(Pet::from(PetName::Sheep)),
            Some(Pet::from(PetName::Sheep)),
            Some(Pet::from(PetName::Sheep)),
        ],
        5,
    )
    .unwrap()
}

pub fn test_blowfish_rally_team(name: &str) -> Team {
    let blowfish = Pet::new(
        PetName::Blowfish,
        None,
        Some(Statistics {
            attack: 1,
            health: 50,
        }),
        1,
    )
    .unwrap();
    Team::new(name, &[Some(blowfish.clone()), None, None, None, None], 5).unwrap()
}

pub fn test_deer_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Deer)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_hippo_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Hippo)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_parrot_team(name: &str) -> Team {
    let mut cricket = Pet::from(PetName::Cricket);
    cricket.set_level(2).unwrap();
    Team::new(
        name,
        &[
            Some(cricket),
            Some(Pet::from(PetName::Parrot)),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_rooster_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Rooster)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_skunk_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Skunk)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_turtle_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Turtle)),
            Some(Pet::from(PetName::Ant)),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_whale_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Whale)),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_crocodile_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Crocodile)),
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Cricket)),
        ],
        5,
    )
    .unwrap()
}

pub fn test_rhino_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Rhino)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_scorpion_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Scorpion)),
            Some(Pet::from(PetName::Ant)),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_shark_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Shark)),
        ],
        5,
    )
    .unwrap()
}

pub fn test_turkey_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Turkey)),
        ],
        5,
    )
    .unwrap()
}

pub fn test_boar_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Boar)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_fly_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Fly)),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_gorilla_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Gorilla)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_leopard_team(name: &str) -> Team {
    Team::new(
        name,
        &[Some(Pet::from(PetName::Leopard)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_mammoth_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Mammoth)),
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Dog)),
        ],
        5,
    )
    .unwrap()
}

pub fn test_snake_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Snake)),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_tiger_team(name: &str) -> Team {
    Team::new(
        name,
        &[
            Some(Pet::from(PetName::Leopard)),
            Some(Pet::from(PetName::Tiger)),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}
