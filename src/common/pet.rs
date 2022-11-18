use serde::{Deserialize, Serialize};

use crate::common::game::Pack;

#[derive(Debug, Serialize, Deserialize)]
pub struct Pet {
    pub name: String,
    pub tier: usize,
    pub attack: usize,
    pub health: usize,
    pub pack: Pack,
    pub effect_trigger: String,
    pub effect: String,
    pub lvl: usize,
}
