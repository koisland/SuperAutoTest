use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::common::battle::{
    state::Statistics,
    team::{Battle, Team},
};
use crate::common::foods::names::FoodName;
use crate::common::pets::{names::PetName, pet::Pet};

#[derive(Debug, Serialize, Deserialize)]
pub struct Teams {
    friends: SimpleTeam,
    enemies: SimpleTeam,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SimplePet {
    name: PetName,
    attack: u8,
    health: u8,
    level: u8,
    food: Option<FoodName>,
}

fn convert_pet(simple_pet: &Option<SimplePet>) -> Option<Pet> {
    if let Some(pet) = simple_pet {
        let stats = Statistics {
            attack: pet.attack as isize,
            health: pet.health as isize,
        };
        Pet::new(pet.name.clone(), Some(stats), pet.level as usize).ok()
    } else {
        None
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SimpleTeam {
    name: String,
    p1: Option<SimplePet>,
    p2: Option<SimplePet>,
    p3: Option<SimplePet>,
    p4: Option<SimplePet>,
    p5: Option<SimplePet>,
}

#[post("/battle", format = "json", data = "<teams>")]
pub fn battle(teams: Json<Teams>) -> Status {
    let mut friends_team = Team::new(
        &teams.friends.name,
        &[
            convert_pet(&teams.friends.p1),
            convert_pet(&teams.friends.p2),
            convert_pet(&teams.friends.p3),
            convert_pet(&teams.friends.p4),
            convert_pet(&teams.friends.p5),
        ],
    );
    let mut enemy_team = Team::new(
        &teams.enemies.name,
        &[
            convert_pet(&teams.enemies.p1),
            convert_pet(&teams.enemies.p2),
            convert_pet(&teams.enemies.p3),
            convert_pet(&teams.enemies.p4),
            convert_pet(&teams.enemies.p5),
        ],
    );

    friends_team.fight(&mut enemy_team, None);
    Status::Ok
}

#[cfg(test)]
mod test {
    use crate::server::rocket;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;
    use std::fs;

    #[test]
    fn test_invalid_team_extra_pet() {
        let invalid_team_six_json =
            fs::read_to_string("docs/examples/battle_invalid.json").unwrap();
        let client = Client::tracked(rocket()).expect("Valid rocket instance");
        let response = client
            .post(uri!(super::battle))
            .header(ContentType::JSON)
            .body(invalid_team_six_json)
            .dispatch();
        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[test]
    fn test_valid_team() {
        let valid_team_json = fs::read_to_string("docs/examples/battle_ant_teams.json").unwrap();
        let client = Client::tracked(rocket()).expect("Valid rocket instance");
        let response = client
            .post(uri!(super::battle))
            .header(ContentType::JSON)
            .body(valid_team_json)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}
