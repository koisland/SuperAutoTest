use rand::random;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::{
    battle::{effect::Effect, stats::Statistics},
    db::record::PetRecord,
    error::SAPTestError,
    foods::food::Food,
    pets::names::PetName,
    SAPDB,
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
    /// An identifier for a pet.
    pub id: Option<String>,
    /// Name for pet.
    pub name: PetName,
    /// Tier of pet.
    pub tier: usize,
    /// [`Statistics`] of pet.
    pub stats: Statistics,
    /// Pet [`Effect`]s.
    pub effect: Vec<Effect>,
    /// Held pet [`Food`] item.
    pub item: Option<Food>,
    /// Cost of pet.
    pub cost: usize,
    /// Seed for pet RNG.
    /// * Used in damage calculation for items like [`Fortune Cookie`](crate::foods::names::FoodName::FortuneCookie)
    pub seed: u64,
    /// Level of pet.
    pub(crate) lvl: usize,
    /// Experience of pet.
    pub(crate) exp: usize,
    /// Pet position on a [`Team`](crate::battle::team::Team).
    pub(crate) pos: Option<usize>,
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
        Pet::new(value, None, None, 1)
    }
}

impl TryFrom<PetRecord> for Pet {
    type Error = SAPTestError;

    fn try_from(record: PetRecord) -> Result<Pet, SAPTestError> {
        let pet_stats = Statistics::new(record.attack, record.health)?;
        let (tier, lvl, cost) = (record.tier, record.lvl, record.cost);
        let pet_name = record.name.clone();
        let effect: Vec<Effect> = record.try_into()?;

        Ok(Pet {
            id: None,
            name: pet_name,
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
        let conn = SAPDB.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM pets WHERE name = ? AND lvl = ?")?;
        let pet_record: PetRecord = stmt
            .query([name.to_string(), lvl.to_string()])?
            .next()?
            .ok_or(SAPTestError::QueryFailure {
                subject: "No Pet Found".to_string(),
                reason: format!("No pet ({name}) found at level ({lvl})."),
            })?
            .try_into()?;

        let mut pet = Pet::try_from(pet_record)?;

        // Use given stats if provided.
        if let Some(pet_stats) = stats {
            pet.stats.attack = pet_stats.attack.clamp(MIN_PET_STATS, MAX_PET_STATS);
            pet.stats.health = pet_stats.health.clamp(MIN_PET_STATS, MAX_PET_STATS);
        };

        // Assign id if any.
        pet.id = id;

        Ok(pet)
    }

    /// Build a custom pet.
    /// * Custom pets have `level` and `tier` of `0` by default.
    /// # Example
    /// ```rust
    /// use sapt::{
    ///     battle::{
    ///         trigger::TRIGGER_START_BATTLE,
    ///         effect::Entity,
    ///         actions::{Action, GainType},
    ///         state::{Position, Status, Target},
    ///     },
    ///     Effect, Food, FoodName, Outcome, Pet, Statistics,
    /// };
    /// let custom_pet = Pet::custom(
    ///     "MelonBear", Some("id_custom_pet_1".to_string()),
    ///     Statistics::new(50, 50).unwrap(),
    ///     &[
    ///         Effect::new(
    ///             Entity::Pet,
    ///             TRIGGER_START_BATTLE,
    ///             Target::Friend,
    ///             Position::Adjacent,
    ///             Action::Gain(GainType::StoredItem(Box::new(Food::try_from(FoodName::Melon).unwrap()))),
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
    /// use sapt::{Pet, PetName, Statistics, battle::actions::{Action, StatChangeType}};
    ///
    /// let ant = Pet::try_from(PetName::Ant).unwrap();
    ///
    /// // Get level 2 ant effect.
    /// let lvl_2_ant_action = &ant.get_effect(2).unwrap()[0].action;
    /// assert_eq!(
    ///     *lvl_2_ant_action,
    ///     Action::Add(StatChangeType::StaticValue(Statistics::new(4,2).unwrap()))
    /// )
    /// ```
    pub fn get_effect(&self, lvl: usize) -> Result<Vec<Effect>, SAPTestError> {
        SAPDB
            .execute_pet_query(
                "SELECT * FROM pets WHERE name = ? AND lvl = ?",
                &[self.name.to_string(), lvl.to_string()],
            )?
            .into_iter()
            .next()
            .ok_or(SAPTestError::QueryFailure {
                subject: "No Pet Effect".to_string(),
                reason: format!("No effect for {} at level {lvl}.", self.name),
            })?
            .try_into()
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
            // Reassign owner if any.
            let owner = self
                .effect
                .first()
                .map(|effect| effect.owner.clone())
                .unwrap_or(None);
            self.effect = self.get_effect(self.lvl)?;
            if let Some(Some(owner)) = owner.map(|pet_ref| pet_ref.upgrade()) {
                for effect in self.effect.iter_mut() {
                    effect.assign_owner(Some(&owner));
                }
            }
            Ok(self)
        }
    }

    /// Helper function to set pet idx for matching on effect triggers.
    /// * Note: This does not update other pets on the same [`Team`](crate::battle::team::Team).
    pub(crate) fn set_pos(&mut self, pos: usize) -> &mut Self {
        self.pos = Some(pos);
        self
    }
}
