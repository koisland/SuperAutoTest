use serde::{Deserialize, Serialize};

use crate::common::game::Pack;

#[derive(Debug, Serialize, Deserialize)]
pub struct Food {
    pub name: String,
    pub tier: usize,
    pub effect: String,
    pub packs: Vec<Pack>,
}

impl Food {
    pub fn new(name: &str, tier: usize, effect: &str, packs: &[Pack]) -> Food {
        Food {
            name: name.to_string(),
            tier,
            effect: effect.to_string(),
            packs: packs.to_vec(),
        }
    }
}
