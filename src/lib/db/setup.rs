use crate::{
    db::record::{FoodRecord, PetRecord},
    error::SAPTestError,
    wiki_scraper::{
        parse_food::parse_food_info, parse_pet::parse_pet_info, parse_tokens::parse_token_info,
    },
};
use log::info;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

/// SAP `SQLite` Database.
/// * Currently, just used to initialize.
/// * Uses [`rusqlite`](https://docs.rs/crate/sqlite/0.29.0) but will migrate to `sqlx` in future.
pub struct SapDB {
    /// Database file.
    pub file: String,
    /// Database pooled connection.
    pub pool: r2d2::Pool<SqliteConnectionManager>,
}

impl SapDB {
    /// Initialize database.
    /// * Creates a `sqlite` file at the specified `file` path with the `pets` and `foods` tables.
    /// * Updates all tables with the most recent information from the SAP wiki.
    /// # Example
    /// ```
    /// use std::path::Path;
    /// use sapt::SapDB;
    ///
    /// let db_path = "./sap.db";
    /// let db = SapDB::new(db_path);
    ///
    /// assert!(db.is_ok());
    /// assert!(Path::new(db_path).exists());
    /// ```
    pub fn new<P>(file: P) -> Result<Self, SAPTestError>
    where
        P: AsRef<Path> + Into<String>,
    {
        let pool = SapDB::create_conn_pool(&file)?;
        let db = SapDB {
            file: file.into(),
            pool,
        };
        db.create_tables()?.update_food_info()?.update_pet_info()?;
        Ok(db)
    }

    /// Create `rusqlite` connection pool.
    fn create_conn_pool<P>(file: P) -> Result<r2d2::Pool<SqliteConnectionManager>, SAPTestError>
    where
        P: AsRef<Path>,
    {
        let manager = SqliteConnectionManager::file(file.as_ref());
        let pool = r2d2::Pool::new(manager)?;
        Ok(pool)
    }

    /// Create tables if they don't exist.
    fn create_tables(&self) -> Result<&Self, SAPTestError> {
        self.pool.get()?.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS pets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                tier INTEGER NOT NULL,
                attack INTEGER NOT NULL,
                health INTEGER NOT NULL,
                pack TEXT NOT NULL,
                effect_trigger TEXT NOT NULL,
                effect TEXT NOT NULL,
                effect_atk INTEGER NOT NULL,
                effect_health INTEGER NOT NULL,
                n_triggers INTEGER NOT NULL,
                temp_effect BOOLEAN NOT NULL,
                lvl INTEGER NOT NULL,
                cost INTEGER NOT NULL,
                CONSTRAINT unq UNIQUE (name, pack, lvl)
            );
            CREATE TABLE IF NOT EXISTS foods (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                tier INTEGER NOT NULL,
                effect TEXT NOT NULL,
                pack TEXT NOT NULL,
                holdable BOOLEAN NOT NULL,
                single_use BOOLEAN NOT NULL,
                end_of_battle BOOLEAN NOT NULL,
                random BOOLEAN NOT NULL,
                n_targets INTEGER NOT NULL,
                effect_atk INTEGER NOT NULL,
                effect_health INTEGER NOT NULL,
                turn_effect BOOLEAN NOT NULL,
                cost INTEGER NOT NULL,
                CONSTRAINT unq UNIQUE (name, pack)
            );",
        )?;

        Ok(self)
    }

    /// Update food information in the database.
    /// * Inserts a new record for each food by `pack`.
    /// * Changes in any field aside from `name` and `pack` will update an entry.
    fn update_food_info(&self) -> Result<&Self, SAPTestError> {
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
        let conn = self.pool.get()?;
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
        Ok(self)
    }

    /// Update pet information in the database.
    /// * Scrapes pet and token information from the Fandom wiki.
    /// * Inserts a new record for each pet by `level` and `pack`.
    /// * Changes in any field aside from `name`, `pack`, and `level` will update an entry.
    fn update_pet_info(&self) -> Result<&Self, SAPTestError> {
        let conn = self.pool.get()?;
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
        Ok(self)
    }

    /// Query database for [`PetRecord`](crate::db::record::PetRecord)s.
    /// # Example
    /// ```
    /// use sapt::SAPDB;
    ///
    /// let stmt = "SELECT * FROM pets";
    /// let query = SAPDB.execute_pet_query(stmt, &[]);
    /// assert!(query.is_ok())
    /// ```
    pub fn execute_pet_query(
        &self,
        sql: &str,
        params: &[String],
    ) -> Result<Vec<PetRecord>, SAPTestError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(sql)?;
        let mut pets_found: Vec<PetRecord> = vec![];

        let mut query = stmt.query(rusqlite::params_from_iter(params))?;
        while let Some(pet_row) = query.next()? {
            pets_found.push(pet_row.try_into()?);
        }
        Ok(pets_found)
    }

    /// Query database for [`FoodRecord`](crate::db::record::FoodRecord)s.
    /// # Example
    /// ```
    /// use sapt::SAPDB;
    /// let stmt = "SELECT * FROM foods";
    /// let query = SAPDB.execute_food_query(stmt, &[]);
    /// assert!(query.is_ok())
    /// ```
    pub fn execute_food_query(
        &self,
        sql: &str,
        params: &[String],
    ) -> Result<Vec<FoodRecord>, SAPTestError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(sql)?;
        let mut foods_found: Vec<FoodRecord> = vec![];

        let mut query = stmt.query(rusqlite::params_from_iter(params))?;
        while let Some(food_row) = query.next()? {
            foods_found.push(food_row.try_into()?);
        }
        Ok(foods_found)
    }
}

#[cfg(test)]
mod test {
    use crate::SAPDB;

    #[test]
    fn test_query_foods() {
        let sql = "SELECT * FROM foods";
        let params: Vec<String> = vec![];
        assert!(SAPDB.execute_food_query(sql, &params).is_ok())
    }

    #[test]
    fn test_query_pets() {
        let sql = "SELECT * FROM pets";
        let params: Vec<String> = vec![];
        assert!(SAPDB.execute_pet_query(sql, &params).is_ok())
    }

    #[test]
    fn test_update_foods() {
        assert!(SAPDB.update_pet_info().is_ok())
    }

    #[test]
    fn test_update_pets() {
        assert!(SAPDB.update_food_info().is_ok())
    }
}
