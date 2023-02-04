use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    num::TryFromIntError,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use crate::{
    error::SAPTestError,
    pets::pet::{MAX_PET_STATS, MIN_PET_STATS},
};

/// Statistics for a [`Pet`](crate::pets::pet::Pet) or an [`Action`].
/// * Generally, a single integer value. ex. `50`
/// * But also, used as a **percentage** for certain pets.
///     * Ex. [`Skunk`](crate::pets::names::PetName::Skunk) or [`Leopard`](crate::pets::names::PetName::Leopard).
#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub struct Statistics {
    /// Attack for stats.
    pub attack: isize,
    /// Health for stats.
    pub health: isize,
}

impl Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.attack, self.health)
    }
}

impl Statistics {
    /// Constructor method for [`Statistics`].
    ///
    /// # Examples
    /// ```
    /// use sapt::Statistics;
    ///
    /// let ant_effect_stats = Statistics::new(2, 1).unwrap();
    /// assert_eq!(
    ///     ant_effect_stats,
    ///     Statistics {attack: 2, health: 1}
    /// )
    /// ```
    pub fn new<A, H>(attack: A, health: H) -> Result<Self, SAPTestError>
    where
        A: TryInto<isize>,
        H: TryInto<isize>,
        A::Error: Into<TryFromIntError>,
        H::Error: Into<TryFromIntError>,
    {
        let attack: isize = attack.try_into().map_err(Into::into)?;
        let health: isize = health.try_into().map_err(Into::into)?;
        Ok(Statistics { attack, health })
    }
}

impl Add for Statistics {
    type Output = Statistics;

    fn add(self, rhs: Self) -> Self::Output {
        Statistics {
            attack: self.attack + rhs.attack,
            health: self.health + rhs.health,
        }
    }
}

impl Sub for Statistics {
    type Output = Statistics;

    fn sub(self, rhs: Self) -> Self::Output {
        Statistics {
            attack: self.attack - rhs.attack,
            health: self.health - rhs.health,
        }
    }
}

impl Mul for Statistics {
    type Output = Statistics;

    fn mul(self, rhs: Self) -> Self::Output {
        let new_atk = (self.attack as f32 * (rhs.attack as f32 / 100.0)).round();
        let new_health = (self.health as f32 * (rhs.health as f32 / 100.0)).round();

        Statistics {
            attack: (new_atk as isize).clamp(MIN_PET_STATS, MAX_PET_STATS),
            health: (new_health as isize).clamp(MIN_PET_STATS, MAX_PET_STATS),
        }
    }
}

impl AddAssign for Statistics {
    fn add_assign(&mut self, rhs: Self) {
        self.attack = (self.attack + rhs.attack).clamp(MIN_PET_STATS, MAX_PET_STATS);
        self.health = (self.health + rhs.health).clamp(MIN_PET_STATS, MAX_PET_STATS);
    }
}

impl SubAssign for Statistics {
    fn sub_assign(&mut self, rhs: Self) {
        self.attack = (self.attack - rhs.attack).clamp(MIN_PET_STATS, MAX_PET_STATS);
        self.health = (self.health - rhs.health).clamp(MIN_PET_STATS, MAX_PET_STATS);
    }
}

impl MulAssign for Statistics {
    fn mul_assign(&mut self, rhs: Self) {
        let new_atk = (self.attack as f32 * (rhs.attack as f32 / 100.0)).round();
        let new_health = (self.health as f32 * (rhs.health as f32 / 100.0)).round();

        self.attack = (new_atk as isize).clamp(MIN_PET_STATS, MAX_PET_STATS);
        self.health = (new_health as isize).clamp(MIN_PET_STATS, MAX_PET_STATS);
    }
}

impl Statistics {
    /// Restrict values of `attack` and `health` to a given `min` and `max`.
    /// # Examples
    /// ```
    /// use sapt::Statistics;
    /// let mut effect_dmg = Statistics::new(-2, -2).unwrap();
    /// let mut stats = Statistics::new(6, 150).unwrap();
    ///
    /// effect_dmg.clamp(0, 50);
    /// stats.clamp(0, 50);
    ///
    /// assert_eq!(effect_dmg, Statistics::new(0, 0).unwrap());
    /// assert_eq!(stats, Statistics::new(6, 50).unwrap());
    /// ```
    pub fn clamp(&mut self, min: isize, max: isize) -> &mut Self {
        self.attack = self.attack.clamp(min, max);
        self.health = self.health.clamp(min, max);
        self
    }
    /// Set [`Statistics`] of any field to another given [`Statistics`] field based on if values are less than or equal to a given `min` value.
    ///
    /// # Examples
    /// ```rust
    /// use sapt::Statistics;
    ///
    /// let mut crab_stats = Statistics::new(3, 1).unwrap();
    /// let gorilla_stats = Statistics::new(6, 9).unwrap();
    ///
    /// // For crab, copy 50% of health. `Mul` impl always treats values as percentages.
    /// let mut copy_crab_stats = gorilla_stats * Statistics::new(0, 50).unwrap();
    /// assert_eq!(copy_crab_stats, Statistics::new(0, 5).unwrap());
    ///
    /// // If any field is less less than 1 (attack), use the provided stats instead.
    /// copy_crab_stats.comp_set_value(&mut crab_stats, 1);
    ///
    /// assert_eq!(copy_crab_stats, Statistics::new(3, 5).unwrap());
    /// ```
    pub fn comp_set_value(&mut self, other: &Statistics, min: isize) -> &Self {
        if self.attack <= min {
            self.attack = other.attack
        }
        if self.health <= min {
            self.health = other.health
        }
        self
    }
    /// Invert attack and health.
    /// # Examples
    /// ```rust
    /// use sapt::Statistics;
    ///
    /// let mut stats = Statistics::new(2, 1).unwrap();
    /// stats.invert();
    ///
    /// assert_eq!(
    ///     stats,
    ///     Statistics {health: 2, attack: 1}
    /// )
    /// ```
    pub fn invert(&mut self) -> &mut Self {
        std::mem::swap(&mut self.attack, &mut self.health);
        self
    }
}
