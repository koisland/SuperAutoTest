use saptest::{
    teams::combat::TeamCombat, Food, FoodName, Pet, PetName, Statistics, Team, TeamViewer,
};

fn rhino_vs_summoner_teams() -> (Team, Team) {
    // https://www.reddit.com/r/superautopets/comments/u06kr7/not_sure_if_rhino_good_or_he_just_got_way_too_far/
    // Tie in attack between shark and cricket causes difference in bee spawn.
    // * Shark wins in this case.
    let sour_sailors_pets = [
        Some(
            Pet::new(
                PetName::Rhino,
                Some(Statistics {
                    attack: 19,
                    health: 21,
                }),
                2,
            )
            .unwrap(),
        ),
        Some(
            Pet::new(
                PetName::Otter,
                Some(Statistics {
                    attack: 19,
                    health: 20,
                }),
                3,
            )
            .unwrap(),
        ),
        Some(
            Pet::new(
                PetName::Bison,
                Some(Statistics {
                    attack: 45,
                    health: 48,
                }),
                2,
            )
            .unwrap(),
        ),
        Some(
            Pet::new(
                PetName::Swan,
                Some(Statistics {
                    attack: 16,
                    health: 18,
                }),
                2,
            )
            .unwrap(),
        ),
        Some(
            Pet::new(
                PetName::Penguin,
                Some(Statistics {
                    attack: 9,
                    health: 9,
                }),
                3,
            )
            .unwrap(),
        ),
    ];
    let mut chunky_wigs_pets = [
        Some(
            Pet::new(
                PetName::Deer,
                Some(Statistics {
                    attack: 9,
                    health: 9,
                }),
                2,
            )
            .unwrap(),
        ),
        Some(
            Pet::new(
                PetName::Cricket,
                Some(Statistics {
                    attack: 6,
                    health: 7,
                }),
                3,
            )
            .unwrap(),
        ),
        Some(
            Pet::new(
                PetName::Sheep,
                Some(Statistics {
                    attack: 4,
                    health: 4,
                }),
                2,
            )
            .unwrap(),
        ),
        Some(
            Pet::new(
                PetName::Turkey,
                Some(Statistics {
                    attack: 5,
                    health: 6,
                }),
                2,
            )
            .unwrap(),
        ),
        Some(
            Pet::new(
                PetName::Shark,
                Some(Statistics {
                    attack: 6,
                    health: 6,
                }),
                1,
            )
            .unwrap(),
        ),
    ];
    for i in [1, 2, 4] {
        chunky_wigs_pets[i].as_mut().unwrap().item = Some(Food::try_from(FoodName::Honey).unwrap())
    }
    chunky_wigs_pets[3].as_mut().unwrap().item = Some(Food::try_from(FoodName::Steak).unwrap());

    (
        Team::new(&sour_sailors_pets, 5).unwrap(),
        Team::new(&chunky_wigs_pets, 5).unwrap(),
    )
}

pub fn rhino() -> Team {
    // https://www.reddit.com/r/superautopets/comments/u06kr7/not_sure_if_rhino_good_or_he_just_got_way_too_far/
    let (mut sour_sailors, mut chunk_wigs) = rhino_vs_summoner_teams();
    // sour_sailors.set_name("The Sour Sailors").unwrap();
    // chunk_wigs.set_name("The Chunk Wigs").unwrap();

    chunk_wigs.fight(&mut sour_sailors).unwrap();

    let chunky_wigs_post_battle_pets = chunk_wigs.all();
    let [bus, bee, zcricket, ram_1, ram_2] = &chunky_wigs_post_battle_pets[..] else { panic!()};
    assert!(
        bus.borrow().stats == Statistics::new(10, 10).unwrap()
            && bee.borrow().stats == Statistics::new(1, 1).unwrap()
            && zcricket.borrow().stats == Statistics::new(3, 3).unwrap()
            && ram_1.borrow().stats == Statistics::new(4, 4).unwrap()
            && ram_2.borrow().stats == Statistics::new(4, 4).unwrap()
    );
    chunk_wigs
}
