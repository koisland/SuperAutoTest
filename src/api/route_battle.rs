use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::common::foods::names::FoodName;
use crate::common::pets::{names::PetName, pet::Pet};
use crate::common::{
    battle::{
        state::{Statistics, TeamFightOutcome},
        team::Team,
    },
    foods::food::Food,
};

const TURN_LIMIT: usize = 300;

#[derive(Debug, Serialize, Deserialize)]
pub struct Teams {
    friends: SimpleTeam,
    enemies: SimpleTeam,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SimplePet {
    name: PetName,
    id: Option<String>,
    attack: u8,
    health: u8,
    level: u8,
    food: Option<FoodName>,
}

fn convert_pet(simple_pet: &Option<SimplePet>) -> Option<Pet> {
    if let Some(s_pet) = simple_pet {
        let stats = Statistics {
            attack: s_pet.attack as isize,
            health: s_pet.health as isize,
        };
        if let Ok(mut pet) = Pet::new(
            s_pet.name.clone(),
            s_pet.id.clone(),
            Some(stats),
            s_pet.level as usize,
        ) {
            // Set item if any.
            let food = s_pet.food.as_ref().map(Food::from);
            pet.item = food;

            Some(pet)
        } else {
            None
        }
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct BattleResponse {
    pub winner: Option<String>,
    pub friends: Vec<Option<Pet>>,
    pub friends_fainted: Vec<Option<Pet>>,
    pub enemies: Vec<Option<Pet>>,
    pub enemies_fainted: Vec<Option<Pet>>,
    pub n_turns: usize,
}

#[post("/battle", format = "json", data = "<teams>")]
pub fn battle(teams: Json<Teams>) -> Json<BattleResponse> {
    let mut friends_team = Team::new(
        &teams.friends.name,
        &[
            convert_pet(&teams.friends.p1),
            convert_pet(&teams.friends.p2),
            convert_pet(&teams.friends.p3),
            convert_pet(&teams.friends.p4),
            convert_pet(&teams.friends.p5),
        ],
        5,
    )
    .unwrap();
    let mut enemy_team = Team::new(
        &teams.enemies.name,
        &[
            convert_pet(&teams.enemies.p1),
            convert_pet(&teams.enemies.p2),
            convert_pet(&teams.enemies.p3),
            convert_pet(&teams.enemies.p4),
            convert_pet(&teams.enemies.p5),
        ],
        5,
    )
    .unwrap();

    let mut fight = friends_team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        // Break if fight continues indefinitely. Dunno what would cause.
        if friends_team.history.curr_turn == TURN_LIMIT {
            fight = TeamFightOutcome::Draw;
            break;
        }
        fight = friends_team.fight(&mut enemy_team)
    }
    let winner_team_name = match fight {
        TeamFightOutcome::Win => Some(friends_team.name),
        TeamFightOutcome::Loss => Some(enemy_team.name),
        TeamFightOutcome::Draw => None,
        TeamFightOutcome::None => None,
    };

    Json(BattleResponse {
        winner: winner_team_name,
        friends: friends_team.friends,
        friends_fainted: friends_team.fainted,
        enemies: enemy_team.friends,
        enemies_fainted: enemy_team.fainted,
        n_turns: friends_team.history.curr_turn,
    })
}

#[cfg(test)]
mod test {
    use crate::api::route_battle::BattleResponse;
    use crate::server::rocket;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::{Client, LocalResponse};
    use std::fs::{self, File};
    use std::io::BufReader;

    #[test]
    fn test_invalid_team_extra_pet() {
        let invalid_team_six_json =
            fs::read_to_string("docs/examples/input_invalid_size.json").unwrap();
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
        let valid_team_json = fs::read_to_string("docs/examples/input_draw.json").unwrap();
        let client = Client::tracked(rocket()).expect("Valid rocket instance");
        let response = client
            .post(uri!(super::battle))
            .header(ContentType::JSON)
            .body(valid_team_json)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn test_battle_draw_outcome() {
        let valid_team_json = fs::read_to_string("docs/examples/input_draw.json").unwrap();
        let reader = BufReader::new(File::open("docs/examples/output_draw.json").unwrap());
        // Expected battle response.
        let exp_battle_response: BattleResponse = serde_json::from_reader(reader).unwrap();

        let client = Client::tracked(rocket()).expect("Valid rocket instance");
        let response = client
            .post(uri!(super::battle))
            .header(ContentType::JSON)
            .body(valid_team_json)
            .dispatch();
        // Good team.
        assert_eq!(response.status(), Status::Ok);
        // Get battle response.
        let response_json: BattleResponse = response.into_json().unwrap();

        // Can't test exactly because ants and random triggers.
        assert_eq!(exp_battle_response.winner, response_json.winner)
    }

    #[test]
    fn test_battle_win_outcome() {
        let valid_team_json = fs::read_to_string("docs/examples/input_win.json").unwrap();
        let reader = BufReader::new(File::open("docs/examples/output_win.json").unwrap());
        // Expected battle response.
        let exp_battle_response: BattleResponse = serde_json::from_reader(reader).unwrap();

        let client = Client::tracked(rocket()).expect("Valid rocket instance");
        let response = client
            .post(uri!(super::battle))
            .header(ContentType::JSON)
            .body(valid_team_json)
            .dispatch();
        // Good team.
        assert_eq!(response.status(), Status::Ok);
        // Get battle response.
        let response_json: BattleResponse = response.into_json().unwrap();

        assert_eq!(exp_battle_response, response_json)
    }
}
