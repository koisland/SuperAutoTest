use crate::{
    api::server::SAPDB,
    common::{pet::Pet, game::Pack}
};
use serde_json::to_string_pretty;
use itertools::Itertools;
use rocket_contrib::json::Json;


// TODO: Add html doc to show basic routes.
#[get("/")]
pub fn index() -> &'static str {
    "Welcome to the unoffical Super Auto Pets API!"
}

#[get("/pet/<name>")]
pub async fn pets(conn: SAPDB, name: &str) -> String {
    // Capitalize first character of name.
    // TODO: Also needs to work for names with two words. ex. Tabby Cat
    let cap_name: String = name
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 {c.to_ascii_uppercase()} else {c} )
        .collect();
    
    let query: Option<Pet> = conn.run(move |c| {
        let mut pack_stmt = c.prepare("SELECT DISTINCT pack FROM pets WHERE name == ?1").unwrap();
        let mut pet_stmt = c.prepare("SELECT * FROM pets WHERE name = ?1").unwrap();

        // If no packs found, empty vector is allowed.
        let mut pet_packs: Vec<Pack> = vec![];
        if let Ok(mut packs_found) = pack_stmt.query([&cap_name]) {
            while let Some(pack_found) = packs_found.next().unwrap_or(None) {
                let pack: String = pack_found.get(0).unwrap();
                pet_packs.push(Pack::new(&pack))
            }
        }

        if let Ok(mut pets_found) = pet_stmt.query([&cap_name]) {
            if let Some(pet_row) = pets_found.next().unwrap_or(None) {
                // Pet effects are 7-9 cols. TODO: Make sep query?
                let pet_effects = (7..=9)
                    .map(|col_n| pet_row.get(col_n).unwrap_or("None".to_string()))
                    .collect_vec();
                let tier: usize = pet_row.get(2).unwrap_or(0);
                let attack: usize = pet_row.get(3).unwrap_or(0);
                let health: usize = pet_row.get(4).unwrap_or(0);
                let effect_trigger: String = pet_row.get(6).unwrap_or("None".to_string());

                return Some(Pet::new(
                    &cap_name,
                    tier,
                    attack,
                    health,
                    &pet_packs,
                    &effect_trigger,
                    &pet_effects
                ));
            }
        }
        // Return None if any query or conversion fails.
        None
    }).await;
    
    if let Some(pet) = query {
        to_string_pretty(&pet).unwrap_or("{}".to_string())
    } else {
        "{}".to_string()
    }
}

#[get("/food/<name>")]
pub async fn foods(conn: SAPDB, name: &str) -> String {
    let response = format!("{{ \"pet\": \"{name}\" }}");

    Json(response).to_string()
}
