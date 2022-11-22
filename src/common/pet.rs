use serde::{Deserialize, Serialize};

use super::{effect::PetEffect, food::Food, game::Pack, team::Team};

#[derive(Debug, Serialize, Deserialize)]
pub struct PetRecord {
    pub name: String,
    pub tier: usize,
    pub attack: usize,
    pub health: usize,
    pub pack: Pack,
    pub effect_trigger: Option<String>,
    pub effect: Option<String>,
    pub lvl: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pet {
    pub name: String,
    pub tier: usize,
    pub attack: usize,
    pub health: usize,
    pub lvl: usize,
    pub effect: PetEffect,
    pub item: Option<Food>,
}

trait Combat {
    fn attack(&self, enemy_team: &mut Team) {}
    fn use_ability(&self, pet_team: &mut Team, enemy_team: &mut Team) {}
}

impl Combat for Pet {
    fn attack(&self, enemy_team: &mut Team) {}
    fn use_ability(&self, pet_team: &mut Team, enemy_team: &mut Team) {}
}
