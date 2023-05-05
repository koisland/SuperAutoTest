use saptest::{FoodName, Pet, PetName, Statistics, Team, TeamShopping, TeamViewer};

/// Demonstrating how to get permanent [`Coconut`](saptest::FoodName::Coconut) using the [`Leech`](saptest::PetName::Leech), [`Gorilla`](saptest::PetName::Gorilla) and [`Parrot`](saptest::PetName::Parrot).
/// * Shop mechanics and effect order.
/// * https://superautopets.fandom.com/f/p/4400000000000044254
/// * The [`Parrot`](saptest::PetName::Parrot) must have higher attack than the [`Leech`](saptest::PetName::Leech) in order for this to work.
pub fn permanent_coconut() -> Team {
    let pets = [
        Some(Pet::try_from(PetName::Seagull).unwrap()),
        Some(Pet::try_from(PetName::Gorilla).unwrap()),
        Some(Pet::new(PetName::Parrot, Some(Statistics::new(5, 3).unwrap()), 1).unwrap()),
        Some(Pet::try_from(PetName::Leech).unwrap()),
    ];
    let mut team = Team::new(&pets, 5).unwrap();
    // Trigger end of turn.
    team.open_shop().unwrap().close_shop().unwrap();

    let parrot = team.nth(2).unwrap();
    assert_eq!(
        parrot.read().unwrap().item.as_ref().unwrap().name,
        FoodName::Coconut
    );
    team
}
