use std::{cell::RefCell, rc::Rc};

use crate::{
    battle::{state::Position, stats::Statistics, team::Team},
    foods::{food::Food, names::FoodName},
    pets::{names::PetName, pet::Pet},
};

pub fn count_pets(friends: &[Rc<RefCell<Pet>>], pet_name: PetName) -> usize {
    friends
        .iter()
        .filter_map(|pet| (pet.borrow().name == pet_name).then_some(1))
        .sum()
}

pub fn test_ant_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_cricket_horse_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Horse).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_mosq_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Mosquito).unwrap(),
            Pet::try_from(PetName::Mosquito).unwrap(),
            Pet::try_from(PetName::Mosquito).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_frilled_dragon_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::FrilledDragon).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_frog_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Frog).unwrap(),
            Pet::try_from(PetName::FrilledDragon).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_moth_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Duck).unwrap(),
            Pet::try_from(PetName::Moth).unwrap(),
            Pet::try_from(PetName::Moth).unwrap(),
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
            duck_w_strawberry,
            Pet::try_from(PetName::Hummingbird).unwrap(),
            Pet::try_from(PetName::Hummingbird).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_iguana_seahorse_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Seahorse).unwrap(),
            Pet::try_from(PetName::Iguana).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_hedgehog_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Hedgehog).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_elephant_peacock_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Elephant).unwrap(),
            Pet::try_from(PetName::Peacock).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_dodo_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Dodo).unwrap(),
            Pet::try_from(PetName::Dodo).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_flamingo_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Flamingo).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
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
    Team::new(&[rat], 5).unwrap()
}

pub fn test_spider_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Spider).unwrap()], 5).unwrap()
}

pub fn test_bat_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Bat).unwrap()], 5).unwrap()
}

pub fn test_atlantic_puffin_team() -> Team {
    let mut strawberry_ant = Pet::try_from(PetName::Ant).unwrap();
    strawberry_ant.item = Some(Food::try_from(FoodName::Strawberry).unwrap());

    Team::new(
        &[
            Pet::try_from(PetName::AtlanticPuffin).unwrap(),
            strawberry_ant.clone(),
            strawberry_ant,
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
            Pet::try_from(PetName::Dove).unwrap(),
            strawberry_ant.clone(),
            strawberry_ant.clone(),
            strawberry_ant,
        ],
        5,
    )
    .unwrap()
}

pub fn test_koala_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Mammoth).unwrap(),
            Pet::try_from(PetName::Koala).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_panda_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Panda).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_pug_team() -> Team {
    let mut exp_1_ant = Pet::try_from(PetName::Ant).unwrap();
    exp_1_ant.add_experience(1).unwrap();

    Team::new(&[exp_1_ant, Pet::try_from(PetName::Pug).unwrap()], 5).unwrap()
}

pub fn test_stork_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Stork).unwrap()], 5).unwrap()
}

pub fn test_racoon_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Racoon).unwrap()], 5).unwrap()
}

pub fn test_toucan_team() -> Team {
    let mut toucan = Pet::try_from(PetName::Toucan).unwrap();
    toucan.item = Some(Food::try_from(FoodName::Honey).unwrap());
    Team::new(
        &[
            toucan,
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_wombat_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Wombat).unwrap()], 5).unwrap()
}

pub fn test_crab_team() -> Team {
    let mut big_ant = Pet::try_from(PetName::Ant).unwrap();
    big_ant.stats.health = 50;
    Team::new(
        &[
            Pet::try_from(PetName::Crab).unwrap(),
            big_ant,
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_badger_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Badger).unwrap(),
            Pet::try_from(PetName::Elephant).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_blowfish_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Elephant).unwrap(),
            Pet::try_from(PetName::Blowfish).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_camel_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Elephant).unwrap(),
            Pet::try_from(PetName::Camel).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_dog_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_dolphin_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Dolphin).unwrap()], 5).unwrap()
}

pub fn test_kangaroo_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Kangaroo).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_ox_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Ox).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_sheep_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Sheep).unwrap()], 5).unwrap()
}

pub fn test_filled_sheep_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Sheep).unwrap(),
            Pet::try_from(PetName::Sheep).unwrap(),
            Pet::try_from(PetName::Sheep).unwrap(),
            Pet::try_from(PetName::Sheep).unwrap(),
            Pet::try_from(PetName::Sheep).unwrap(),
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
    Team::new(&[blowfish.clone()], 5).unwrap()
}

pub fn test_aardvark_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Aardvark).unwrap()], 5).unwrap()
}

pub fn test_bear_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Bear).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_seagull_team() -> Team {
    let seagull = Pet::try_from(PetName::Seagull).unwrap();
    let mut team = Team::new(&[Pet::try_from(PetName::Cricket).unwrap(), seagull], 5).unwrap();
    team.set_item(
        Position::Last,
        Some(Food::try_from(FoodName::Honey).unwrap()),
    )
    .unwrap();
    team
}

pub fn test_blobfish_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Blobfish).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
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
            Pet::try_from(PetName::Blobfish).unwrap(),
            dog_w_exp,
            Pet::try_from(PetName::Clownfish).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_toad_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Toad).unwrap(),
            Pet::try_from(PetName::Mosquito).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_woodpecker_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Woodpecker).unwrap()], 5).unwrap()
}

pub fn test_woodpecker_self_hurt_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Woodpecker).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_deer_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Deer).unwrap()], 5).unwrap()
}

