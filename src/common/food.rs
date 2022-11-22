use serde::{Deserialize, Serialize};

use crate::common::game::Pack;

use super::effect::FoodEffect;

#[derive(Debug, Serialize, Deserialize)]
pub struct FoodRecord {
    pub name: String,
    pub tier: usize,
    pub effect: String,
    pub pack: Pack,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Food {
    pub name: String,
    pub tier: usize,
    pub effect: FoodEffect,
    pub can_hold: bool,
}
