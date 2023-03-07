use saptest::{Food, FoodName, Pet, PetName, Team, TeamCombat, TeamViewer};


/// Classic [`Mushroom`](saptest::FoodName::Mushroom)ed [`Scorpion`](saptest::PetName::Scorpion) tactic.
/// * https://youtu.be/NSqjuA32AoA?t=149
/// * Scorpion spawns with [`Peanut`](saptest::FoodName::Peanut) on summon with [`Mushroom`](saptest::FoodName::Mushroom).
pub fn mushroom_scorpion() -> Team {
    let mut scorpion = Pet::try_from(PetName::Scorpion).unwrap();
    scorpion.item = Some(Food::try_from(FoodName::Mushroom).unwrap());
    let mut gorilla = Pet::try_from(PetName::Gorilla).unwrap();
    gorilla.item = Some(Food::try_from(FoodName::Melon).unwrap());

    let mut team = Team::new(
        &[Some(scorpion), Some(Pet::try_from(PetName::Dog).unwrap())],
        5,
    )
    .unwrap();
    let mut enemy_team = Team::new(&[Some(gorilla)], 5).unwrap();

    // Mushroom scorpion attacks gorilla.
    team.fight(&mut enemy_team).unwrap();

    let respawned_scorpion = team.first().unwrap();
    // Gain peanut on respawn.
    assert_eq!(
        respawned_scorpion.borrow().item.as_ref().unwrap().name,
        FoodName::Peanut
    );
    team
}
