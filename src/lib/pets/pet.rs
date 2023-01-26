use rand::random;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display};

use crate::{
    battle::{effect::Effect, state::Statistics},
    db::{setup::get_connection, utils::map_row_to_pet},
    foods::food::Food,
    pets::names::PetName,
};

/// Minimum pet level.
pub const MIN_PET_LEVEL: usize = 1;
/// Maximum pet level.
pub const MAX_PET_LEVEL: usize = 3;
/// Minimum pet stats value.
pub const MIN_PET_STATS: isize = 0;
/// Maximum pet stats value.
pub const MAX_PET_STATS: isize = 50;

/// A Super Auto Pet.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct Pet {
    /// An ID for a pet.
    pub id: Option<String>,
    /// Name for pet.
    pub name: PetName,
    /// Tier of pet.
    pub tier: usize,
    /// [`Statistics`] of pet.
    pub stats: Statistics,
    /// Level of pet.
    pub lvl: usize,
    /// Experience of pet.
    pub exp: usize,
    /// Pet [`Effect`]s.
    pub effect: Vec<Effect>,
    /// Held pet [`Food`] item.
    pub item: Option<Food>,
    /// Pet position on a [`Team`](crate::battle::team::Team).
    pub pos: Option<usize>,
    /// Cost of pet.
    pub cost: usize,
    /// Seed for pet RNG.
    /// * Used in damage calculation for items [`Fortune Cookie`](crate::foods::names::FoodName::FortuneCookie)
    pub seed: u64,
}

