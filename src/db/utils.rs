use crate::db::{
    pack::Pack,
    record::{FoodRecord, PetRecord},
};
use rusqlite::{Error, Row};
use std::{fmt::Write, str::FromStr};

pub fn map_row_to_pet(pet_row: &Row) -> Result<PetRecord, Error> {
    let pack: String = pet_row.get(5)?;
    let is_temp_effect_str: String = pet_row.get(11)?;
    Ok(PetRecord {
        name: pet_row.get(1)?,
        tier: pet_row.get(2)?,
        attack: pet_row.get(3)?,
        health: pet_row.get(4)?,
        pack: Pack::from_str(&pack).unwrap(),
        effect_trigger: pet_row.get(6)?,
        effect: pet_row.get(7)?,
        effect_atk: pet_row.get(8)?,
        effect_health: pet_row.get(9)?,
        n_triggers: pet_row.get(10)?,
        temp_effect: is_temp_effect_str == *"true",
        lvl: pet_row.get(12)?,
    })
}

pub fn map_row_to_food(food_row: &Row) -> Result<FoodRecord, Error> {
    let pack: String = food_row.get(4)?;
    let holdable_str: String = food_row.get(5)?;
    let single_use_str: String = food_row.get(6)?;
    let end_of_battle_str: String = food_row.get(7)?;
    let random_str: String = food_row.get(8)?;
    let turn_effect_str: String = food_row.get(12)?;
    Ok(FoodRecord {
        name: food_row.get(1)?,
        tier: food_row.get(2)?,
        effect: food_row.get(3)?,
        pack: Pack::from_str(&pack).unwrap(),
        holdable: holdable_str == *"true",
        single_use: single_use_str == *"true",
        end_of_battle: end_of_battle_str == *"true",
        random: random_str == *"true",
        n_targets: food_row.get(9)?,
        effect_atk: food_row.get(10)?,
        effect_health: food_row.get(11)?,
        turn_effect: turn_effect_str == *"true",
    })
}

/// Dynamically grow SQL statement given params.
pub fn setup_param_query(table: &str, params: &[(&str, &Vec<String>)]) -> String {
    let mut sql_stmt = format!("SELECT * FROM {table} WHERE ");

    // Iterate through params and set up SQL statement.
    // No user param values are inserted.
    for (i, (param_name, param_value)) in params.iter().enumerate() {
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
            let _ = write!(sql_stmt, "{} {} ({})", param_name, sql_in, params_string);
        } else {
            let _ = write!(
                sql_stmt,
                "{} {} ({}) AND ",
                param_name, sql_in, params_string
            );
        }
    }
    sql_stmt
}

#[cfg(test)]
mod test {
    use super::setup_param_query;

    #[test]
    fn test_build_param_query() {
        let name_params = vec!["apple".to_string(), "coconut".to_string()];
        let stmt = setup_param_query("foods", &[("name", &name_params)]);
        assert_eq!("SELECT * FROM foods WHERE name IN (?, ?)", &stmt)
    }

    #[test]
    fn test_build_empty_param_query() {
        let name_params = vec!["apple".to_string(), "coconut".to_string()];
        let pack_params: Vec<String> = vec![];
        let stmt = setup_param_query("foods", &[("name", &name_params), ("pack", &pack_params)]);
        assert_eq!(
            "SELECT * FROM foods WHERE name IN (?, ?) AND pack NOT IN ()",
            &stmt
        )
    }
}
