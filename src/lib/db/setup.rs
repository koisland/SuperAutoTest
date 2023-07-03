use crate::{
    db::{
        query::SAPQuery,
        record::{FoodRecord, PetRecord, SAPRecord},
    },
    error::SAPTestError,
    wiki_scraper::{
        parse_food::parse_food_info, parse_names::parse_names_info, parse_pet::parse_pet_info,
        parse_tokens::parse_token_info, parse_toy::parse_toy_info,
    },
    Entity, CONFIG,
};
use log::info;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

const PET_URL: &str = "https://superautopets.fandom.com/wiki/Pets?action=raw";
const FOOD_URL: &str = "https://superautopets.fandom.com/wiki/Food?action=raw";
const TOKEN_URL: &str = "https://superautopets.fandom.com/wiki/Tokens?action=raw";
const TOYS_URL: &str = "https://superautopets.fandom.com/wiki/Toys?action=raw";
const NAMES_URL: &str = "https://superautopets.fandom.com/wiki/Team_Names?action=raw";

/// A Super Auto Pets database.
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
    /// use saptest::SapDB;
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

        // Update on startup if enabled.
        if CONFIG.database.update_on_startup {
            db.create_tables()?
                .update_food_info()?
                .update_pet_info()?
                .update_toy_info()?
                .update_name_info()?;
        }

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
            CREATE TABLE IF NOT EXISTS names (
                id INTEGER PRIMARY KEY,
                word_category TEXT NOT NULL,
                word TEXT NOT NULL,
                CONSTRAINT unq UNIQUE (word_category, word)
            );
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
                img_url TEXT,
                is_token BOOLEAN NOT NULL,
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
                img_url TEXT,
                CONSTRAINT unq UNIQUE (name, pack)
            );
            CREATE TABLE IF NOT EXISTS toys (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                tier INTEGER NOT NULL,
                effect_trigger TEXT NOT NULL,
                effect TEXT NOT NULL,
                effect_atk INTEGER NOT NULL,
                effect_health INTEGER NOT NULL,
                n_triggers INTEGER NOT NULL,
                temp_effect BOOLEAN NOT NULL,
                lvl INTEGER NOT NULL,
                source TEXT NOT NULL,
                img_url TEXT,
                hard_mode BOOLEAN NOT NULL,
                CONSTRAINT unq UNIQUE (name, lvl)
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
                turn_effect, cost, img_url
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
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
                cost = ?13,
                img_url = ?14
            WHERE
                tier != ?2 OR
                effect != ?3
            ;
        ";
        let conn = self.pool.get()?;
        let mut n_rows_updated: usize = 0;

        let food_url = CONFIG.database.foods_version.map_or_else(
            || FOOD_URL.to_owned(),
            |id| format!("{FOOD_URL}&oldid={id}"),
        );
        let foods = parse_food_info(&food_url)?;
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
                    &food.img_url.to_string(),
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
                lvl, cost, img_url, is_token
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(name, pack, lvl) DO UPDATE SET
                tier = ?2,
                attack = ?3,
                health = ?4,
                effect_trigger = ?6,
                effect = ?7,
                effect_atk = ?8,
                effect_health = ?9,
                n_triggers = ?10,
                temp_effect = ?11,
                img_url = ?14,
                is_token = ?15
            WHERE
                tier != ?2 OR
                attack != ?3 OR
                health != ?4 OR
                effect_trigger != ?6 OR
                effect != ?7
            ;
        ";
        let mut n_rows_updated: usize = 0;

        // Use older version if available.
        let pet_url = CONFIG
            .database
            .pets_version
            .map_or_else(|| PET_URL.to_owned(), |id| format!("{PET_URL}&oldid={id}"));
        let token_url = CONFIG.database.tokens_version.map_or_else(
            || TOKEN_URL.to_owned(),
            |id| format!("{TOKEN_URL}&oldid={id}"),
        );

        let mut pets = parse_pet_info(&pet_url)?;
        let tokens = parse_token_info(&token_url)?;
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
                    &pet.img_url.to_string(),
                    &pet.is_token.to_string(),
                ],
            )?;
            n_rows_updated += n_rows;
        }
        info!(target: "db", "{} rows updated in \"pet\" table.", n_rows_updated);
        Ok(self)
    }

    /// Update toy information in the database.
    /// * Scrapes toy (hard and normal) information from the Fandom wiki.
    /// * Inserts a new record for each pet by `level`
    /// * Changes in any field aside from `name` and `level` will update an entry.
    fn update_toy_info(&self) -> Result<&Self, SAPTestError> {
        let conn = self.pool.get()?;
        // Read in insert or replace SQL.
        let sql_insert_pet = "
            INSERT INTO toys (
                name, tier, effect_trigger, effect, effect_atk, effect_health,
                n_triggers, temp_effect,
                lvl, source, img_url, hard_mode
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(name, lvl) DO UPDATE SET
                tier = ?2,
                effect_trigger = ?3,
                effect = ?4,
                effect_atk = ?5,
                effect_health = ?6,
                n_triggers = ?7,
                temp_effect = ?8,
                source = ?10,
                img_url = ?11,
                hard_mode = ?12
            WHERE
                tier != ?2 OR
                effect_atk != ?5 OR
                effect_health != ?6 OR
                effect_trigger != ?3 OR
                effect != ?4
            ;
        ";
        let mut n_rows_updated: usize = 0;

        // Use older version if available.
        let toys_url = CONFIG
            .database
            .toys_version
            .map_or_else(|| TOYS_URL.to_owned(), |id| format!("{PET_URL}&oldid={id}"));

        let toys = parse_toy_info(&toys_url)?;

        // Add each toy.
        for toy in toys.iter() {
            // Creating a new row for each pack and level a pet belongs to.
            // Each pet constrained by name and pack so will replace if already exists.
            let n_rows = conn.execute(
                sql_insert_pet,
                [
                    &toy.name.to_string(),
                    &toy.tier.to_string(),
                    &toy.effect_trigger
                        .clone()
                        .unwrap_or_else(|| "None".to_string()),
                    &toy.effect.clone().unwrap_or_else(|| "None".to_string()),
                    &toy.effect_atk.to_string(),
                    &toy.effect_health.to_string(),
                    &toy.n_triggers.to_string(),
                    &toy.temp_effect.to_string(),
                    &toy.lvl.to_string(),
                    &toy.source.clone().unwrap_or_else(|| "None".to_string()),
                    &toy.img_url.to_string(),
                    &toy.hard_mode.to_string(),
                ],
            )?;
            n_rows_updated += n_rows;
        }
        info!(target: "db", "{} rows updated in \"pet\" table.", n_rows_updated);
        Ok(self)
    }

    fn update_name_info(&self) -> Result<&Self, SAPTestError> {
        let conn = self.pool.get()?;
        // Read in insert or replace SQL.
        let sql_insert_names = "
            INSERT OR IGNORE INTO names (word_category, word) VALUES (?1, ?2);
        ";

        let names_url = CONFIG.database.names_version.map_or_else(
            || NAMES_URL.to_owned(),
            |id| format!("{NAMES_URL}&oldid={id}"),
        );
        let words = parse_names_info(&names_url)?;
        let n_words = words.len();
        for word in words.into_iter() {
            conn.execute(sql_insert_names, [word.word_type.to_string(), word.word])?;
        }
        info!(target: "db", "{} rows updated in \"names\" table.", n_words);
        Ok(self)
    }

    /// Execute `SELECT` query in the Super Auto Pets database with a [`SAPQuery`].
    ///
    /// # Examples
    /// ---
    /// Pet Query
    /// ```
    /// use saptest::{SAPDB, SAPQuery, Entity, PetName, db::{pack::Pack, record::SAPRecord}};
    ///
    /// let mut query = SAPQuery::builder();
    /// query.set_table(Entity::Pet)
    ///     .set_param("name", vec![PetName::Tiger])
    ///     .set_param("lvl", vec![2])
    ///     .set_param("pack", vec![Pack::Turtle]);
    ///
    /// let pets = SAPDB.execute_query(query).unwrap();
    /// let Some(SAPRecord::Pet(record)) = pets.first() else { panic!("No Record found.")};
    /// assert!(record.name == PetName::Tiger && record.lvl == 2 && record.pack == Pack::Turtle)
    /// ```
    /// ---
    /// Food Query
    /// ```
    /// use saptest::{SAPDB, SAPQuery, Entity, FoodName, db::{pack::Pack, record::SAPRecord}};

    /// let mut query = SAPQuery::builder();
    /// query.set_table(Entity::Food)
    ///     .set_param("name", vec![FoodName::Apple])
    ///     .set_param("pack", vec![Pack::Turtle]);
    ///
    /// let foods = SAPDB.execute_query(query).unwrap();
    ///
    /// let Some(SAPRecord::Food(record)) = foods.first() else { panic!("No Record found.")};
    /// assert!(record.name == FoodName::Apple && record.pack == Pack::Turtle)
    /// ```
    pub fn execute_query(&self, sap_query: SAPQuery) -> Result<Vec<SAPRecord>, SAPTestError> {
        let conn = self.pool.get()?;
        let mut records: Vec<SAPRecord> = vec![];

        let mut stmt = conn.prepare(&sap_query.as_sql()?)?;
        // ^ Requires a table. Safe to unwrap.
        let table = sap_query.table.unwrap();

        let mut query = stmt.query(rusqlite::params_from_iter(sap_query.flat_params()))?;
        while let Some(row) = query.next()? {
            let record = match table {
                Entity::Pet => SAPRecord::Pet(row.try_into()?),
                Entity::Food => SAPRecord::Food(row.try_into()?),
            };
            records.push(record);
        }
        Ok(records)
    }

    pub(crate) fn execute_sql_query(
        &self,
        sql: &str,
        params: &[String],
    ) -> Result<Vec<SAPRecord>, SAPTestError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(sql)?;
        let mut records: Vec<SAPRecord> = vec![];

        let mut query = stmt.query(rusqlite::params_from_iter(params))?;
        while let Some(row) = query.next()? {
            // Try converting records to valid types.
            let record = if let Ok(record) = TryInto::<PetRecord>::try_into(row) {
                SAPRecord::Pet(record)
            } else if let Ok(record) = TryInto::<FoodRecord>::try_into(row) {
                SAPRecord::Food(record)
            } else {
                return Err(SAPTestError::QueryFailure {
                    subject: "Invalid Record Conversion".to_string(),
                    reason: format!("Cannot form query ({sql}) with params {params:?} results into a valid record type.")
                })?;
            };
            records.push(record);
        }
        Ok(records)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        db::{
            pack::Pack,
            query::SAPQuery,
            record::{FoodRecord, PetRecord, SAPRecord},
        },
        Entity, FoodName, PetName, SAPDB,
    };

    #[test]
    fn test_query_no_params() {
        let mut food_query = SAPQuery::builder();
        food_query.set_table(Entity::Food);

        let mut pet_query = SAPQuery::builder();
        pet_query.set_table(Entity::Pet);

        let foods = SAPDB.execute_query(food_query);
        let pets = SAPDB.execute_query(pet_query);
        assert!(foods.is_ok());
        assert!(pets.is_ok());
    }

    #[test]
    fn test_query_params_food() {
        let mut food_query = SAPQuery::builder();

        food_query
            .set_table(Entity::Food)
            .set_param("name", vec![FoodName::Apple])
            .set_param("pack", vec![Pack::Turtle]);

        let foods = SAPDB.execute_query(food_query).unwrap();

        let SAPRecord::Food(record) = foods.first().unwrap() else { panic!("No Record found.")};
        assert!(record.name == FoodName::Apple && record.pack == Pack::Turtle)
    }

    #[test]
    fn test_query_params_pets() {
        let mut pet_query = SAPQuery::builder();

        pet_query
            .set_table(Entity::Pet)
            .set_param("name", vec![PetName::Tiger])
            .set_param("lvl", vec![2])
            .set_param("pack", vec![Pack::Turtle]);

        let pets = SAPDB.execute_query(pet_query).unwrap();
        let SAPRecord::Pet(record) = pets.first().unwrap() else { panic!("No Record found.")};
        assert!(record.name == PetName::Tiger && record.lvl == 2 && record.pack == Pack::Turtle)
    }

    #[test]
    fn test_query_sql_foods() {
        let sql = "SELECT * FROM foods";
        let params: Vec<String> = vec![];
        let records = SAPDB.execute_sql_query(sql, &params).unwrap();
        let first_record = &records[0];
        assert!(TryInto::<FoodRecord>::try_into(first_record.clone()).is_ok());
        assert!(TryInto::<PetRecord>::try_into(first_record.clone()).is_err())
    }

    #[test]
    fn test_query_sql_pets() {
        let sql = "SELECT * FROM pets";
        let params: Vec<String> = vec![];
        let records = SAPDB.execute_sql_query(sql, &params).unwrap();
        let first_record = &records[0];
        assert!(TryInto::<FoodRecord>::try_into(first_record.clone()).is_err());
        assert!(TryInto::<PetRecord>::try_into(first_record.clone()).is_ok())
    }

    #[test]
    fn test_update_foods() {
        assert!(SAPDB.update_pet_info().is_ok())
    }

    #[test]
    fn test_update_pets() {
        assert!(SAPDB.update_food_info().is_ok())
    }

    #[test]
    fn test_update_names() {
        assert!(SAPDB.update_name_info().is_ok())
    }
}
