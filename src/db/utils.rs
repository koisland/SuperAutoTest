use crate::{
    common::{food::FoodRecord, game::Pack, pet::PetRecord},
    wiki_scraper::parser::{parse_pet_info, read_wiki_url},
};
use log::error;
use rusqlite::{Error, Row};
use serde_json::to_writer_pretty;
use std::fs::File;

#[allow(dead_code)]
fn write_pet_info(output: &str) {
    if let Ok(wiki_urls) = read_wiki_url(crate::SCRAPER_SOURCES) {
        let res = parse_pet_info(&wiki_urls.pets);
        if let Ok(all_pets) = res {
            let file = File::create(output).expect("Can't create file.");
            to_writer_pretty(file, &all_pets).expect("Unable to serialize pet info.");
        } else {
            error!(target: "scraper", "{:?}", res.unwrap_err())
        }
    }
}

pub fn map_row_to_pet(pet_row: &Row) -> Result<PetRecord, Error> {
    let pack: String = pet_row.get(5)?;
    Ok(PetRecord {
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

pub fn map_row_to_food(food_row: &Row) -> Result<FoodRecord, Error> {
    let pack: String = food_row.get(4)?;
    Ok(FoodRecord {
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
