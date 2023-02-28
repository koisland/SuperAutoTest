use crate::{
    effects::{
        actions::Action,
        state::{Outcome, Position, Target},
    },
    error::SAPTestError,
    FoodName, Pet, PetName,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    fmt::Display,
    rc::{Rc, Weak},
};

/// A Super Auto Pets entity.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
pub enum Entity {
    #[default]
    /// A [`Pet`](crate::pets::pet::Pet).
    Pet,
    /// A [`Food`](crate::foods::food::Food).
    Food,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
/// [`Entity`] names.
pub enum EntityName {
    /// Pet name.
    Pet(PetName),
    /// Food name.
    Food(FoodName),
}

/// An effect for an [`Entity`] in Super Auto Pets.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Effect {
    /// Owner of effect.
    pub entity: Entity,
    #[serde(skip)]
    /// Idx of owner.
    pub(crate) owner: Option<Weak<RefCell<Pet>>>,
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
        self.entity == other.entity
            && self.trigger == other.trigger
            && self.target == other.target
            && self.position == other.position
            && self.action == other.action
            && self.uses == other.uses
            && self.temp == other.temp
    }
}

impl TryFrom<&Effect> for Rc<RefCell<Pet>> {
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
    ///         effect::Entity,
    ///         trigger::TRIGGER_SELF_FAINT,
    ///         state::{Position, Target, ItemCondition, Outcome},
    ///         actions::{Action, StatChangeType}
    ///     }
    /// };
    /// let lvl_1_ant_effect = Effect::new(
    ///     Entity::Pet,
    ///     TRIGGER_SELF_FAINT,
    ///     Target::Friend,
    ///     Position::Any(ItemCondition::None),
    ///     Action::Add(StatChangeType::StaticValue(Statistics {attack: 2, health: 1})),
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
    pub fn get_owner(&self) -> Option<Weak<RefCell<Pet>>> {
        self.owner.as_ref().cloned()
    }

    /// Assign this effect to a pet.
    /// * Used in [`Team`](crate::Team) to assign and track owners of [`Effect`]s.
    /// # Example
    /// ```rust
    /// use std::{rc::Rc, cell::RefCell};
    /// use saptest::{Pet, PetName, Effect};
    ///
    /// let ant = Pet::try_from(PetName::Ant).unwrap();
    /// let rc_ant = Rc::new(RefCell::new(ant));
    ///
    /// // Create an example effect.
    /// let mut effect = Effect::default();
    /// // ^ and its trigger now belongs to ant.
    /// effect.assign_owner(Some(&rc_ant));
    /// assert!(effect.get_owner().unwrap().ptr_eq(&Rc::downgrade(&rc_ant)))
    /// ```
    pub fn assign_owner(&mut self, owner: Option<&Rc<RefCell<Pet>>>) -> &mut Self {
        let owner_ref = owner.map(Rc::downgrade);
        self.owner = owner_ref.clone();
        self.trigger.affected_pet = owner_ref;
        self
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
    /// use saptest::{Pet, PetName, Effect, effects::effect::Modify};
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
    /// use saptest::{Pet, PetName, Effect, effects::effect::Modify};
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
