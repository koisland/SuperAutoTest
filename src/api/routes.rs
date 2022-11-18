use crate::{api::server::SapDB, common::food::Food, common::game::Pack, common::pet::Pet};
use rocket::response::content::RawJson;
use rusqlite::{Error, Row};
use serde_json::to_string_pretty;

const QUERY_FOOD_PARAMS: [&str; 3] = ["name", "pack", "tier"];
const QUERY_PET_PARAMS: [&str; 5] = ["name", "tier", "lvl", "pack", "effect_trigger"];

// TODO: Add html doc to show basic routes.
#[get("/")]
pub fn index() -> &'static str {
    "Welcome to the unoffical Super Auto Pets API!"
}

pub fn capitalize_names(name: &str) -> String {
    let cap_name: String = name
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect();
    cap_name
}

pub fn map_row_to_pet(pet_row: &Row) -> Result<Pet, Error> {
    let pack: String = pet_row.get(5)?;
    Ok(Pet {
        name: pet_row.get(1)?,
        tier: pet_row.get(2)?,
        attack: pet_row.get(3)?,
        health: pet_row.get(4)?,
        pack: Pack::new(&pack),
        effect_trigger: pet_row.get(6)?,
        effect: pet_row.get(7)?,
        lvl: pet_row.get(8)?,
    })
}

pub fn map_row_to_food(food_row: &Row) -> Result<Food, Error> {
    let pack: String = food_row.get(4)?;
    Ok(Food {
        name: food_row.get(1)?,
        tier: food_row.get(2)?,
        effect: food_row.get(3)?,
        pack: Pack::new(&pack),
    })
}

pub fn setup_param_query(table: &str, params: &[String], param_names: &[&str]) -> String {
    let mut sql_stmt = format!("SELECT * FROM {table} WHERE ");

    // Check that params and length are equivalent.
    assert!(params.len() == param_names.len(), "Length of params and their names is different.");
    // Check that at least one param given
    assert!(params.len() >= 1, "Less than one param given. Will result in malformed SQL.");

    // Iterate through params and set up SQL statement.
    // No user param values are inserted.
    for (i, (param_name, param_value)) in param_names.iter().zip(params).enumerate() {
        // If value is empty, use not in to get all other values.
        let sql_in = if param_value.is_empty() {
            "NOT IN"
        } else {
            "IN"
        };
        if i + 1 == params.len() {
            sql_stmt.push_str(&format!("{param_name} {sql_in} (?{})", i + 1))
        } else {
            sql_stmt.push_str(&format!("{param_name} {sql_in} (?{}) AND ", i + 1))
        }
    }
    sql_stmt
}

#[get("/pet?<name>&<level>&<tier>&<pack>&<effect_trigger>")]
pub async fn pets(
    conn: SapDB,
    name: Option<&str>,
    level: Option<u8>,
    tier: Option<u8>,
    pack: Option<&str>,
    effect_trigger: Option<&str>,
) -> Option<RawJson<String>> {
    // Set defaults if no param given.
    let pet_name = name.map_or("".to_string(), capitalize_names);
    let pet_tier = tier.map_or("".to_string(), |tier| tier.to_string());
    let pet_level = level.map_or("".to_string(), |lvl| lvl.to_string());
    let pack_name = pack.map_or("".to_string(), capitalize_names);
    let effect_trigger_name = effect_trigger.map_or("".to_string(), |effect_trigger| {
        capitalize_names(effect_trigger)
    });

    let sql_params: [String; 5] = [
        pet_name,
        pet_tier,
        pet_level,
        pack_name,
        effect_trigger_name,
    ];
    let sql_stmt = setup_param_query("pets", &sql_params, &QUERY_PET_PARAMS);

    let query: Result<Vec<Pet>, Error> = conn
        .run(move |c| {
            let mut pet_stmt = c.prepare(&sql_stmt).unwrap();
            let mut pets: Vec<Pet> = vec![];

            if let Ok(mut pets_found) = pet_stmt.query(sql_params) {
                while let Some(pet_row) = pets_found.next().unwrap_or(None) {
                    let pet = map_row_to_pet(pet_row)?;
                    pets.push(pet);
                }
            }
            Ok(pets)
        })
        .await;
    if let Ok(res) = query {
        Some(RawJson(to_string_pretty(&res).unwrap()))
    } else {
        None
    }
}

#[get("/food?<name>&<tier>&<pack>")]
pub async fn foods(
    conn: SapDB,
    name: Option<&str>,
    tier: Option<u8>,
    pack: Option<&str>,
) -> Option<RawJson<String>> {
    let food_name = name.map_or("".to_string(), capitalize_names);
    let pack_name = pack.map_or("".to_string(), capitalize_names);
    let tier_name = tier.map_or("".to_string(), |tier| tier.to_string());

    let sql_params: [String; 3] = [food_name, pack_name, tier_name];
    let sql_stmt = setup_param_query("foods", &sql_params, &QUERY_FOOD_PARAMS);

    let query: Result<Vec<Food>, Error> = conn
        .run(move |c| {
            let mut food_stmt = c.prepare(&sql_stmt).unwrap();
            let mut foods: Vec<Food> = vec![];

            if let Ok(mut foods_found) = food_stmt.query(sql_params) {
                while let Some(food_row) = foods_found.next().unwrap_or(None) {
                    let food = map_row_to_food(food_row)?;
                    foods.push(food);
                }
            }
            Ok(foods)
        })
        .await;
    if let Ok(res) = query {
        Some(RawJson(to_string_pretty(&res).unwrap()))
    } else {
        None
    }
}
