use crate::{api::server::SapDB, common::food::Food, common::game::Pack, common::pet::Pet};
use itertools::Itertools;
use rocket::response::content::RawJson;
use rusqlite::{Error, Row};
use serde_json::to_string_pretty;

const QUERY_FOOD_PARAMS: [&str; 3] = ["name", "pack", "tier"];
const QUERY_PET_PARAMS: [&str; 5] = ["name", "tier", "lvl", "pack", "effect_trigger"];

// TODO: Add static html doc to show basic routes.
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

/// Dynamically grow SQL statement given params.
pub fn setup_param_query(table: &str, params: &[Vec<String>], param_names: &[&str]) -> String {
    let mut sql_stmt = format!("SELECT * FROM {table} WHERE ");

    // Check that params and length are equivalent.
    assert!(
        params.len() == param_names.len(),
        "Length of params and their names is different."
    );
    // Check that at least one param given
    assert!(
        !params.is_empty(),
        "Less than one param given. Will result in malformed SQL."
    );

    // Iterate through params and set up SQL statement.
    // No user param values are inserted.
    for (i, (param_name, param_value)) in param_names.iter().zip(params).enumerate() {
        // If value is empty, use NOT IN to get all other values.
        let sql_in = if param_value.iter().all(|param| param.is_empty()) {
            "NOT IN"
        } else {
            "IN"
        };
        // Set number of query params.
        let n_elems = param_value.len();
        let params_string = vec!["?"; n_elems].join(", ");

        // If at end of params, don't include AND.
        if i + 1 == params.len() {
            sql_stmt.push_str(&format!("{param_name} {sql_in} ({})", params_string))
        } else {
            sql_stmt.push_str(&format!("{param_name} {sql_in} ({}) AND ", params_string))
        }
    }
    sql_stmt
}

#[get("/pet?<name>&<level>&<tier>&<pack>&<effect_trigger>")]
pub async fn pets(
    conn: SapDB,
    name: Option<Vec<&str>>,
    level: Option<Vec<u8>>,
    tier: Option<Vec<u8>>,
    pack: Option<Vec<&str>>,
    effect_trigger: Option<Vec<&str>>,
) -> Option<RawJson<String>> {
    // Set defaults if no param given.
    let pet_name = name.map_or(vec![], |names| {
        names
            .iter()
            .map(|name| capitalize_names(name))
            .collect_vec()
    });
    let pet_tier = tier.map_or(vec![], |tiers| {
        tiers.iter().map(|tier| tier.to_string()).collect_vec()
    });
    let pet_level = level.map_or(vec![], |lvls| {
        lvls.iter().map(|lvl| lvl.to_string()).collect_vec()
    });
    let pack_name = pack.map_or(vec![], |packs| {
        packs
            .iter()
            .map(|pack| capitalize_names(pack))
            .collect_vec()
    });
    let effect_trigger_name = effect_trigger.map_or(vec![], |effect_triggers| {
        effect_triggers
            .iter()
            .map(|trigger| capitalize_names(trigger))
            .collect_vec()
    });

    let sql_params: [Vec<String>; 5] = [
        pet_name,
        pet_tier,
        pet_level,
        pack_name,
        effect_trigger_name,
    ];

    let sql_stmt = setup_param_query("pets", &sql_params, &QUERY_PET_PARAMS);
    let flat_sql_params: Vec<String> = sql_params.into_iter().flatten().collect_vec();

    let query: Result<Vec<Pet>, Error> = conn
        .run(move |c| {
            let mut pet_stmt = c.prepare(&sql_stmt).unwrap();
            let mut pets: Vec<Pet> = vec![];

            if let Ok(mut pets_found) = pet_stmt.query(rusqlite::params_from_iter(flat_sql_params))
            {
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
    name: Option<Vec<&str>>,
    tier: Option<Vec<u8>>,
    pack: Option<Vec<&str>>,
) -> Option<RawJson<String>> {
    let food_name = name.map_or(vec![], |food_names| {
        food_names
            .iter()
            .map(|name| capitalize_names(name))
            .collect_vec()
    });
    let pack_name = pack.map_or(vec![], |pack_names| {
        pack_names
            .iter()
            .map(|name| capitalize_names(name))
            .collect_vec()
    });
    let tier_name = tier.map_or(vec![], |tiers| {
        tiers.iter().map(|tier| tier.to_string()).collect_vec()
    });

    let sql_params: [Vec<String>; 3] = [food_name, pack_name, tier_name];
    let sql_stmt = setup_param_query("foods", &sql_params, &QUERY_FOOD_PARAMS);
    let flat_sql_params: Vec<String> = sql_params.into_iter().flatten().collect_vec();

    let query: Result<Vec<Food>, Error> = conn
        .run(move |c| {
            let mut food_stmt = c.prepare(&sql_stmt).unwrap();
            let mut foods: Vec<Food> = vec![];

            if let Ok(mut foods_found) =
                food_stmt.query(rusqlite::params_from_iter(flat_sql_params))
            {
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
