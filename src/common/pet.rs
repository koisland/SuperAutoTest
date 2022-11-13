use serde::{Deserialize, Serialize};

use crate::common::game::Pack;

#[derive(Debug, Serialize, Deserialize)]
pub struct Pet {
    pub name: String,
    pub tier: usize,
    pub attack: usize,
    pub health: usize,
    pub packs: Vec<Pack>,
    pub effect_trigger: String,
    pub effects: Vec<String>,
}

impl Pet {
    pub fn new(
        name: &str,
        tier: usize,
        attack: usize,
        health: usize,
        packs: &[Pack],
        effect_trigger: &str,
        effects: &[String],
    ) -> Pet {
        Pet {
            name: name.to_string(),
            tier,
            attack,
            health,
            packs: packs.to_vec(),
            effect_trigger: effect_trigger.to_string(),
            effects: effects.to_vec(),
        }
    }
}
