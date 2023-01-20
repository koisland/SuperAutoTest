use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, error::Error, fmt::Display};

use crate::{
    common::{
        battle::{effect::Effect, state::Statistics},
        foods::food::Food,
        pets::{effects::get_pet_effect, names::PetName},
        regex_patterns::*,
    },
    db::{setup::get_connection, utils::map_row_to_pet},
};

pub const MIN_PET_LEVEL: usize = 1;
pub const MAX_PET_LEVEL: usize = 3;
pub const MIN_PET_STATS: isize = 0;
pub const MAX_PET_STATS: isize = 50;

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
    pub id: Option<String>,
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

impl From<PetName> for Pet {
    fn from(value: PetName) -> Pet {
        let def_name = value.to_string();
        Pet::new(value, Some(def_name), None, 1).expect("Cannot create default pet.")
    }
}

impl Pet {
    /// Create a new `Pet` with given id, stats, and level.
    pub fn new(
        name: PetName,
        id: Option<String>,
        stats: Option<Statistics>,
        lvl: usize,
    ) -> Result<Pet, Box<dyn Error>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM pets WHERE name = ? AND lvl = ?")?;
        let pet_record = stmt.query_row([name.to_string(), lvl.to_string()], map_row_to_pet)?;

        // Use default stats at level if stats not provided.
        let pet_stats = stats.unwrap_or(Statistics {
            attack: isize::try_from(pet_record.attack)?.clamp(MIN_PET_STATS, MAX_PET_STATS),
            health: isize::try_from(pet_record.health)?.clamp(MIN_PET_STATS, MAX_PET_STATS),
        });

        let effect = get_pet_effect(
            &name,
            &pet_stats,
            Statistics {
                attack: isize::try_from(pet_record.effect_atk)?.clamp(MIN_PET_STATS, MAX_PET_STATS),
                health: isize::try_from(pet_record.effect_health)?
                    .clamp(MIN_PET_STATS, MAX_PET_STATS),
            },
            lvl,
            pet_record.n_triggers,
        );

        Ok(Pet {
            id,
            name,
            tier: pet_record.tier,
            stats: pet_stats,
            lvl: pet_record.lvl,
            effect,
            item: None,
            pos: None,
        })
    }

    /// Get the effect of this `Pet` at a given level.
    pub fn get_effect(&self, lvl: usize) -> Result<Option<Effect>, Box<dyn Error>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM pets WHERE name = ? AND lvl = ?")?;
        // Get pet stats and n_triggers from sqlite db. Otherwise, set to default.
        let (pet_effect_stats, n_triggers) = if let Ok(pet_record) =
            stmt.query_row([self.name.to_string(), lvl.to_string()], map_row_to_pet)
        {
            (
                Statistics {
                    attack: isize::try_from(pet_record.effect_atk)?
                        .clamp(MIN_PET_STATS, MAX_PET_STATS),
                    health: isize::try_from(pet_record.effect_health)?
                        .clamp(MIN_PET_STATS, MAX_PET_STATS),
                },
                pet_record.n_triggers,
            )
        } else {
            (Statistics::default(), 1)
        };

        // Get new effect and replace.
        Ok(get_pet_effect(
            &self.name,
            &self.stats,
            pet_effect_stats,
            lvl,
            n_triggers,
        ))
    }

    /// Set the level of this `Pet`.
    pub fn set_level(&mut self, lvl: usize) -> Result<&mut Self, Box<dyn Error>> {
        if !(MIN_PET_LEVEL..=MAX_PET_LEVEL).contains(&lvl) {
            Err("Not a valid level.".into())
        } else {
            self.lvl = lvl;
            self.effect = self.get_effect(self.lvl)?;
            Ok(self)
        }
    }

    /// Helper function to set pet position for matching on effect triggers.
    ///
    /// * Note: This does not update other pets on the same team.
    pub fn set_pos(&mut self, pos: usize) -> &mut Self {
        self.pos = Some(pos);
        if let Some(effect) = self.effect.as_mut() {
            effect.trigger.idx = Some(pos)
        }
        self
    }
}
