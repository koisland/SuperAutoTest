use serde::{Deserialize, Serialize};

use crate::common::game::Pack;

#[derive(Debug, Serialize, Deserialize)]
pub struct Pet {
    pub name: String,
    pub tier: usize,
    pub attack: usize,
    pub health: usize,
    pub pack: Pack,
    pub effect_trigger: Option<String>,
    pub effect: Option<String>,
    pub lvl: usize,
}
