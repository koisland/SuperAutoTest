use std::{cell::RefCell, rc::Rc};

use crate::{Food, FoodName, Pet, PetName, Position, Statistics, Team};

pub fn count_pets(friends: &[Option<Rc<RefCell<Pet>>>], pet_name: PetName) -> usize {
    friends
        .iter()
        .flatten()
        .filter_map(|pet| (pet.borrow().name == pet_name).then_some(1))
        .sum()
}

pub fn test_ant_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
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
        ],
        5,
    )
    .unwrap()
}

pub fn test_rat_team(lvl: usize) -> Team {
    let mut rat = Pet::try_from(PetName::Rat).unwrap();
    rat.set_level(lvl).unwrap();

    Team::new(&[Some(rat)], 5).unwrap()
}

pub fn test_spider_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Spider).unwrap())], 5).unwrap()
}

pub fn test_bat_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Bat).unwrap())], 5).unwrap()
}

pub fn test_atlantic_puffin_team() -> Team {
    let mut strawberry_ant = Pet::try_from(PetName::Ant).unwrap();
    strawberry_ant.item = Some(Food::try_from(FoodName::Strawberry).unwrap());

    Team::new(
        &[
            Some(Pet::try_from(PetName::AtlanticPuffin).unwrap()),
            Some(strawberry_ant.clone()),
            Some(strawberry_ant),
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
            Some(strawberry_ant),
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
        ],
        5,
    )
    .unwrap()
}

pub fn test_pug_team() -> Team {
    let mut exp_1_ant = Pet::try_from(PetName::Ant).unwrap();
    exp_1_ant.add_experience(1).unwrap();

    Team::new(
        &[Some(exp_1_ant), Some(Pet::try_from(PetName::Pug).unwrap())],
        5,
    )
    .unwrap()
}

pub fn test_stork_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Stork).unwrap())], 5).unwrap()
}

pub fn test_racoon_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Racoon).unwrap())], 5).unwrap()
}

pub fn test_toucan_team() -> Team {
    let mut toucan = Pet::try_from(PetName::Toucan).unwrap();
    toucan.item = Some(Food::try_from(FoodName::Honey).unwrap());
    Team::new(
        &[
            Some(toucan),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_shrimp_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Shrimp).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_swan_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Swan).unwrap())], 5).unwrap()
}

// Frigatebird uses peacock.

pub fn test_goldfish_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::GoldFish).unwrap())], 5).unwrap()
}

pub fn test_dromedary_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Dromedary).unwrap())], 5).unwrap()
}

pub fn test_tabby_cat_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::TabbyCat).unwrap()),
        ],
        5,
    )
    .unwrap()
}

// No guinea pig.

pub fn test_jellyfish_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Jellyfish).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_salamander_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Salamander).unwrap())], 5).unwrap()
}

pub fn test_yak_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Yak).unwrap())], 5).unwrap()
}

pub fn test_wombat_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Wombat).unwrap())], 5).unwrap()
}

pub fn test_crab_team() -> Team {
    let mut big_ant = Pet::try_from(PetName::Ant).unwrap();
    big_ant.stats.health = 50;
    Team::new(
        &[
            Some(Pet::try_from(PetName::Crab).unwrap()),
            Some(big_ant),
            Some(Pet::try_from(PetName::Ant).unwrap()),
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
        ],
        5,
    )
    .unwrap()
}

pub fn test_dolphin_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Dolphin).unwrap())], 5).unwrap()
}

pub fn test_kangaroo_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Kangaroo).unwrap()),
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
        ],
        5,
    )
    .unwrap()
}

pub fn test_sheep_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Sheep).unwrap())], 5).unwrap()
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
    Team::new(&[Some(blowfish.clone())], 5).unwrap()
}

pub fn test_aardvark_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Aardvark).unwrap())], 5).unwrap()
}

