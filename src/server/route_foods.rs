use super::{main::SapDB, utils::capitalize_names};
use itertools::Itertools;
use rocket::serde::json::Json;
use sapt::db::{query::query_food, record::FoodRecord, utils::setup_param_query};

const QUERY_FOOD_PARAMS: [&str; 3] = ["name", "pack", "tier"];

#[get("/food?<name>&<tier>&<pack>")]
pub async fn foods(
    conn: SapDB,
    name: Option<Vec<&str>>,
    tier: Option<Vec<u8>>,
    pack: Option<Vec<&str>>,
) -> Json<Vec<FoodRecord>> {
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
    // Will panic if length of query param names and sql_params not equal.
    let named_params: Vec<(&str, &Vec<String>)> = sql_params
        .iter()
        .enumerate()
        .map(|(i, vals)| (QUERY_FOOD_PARAMS[i], vals))
        .collect();
    let sql_stmt = setup_param_query("foods", &named_params);
    let flat_sql_params: Vec<String> = sql_params.into_iter().flatten().collect_vec();

    let query: Result<Vec<FoodRecord>, rusqlite::Error> = conn
        .run(move |c| query_food(c, &sql_stmt, &flat_sql_params))
        .await;

    Json(query.unwrap_or_default())
}

#[cfg(test)]
mod test {
    use crate::server::main::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use sapt::{
        db::{pack::Pack, record::FoodRecord},
        FoodName,
    };

    #[test]
    fn test_get_single_food_entry() {
        let client = Client::tracked(rocket()).expect("Valid rocket instance");
        let response = client
            .get("/food?tier=3&pack=Star&name=pineapple")
            .dispatch();

        assert_eq!(response.status(), Status::Ok);

        let records: Vec<FoodRecord> =
            serde_json::from_str(&response.into_string().unwrap()).unwrap();

        assert!(records.len() == 1);

        assert_eq!(
            records.get(0).unwrap(),
            &FoodRecord {
                name: FoodName::Pineapple,
                tier: 3,
                effect: "Give one pet Pineapple. Ability deals +2 damage".to_string(),
                pack: Pack::Star,
                holdable: true,
                single_use: false,
                end_of_battle: false,
                random: false,
                n_targets: 1,
                effect_atk: 2,
                effect_health: 0,
                turn_effect: false,
                cost: 3
            }
        );
    }

    #[test]
    fn test_get_multiple_food_entries() {
        let client = Client::tracked(rocket()).expect("Valid rocket instance");
        let response = client.get("/food?tier=3&pack=Star").dispatch();

        assert_eq!(response.status(), Status::Ok);

        let records: Vec<FoodRecord> =
            serde_json::from_str(&response.into_string().unwrap()).unwrap();

        assert!(records.len() == 2);

        assert_eq!(
            records,
            vec![
                FoodRecord {
                    name: FoodName::Pineapple,
                    tier: 3,
                    effect: "Give one pet Pineapple. Ability deals +2 damage".to_string(),
                    pack: Pack::Star,
                    holdable: true,
                    single_use: false,
                    end_of_battle: false,
                    random: false,
                    n_targets: 1,
                    effect_atk: 2,
                    effect_health: 0,
                    turn_effect: false,
                    cost: 3
                },
                FoodRecord {
                    name: FoodName::Cucumber,
                    tier: 3,
                    effect: "Give one pet Cucumber. Gain +1 health at end of turn".to_string(),
                    pack: Pack::Star,
                    holdable: true,
                    single_use: false,
                    end_of_battle: false,
                    random: false,
                    n_targets: 1,
                    effect_atk: 0,
                    effect_health: 1,
                    turn_effect: true,
                    cost: 3
                },
            ]
        );
    }
}
