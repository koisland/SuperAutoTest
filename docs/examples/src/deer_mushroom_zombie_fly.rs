use saptest::{Food, FoodName, Pet, PetName, Team, TeamCombat, TeamViewer};

/// Summon ordering. Pet effects activate first then food item effects.
/// * https://youtu.be/NSqjuA32AoA?t=458
pub fn deer_fly_mushroom() -> Team {
    let mut deer_w_mush = Pet::try_from(PetName::Deer).unwrap();
    deer_w_mush.item = Some(Food::try_from(FoodName::Mushroom).unwrap());
    let pets = [
        Some(deer_w_mush),
        Some(Pet::try_from(PetName::Fly).unwrap()),
        Some(Pet::try_from(PetName::Shark).unwrap()),
    ];

    let mut team = Team::new(&pets, 5).unwrap();
    let mut enemy_team = Team::new(&pets, 5).unwrap();

    team.fight(&mut enemy_team).unwrap();

    // Correct spawn order.
    assert!(
        team.first().unwrap().borrow().name == PetName::Deer
            && team.nth(1).unwrap().borrow().name == PetName::ZombieFly
            && team.nth(2).unwrap().borrow().name == PetName::Bus
    );
    team
}
