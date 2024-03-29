use itertools::Itertools;
use saptest::{
    db::{pack::Pack, record::PetRecord},
    Entity, Pet, Team, SAPDB,
};

pub fn factorial(n: u128) -> u128 {
    (1..=n).product()
}
fn combinations(n: u128, choose: u128) -> u128 {
    factorial(n) / (factorial(choose) * factorial(n - choose))
}
// fn permutations(n: u128, choose: u128) -> u128 {
//     factorial(n) / factorial(n-choose)
// }

pub fn five_pet_combinations() {
    let mut query = saptest::SAPQuery::builder();
    query
        .set_table(Entity::Pet)
        .set_param("tier", vec![1, 2, 3])
        .set_param("lvl", vec![1])
        .set_param("pack", vec![Pack::Turtle]);

    let mut turtle_records = SAPDB
        .execute_query(query)
        .unwrap()
        .into_iter()
        .map(|record| TryInto::<PetRecord>::try_into(record).ok())
        .collect_vec();

    // Include combinations with two slots.
    turtle_records.extend([None, None]);

    let n = turtle_records.len().try_into().unwrap();
    let choose = 5;

    let n_comb = combinations(n, choose);
    // let n_perm = permutations(n, choose);

    // Calculate chunk size.
    let n_threads = 8;
    let chunk_size = n_comb / n_threads;
    // Combine with itertools to generate all pet combinations.
    let pet_comb = turtle_records
        .into_iter()
        .combinations(5)
        .chunks(chunk_size.try_into().unwrap());

    // Create a team for each pet combination.
    let mut handles = vec![];
    for chunk in &pet_comb {
        // Chunk object not thread-safe so collect into vec.
        let chunk = chunk.into_iter().collect_vec();
        println!("{}", chunk.len());

        handles.push(std::thread::spawn(|| {
            let mut teams: Vec<Team> = vec![];
            for comb in chunk.into_iter() {
                let pets: Vec<Option<Pet>> = comb
                    .into_iter()
                    .map(|pet| pet.map(|rec| rec.try_into().unwrap()))
                    .collect_vec();
                let team = Team::new(&pets, 5).unwrap();
                teams.push(team)
            }
            teams
        }))
    }
    let res = handles
        .into_iter()
        .filter_map(|handle| handle.join().ok())
        .flatten()
        .collect_vec();

    println!("{}", res.len())
}
