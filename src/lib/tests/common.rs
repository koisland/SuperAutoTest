use crate::{
    battle::{state::Statistics, team::Team},
    foods::{food::Food, names::FoodName},
    pets::{names::PetName, pet::Pet},
};

pub fn count_pets(friends: &[Option<Pet>], pet_name: PetName) -> usize {
    friends
        .iter()
        .filter_map(|pet| {
            if let Some(pet) = pet {
                (pet.name == pet_name).then_some(1)
            } else {
                None
            }
        })
        .sum::<usize>()
}

pub fn test_ant_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_cricket_horse_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Horse).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_mosq_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_frilled_dragon_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::FrilledDragon).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_frog_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Frog).unwrap()),
            Some(Pet::try_from(PetName::FrilledDragon).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_moth_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Duck).unwrap()),
            Some(Pet::try_from(PetName::Moth).unwrap()),
            Some(Pet::try_from(PetName::Moth).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_hummingbird_team() -> Team {
    let mut duck_w_strawberry = Pet::try_from(PetName::Duck).unwrap();
    duck_w_strawberry.item = Some(Food::try_from(FoodName::Strawberry).unwrap());

    Team::new(
        &[
            Some(duck_w_strawberry),
            Some(Pet::try_from(PetName::Hummingbird).unwrap()),
            Some(Pet::try_from(PetName::Hummingbird).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_iguana_seahorse_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Seahorse).unwrap()),
            Some(Pet::try_from(PetName::Iguana).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_hedgehog_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Hedgehog).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_elephant_peacock_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Elephant).unwrap()),
            Some(Pet::try_from(PetName::Peacock).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_dodo_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dodo).unwrap()),
            Some(Pet::try_from(PetName::Dodo).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_flamingo_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Flamingo).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_rat_team(lvl: usize) -> Team {
    let mut rat = Pet::try_from(PetName::Rat).unwrap();
    if let Err(error) = rat.set_level(lvl) {
        println!("{:?}", error)
    };

    let pets = [Some(rat), None, None, None, None];
    Team::new(&pets, 5).unwrap()
}

