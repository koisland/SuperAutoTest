use crate::common::game::Pack;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FoodRecord {
    pub name: String,
    pub tier: usize,
    pub effect: String,
    pub pack: Pack,
}