pub fn test_bear_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Bear).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_seagull_team() -> Team {
    let seagull = Some(Pet::try_from(PetName::Seagull).unwrap());
    let mut team = Team::new(
        &[Some(Pet::try_from(PetName::Cricket).unwrap()), seagull],
        5,
    )
    .unwrap();
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
            Some(Pet::try_from(PetName::Blobfish).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
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
        ],
        5,
    )
    .unwrap()
}

pub fn test_woodpecker_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Woodpecker).unwrap())], 5).unwrap()
}

pub fn test_woodpecker_self_hurt_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Woodpecker).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_deer_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Deer).unwrap())], 5).unwrap()
}

pub fn test_hippo_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Hippo).unwrap())], 5).unwrap()
}

pub fn test_parrot_team() -> Team {
    let mut cricket = Pet::try_from(PetName::Cricket).unwrap();
    cricket.set_level(2).unwrap();
    Team::new(
        &[Some(cricket), Some(Pet::try_from(PetName::Parrot).unwrap())],
        5,
    )
    .unwrap()
}

pub fn test_rooster_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Rooster).unwrap())], 5).unwrap()
}

pub fn test_skunk_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Skunk).unwrap())], 5).unwrap()
}

pub fn test_turtle_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Turtle).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
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
    Team::new(&[Some(Pet::try_from(PetName::Porcupine).unwrap())], 5).unwrap()
}

pub fn test_caterpillar_team() -> Team {
    let mut lvl_3_caterpillar = Pet::try_from(PetName::Caterpillar).unwrap();
    lvl_3_caterpillar.set_level(3).unwrap();

    let big_dog = Pet::new(
        PetName::Dog,
        None,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();

    Team::new(&[Some(lvl_3_caterpillar), Some(big_dog)], 5).unwrap()
}

pub fn test_anteater_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Anteater).unwrap())], 5).unwrap()
}

pub fn test_donkey_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Donkey).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_eel_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Eel).unwrap())], 5).unwrap()
}

pub fn test_hawk_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Hawk).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
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
    Team::new(&[Some(Pet::try_from(PetName::Rhino).unwrap())], 5).unwrap()
}

pub fn test_scorpion_team() -> Team {
    let mut team = Team::new(&[Some(Pet::try_from(PetName::Scorpion).unwrap())], 5).unwrap();
    team.set_item(
        Position::First,
        Some(Food::try_from(FoodName::Peanut).unwrap()),
    )
    .unwrap();
    team
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
    Team::new(&[Some(Pet::try_from(PetName::Boar).unwrap())], 5).unwrap()
}

pub fn test_fly_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Fly).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_gorilla_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Gorilla).unwrap())], 5).unwrap()
}

pub fn test_leopard_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Leopard).unwrap())], 5).unwrap()
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
        ],
        5,
    )
    .unwrap()
}

pub fn test_hyena_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Hyena).unwrap()),
            Some(Pet::try_from(PetName::Gorilla).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_lionfish_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Lionfish).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_eagle_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Eagle).unwrap())], 5).unwrap()
}

pub fn test_microbe_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Microbe).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_lion_highest_tier_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Lion).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_lion_lowest_tier_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Lion).unwrap()),
            Some(Pet::try_from(PetName::Gorilla).unwrap()),
            Some(Pet::try_from(PetName::Gorilla).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_swordfish_team() -> Team {
    let swordfish = Pet::new(
        PetName::Swordfish,
        None,
        Some(Statistics {
            attack: 25,
            health: 25,
        }),
        1,
    )
    .unwrap();
    let gorilla = Pet::new(
        PetName::Gorilla,
        None,
        Some(Statistics {
            attack: 50,
            health: 50,
        }),
        1,
    )
    .unwrap();
    Team::new(&[Some(swordfish), Some(gorilla)], 5).unwrap()
}

pub fn test_triceratops_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Triceratops).unwrap()),
            Some(Pet::try_from(PetName::Gorilla).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_vulture_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Vulture).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_alpaca_team() -> Team {
    let mut team = Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Alpaca).unwrap()),
            Some(Pet::try_from(PetName::Alpaca).unwrap()),
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
    Team::new(
        &[
            Some(Pet::try_from(PetName::Tapir).unwrap()),
            Some(Pet::try_from(PetName::Tapir).unwrap()),
            Some(Pet::try_from(PetName::Tiger).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_walrus_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Walrus).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_white_tiger_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::WhiteTiger).unwrap()),
            Some(Pet::try_from(PetName::Deer).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_octopus_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Octopus).unwrap())], 5).unwrap()
}

