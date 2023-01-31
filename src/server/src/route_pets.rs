use crate::{SapDB, utils::capitalize_names};
use sapt::db::{query::query_pet, record::PetRecord, utils::setup_param_query};

use itertools::Itertools;
use rocket::serde::json::Json;

const QUERY_PET_PARAMS: [&str; 5] = ["name", "tier", "lvl", "pack", "effect_trigger"];

#[get("/pet?<name>&<level>&<tier>&<pack>&<effect_trigger>")]
pub async fn pets(
    conn: SapDB,
    name: Option<Vec<&str>>,
    level: Option<Vec<u8>>,
    tier: Option<Vec<u8>>,
    pack: Option<Vec<&str>>,
    effect_trigger: Option<Vec<&str>>,
) -> Json<Vec<PetRecord>> {
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

    // Will panic if length of query param names and sql_params not equal.
    let named_params: Vec<(&str, &Vec<String>)> = sql_params
        .iter()
        .enumerate()
        .map(|(i, vals)| (QUERY_PET_PARAMS[i], vals))
        .collect();

    let sql_stmt = setup_param_query("pets", &named_params);
    let flat_sql_params: Vec<String> = sql_params.into_iter().flatten().collect_vec();

    let query: Result<Vec<PetRecord>, rusqlite::Error> = conn
        .run(move |c| query_pet(c, &sql_stmt, &flat_sql_params))
        .await;

    Json(query.unwrap_or_default())
}

#[cfg(test)]
mod test {
    use crate::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use sapt::{
        db::{pack::Pack, record::PetRecord},
        PetName,
    };

    #[test]
    fn test_get_single_pet_entry() {
        let client = Client::tracked(rocket()).expect("Valid rocket instance");
        let response = client
            .get("/pet?name=ant&level=1&tier=1&pack=Turtle&effect_trigger=faint")
            .dispatch();

        assert_eq!(response.status(), Status::Ok);

        let records: Vec<PetRecord> =
            serde_json::from_str(&response.into_string().unwrap()).unwrap();

        assert!(records.len() == 1);
        assert_eq!(
            records.get(0).unwrap(),
            &PetRecord {
                name: PetName::Ant,
                tier: 1,
                attack: 2,
                health: 1,
                pack: Pack::Turtle,
                effect_trigger: Some("Faint".to_string(),),
                effect: Some("Give one random friend +2 attack and +1 health.".to_string(),),
                effect_atk: 2,
                effect_health: 1,
                n_triggers: 1,
                temp_effect: false,
                lvl: 1,
                cost: 3
            }
        )
    }

    #[test]
    fn test_get_multiple_pet_entries() {
        let client = Client::tracked(rocket()).expect("Valid rocket instance");
        let response = client
            .get("/pet?name=ant&level=1&tier=1&pack=Turtle&name=cricket")
            .dispatch();

        assert_eq!(response.status(), Status::Ok);

        let records: Vec<PetRecord> =
            serde_json::from_str(&response.into_string().unwrap()).unwrap();

        assert!(records.len() == 2);

        assert_eq!(
            records,
            vec![
                PetRecord {
                    name: PetName::Ant,
                    tier: 1,
                    attack: 2,
                    health: 1,
                    pack: Pack::Turtle,
                    effect_trigger: Some("Faint".to_string(),),
                    effect: Some("Give one random friend +2 attack and +1 health.".to_string(),),
                    effect_atk: 2,
                    effect_health: 1,
                    n_triggers: 1,
                    temp_effect: false,
                    lvl: 1,
                    cost: 3
                },
                PetRecord {
                    name: PetName::Cricket,
                    tier: 1,
                    attack: 1,
                    health: 2,
                    pack: Pack::Turtle,
                    effect_trigger: Some("Faint".to_string(),),
                    effect: Some("Summon one 1/1 Zombie Cricket.".to_string(),),
                    effect_atk: 1,
                    effect_health: 1,
                    n_triggers: 1,
                    temp_effect: false,
                    lvl: 1,
                    cost: 3
                },
            ]
        )
    }
}
