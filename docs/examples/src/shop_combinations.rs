use std::io::Write;
use itertools::Itertools;
use saptest::{Shop, Team, TeamShopping, db::{record::PetRecord, pack::Pack}, SAPDB, Entity, Pet, PetName};


#[test]
pub fn generate_possible_t1_teams() {
    let tiers = vec![1.to_string()];
    let levels = vec![1.to_string()];
    let packs = vec![Pack::Turtle.to_string()];
    let mut turtle_pet_records = SAPDB
        .execute_query(
            Entity::Pet,
            &[("tier", &tiers), ("pack", &packs), ("lvl", &levels)]
        )
        .unwrap()
        .into_iter()
        .map(|record| TryInto::<PetRecord>::try_into(record).ok())
        .filter(|record| record.as_ref().map(|record| &record.name) != Some(&PetName::Sloth))
        .collect_vec();

    // Include combinations with empty slots.
    turtle_pet_records.push(None);

    // Calculate chunk size.
    // Combine with itertools to generate all pet combinations.
    let pet_comb = turtle_pet_records.into_iter().combinations_with_replacement(3).chunks(2_000);

    // Create a team for each pet combination.
    let mut handles = vec![];
    for chunk in &pet_comb {
        // Chunk object not thread-safe so collect into vec.
        let chunk = chunk.into_iter().collect_vec();
        println!("{}", chunk.len());

        handles.push(
            std::thread::spawn(|| {
                let mut teams: Vec<Team> = vec![];
                for comb in chunk.into_iter() {
                    let pets: Vec<Option<Pet>> = comb.into_iter().map(|pet| pet.map(|record| record.try_into().unwrap())).collect_vec();
                    let team = Team::new(&pets, 5).unwrap();    
                    teams.push(team)
                }
                teams
            })
        )
    }
    let res = handles.into_iter().filter_map(|handle| handle.join().ok()).flatten().collect_vec();

    if let Ok(mut file) = std::fs::File::create("t1_team.txt") {
        for team in res.iter() {
            file.write(team.to_string().as_bytes()).unwrap();
            file.write(&[b'\n']).unwrap();
        }
    }
}