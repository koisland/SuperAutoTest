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

pub fn test_cricket_horse_team() -> Team {
    Team::new(
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

pub fn test_mosq_team() -> Team {
    Team::new(
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

pub fn test_frilled_dragon_team() -> Team {
    Team::new(
        &[
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::FrilledDragon)),
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
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Frog)),
            Some(Pet::from(PetName::FrilledDragon)),
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
            Some(Pet::from(PetName::Duck)),
            Some(Pet::from(PetName::Moth)),
            Some(Pet::from(PetName::Moth)),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_hummingbird_team() -> Team {
    let mut duck_w_strawberry = Pet::from(PetName::Duck);
    duck_w_strawberry.item = Some(Food::from(FoodName::Strawberry));

    Team::new(
        &[
            Some(duck_w_strawberry),
            Some(Pet::from(PetName::Hummingbird)),
            Some(Pet::from(PetName::Hummingbird)),
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
            Some(Pet::from(PetName::Seahorse)),
            Some(Pet::from(PetName::Iguana)),
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

pub fn test_elephant_peacock_team() -> Team {
    Team::new(
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

pub fn test_dodo_team() -> Team {
    Team::new(
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

pub fn test_flamingo_team() -> Team {
    Team::new(
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

pub fn test_rat_team(lvl: usize) -> Team {
    let mut rat = Pet::from(PetName::Rat);
    if let Err(error) = rat.set_level(lvl) {
        println!("{:?}", error)
    };

    let pets = [Some(rat), None, None, None, None];
    Team::new(&pets, 5).unwrap()
}

pub fn test_spider_team() -> Team {
    Team::new(
        &[Some(Pet::from(PetName::Spider)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_bat_team() -> Team {
    Team::new(&[Some(Pet::from(PetName::Bat)), None, None, None, None], 5).unwrap()
}

pub fn test_atlantic_puffin_team() -> Team {
    let mut strawberry_ant = Pet::from(PetName::Ant);
    strawberry_ant.item = Some(Food::from(FoodName::Strawberry));

    Team::new(
        &[
            Some(Pet::from(PetName::AtlanticPuffin)),
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
    let mut strawberry_ant = Pet::from(PetName::Ant);
    strawberry_ant.item = Some(Food::from(FoodName::Strawberry));

    Team::new(
        &[
            Some(Pet::from(PetName::Dove)),
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
            Some(Pet::from(PetName::Mammoth)),
            Some(Pet::from(PetName::Koala)),
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
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Panda)),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_pug_team() -> Team {
    let mut exp_1_ant = Pet::from(PetName::Ant);
    exp_1_ant.add_experience(1).unwrap();

    Team::new(
        &[
            Some(exp_1_ant),
            Some(Pet::from(PetName::Pug)),
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
        &[Some(Pet::from(PetName::Stork)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_racoon_team() -> Team {
    Team::new(
        &[Some(Pet::from(PetName::Racoon)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_toucan_team() -> Team {
    let mut toucan = Pet::from(PetName::Toucan);
    toucan.item = Some(Food::from(FoodName::Honey));
    Team::new(
        &[
            Some(toucan),
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Dog)),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_wombat_team() -> Team {
    Team::new(
        &[Some(Pet::from(PetName::Wombat)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_crab_team() -> Team {
    let mut big_ant = Pet::from(PetName::Ant);
    big_ant.stats.health = 50;
    Team::new(
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

pub fn test_badger_team() -> Team {
    Team::new(
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

pub fn test_blowfish_team() -> Team {
    Team::new(
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

pub fn test_camel_team() -> Team {
    Team::new(
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

pub fn test_dog_team() -> Team {
    Team::new(
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

pub fn test_dolphin_team() -> Team {
    Team::new(
        &[Some(Pet::from(PetName::Dolphin)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_kangaroo_team() -> Team {
    Team::new(
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

pub fn test_ox_team() -> Team {
    Team::new(
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

pub fn test_sheep_team() -> Team {
    Team::new(
        &[Some(Pet::from(PetName::Sheep)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_filled_sheep_team() -> Team {
    Team::new(
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
        &[Some(Pet::from(PetName::Aardvark)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_bear_team() -> Team {
    Team::new(
        &[
            Some(Pet::from(PetName::Bear)),
            Some(Pet::from(PetName::Dog)),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_seagull_team() -> Team {
    let mut seagull = Pet::from(PetName::Seagull);
    seagull.item = Some(Food::from(FoodName::Honey));
    Team::new(
        &[
            Some(Pet::from(PetName::Cricket)),
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
            Some(Pet::from(PetName::Blobfish)),
            Some(Pet::from(PetName::Dog)),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_clownfish_team() -> Team {
    let mut dog_w_exp = Pet::from(PetName::Dog);
    dog_w_exp.add_experience(1).unwrap();
    Team::new(
        &[
            Some(Pet::from(PetName::Blobfish)),
            Some(dog_w_exp),
            Some(Pet::from(PetName::Clownfish)),
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
            Some(Pet::from(PetName::Toad)),
            Some(Pet::from(PetName::Mosquito)),
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
        &[Some(Pet::from(PetName::Woodpecker)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_woodpecker_self_hurt_team() -> Team {
    Team::new(
        &[
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Cricket)),
            Some(Pet::from(PetName::Woodpecker)),
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_deer_team() -> Team {
    Team::new(&[Some(Pet::from(PetName::Deer)), None, None, None, None], 5).unwrap()
}

pub fn test_hippo_team() -> Team {
    Team::new(
        &[Some(Pet::from(PetName::Hippo)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_parrot_team() -> Team {
    let mut cricket = Pet::from(PetName::Cricket);
    cricket.set_level(2).unwrap();
    Team::new(
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

pub fn test_rooster_team() -> Team {
    Team::new(
        &[Some(Pet::from(PetName::Rooster)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_skunk_team() -> Team {
    Team::new(
        &[Some(Pet::from(PetName::Skunk)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_turtle_team() -> Team {
    Team::new(
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

pub fn test_whale_team() -> Team {
    Team::new(
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

pub fn test_armadillo_team() -> Team {
    Team::new(
        &[
            Some(Pet::from(PetName::Armadillo)),
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Dog)),
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_doberman_team() -> Team {
    Team::new(
        &[
            Some(Pet::from(PetName::Doberman)),
            Some(Pet::from(PetName::Mammoth)),
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
            Some(Pet::from(PetName::Doberman)),
            Some(Pet::from(PetName::Ant)),
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
            Some(Pet::from(PetName::Lynx)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
            Some(Pet::from(PetName::Ant)),
        ],
        5,
    )
    .unwrap()
}

pub fn test_porcupine_team() -> Team {
    Team::new(
        &[
            Some(Pet::from(PetName::Doberman)),
            Some(Pet::from(PetName::Ant)),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_caterpillar_team() -> Team {
    Team::new(
        &[
            Some(Pet::from(PetName::Caterpillar)),
            None,
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
        &[Some(Pet::from(PetName::Anteater)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_donkey_team() -> Team {
    Team::new(
        &[
            Some(Pet::from(PetName::Dog)),
            Some(Pet::from(PetName::Donkey)),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_eel_team() -> Team {
    Team::new(&[Some(Pet::from(PetName::Eel)), None, None, None, None], 5).unwrap()
}

pub fn test_hawk_team() -> Team {
    Team::new(
        &[
            Some(Pet::from(PetName::Hawk)),
            Some(Pet::from(PetName::Ant)),
            None,
            None,
            None,
        ],
        5,
    )
    .unwrap()
}

pub fn test_pelican_team() -> Team {
    let mut strawberry_ant = Pet::from(PetName::Ant);
    strawberry_ant.item = Some(Food::from(FoodName::Strawberry));

    Team::new(
        &[
            Some(Pet::from(PetName::Pelican)),
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

pub fn test_rhino_team() -> Team {
    Team::new(
        &[Some(Pet::from(PetName::Rhino)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_scorpion_team() -> Team {
    Team::new(
        &[
            Some(Pet::from(PetName::Scorpion)),
            Some(Pet::from(PetName::Scorpion)),
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

pub fn test_turkey_team() -> Team {
    Team::new(
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

pub fn test_boar_team() -> Team {
    Team::new(&[Some(Pet::from(PetName::Boar)), None, None, None, None], 5).unwrap()
}

pub fn test_fly_team() -> Team {
    Team::new(
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

pub fn test_gorilla_team() -> Team {
    Team::new(
        &[Some(Pet::from(PetName::Gorilla)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_leopard_team() -> Team {
    Team::new(
        &[Some(Pet::from(PetName::Leopard)), None, None, None, None],
        5,
    )
    .unwrap()
}

pub fn test_mammoth_team() -> Team {
    Team::new(
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

pub fn test_snake_team() -> Team {
    Team::new(
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

pub fn test_tiger_team() -> Team {
    Team::new(
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
