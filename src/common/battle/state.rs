use crate::common::pets::pet::MAX_PET_HEALTH;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Statistics {
    pub attack: usize,
    pub health: usize,
}

impl Statistics {
    /// Add some `Statistics` to another capping at `50`.
    pub fn add(&mut self, stats: &Statistics) -> &Self {
        self.attack = (self.attack + stats.attack).clamp(0, MAX_PET_HEALTH);
        self.health = (self.health + stats.health).clamp(0, MAX_PET_HEALTH);
        self
    }
    #[allow(dead_code)]
    /// Subtract some `Statistics` from another.
    pub fn sub(&mut self, stats: &Statistics) -> &Self {
        self.attack = self.attack.saturating_sub(stats.attack);
        self.health = self.health.saturating_sub(stats.health);
        self
    }
}

impl Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.attack, self.health)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Position {
    Any,
    All,
    OnSelf,
    Trigger,
    Range(RangeInclusive<isize>),
    Specific(isize),
    None,
}

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Action {
    Add(Statistics),
    Remove(Statistics),
    CopyStatsHealthiest,
    Negate(Statistics),
    Gain(Box<Food>),
    Summon(Option<Box<Pet>>),
    None,
    NotImplemented,
}
