use crate::common::battle::state::{Action, Outcome, Position, Target};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Entity {
    Pet,
    Food,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Effect {
    pub entity: Entity,
    pub trigger: Outcome,
    pub target: Target,
    pub position: Position,
    pub action: Action,
    pub uses: Option<usize>,
    pub temp: bool,
}

impl Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Effect (Uses: {:?}): ({:?}) - Trigger: {} - Action: {:?} on {:?} ({:?}) ]",
            self.uses, self.entity, self.trigger, self.action, self.target, self.position
        )
    }
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
            *uses += n
        };
        self
    }

    fn remove_uses(&mut self, n: usize) -> &Self {
        if let Some(uses) = self.uses.as_mut() {
            if *uses >= n {
                *uses -= n
            }
        };
        self
    }
}
