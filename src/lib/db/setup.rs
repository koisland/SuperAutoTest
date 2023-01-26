// https://docs.rs/crate/sqlite/0.29.0

use rusqlite::Connection;
use std::error::Error;
use std::fs::read_to_string;

/// Get database connection.
pub fn get_connection() -> Result<Connection, Box<dyn Error>> {
    let db = Connection::open(crate::DB_FNAME)?;
    Ok(db)
}

/// Create tables if they don't exist.
pub fn create_tables(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let sql_create_tables = read_to_string(crate::DB_CREATE_SQL)?;
    conn.execute_batch(&sql_create_tables)?;
    Ok(())
}
