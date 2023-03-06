use saptest::{FoodName, Pet, PetName, Statistics, Team, TeamShopping, TeamViewer};


pub fn permanent_coconut() -> Team {
    // This is an forum post on how to get permanent coconut.
    // https://superautopets.fandom.com/f/p/4400000000000044254
    // It relies on the order of pets on ending turn.
    // The parrot must have higher attack than the leech in order for this to work.
    let pets = [
        Some(Pet::try_from(PetName::Seagull).unwrap()),
        Some(Pet::try_from(PetName::Gorilla).unwrap()),
        Some(
            Pet::new(
                PetName::Parrot,
                Some(Statistics::new(5, 3).unwrap()),
                1,
            )
            .unwrap(),
        ),
        Some(Pet::try_from(PetName::Leech).unwrap()),
    ];
    let mut team = Team::new(&pets, 5).unwrap();
    // Trigger end of turn.
    team.open_shop().unwrap().close_shop().unwrap();

    let parrot = team.nth(2).unwrap();
    assert_eq!(
        parrot.borrow().item.as_ref().unwrap().name,
        FoodName::Coconut
    );
    team
}
