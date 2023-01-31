use sapt::db::{
    query::{update_food_info, update_pet_info},
    setup::{create_tables, get_connection},
};

fn main() {
    let conn = get_connection().unwrap();
    create_tables(&conn).unwrap();
    update_food_info(&conn).unwrap();
    update_pet_info(&conn).unwrap();
}
