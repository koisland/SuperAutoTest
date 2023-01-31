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
    /// Owner target of effect.
    pub(crate) owner_target: Option<Target>,
    /// Idx of owner.
    pub(crate) owner_idx: Option<usize>,
    /// Effect trigger.
    pub trigger: Outcome,
    /// Target of the effect.
    pub target: Target,
    /// Position of target to affect.
    pub position: Position,
    /// Action to take.
    pub action: Action,
    /// Number of uses of [`Effect`] per trigger.
    /// * `None` indicates unlimited uses.
    pub uses: Option<usize>,
    /// If the [`Effect`] is temporary or not.
    pub temp: bool,
}

impl Effect {
    /// Generate a new effect.
    /// # Example
    /// ```rust
    /// use sapt::{
    ///     Effect, Outcome, Statistics,
    ///     battle::{
    ///         effect::Entity,
    ///         trigger::TRIGGER_SELF_FAINT,
    ///         state::{Position, Action, Target, Condition}
    ///     }
    /// };
    /// let lvl_1_ant_effect = Effect::new(
    ///     Entity::Pet,
    ///     TRIGGER_SELF_FAINT,
    ///     Target::Friend,
    ///     Position::Any(Condition::None),
    ///     Action::Add(Statistics {attack: 2, health: 1}),
    ///     Some(1),
    ///     false
    /// );
    /// ```
    pub fn new(
        effect_type: Entity,
        trigger: Outcome,
        target: Target,
        position: Position,
        action: Action,
        uses: Option<usize>,
        temporary: bool,
    ) -> Self {
        Effect {
            entity: effect_type,
            owner_target: None,
            owner_idx: None,
            trigger,
            target,
            position,
            action,
            uses,
            temp: temporary,
        }
    }
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
    /// # Example
    /// ```
    /// use sapt::{Pet, PetName, Effect, battle::effect::Modify};
    ///
    /// let mut dolphin = Pet::try_from(PetName::Dolphin).unwrap();
    /// let dolphin_effect = dolphin.effect.first_mut().unwrap();
    /// assert_eq!(dolphin_effect.uses, Some(1));
    /// // Add a use.
    /// dolphin_effect.add_uses(1);
    /// assert_eq!(dolphin_effect.uses, Some(2));
    /// ```
    fn add_uses(&mut self, n: usize) -> &Self;

    /// Remove `n` uses to a [`Effect`] if possible.
    /// # Example
    /// ```
    /// use sapt::{Pet, PetName, Effect, battle::effect::Modify};
    ///
    /// let mut dolphin = Pet::try_from(PetName::Dolphin).unwrap();
    /// let dolphin_effect = dolphin.effect.first_mut().unwrap();
    /// assert_eq!(dolphin_effect.uses, Some(1));
    /// // Add a use.
    /// dolphin_effect.remove_uses(1);
    /// assert_eq!(dolphin_effect.uses, Some(0));
    /// ```
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
