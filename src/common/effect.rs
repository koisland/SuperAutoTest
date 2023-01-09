use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

use super::{food::Food, pet::Pet};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Statistics {
    pub attack: usize,
    pub health: usize,
}

impl Statistics {
    /// Add some `Statistics` to another capping at `50`.
    pub fn add(&mut self, stats: &Statistics) -> &Self {
        self.attack = (self.attack + stats.attack).clamp(0, 50);
        self.health = (self.health + stats.health).clamp(0, 50);
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Position {
    Any,
    All,
    Trigger,
    Specific(usize),
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum EffectType {
    Pet,
    Food,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Effect {
    pub effect_type: EffectType,
    pub trigger: Outcome,
    pub target: Target,
    pub position: Position,
    pub effect: EffectAction,
    pub uses: Option<Rc<RefCell<usize>>>,
}

pub trait Modify {
    /// Add `n` uses to a `Effect` if possible.
    fn add_uses(&mut self, n: usize) -> &Self;

    /// Remove `n` uses to a `Effect` if possible.
    fn remove_uses(&mut self, n: usize) -> &Self;
}

impl Modify for Effect {
    fn add_uses(&mut self, n: usize) -> &Self {
        if let Some(uses) = self.uses.as_mut() {
            *uses.borrow_mut() += n
        };
        self
    }

    fn remove_uses(&mut self, n: usize) -> &Self {
        if let Some(uses) = self.uses.as_mut() {
            if *uses.borrow() >= n {
                *uses.borrow_mut() -= n
            }
        };
        self
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Target {
    OnSelf,
    Friend,
    Enemy,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Outcome {
    pub status: Status,
    pub target: Target,
    pub position: Position,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum EffectAction {
    Add(Statistics),
    Remove(Statistics),
    Negate(Statistics),
    Gain(Box<Food>),
    Summon(Option<Box<Pet>>),
    None,
    NotImplemented,
}
