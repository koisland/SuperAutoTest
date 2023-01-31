use crate::{
    db::{
        record::{FoodRecord, PetRecord},
        utils::{map_row_to_food, map_row_to_pet},
    },
    wiki_scraper::{
        parse_food::parse_food_info, parse_pet::parse_pet_info, parse_tokens::parse_token_info,
    },
};
use log::info;
use rusqlite::Connection;
use std::error::Error;

/// Query pets generating a list of [`PetRecord`]s.
pub fn query_pet(
    conn: &Connection,
    sql: &str,
    params: &[String],
) -> Result<Vec<PetRecord>, rusqlite::Error> {
    let mut pet_stmt = conn.prepare(sql)?;
    let mut pets: Vec<PetRecord> = vec![];

    if let Ok(mut pets_found) = pet_stmt.query(rusqlite::params_from_iter(params)) {
        while let Some(pet_row) = pets_found.next().unwrap_or(None) {
            let pet = map_row_to_pet(pet_row)?;
            pets.push(pet);
        }
    }
    Ok(pets)
}

/// Query foods generating a list of [`FoodRecord`]s.
pub fn query_food(
    conn: &Connection,
    sql: &str,
    params: &[String],
) -> Result<Vec<FoodRecord>, rusqlite::Error> {
    let mut food_stmt = conn.prepare(sql)?;
    let mut foods: Vec<FoodRecord> = vec![];

    if let Ok(mut foods_found) = food_stmt.query(rusqlite::params_from_iter(params)) {
        while let Some(food_row) = foods_found.next().unwrap_or(None) {
            let food = map_row_to_food(food_row)?;
            foods.push(food);
        }
    }
    Ok(foods)
}

/// Update pet information in the database.
/// * Scrapes pet and token information from the Fandom wiki.
/// * Inserts a new record for each pet by `level` and `pack`.
/// * Changes in any field aside from `name`, `pack`, and `level` will update an entry.
pub fn update_pet_info(conn: &Connection) -> Result<(), Box<dyn Error>> {
    // Read in insert or replace SQL.
    let sql_insert_pet = "
INSERT INTO pets (
    name, tier, attack, health, pack,
    effect_trigger, effect, effect_atk, effect_health, n_triggers, temp_effect,
    lvl, cost
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
ON CONFLICT(name, pack, lvl) DO UPDATE SET
    tier = ?2,
    attack = ?3,
    health = ?4,
    effect_trigger = ?6,
    effect = ?7,
    effect_atk = ?8,
    effect_health = ?9,
    n_triggers = ?10,
    temp_effect = ?11
WHERE
    tier != ?2 OR
    attack != ?3 OR
    health != ?4 OR
    effect_trigger != ?6 OR
    effect != ?7
;
    ";
    let mut n_rows_updated: usize = 0;

    let mut pets = parse_pet_info(crate::PET_URL)?;
    let tokens = parse_token_info(crate::TOKEN_URL)?;
    pets.extend(tokens);

    // Add each pet.
    for pet in pets.iter() {
        // Creating a new row for each pack and level a pet belongs to.
        // Each pet constrained by name and pack so will replace if already exists.
        let n_rows = conn.execute(
            sql_insert_pet,
            [
                &pet.name.to_string(),
                &pet.tier.to_string(),
                &pet.attack.to_string(),
                &pet.health.to_string(),
                &pet.pack.to_string(),
                &pet.effect_trigger
                    .clone()
                    .unwrap_or_else(|| "None".to_string()),
                &pet.effect.clone().unwrap_or_else(|| "None".to_string()),
                &pet.effect_atk.to_string(),
                &pet.effect_health.to_string(),
                &pet.n_triggers.to_string(),
                &pet.temp_effect.to_string(),
                &pet.lvl.to_string(),
                &pet.cost.to_string(),
            ],
        )?;
        n_rows_updated += n_rows;
    }
    info!(target: "db", "{} rows updated in \"pet\" table.", n_rows_updated);
    Ok(())
}

/// Update food information in the database.
/// * Inserts a new record for each food by `pack`.
/// * Changes in any field aside from `name` and `pack` will update an entry.
pub fn update_food_info(conn: &Connection) -> Result<(), Box<dyn Error>> {
    // Read in insert or replace SQL.
    let sql_insert_food = "
INSERT INTO foods (
    name, tier, effect, pack,
    holdable, single_use, end_of_battle,
    random, n_targets,
    effect_atk, effect_health,
    turn_effect, cost
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
ON CONFLICT(name, pack) DO UPDATE SET
    tier = ?2,
    effect = ?3,
    pack = ?4,
    holdable = ?5,
    single_use = ?6,
    end_of_battle = ?7,
    random = ?8,
    n_targets = ?9,
    effect_atk = ?10,
    effect_health = ?11,
    turn_effect = ?12,
    cost = ?13
WHERE
    tier != ?2 OR
    effect != ?3
;
    ";
    let mut n_rows_updated: usize = 0;

    let foods = parse_food_info(crate::FOOD_URL)?;
    for food in foods.iter() {
        let n_rows = conn.execute(
            sql_insert_food,
            [
                &food.name.to_string(),
                &food.tier.to_string(),
                &food.effect,
                &food.pack.to_string(),
                &food.holdable.to_string(),
                &food.single_use.to_string(),
                &food.end_of_battle.to_string(),
                &food.random.to_string(),
                &food.n_targets.to_string(),
                &food.effect_atk.to_string(),
                &food.effect_health.to_string(),
                &food.turn_effect.to_string(),
                &food.cost.to_string(),
            ],
        )?;
        n_rows_updated += n_rows;
    }
    info!(target: "db", "{} rows updated in \"food\" table.", n_rows_updated);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::{query_food, query_pet, update_food_info, update_pet_info};
    use crate::db::setup::get_connection;

    #[test]
    fn test_query_foods() {
        let conn = get_connection().unwrap();
        let sql = "SELECT * FROM foods";
        let params: Vec<String> = vec![];
        assert!(query_food(&conn, sql, &params).is_ok())
    }

    #[test]
    fn test_query_pets() {
        let conn = get_connection().unwrap();
        let sql = "SELECT * FROM pets";
        let params: Vec<String> = vec![];
        assert!(query_pet(&conn, sql, &params).is_ok())
    }

    #[test]
    fn test_update_foods() {
        let conn = get_connection().unwrap();
        assert!(update_pet_info(&conn).is_ok())
    }

    #[test]
    fn test_update_pets() {
        let conn = get_connection().unwrap();
        assert!(update_food_info(&conn).is_ok())
    }
}
