use crate::{
    api::server::SapDB,
    common::food::FoodRecord,
    common::{pet::PetRecord, team::Team},
    db::{
        query::{query_food, query_pet},
        utils::setup_param_query,
    },
};
use itertools::Itertools;
use rocket::{form::Form, response::content::RawJson, serde::json::Json};
use rusqlite::Error;
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

    let query: Result<Vec<PetRecord>, Error> = conn
        .run(move |c| query_pet(c, &sql_stmt, &flat_sql_params))
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

    let query: Result<Vec<FoodRecord>, Error> = conn
        .run(move |c| query_food(c, &sql_stmt, &flat_sql_params))
        .await;
    if let Ok(res) = query {
        Some(RawJson(to_string_pretty(&res).unwrap()))
    } else {
        None
    }
}

#[derive(FromForm)]
pub struct Teams {
    team_1: Json<Team>,
    team_2: Json<Team>,
}

#[post("/battle", data = "<teams>")]
pub fn battle(teams: Form<Teams>) {}
// { pet: Cat, lvl: 1, stats: {atk: 1, health: 2}, food: None}
