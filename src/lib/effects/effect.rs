use crate::{
    effects::{
        actions::Action,
        state::{Outcome, Position, Target},
    },
    error::SAPTestError,
    toys::names::ToyName,
    FoodName, Pet, PetName,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock, Weak};

/// A Super Auto Pets entity.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
pub enum Entity {
    #[default]
    /// A [`Pet`](crate::pets::pet::Pet).
    Pet,
    /// A [`Food`](crate::foods::food::Food).
    Food,
    /// A [`Toy`](crate::toys::toy::Toy)
    Toy,
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
/// [`Entity`] names.
pub enum EntityName {
    /// Pet name.
    Pet(PetName),
    /// Food name.
    Food(FoodName),
    /// Toy name.
    Toy(ToyName),
}

/// An effect for an [`Entity`] in Super Auto Pets.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Effect {
    #[serde(skip)]
    /// Idx of owner.
    pub(crate) owner: Option<Weak<RwLock<Pet>>>,
    /// Effect trigger.
    pub trigger: Outcome,
    /// Target of the effect.
    pub target: Target,
    /// Position of target to affect.
    pub position: Position,
    /// Action to take.
    pub action: Action,
    /// Number of uses of effect per trigger.
    /// * `None` indicates unlimited uses.
    pub uses: Option<usize>,
    /// If the effect is temporary or not.
    pub temp: bool,
}

impl PartialEq for Effect {
    fn eq(&self, other: &Self) -> bool {
        self.trigger == other.trigger
            && self.target == other.target
            && self.position == other.position
            && self.action == other.action
            && self.uses == other.uses
            && self.temp == other.temp
    }
}

impl TryFrom<&Effect> for Arc<RwLock<Pet>> {
    type Error = SAPTestError;

    fn try_from(effect: &Effect) -> Result<Self, Self::Error> {
        effect
            .owner
            .as_ref()
            .ok_or(SAPTestError::InvalidTeamAction {
                subject: "Missing Effect Owner".to_string(),
                reason: format!("{effect:?} has no owner."),
            })?
            .upgrade()
            .ok_or(SAPTestError::InvalidTeamAction {
                subject: "Dropped Owner".to_string(),
                reason: "Pet reference dropped.".to_string(),
            })
    }
}

impl Effect {
    /// Generate a new effect.
    /// # Example
    /// ```rust
    /// use saptest::{
    ///     Effect, Statistics,
    ///     effects::{
    ///         trigger::TRIGGER_SELF_FAINT,
    ///         state::{Position, Target, ItemCondition, Outcome},
    ///         actions::{Action, StatChangeType}
    ///     }
    /// };
    /// let lvl_1_ant_effect = Effect::new(
    ///     TRIGGER_SELF_FAINT,
    ///     Target::Friend,
    ///     Position::Any(ItemCondition::None),
    ///     Action::Add(StatChangeType::Static(Statistics {attack: 2, health: 1})),
    ///     Some(1),
    ///     false
    /// );
    /// ```
    pub fn new(
        trigger: Outcome,
        target: Target,
        position: Position,
        action: Action,
        uses: Option<usize>,
        temporary: bool,
    ) -> Self {
        Effect {
            owner: None,
            trigger,
            target,
            position,
            action,
            uses,
            temp: temporary,
        }
    }

    /// Get owner of effect.
    /// # Example
    /// ```
    /// use saptest::Effect;
    /// // By default, no effect owner.
    /// let mut effect = Effect::default();
    /// assert!(effect.get_owner().is_none());
    /// ```
    pub fn get_owner(&self) -> Option<Weak<RwLock<Pet>>> {
        self.owner.as_ref().cloned()
    }

    /// Assign this effect to a pet.
    /// * Used in [`Team`](crate::Team) to assign and track owners of [`Effect`]s.
    /// # Example
    /// ```rust
    /// use std::sync::{Arc, RwLock};
    /// use saptest::{Pet, PetName, Effect};
    ///
    /// let ant = Pet::try_from(PetName::Ant).unwrap();
    /// let rc_ant = Arc::new(RwLock::new(ant));
    ///
    /// // Create an example effect.
    /// let mut effect = Effect::default();
    /// // ^ and its trigger now belongs to ant.
    /// effect.assign_owner(Some(&rc_ant));
    /// assert!(effect.get_owner().unwrap().ptr_eq(&Arc::downgrade(&rc_ant)))
    /// ```
    pub fn assign_owner(&mut self, owner: Option<&Arc<RwLock<Pet>>>) -> &mut Self {
        let owner_ref = owner.map(Arc::downgrade);
        self.owner = owner_ref.clone();
        self.trigger.affected_pet = owner_ref;
        self
    }

    /// Used to check if can be triggered by the provided [`Outcome`].
    ///
    /// Must be:
    /// * Exact match of trigger or a non-specific position and matches exactly on position, affected [`Target`], and [`Status`](crate::effects::state::Status).
    /// * Not out of uses.
    /// # Example
    /// ```
    /// use saptest::{Pet, PetName, effects::trigger::TRIGGER_START_BATTLE};
    ///
    /// let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// let mosquito_effect = mosquito.effect.first().unwrap();
    /// assert!(mosquito_effect.check_activates(&TRIGGER_START_BATTLE));
    /// ```
    pub fn check_activates(&self, trigger: &Outcome) -> bool {
        let exact_match = self.trigger == *trigger;
        // Allows triggers for effects that activate on any position/positions. ex. Horse.
        let non_specific_match = self.trigger.position.is_non_specific()
            && self.trigger.position == trigger.position
            && self.trigger.affected_team == trigger.affected_team
            && self.trigger.status == trigger.status;
        // Either match and not out of uses.
        exact_match || non_specific_match && self.uses != Some(0)
    }
}

/// Allow modification of an [`Effect`].
pub trait EffectModify {
    /// Add `n` uses to a [`Effect`] if possible.
    /// # Example
    /// ```
    /// use saptest::{Pet, PetName, Effect, effects::effect::EffectModify};
    ///
    /// let mut dolphin = Pet::try_from(PetName::Dolphin).unwrap();
    /// let dolphin_effect = dolphin.effect.first_mut().unwrap();
    /// assert_eq!(dolphin_effect.uses, Some(1));
    /// // Add a use.
    /// dolphin_effect.add_uses(1);
    /// assert_eq!(dolphin_effect.uses, Some(2));
    /// ```
    fn add_uses(&mut self, n: usize) -> &mut Self;

    /// Remove `n` uses to a [`Effect`] if possible.
    /// # Example
    /// ```
    /// use saptest::{Pet, PetName, Effect, effects::effect::EffectModify};
    ///
    /// let mut dolphin = Pet::try_from(PetName::Dolphin).unwrap();
    /// let dolphin_effect = dolphin.effect.first_mut().unwrap();
    /// assert_eq!(dolphin_effect.uses, Some(1));
    /// // Add a use.
    /// dolphin_effect.remove_uses(1);
    /// assert_eq!(dolphin_effect.uses, Some(0));
    /// ```
    fn remove_uses(&mut self, n: usize) -> &mut Self;
}

impl EffectModify for Effect {
    fn add_uses(&mut self, n: usize) -> &mut Self {
        if let Some(uses) = self.uses.as_mut() {
            *uses += n
        };
        self
    }

    fn remove_uses(&mut self, n: usize) -> &mut Self {
        if let Some(uses) = self.uses.as_mut() {
            if *uses >= n {
                *uses -= n
            }
        };
        self
    }
}
