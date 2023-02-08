use itertools::Itertools;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use sapt::{
    battle::{
        stats::Statistics,
        state::TeamFightOutcome,
        team::Team,
    },
    foods::{food::Food, names::FoodName},
    pets::{names::PetName, pet::Pet},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Teams {
    friends: SimpleTeam,
    enemies: SimpleTeam,
    seed: Option<u64>,
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
        let mut pet = Pet::new(
            s_pet.name.clone(),
            s_pet.id.clone(),
            Some(stats),
            s_pet.level as usize,
        ).unwrap();
        // Set item if any.
        let food = s_pet.food.as_ref().map(|food| Food::try_from(food).unwrap());
        pet.item = food;
        Some(pet)
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

impl From<&SimpleTeam> for Team {
    fn from(team: &SimpleTeam) -> Self {
        let pets = [
            convert_pet(&team.p1),
            convert_pet(&team.p2),
            convert_pet(&team.p3),
            convert_pet(&team.p4),
            convert_pet(&team.p5),
        ].into_iter().filter_map(|pet| pet.as_ref().cloned()).collect_vec();
        let mut team = Team::new(
            &pets,
            5,
        ).expect("Unable to create team.");
        team.name = team.name.to_string();
        team
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BattleResponse {
    pub winner: Option<String>,
    pub friends: Vec<Pet>,
    pub friends_fainted: Vec<Pet>,
    pub enemies: Vec<Pet>,
    pub enemies_fainted: Vec<Pet>,
    pub n_turns: usize,
}

#[post("/battle", format = "json", data = "<teams>")]
pub fn battle(teams: Json<Teams>) -> Json<BattleResponse> {
    let mut friends_team = Team::from(&teams.friends);
    let mut enemy_team = Team::from(&teams.enemies);

    if let Some(seed) = teams.seed {
        friends_team.set_seed(seed);
        enemy_team.set_seed(seed);
    }

    let mut fight = friends_team.fight(&mut enemy_team);
    while let TeamFightOutcome::None = fight {
        fight = friends_team.fight(&mut enemy_team)
    }
    let winner_team_name = match fight {
        TeamFightOutcome::Win => Some(friends_team.name),
        TeamFightOutcome::Loss => Some(enemy_team.name),
        _ => None,
    };

    Json(BattleResponse {
        winner: winner_team_name,
        friends: friends_team.friends.iter().map(|pet| pet.borrow().clone()).collect_vec(),
        friends_fainted: friends_team.fainted.iter().map(|pet| pet.borrow().clone()).collect_vec(),
        enemies: enemy_team.friends.iter().map(|pet| pet.borrow().clone()).collect_vec(),
        enemies_fainted: enemy_team.fainted.iter().map(|pet| pet.borrow().clone()).collect_vec(),
        n_turns: friends_team.history.curr_phase,
    })
}

#[cfg(test)]
mod test {
    use crate::{rocket, route_battle::BattleResponse};
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;
    use std::fs::{self, File};
    use std::io::BufReader;

    #[test]
    fn test_invalid_team_extra_pet() {
        let invalid_team_six_json =
            fs::read_to_string("examples/input_invalid_size.json").unwrap();
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
        let valid_team_json = fs::read_to_string("examples/input_draw.json").unwrap();
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
        let valid_team_json = fs::read_to_string("examples/input_draw.json").unwrap();
        let reader = BufReader::new(File::open("examples/output_draw.json").unwrap());
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
        // let response_json: BattleResponse = response.into_json().unwrap();

        // assert_eq!(exp_battle_response, response_json)
    }

    #[test]
    fn test_battle_win_outcome() {
        let valid_team_json = fs::read_to_string("examples/input_win.json").unwrap();
        let reader = BufReader::new(File::open("examples/output_win.json").unwrap());
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
        // let response_json: BattleResponse = response.into_json().unwrap();

        // assert_eq!(exp_battle_response, response_json)
    }

    #[test]
    fn test_battle_loss_outcome() {
        let valid_team_json = fs::read_to_string("examples/input_loss.json").unwrap();
        let reader = BufReader::new(File::open("examples/output_loss.json").unwrap());
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
        // let response_json: BattleResponse = response.into_json().unwrap();

        // assert_eq!(exp_battle_response, response_json)
    }

    #[test]
    fn test_battle_team_seeded() {
        let team_json = fs::read_to_string("examples/input_draw_seeded.json").unwrap();
        let reader = BufReader::new(File::open("examples/output_draw_seeded.json").unwrap());
        // Expected battle response.
        let exp_battle_response: BattleResponse = serde_json::from_reader(reader).unwrap();

        let client = Client::tracked(rocket()).expect("Valid rocket instance");
        let response = client
            .post(uri!(super::battle))
            .header(ContentType::JSON)
            .body(team_json)
            .dispatch();

        // let response_json: BattleResponse = response.into_json().unwrap();

        // assert_eq!(exp_battle_response, response_json)
    }
}