pub fn test_spider_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Spider).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_bat_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Bat).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_atlantic_puffin_team() -> Team {
    let mut strawberry_ant = Pet::try_from(PetName::Ant).unwrap();
    strawberry_ant.item = Some(Food::try_from(FoodName::Strawberry).unwrap());

    Team::new(
        &[
            Some(Pet::try_from(PetName::AtlanticPuffin).unwrap()),
            Some(strawberry_ant.clone()),
            Some(strawberry_ant),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_dove_team() -> Team {
    let mut strawberry_ant = Pet::try_from(PetName::Ant).unwrap();
    strawberry_ant.item = Some(Food::try_from(FoodName::Strawberry).unwrap());

    Team::new(
        &[
            Some(Pet::try_from(PetName::Dove).unwrap()),
            Some(strawberry_ant.clone()),
            Some(strawberry_ant.clone()),
            Some(strawberry_ant),
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_koala_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Mammoth).unwrap()),
            Some(Pet::try_from(PetName::Koala).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_panda_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Panda).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_pug_team() -> Team {
    let mut exp_1_ant = Pet::try_from(PetName::Ant).unwrap();
    exp_1_ant.add_experience(1).unwrap();

    Team::new(
        &[
            Some(exp_1_ant),
            Some(Pet::try_from(PetName::Pug).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_stork_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Stork).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_racoon_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Racoon).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_toucan_team() -> Team {
    let mut toucan = Pet::try_from(PetName::Toucan).unwrap();
    toucan.item = Some(Food::try_from(FoodName::Honey).unwrap());
    Team::new(
        &[
            Some(toucan),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_wombat_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Wombat).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_crab_team() -> Team {
    let mut big_ant = Pet::try_from(PetName::Ant).unwrap();
    big_ant.stats.health = 50;
    Team::new(
        &[
            Some(Pet::try_from(PetName::Crab).unwrap()),
            Some(big_ant),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_badger_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Badger).unwrap()),
            Some(Pet::try_from(PetName::Elephant).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_blowfish_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Elephant).unwrap()),
            Some(Pet::try_from(PetName::Blowfish).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_camel_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Elephant).unwrap()),
            Some(Pet::try_from(PetName::Camel).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_dog_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_dolphin_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dolphin).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_kangaroo_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Kangaroo).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_ox_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Ox).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_sheep_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Sheep).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_filled_sheep_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Sheep).unwrap()),
            Some(Pet::try_from(PetName::Sheep).unwrap()),
            Some(Pet::try_from(PetName::Sheep).unwrap()),
            Some(Pet::try_from(PetName::Sheep).unwrap()),
            Some(Pet::try_from(PetName::Sheep).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_blowfish_rally_team() -> Team {
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
    Team::new(&[Some(blowfish.clone()), None, None, None, None], 5).unwrap()
}

pub fn test_aardvark_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Aardvark).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_bear_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Bear).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_seagull_team() -> Team {
    let mut seagull = Pet::try_from(PetName::Seagull).unwrap();
    seagull.item = Some(Food::try_from(FoodName::Honey).unwrap());
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(seagull),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_blobfish_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Blobfish).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_clownfish_team() -> Team {
    let mut dog_w_exp = Pet::try_from(PetName::Dog).unwrap();
    dog_w_exp.add_experience(1).unwrap();
    Team::new(
        &[
            Some(Pet::try_from(PetName::Blobfish).unwrap()),
            Some(dog_w_exp),
            Some(Pet::try_from(PetName::Clownfish).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_toad_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Toad).unwrap()),
            Some(Pet::try_from(PetName::Mosquito).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_woodpecker_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Woodpecker).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_woodpecker_self_hurt_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Woodpecker).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_deer_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Deer).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_hippo_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Hippo).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_parrot_team() -> Team {
    let mut cricket = Pet::try_from(PetName::Cricket).unwrap();
    cricket.set_level(2).unwrap();
    Team::new(
        &[
            Some(cricket),
            Some(Pet::try_from(PetName::Parrot).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_rooster_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Rooster).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_skunk_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Skunk).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_turtle_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Turtle).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_whale_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Whale).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_armadillo_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Armadillo).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_doberman_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Doberman).unwrap()),
            Some(Pet::try_from(PetName::Mammoth).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_doberman_highest_tier_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Doberman).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_lynx_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Lynx).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_porcupine_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Porcupine).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_caterpillar_team() -> Team {
    let mut lvl_3_caterpillar = Pet::try_from(PetName::Caterpillar).unwrap();
    lvl_3_caterpillar.set_level(3).unwrap();

    Team::new(
        &[
            Some(lvl_3_caterpillar),
            Some(
                Pet::new(
                    PetName::Dog,
                    None,
                    Some(Statistics {
                        attack: 50,
                        health: 50,
                    }),
                    1,
                )
                .unwrap(),
            ),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_anteater_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Anteater).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_donkey_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Donkey).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_eel_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Eel).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_hawk_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Hawk).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_pelican_team() -> Team {
    let mut strawberry_ant = Pet::try_from(PetName::Ant).unwrap();
    strawberry_ant.item = Some(Food::try_from(FoodName::Strawberry).unwrap());

    Team::new(
        &[
            Some(Pet::try_from(PetName::Pelican).unwrap()),
            Some(strawberry_ant),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_crocodile_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Crocodile).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_rhino_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Rhino).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_scorpion_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Scorpion).unwrap()),
            Some(Pet::try_from(PetName::Scorpion).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_shark_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Shark).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_turkey_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Turkey).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_boar_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Boar).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_fly_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Fly).unwrap()),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_gorilla_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Gorilla).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_leopard_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Leopard).unwrap()),
            None,
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_mammoth_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Mammoth).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_snake_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Snake).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_tiger_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Leopard).unwrap()),
            Some(Pet::try_from(PetName::Tiger).unwrap()),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}