pub fn test_hippo_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Hippo).unwrap()], 5).unwrap()
}

pub fn test_parrot_team() -> Team {
    let mut cricket = Pet::try_from(PetName::Cricket).unwrap();
    cricket.set_level(2).unwrap();
    Team::new(&[cricket, Pet::try_from(PetName::Parrot).unwrap()], 5).unwrap()
}

pub fn test_rooster_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Rooster).unwrap()], 5).unwrap()
}

pub fn test_skunk_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Skunk).unwrap()], 5).unwrap()
}

pub fn test_turtle_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Turtle).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_whale_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Whale).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_armadillo_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Armadillo).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_doberman_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Doberman).unwrap(),
            Pet::try_from(PetName::Mammoth).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_doberman_highest_tier_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Doberman).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_lynx_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Lynx).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_porcupine_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Porcupine).unwrap()], 5).unwrap()
}

pub fn test_caterpillar_team() -> Team {
    let mut lvl_3_caterpillar = Pet::try_from(PetName::Caterpillar).unwrap();
    lvl_3_caterpillar.set_level(3).unwrap();

    Team::new(
        &[
            lvl_3_caterpillar,
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
        ],
        5,
    )
    .unwrap()
}

pub fn test_anteater_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Anteater).unwrap()], 5).unwrap()
}

pub fn test_donkey_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Donkey).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_eel_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Eel).unwrap()], 5).unwrap()
}

pub fn test_hawk_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Hawk).unwrap(),
            Pet::try_from(PetName::Ant).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_pelican_team() -> Team {
    let mut strawberry_ant = Pet::try_from(PetName::Ant).unwrap();
    strawberry_ant.item = Some(Food::try_from(FoodName::Strawberry).unwrap());

    Team::new(
        &[Pet::try_from(PetName::Pelican).unwrap(), strawberry_ant],
        5,
    )
    .unwrap()
}

pub fn test_crocodile_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Crocodile).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_rhino_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Rhino).unwrap()], 5).unwrap()
}

pub fn test_scorpion_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Scorpion).unwrap(),
            Pet::try_from(PetName::Scorpion).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_shark_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Shark).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_turkey_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Turkey).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_boar_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Boar).unwrap()], 5).unwrap()
}

pub fn test_fly_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Fly).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_gorilla_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Gorilla).unwrap()], 5).unwrap()
}

pub fn test_leopard_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Leopard).unwrap()], 5).unwrap()
}

pub fn test_mammoth_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Mammoth).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_snake_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Snake).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_tiger_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Leopard).unwrap(),
            Pet::try_from(PetName::Tiger).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_hyena_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Hyena).unwrap(),
            Pet::try_from(PetName::Gorilla).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_lionfish_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Lionfish).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_eagle_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Eagle).unwrap()], 5).unwrap()
}

pub fn test_microbe_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Microbe).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_lion_highest_tier_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Lion).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_lion_lowest_tier_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Lion).unwrap(),
            Pet::try_from(PetName::Gorilla).unwrap(),
            Pet::try_from(PetName::Gorilla).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_swordfish_team() -> Team {
    Team::new(
        &[
            Pet::new(
                PetName::Swordfish,
                None,
                Some(Statistics {
                    attack: 25,
                    health: 25,
                }),
                1,
            )
            .unwrap(),
            Pet::new(
                PetName::Gorilla,
                None,
                Some(Statistics {
                    attack: 50,
                    health: 50,
                }),
                1,
            )
            .unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_triceratops_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Triceratops).unwrap(),
            Pet::try_from(PetName::Gorilla).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_vulture_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Vulture).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_alpaca_team() -> Team {
    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Alpaca).unwrap(),
            Pet::try_from(PetName::Alpaca).unwrap(),
        ],
        5,
    )
    .unwrap();

    // Give mushroom to alpaca.
    team.set_item(
        Position::Relative(-1),
        Some(Food::try_from(FoodName::Mushroom).unwrap()),
    )
    .unwrap();
    team
}

pub fn test_tapir_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Tapir).unwrap()], 5).unwrap()
}

pub fn test_walrus_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Walrus).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_white_tiger_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::WhiteTiger).unwrap(),
            Pet::try_from(PetName::Deer).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_octopus_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Octopus).unwrap()], 5).unwrap()
}

pub fn test_orca_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Orca).unwrap()], 5).unwrap()
}

pub fn test_piranha_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Piranha).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_reindeer_team() -> Team {
    Team::new(&[Pet::try_from(PetName::Reindeer).unwrap()], 5).unwrap()
}

pub fn test_sabertooth_team() -> Team {
    Team::new(&[Pet::try_from(PetName::SabertoothTiger).unwrap()], 5).unwrap()
}

pub fn test_spinosaurus_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Spinosaurus).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_stegosaurus_team() -> Team {
    Team::new(
        &[
            Pet::try_from(PetName::Dog).unwrap(),
            Pet::try_from(PetName::Stegosaurus).unwrap(),
        ],
        5,
    )
    .unwrap()
}

pub fn test_velociraptor_team() -> Team {
    let mut team = Team::new(
        &[
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Cricket).unwrap(),
            Pet::try_from(PetName::Velociraptor).unwrap(),
        ],
        5,
    )
    .unwrap();
    team.set_item(
        Position::Range(-2..=-1),
        Some(Food::try_from(FoodName::Strawberry).unwrap()),
    )
    .unwrap();
    team
}
