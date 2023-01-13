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

pub const MAX_PET_STATS: usize = 50;

#[allow(dead_code)]
pub fn num_regex(pattern: &LRegex, string: &str) -> Option<usize> {
    if let Some(cap) = pattern.captures(string) {
        cap.get(1)
            .map(|mtch| mtch.as_str().parse::<usize>().unwrap())
    } else {
        None
    }
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
        stats: Option<Statistics>,
        lvl: usize,
        item: Option<Food>,
        pos: Option<usize>,
    ) -> Result<Pet, Box<dyn Error>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM pets WHERE name = ? AND lvl = ?")?;
        let pet_record = stmt.query_row([name.to_string(), lvl.to_string()], map_row_to_pet)?;

        // Use default stats at level if stats not provided.
        let pet_stats = stats.unwrap_or(Statistics {
            attack: pet_record.attack,
            health: pet_record.health,
        });

        let effect = get_pet_effect(
            &name,
            &pet_stats,
            Statistics {
                attack: pet_record.effect_atk,
                health: pet_record.effect_health,
            },
            lvl,
            pet_record.n_triggers,
        );

        Ok(Pet {
            name,
            tier: pet_record.tier,
            stats: pet_stats,
            lvl: pet_record.lvl,
            effect,
            item,
            pos,
        })
    }

    pub fn levelup(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM pets WHERE name = ? AND lvl = ?")?;
        let pet_record = stmt.query_row(
            [self.name.to_string(), self.lvl.clamp(1, 3).to_string()],
            map_row_to_pet,
        )?;

        // Get new effect and replace.
        let effect = get_pet_effect(
            &self.name,
            &self.stats,
            Statistics {
                attack: pet_record.effect_atk,
                health: pet_record.effect_health,
            },
            (self.lvl + 1).clamp(1, 3),
            pet_record.n_triggers,
        );
        self.effect = effect;

        Ok(self)
    }
}
