use rand::random;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::{
    battle::{effect::Effect, state::Action, stats::Statistics},
    db::{setup::get_connection, utils::map_row_to_pet},
    error::SAPTestError,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub(crate) lvl: usize,
    /// Experience of pet.
    pub(crate) exp: usize,
    /// Pet [`Effect`]s.
    pub effect: Vec<Effect>,
    /// Held pet [`Food`] item.
    pub item: Option<Food>,
    /// Pet position on a [`Team`](crate::battle::team::Team).
    pub(crate) pos: Option<usize>,
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

impl TryFrom<PetName> for Pet {
    type Error = SAPTestError;

    fn try_from(value: PetName) -> Result<Pet, SAPTestError> {
        let def_name = value.to_string();
        Pet::new(value, Some(def_name), None, 1)
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
    ///     Some(Statistics::new(2, 1).unwrap()),
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
    ) -> Result<Pet, SAPTestError> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM pets WHERE name = ? AND lvl = ?")?;
        let mut pet_record = stmt.query_row([name.to_string(), lvl.to_string()], map_row_to_pet)?;

        // Use record stats if none provided.
        let pet_stats = if let Some(pet_stats) = stats {
            let atk = pet_stats.attack.clamp(MIN_PET_STATS, MAX_PET_STATS);
            let health = pet_stats.health.clamp(MIN_PET_STATS, MAX_PET_STATS);
            pet_record.attack = atk.try_into()?;
            pet_record.health = health.try_into()?;
            Statistics::new(atk, health)?
        } else {
            Statistics::new(pet_record.attack, pet_record.health)?
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

    /// Build a custom pet.
    /// * Custom pets have `level` and `tier` of `0` by default.
    /// # Example
    /// ```rust
    /// use sapt::{
    ///     battle::{
    ///         effect::Entity,
    ///         state::{Action, Position, Status, Target},
    ///     },
    ///     Effect, Food, FoodName, Outcome, Pet, Statistics,
    /// };
    /// let custom_pet = Pet::custom(
    ///     "MelonBear", Some("id_custom_pet_1".to_string()),
    ///     Statistics::new(50, 50).unwrap(),
    ///     &[
    ///         Effect::new(
    ///             Entity::Pet,
    ///             Outcome {
    ///                 status: Status::StartOfBattle,
    ///                 to_target: Target::None,
    ///                 from_target: Target::None,
    ///                 position: Position::None,
    ///                 to_idx: None,
    ///                 from_idx: None,
    ///                 stat_diff: None,
    ///             },
    ///             Target::Friend,
    ///             Position::Adjacent,
    ///             Action::Gain(Some(Box::new(Food::try_from(FoodName::Melon).unwrap()))),
    ///             Some(1),
    ///             false,
    ///     )],
    /// );
    /// ```
    pub fn custom(name: &str, id: Option<String>, stats: Statistics, effect: &[Effect]) -> Pet {
        let mut adj_stats = stats;
        adj_stats.clamp(MIN_PET_STATS, MAX_PET_STATS);

        Pet {
            id,
            tier: 0,
            name: PetName::Custom(name.to_string()),
            stats: adj_stats,
            lvl: 1,
            exp: 0,
            effect: effect.to_vec(),
            item: None,
            pos: None,
            cost: 3,
            seed: random(),
        }
    }

    /// Get the effect of this pet at a given level.
    /// # Examples
    /// ```rust
    /// use sapt::{Pet, PetName, Statistics, battle::state::Action};
    ///
    /// let ant = Pet::try_from(PetName::Ant).unwrap();
    ///
    /// // Get level 2 ant effect.
    /// let lvl_2_ant_action = &ant.get_effect(2).unwrap()[0].action;
    /// assert_eq!(
    ///     *lvl_2_ant_action,
    ///     Action::Add(Statistics::new(4,2).unwrap())
    /// )
    /// ```
    pub fn get_effect(&self, lvl: usize) -> Result<Vec<Effect>, SAPTestError> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM pets WHERE name = ? AND lvl = ?")?;
        // Get pet stats and n_triggers from sqlite db. Otherwise, set to default.
        if let Ok(pet_record) =
            stmt.query_row([self.name.to_string(), lvl.to_string()], map_row_to_pet)
        {
            let effects: Vec<Effect> = pet_record.try_into()?;
            Ok(effects)
        } else {
            Err(SAPTestError::QueryFailure {
                subject: "No Effect".to_string(),
                reason: format!("No effect for {} at level {lvl}.", self.name),
            })
        }
    }

    /// Get pet experience.
    /// # Example
    /// ```
    /// use sapt::{Pet, PetName};
    ///
    /// let pet = Pet::try_from(PetName::Ant).unwrap();
    ///
    /// assert_eq!(pet.get_experience(), 0)
    /// ```
    pub fn get_experience(&self) -> usize {
        self.exp
    }

    /// Get pet level.
    /// # Example
    /// ```
    /// use sapt::{Pet, PetName};
    ///
    /// let pet = Pet::try_from(PetName::Ant).unwrap();
    ///
    /// assert_eq!(pet.get_level(), 1)
    /// ```
    pub fn get_level(&self) -> usize {
        self.lvl
    }

    /// Add an experience point to a pet.
    /// * This will also increase health (`+1`) and attack (`+1`) per experience point.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName};
    /// let mut pet = Pet::try_from(PetName::Ant).unwrap();
    ///
    /// // Add single point.
    /// pet.add_experience(1).unwrap();
    /// assert!(pet.get_experience() == 1 && pet.get_level() == 1);
    /// assert!(pet.stats.attack == 3 && pet.stats.health == 2);
    ///
    /// // Add three points to reach level 2 and 4 total exp points.
    /// pet.add_experience(3).unwrap();
    /// assert!(pet.get_experience() == 4 && pet.get_level() == 2);
    /// assert!(pet.stats.attack == 6 && pet.stats.health == 5);
    ///
    /// // Add one point to reach level cap.
    /// pet.add_experience(1).unwrap();
    /// assert!(pet.get_experience() == 5 && pet.get_level() == 3);
    /// assert!(pet.stats.attack == 7 && pet.stats.health == 6);
    ///
    /// // Additional experience is not allowed.
    /// assert!(pet.add_experience(3).is_err())
    /// ```
    pub fn add_experience(&mut self, mut exp: usize) -> Result<&mut Self, SAPTestError> {
        match self.lvl {
            MAX_PET_LEVEL => {
                return Err(SAPTestError::InvalidPetAction {
                    subject: "Max Level".to_string(),
                    reason: format!("{} at max level {}.", self.name, self.lvl),
                });
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
    /// * Note: This only adjusts level and effect. Stats and previous experience are unaltered.
    /// # Examples
    /// ```rust
    /// use sapt::{Pet, PetName};
    /// let mut pet = Pet::try_from(PetName::Ant).unwrap();
    ///
    /// assert!(pet.set_level(2).is_ok());
    /// assert_eq!(pet.get_level(), 2);
    /// // Invalid level.
    /// assert!(pet.set_level(5).is_err());
    /// ```
    pub fn set_level(&mut self, lvl: usize) -> Result<&mut Self, SAPTestError> {
        if !(MIN_PET_LEVEL..=MAX_PET_LEVEL).contains(&lvl) {
            Err(SAPTestError::InvalidPetAction {
                subject: "Invalid Level".to_string(),
                reason: format!("{} cannot be set to {}.", self.name, lvl),
            })
        } else {
            self.lvl = lvl;
            self.effect = self.get_effect(self.lvl)?;
            Ok(self)
        }
    }

    /// Helper function to set pet idx for matching on effect triggers.
    /// * Note: This does not update other pets on the same [`Team`](crate::battle::team::Team).
    pub(crate) fn set_pos(&mut self, pos: usize) -> &mut Self {
        self.pos = Some(pos);
        self
    }

    /// Updates missing food items from an [`Action::Gain`](crate::battle::state::Action::Gain) effect.
    /// * Specifically for [`Toucan`](crate::pets::names::PetName::Toucan).
    ///
    /// ```rust
    /// use sapt::{Pet, PetName, Food, FoodName, Team, EffectApply, battle::state::Action};
    ///
    /// let honey = Food::try_from(FoodName::Honey).unwrap();
    /// let mut toucan = Pet::try_from(PetName::Toucan).unwrap();
    /// toucan.item = Some(honey.clone());
    ///
    /// assert_eq!(
    ///     toucan.effect.first().unwrap().action,
    ///     Action::Gain(None)
    /// );
    ///
    /// let team = Team::new(&[Some(toucan)], 5).unwrap();
    ///
    /// assert_eq!(
    ///     team.friends
    ///         .first().unwrap().as_ref().unwrap()
    ///         .effect.first().unwrap()
    ///         .action,
    ///     Action::Gain(Some(Box::new(honey)))
    /// )
    /// ```
    pub(crate) fn update_missing_food_effects(&mut self) {
        for effect in self.effect.iter_mut() {
            let effect_missing_food = if let Action::Gain(food) = &effect.action {
                food.is_none()
            } else {
                false
            };
            if self.item.as_ref().is_some() && effect_missing_food {
                effect.action = Action::Gain(Some(Box::new(self.item.as_ref().unwrap().clone())))
            }
        }
    }
}