impl PartialEq for Pet {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.tier == other.tier
            && self.stats == other.stats
            && self.lvl == other.lvl
            && self.exp == other.exp
            && self.effect == other.effect
            && self.item == other.item
            && self.pos == other.pos
            && self.cost == other.cost
    }
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
    /// Create a new pet.
    /// * All [`Effect`]s are determined by the given `stats` and `lvl`.
    ///     * To use custom [`Effect`]s, use the `Pet { ... }` constructor.
    /// * Providing `None` for `stats` will yield the default [`Statistics`] for the pet at the given `lvl`.
    /// * By default, pets are randomly seeded.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Statistics};
    ///
    /// let pet = Pet::new(
    ///     PetName::Ant,
    ///     Some("Ant".to_string()),
    ///     Some(Statistics::new(2, 1)),
    ///     1
    /// );
    /// let pet_with_no_stats = Pet::new(
    ///     PetName::Ant,
    ///     Some("Ant".to_string()),
    ///     None,
    ///     1
    /// );
    /// assert!(pet.is_ok() && pet_with_no_stats.is_ok());
    /// assert_eq!(
    ///     pet.unwrap().stats,
    ///     pet_with_no_stats.unwrap().stats
    /// )
    /// ```
    pub fn new(
        name: PetName,
        id: Option<String>,
        stats: Option<Statistics>,
        lvl: usize,
    ) -> Result<Pet, Box<dyn Error>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM pets WHERE name = ? AND lvl = ?")?;
        let mut pet_record = stmt.query_row([name.to_string(), lvl.to_string()], map_row_to_pet)?;

        // // Use default stats at level if stats not provided.
        // let pet_stats = stats.unwrap_or(Statistics {
        //     attack: isize::try_from(pet_record.attack)?.clamp(MIN_PET_STATS, MAX_PET_STATS),
        //     health: isize::try_from(pet_record.health)?.clamp(MIN_PET_STATS, MAX_PET_STATS),
        // });
        let pet_stats = if let Some(pet_stats) = stats {
            let atk = pet_stats.attack.clamp(MIN_PET_STATS, MAX_PET_STATS);
            let health = pet_stats.health.clamp(MIN_PET_STATS, MAX_PET_STATS);
            pet_record.attack = atk.try_into()?;
            pet_record.health = health.try_into()?;
            Statistics::new(atk, health)
        } else {
            Statistics::new(pet_record.attack.try_into()?, pet_record.health.try_into()?)
        };
        let (tier, lvl, cost) = (pet_record.tier, pet_record.lvl, pet_record.cost);
        let effect: Vec<Effect> = pet_record.try_into()?;

        Ok(Pet {
            id,
            name,
            tier,
            stats: pet_stats,
            lvl,
            exp: 0,
            effect,
            item: None,
            pos: None,
            cost,
            seed: random(),
        })
    }

    /// Get the effect of this pet at a given level.
    /// # Examples
    /// ```rust
    /// use sapt::{Pet, PetName, Statistics, battle::state::Action};
    ///
    /// let ant = Pet::from(PetName::Ant);
    ///
    /// // Get level 2 ant effect.
    /// let lvl_2_ant_action = &ant.get_effect(2).unwrap()[0].action;
    /// assert_eq!(
    ///     *lvl_2_ant_action,
    ///     Action::Add(Statistics::new(4,2))
    /// )
    /// ```
    pub fn get_effect(&self, lvl: usize) -> Result<Vec<Effect>, Box<dyn Error>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM pets WHERE name = ? AND lvl = ?")?;
        // Get pet stats and n_triggers from sqlite db. Otherwise, set to default.
        if let Ok(pet_record) =
            stmt.query_row([self.name.to_string(), lvl.to_string()], map_row_to_pet)
        {
            let effect: Vec<Effect> = pet_record.try_into()?;
            Ok(effect)
        } else {
            Err("No effect for pet at level.".into())
        }
    }

    #[allow(dead_code)]
    /// Add an experience point to a pet.
    /// * This will also update health and attack.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName};
    /// let mut pet = Pet::from(PetName::Ant);
    ///
    /// // Add single point.
    /// pet.add_experience(1).unwrap();
    /// assert!(pet.exp == 1 && pet.lvl == 1);
    /// assert!(pet.stats.attack == 3 && pet.stats.health == 2);
    ///
    /// // Add three points to reach level 2 and 4 total exp points.
    /// pet.add_experience(3).unwrap();
    /// assert!(pet.exp == 4 && pet.lvl == 2);
    /// assert!(pet.stats.attack == 6 && pet.stats.health == 5);
    ///
    /// // Add one point to reach level cap.
    /// pet.add_experience(1).unwrap();
    /// assert!(pet.exp == 5 && pet.lvl == 3);
    /// assert!(pet.stats.attack == 7 && pet.stats.health == 6);
    ///
    /// // Additional experience is not allowed.
    /// assert!(pet.add_experience(3).is_err())
    /// ```
    pub fn add_experience(&mut self, mut exp: usize) -> Result<&mut Self, Box<dyn Error>> {
        match self.lvl {
            MAX_PET_LEVEL => {
                return Err("Already at level cap.".into());
            }
            _ => {
                // lvl 1 -> lvl 2 (1 * (1-1) + (1+1)) = 2
                // lvl 2 -> lvl 3 (2 * (2-1) + (2+1)) = 5
                // lvl 3 -> lvl 4 (3 * (3-1) + (3+1)) = 10
                loop {
                    // Calculate required exp to level up.
                    let req_exp = (self.lvl * (self.lvl - 1)) + (self.lvl + 1);
                    if self.exp + exp >= req_exp {
                        self.exp += exp;
                        self.lvl += 1;

                        // Update effect and health.
                        self.set_level(self.lvl)?;
                        for _ in 0..exp {
                            self.stats.attack += 1;
                            self.stats.health += 1;
                        }
                        // Exit at level cap.
                        if self.lvl >= MAX_PET_LEVEL {
                            break;
                        }
                        // Exp already added so set to 0.
                        exp = 0
                    } else {
                        self.exp += exp;

                        for _ in 0..exp {
                            self.stats.attack += 1;
                            self.stats.health += 1;
                        }

                        break;
                    }
                }
            }
        };
        Ok(self)
    }

    /// Set the level of this pet.
    ///
    /// # Examples
    /// ```rust
    /// use sapt::{Pet, PetName};
    /// let mut pet = Pet::from(PetName::Ant);
    ///
    /// assert!(pet.set_level(2).is_ok());
    /// assert_eq!(pet.lvl, 2);
    /// // Invalid level.
    /// assert!(pet.set_level(5).is_err());
    /// ```
    pub fn set_level(&mut self, lvl: usize) -> Result<&mut Self, Box<dyn Error>> {
        if !(MIN_PET_LEVEL..=MAX_PET_LEVEL).contains(&lvl) {
            Err("Not a valid level.".into())
        } else {
            self.lvl = lvl;
            self.effect = self.get_effect(self.lvl)?;
            Ok(self)
        }
    }

    /// Helper function to set pet idx for matching on effect triggers.
    /// * Note: This does not update other pets on the same [`Team`](crate::battle::team::Team).
    pub fn set_pos(&mut self, pos: usize) -> &mut Self {
        self.pos = Some(pos);
        for effect in self.effect.iter_mut() {
            effect.trigger.idx = Some(pos)
        }
        self
    }
}
