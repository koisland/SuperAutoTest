use crate::common::pets::pet::MAX_PET_STATS;

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Statistics {
    pub attack: usize,
    pub health: usize,
}

impl Statistics {
    /// Add some `Statistics` to another capping at `50`.
    pub fn add(&mut self, stats: &Statistics) -> &mut Self {
        self.attack = (self.attack + stats.attack).clamp(1, MAX_PET_STATS);
        self.health = (self.health + stats.health).clamp(1, MAX_PET_STATS);
        self
    }
    #[allow(dead_code)]
    /// Subtract some `Statistics` from another.
    pub fn sub(&mut self, stats: &Statistics) -> &mut Self {
        self.attack = self.attack.saturating_sub(stats.attack);
        self.health = self.health.saturating_sub(stats.health);
        self
    }
    /// Multiply some `Statistics` by another.
    pub fn mult(&self, perc_stats_mult: &Statistics) -> Statistics {
        let new_atk = (self.attack as f32 * (perc_stats_mult.attack as f32 / 100.0)).round();
        let new_health = (self.health as f32 * (perc_stats_mult.health as f32 / 100.0)).round();
        Statistics {
            attack: (new_atk as usize).clamp(0, MAX_PET_STATS),
            health: (new_health as usize).clamp(0, MAX_PET_STATS),
        }
    }
    /// Set `Statistics` to another given `Statistics` based on if values are less than or equal to a given `min` value.
    pub fn comp_set_value(&mut self, other: &Statistics, min: usize) -> &Self {
        if self.attack <= min {
            self.attack = other.attack
        }
        if self.health <= min {
            self.health = other.health
        }
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
    None,
}

/// Target team for an effect.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Target {
    Friend,
    Enemy,
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
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::RangeInclusive};

use crate::common::{foods::food::Food, pets::pet::Pet};

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
    Gain(Box<Food>),
    Summon(Option<Box<Pet>>),
    None,
    NotImplemented,
}
