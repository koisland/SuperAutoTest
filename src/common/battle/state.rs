use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{AddAssign, MulAssign, RangeInclusive},
};

use crate::common::{
    foods::food::Food,
    pets::pet::{Pet, MAX_PET_STATS, MIN_PET_STATS},
};

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Statistics {
    pub attack: isize,
    pub health: isize,
}

impl AddAssign for Statistics {
    fn add_assign(&mut self, rhs: Self) {
        self.attack = (self.attack + rhs.attack).clamp(MIN_PET_STATS, MAX_PET_STATS);
        self.health = (self.health + rhs.health).clamp(MIN_PET_STATS, MAX_PET_STATS);
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
    pub fn clamp(&mut self, min: isize, max: isize) -> &mut Self {
        self.attack = self.attack.clamp(min, max);
        self.health = self.health.clamp(min, max);
        self
    }
    /// Set `Statistics` to another given `Statistics` based on if values are less than or equal to a given `min` value.
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
    pub fn invert(&mut self) -> &mut Self {
        std::mem::swap(&mut self.attack, &mut self.health);
        self
    }
}

impl Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.attack, self.health)
    }
}

/// Conditions to select pets by.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Condition {
    Healthiest,
    Illest,
    Strongest,
    Weakest,
}

/// Positions to select pets by.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Position {
    Any,
    All,
    OnSelf,
    Trigger,
    Range(RangeInclusive<isize>),
    Specific(isize),
    Condition(Condition),
    Multiple(Vec<Position>),
    None,
}

/// Target team for an effect.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Target {
    Friend,
    Enemy,
    Either,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq)]
pub struct Outcome {
    pub status: Status,
    pub target: Target,
    pub position: Position,
    pub idx: Option<usize>,
}

impl PartialEq for Outcome {
    fn eq(&self, other: &Self) -> bool {
        self.status == other.status
            && self.target == other.target
            && self.position == other.position
    }
}

impl Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Status: {:?}, Target: {:?}, Position: {:?}, Index: {:?}]",
            self.status, self.target, self.position, self.idx
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Status {
    StartBattle,
    Attack,
    Hurt,
    Faint,
    KnockOut,
    Summoned,
    Pushed,
    None,
    NotImplemented,
}

/// General Pet attribute use for `Action::Copy`.
///
/// Statistics for `Health` or `Attack` are a set percentage.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum CopyAttr {
    PercentStats(Statistics),
    Effect,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Action {
    Add(Statistics),
    Remove(Statistics),
    Copy(CopyAttr, Position),
    Negate(Statistics),
    Critical(usize),
    Kill,
    Invincible,
    Gain(Box<Food>),
    Summon(Option<Box<Pet>>),
    Multiple(Vec<Action>),
    None,
    NotImplemented,
}
