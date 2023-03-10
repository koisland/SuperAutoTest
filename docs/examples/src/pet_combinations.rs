use itertools::Itertools;
use saptest::{SAPDB, db::{pack::Pack, record::PetRecord}, Pet, Team, Entity};

/// Generate all possible 5-pet combinations from the turtle pack.
pub fn five_pet_combinations() -> Vec<Team> {
    let tiers = vec![1.to_string()];
    let packs = vec![Pack::Turtle.to_string()];
    let turtle_records = SAPDB
        .execute_query(
            Entity::Pet,
            &[("tier", &tiers), ("pack", &packs)]
        )
        .unwrap()
        .into_iter()
        .filter_map(|record| TryInto::<PetRecord>::try_into(record).ok())
        .collect_vec();

    // Combine with itertools to generate all pet combinations.
    let pet_comb = turtle_records.into_iter().combinations(5);

    // Create a team for each pet combination.
    // Note: Pets and Teams are not currently thread-safe. In the future this may change.
    let mut teams = vec![];

    for comb in pet_comb {
        let pets: Vec<Option<Pet>> = comb.into_iter().map(|pet| Some(pet.try_into().unwrap())).collect_vec();
        let team = Team::new(&pets, 5).unwrap();
        teams.push(team)
    }

    teams
}
