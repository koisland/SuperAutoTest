use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display};

use crate::{
    common::{
        battle::effect::Effect,
        battle::state::Statistics,
        foods::food::Food,
        pets::{effects::get_pet_effect, names::PetName},
        regex_patterns::*,
    },
    db::{setup::get_connection, utils::map_row_to_pet},
};

#[allow(dead_code)]
pub fn num_regex(pattern: &LRegex, string: &str) -> Result<usize, Box<dyn Error>> {
    Ok(pattern.captures(string).map_or(Ok(0), |cap| {
        cap.get(1)
            .map_or(Ok(0), |mtch| mtch.as_str().parse::<usize>())
    })?)
}

/// A Super Auto Pet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pet {
    pub name: PetName,
    pub tier: usize,
    pub stats: Statistics,
    pub lvl: usize,
    pub effect: Option<Effect>,
    pub item: Option<Food>,
    pub pos: Option<usize>,
}

impl Display for Pet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}: ({},{}) (Level: {}) (Pos: {:?}) (Item: {:?})]",
            self.name, self.stats.attack, self.stats.health, self.lvl, self.pos, self.item
        )
    }
}

#[allow(dead_code)]
impl Pet {
    /// Create a new `Pet` with given stats and level
    pub fn new(
        name: PetName,
        stats: Statistics,
        lvl: usize,
        item: Option<Food>,
        pos: Option<usize>,
    ) -> Result<Pet, Box<dyn Error>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM pets where name = ? and lvl = ?")?;
        let pet_record = stmt.query_row([name.to_string(), lvl.to_string()], map_row_to_pet)?;
        let pet_effect = pet_record.effect.unwrap_or_else(|| "None".to_string());

        let mut effect_stats = Statistics {
            attack: num_regex(RGX_ATK, &pet_effect).ok().unwrap_or(0),
            health: num_regex(RGX_HEALTH, &pet_effect).ok().unwrap_or(0),
        };
        // If a pet has a summon effect, replace attack and health stats from effect_stats.
        if pet_effect.contains("Summon") {
            effect_stats.attack = num_regex(RGX_SUMMON_ATK, &pet_effect).ok().unwrap_or(1);
            effect_stats.health = num_regex(RGX_SUMMON_HEALTH, &pet_effect).ok().unwrap_or(1);
        }
        let n_triggers = num_regex(RGX_N_TRIGGERS, &pet_effect).ok().unwrap_or(1);
        // TODO: Parse from pet description.
        let effect = get_pet_effect(&name, effect_stats, lvl, n_triggers);

        Ok(Pet {
            name,
            tier: pet_record.tier,
            stats,
            lvl: pet_record.lvl,
            effect,
            item,
            pos,
        })
    }
}