pub fn test_orca_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Orca).unwrap())], 5).unwrap()
}

pub fn test_piranha_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Piranha).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_reindeer_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Reindeer).unwrap())], 5).unwrap()
}

pub fn test_sabertooth_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::SabertoothTiger).unwrap())], 5).unwrap()
}

pub fn test_spinosaurus_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Spinosaurus).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_stegosaurus_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Stegosaurus).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_velociraptor_team() -> Team {
    let mut team = Team::new(
        &[
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Cricket).unwrap()),
            Some(Pet::try_from(PetName::Velociraptor).unwrap()),
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

pub fn test_beaver_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Beaver).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_duck_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Duck).unwrap())], 5).unwrap()
}

pub fn test_fish_team() -> Team {
    let mut fish = Pet::try_from(PetName::Fish).unwrap();
    fish.add_experience(1).unwrap();
    Team::new(
        &[
            Some(fish),
            Some(Pet::try_from(PetName::Duck).unwrap()),
            Some(Pet::try_from(PetName::Duck).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_pig_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Pig).unwrap())], 5).unwrap()
}

pub fn test_chinchilla_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Chinchilla).unwrap())], 5).unwrap()
}

pub fn test_marmoset_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Marmoset).unwrap())], 5).unwrap()
}

pub fn test_beetle_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Beetle).unwrap())], 5).unwrap()
}

pub fn test_bluebird_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Bluebird).unwrap()),
            Some(Pet::try_from(PetName::Bluebird).unwrap()),
            Some(Pet::try_from(PetName::Bluebird).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_ladybug_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Ladybug).unwrap()),
            Some(Pet::try_from(PetName::Ladybug).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_cockroach_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Cockroach).unwrap())], 5).unwrap()
}

pub fn test_duckling_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Duckling).unwrap())], 5).unwrap()
}

pub fn test_kiwi_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Kiwi).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_mouse_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Mouse).unwrap()),
            Some(Pet::try_from(PetName::Ant).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_pillbug_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Pillbug).unwrap()),
            Some(Pet::try_from(PetName::Bluebird).unwrap()),
            Some(Pet::try_from(PetName::Bluebird).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_giraffe_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Giraffe).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_rabbit_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Rabbit).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_emperor_tamarin_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::EmperorTamarin).unwrap())], 5).unwrap()
}

pub fn test_wasp_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Wasp).unwrap())], 5).unwrap()
}

pub fn test_hatching_chick_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::HatchingChick).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_owl_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::Owl).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_puppy_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Puppy).unwrap()),
            Some(Pet::try_from(PetName::Puppy).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_tropicalfish_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Dog).unwrap()),
            Some(Pet::try_from(PetName::TropicalFish).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_capybara_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Capybara).unwrap())], 5).unwrap()
}

pub fn test_cassowary_team() -> Team {
    let mut cassowary = Pet::try_from(PetName::Cassowary).unwrap();
    cassowary.item = Some(Food::try_from(FoodName::Strawberry).unwrap());
    Team::new(&[Some(cassowary)], 5).unwrap()
}

pub fn test_leech_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Peacock).unwrap()),
            Some(Pet::try_from(PetName::Leech).unwrap()),
        ],
        5,
    )
    .unwrap()
}

pub fn test_okapi_team() -> Team {
    Team::new(&[Some(Pet::try_from(PetName::Okapi).unwrap())], 5).unwrap()
}

pub fn test_starfish_team() -> Team {
    Team::new(
        &[
            Some(Pet::try_from(PetName::Duck).unwrap()),
            Some(Pet::try_from(PetName::Starfish).unwrap()),
            Some(Pet::try_from(PetName::Dog).unwrap()),
        ],
        5,
    )
    .unwrap()
}
