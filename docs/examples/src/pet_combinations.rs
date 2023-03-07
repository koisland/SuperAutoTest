use itertools::Itertools;
use saptest::{SAPDB, db::pack::Pack, Pet, Team};

/// Generate all possible 5-pet combinations from the turtle pack.
pub fn five_pet_combinations() -> Vec<Team> {
    let turtle_records = SAPDB.execute_pet_query(
        "SELECT * FROM pets WHERE lvl = ? AND pack = ?",
        &[1.to_string(), Pack::Turtle.to_string()]
    ).unwrap();

    // Combine with itertools to generate all pet combinations.
    let pet_comb = turtle_records.into_iter().combinations(5);

    // Create a team for each pet combination.
    // Note: Pets and Teams are not currently thread-safe. In the future this may change.
    let mut teams = vec![];
    for comb in pet_comb.into_iter() {
        let pets: Vec<Option<Pet>> = comb.into_iter().map(|pet| Some(pet.try_into().unwrap())).collect_vec();
        let team = Team::new(&pets, 5).unwrap();
        teams.push(team)
    }

    teams
}
