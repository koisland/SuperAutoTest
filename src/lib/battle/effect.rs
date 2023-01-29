use crate::battle::state::{Action, Outcome, Position, Target};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Owner of [`Effect`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Entity {
    /// A [`Pet`](crate::pets::pet::Pet).
    Pet,
    /// A [`Food`](crate::foods::food::Food).
    Food,
}

/// An effect for an [`Entity`] in Super Auto Pets.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Effect {
    /// Owner of effect.
    pub entity: Entity,
    /// Idx of owner.
    pub owner_idx: Option<usize>,
    /// Effect trigger.
    pub trigger: Outcome,
    /// Target of the effect.
    pub target: Target,
    /// Position of target to affect.
    pub position: Position,
    /// Action to take.
    pub action: Action,
    /// Number of uses of [`Effect`].
    /// * `None` indicates unlimited uses.
    pub uses: Option<usize>,
    /// If the [`Effect`] is temporary or not.
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

/// Allow modification of an [`Effect`].
pub trait Modify {
    /// Add `n` uses to a [`Effect`] if possible.
    fn add_uses(&mut self, n: usize) -> &Self;

    /// Remove `n` uses to a [`Effect`] if possible.
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
