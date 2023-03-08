use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    ops::RangeInclusive,
    rc::{Rc, Weak},
};

use crate::{
    effects::{effect::EntityName, stats::Statistics},
    pets::pet::Pet,
    shop::store::ShopState,
    teams::team::TeamFightOutcome,
    FoodName,
};

use super::actions::Action;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
/// Possible equality conditions to check.
pub enum EqualityCondition {
    /// Is same pet.
    IsSelf,
    /// Is this tier.
    Tier(usize),
    /// Has same name.
    Name(EntityName),
    /// Is this level.
    Level(usize),
    /// Has this [`Action`].
    Action(Box<Action>),
    /// Triggered by this [`Status`].
    Trigger(Status),
    /// Is frozen. Only available for shops.
    Frozen,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
/// Conditions a `Team` is in.
pub enum TeamCondition {
    /// Previous team fight was some outcome.
    PreviousBattle(TeamFightOutcome),
    /// Has this many open slots.
    OpenSpaceEqual(usize),
    /// Has this many pets on team.
    NumberPetsEqual(usize),
    /// Has this many or more pets on team.
    NumberPetsGreaterEqual(usize),
    /// Number of fainted pets is a multiple of this value.
    NumberFaintedMultiple(usize),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
/// Conditions a `Shop` is in.
pub enum ShopCondition {
    /// Shop is in this state.
    InState(ShopState),
    /// Gold is equal to this amount.
    GoldEqual(usize),
    /// Gold is greater than or equal to this amount.
    GoldGreaterEqual(usize),
}

/// Conditions to select [`Pet`]s or [`ShopItem`](crate::shop::store::ShopItem) by.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ItemCondition {
    /// Is the healthiest (highest health) pet.
    Healthiest,
    /// Is the illest (lowest health) pet.
    Illest,
    /// Is the strongest (highest attack) pet.
    Strongest,
    /// Is the weakest (lowest attack) pet.
    Weakest,
    /// Is highest tier pet.
    HighestTier,
    /// Is lowest tier pet.
    LowestTier,
    /// Multiple conditions.
    Multiple(Vec<ItemCondition>),
    /// Multiple conditions. All must be met to be included.
    MultipleAll(Vec<ItemCondition>),
    /// Has the quality.
    Equal(EqualityCondition),
    /// Doesn't have this quality.
    NotEqual(EqualityCondition),
    /// All alive pets.
    None,
}

/// Positions to select pets by.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub enum Position {
    /// Some number of [`Pet`]s based on a given [`ItemCondition`].
    /// 1. [`ItemCondition`]s to select pets by.
    /// 2. Number of pets to select.
    /// 3. Shuffle any found pets.
    ///
    /// Note: These positions are non-overlapping.
    N(ItemCondition, usize, bool),
    /// Any [`Pet`] that matches a given [`ItemCondition`].
    Any(ItemCondition),
    /// All [`Pet`]s that match a given [`ItemCondition`].
    All(ItemCondition),
    /// Position of self.
    OnSelf,
    /// Pet affected in [`Outcome`] trigger.
    TriggerAffected,
    /// Pet causing in [`Outcome`] trigger.
    TriggerAfflicting,
    /// First pet on [`Team`](crate::teams::team::Team).
    First,
    /// Last pet on [`Team`](crate::teams::team::Team).
    Last,
    /// Opposite team's pet at the current pet index.
    Opposite,
    /// All [`Pet`]s ahead of current pet.
    Ahead,
    /// A specified range on a [`Team`](crate::teams::team::Team).
    Range(RangeInclusive<isize>),
    /// A [`Pet`] relative to current [`Pet`].
    /// * Note: Empty slots are taken into consideration.
    Relative(isize),
    /// Nearest pet(s) ahead or behind from current [`Pet`].
    /// * Negative values check pets behind.
    /// * Positive values check pets ahead.
    Nearest(isize),
    /// Multiple [`Position`]s.
    Multiple(Vec<Position>),
    /// All [`Pet`]'s adjacent to current index.
    Adjacent,
    #[default]
    /// No position.
    None,
}

impl Position {
    /// Check if position is a non-specific to current pet.
    /// * [`Position::Any`]
    /// * [`Position::All`]
    /// * [`Position::Nearest`]
    /// * [`Position::Relative`]
    /// * [`Position::Ahead`]
    /// * [`Position::None`]
    pub fn is_non_specific(&self) -> bool {
        matches!(
            self,
            Position::Any(_)
                | Position::All(_)
                | Position::Relative(_)
                | Position::N(_, _, _)
                | Position::Nearest(_)
                | Position::Ahead
                | Position::None
        )
    }
}

/// Target for an [`Effect`](crate::Effect).
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default, Hash)]
pub enum Target {
    /// Friend team.
    Friend,
    /// Enemy team.
    Enemy,
    /// Shop.
    Shop,
    /// Either `Friend` or `Enemy` team.
    /// * Ex. [Badger](crate::pets::names::PetName::Badger)
    Either,
    #[default]
    /// No target.
    None,
}

/// The outcome of any [`Pet`] action. Serve as [`Effect`](crate::Effect) triggers in battle.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Outcome {
    /// Status of a [`Pet`].
    pub status: Status,
    #[serde(skip)]
    /// The affected pet.
    pub(crate) affected_pet: Option<Weak<RefCell<Pet>>>,
    /// The affected team.
    pub affected_team: Target,
    #[serde(skip)]
    /// The pet causing the status_update.
    pub(crate) afflicting_pet: Option<Weak<RefCell<Pet>>>,
    /// The team causing the status update.
    pub afflicting_team: Target,
    /// General position on `affected_team`.
    pub position: Position,
    /// Difference in [`Statistics`] after status update from initial state.
    pub(crate) stat_diff: Option<Statistics>,
}

impl PartialEq for Outcome {
    fn eq(&self, other: &Self) -> bool {
        let same_affected_pet = if let (Some(pet), Some(other_pet)) =
            (self.affected_pet.as_ref(), other.affected_pet.as_ref())
        {
            pet.ptr_eq(other_pet)
        } else {
            self.affected_pet.is_none() && other.affected_pet.is_none()
        };
        same_affected_pet
            && self.status == other.status
            && self.position == other.position
            && self.affected_team == other.affected_team
            && self.afflicting_team == other.afflicting_team
    }
}

impl Default for Outcome {
    fn default() -> Self {
        Self {
            status: Status::None,
            affected_pet: Default::default(),
            affected_team: Target::None,
            afflicting_pet: Default::default(),
            afflicting_team: Target::None,
            position: Position::None,
            stat_diff: Default::default(),
        }
    }
}

impl Outcome {
    /// Attach the affected pet to this trigger.
    /// # Example.
    /// ```
    /// use std::{rc::Rc, cell::RefCell};
    /// use saptest::{Pet, PetName, effects::trigger::TRIGGER_SELF_FAINT};
    ///
    /// let ant = Rc::new(RefCell::new(Pet::try_from(PetName::Ant).unwrap()));
    /// let mut faint_trigger = TRIGGER_SELF_FAINT.clone();
    /// // Set affected pet to be ant.
    /// faint_trigger.set_affected(&ant);
    ///
    /// let affected_pet = faint_trigger.get_affected().unwrap();
    /// assert!(affected_pet.ptr_eq(&Rc::downgrade(&ant)));
    /// ```
    pub fn set_affected(&mut self, pet: &Rc<RefCell<Pet>>) -> &mut Self {
        self.affected_pet = Some(Rc::downgrade(pet));
        self
    }

    /// Attach the afflicting pet to this trigger.
    /// # Example.
    /// ```
    /// use std::{rc::Rc, cell::RefCell};
    /// use saptest::{Pet, PetName, effects::trigger::TRIGGER_SELF_FAINT};
    ///
    /// let ant = Rc::new(RefCell::new(Pet::try_from(PetName::Ant).unwrap()));
    /// let mosquito = Rc::new(RefCell::new(Pet::try_from(PetName::Mosquito).unwrap()));
    /// let mut faint_trigger = TRIGGER_SELF_FAINT.clone();
    /// // Set affected pet to be ant and afflicting pet to be mosquito.
    /// faint_trigger.set_affected(&ant).set_afflicting(&mosquito);
    ///
    /// let afflicting_pet = faint_trigger.get_afflicting().unwrap();
    /// assert!(afflicting_pet.ptr_eq(&Rc::downgrade(&mosquito)));
    /// ```
    pub fn set_afflicting(&mut self, pet: &Rc<RefCell<Pet>>) -> &mut Self {
        self.afflicting_pet = Some(Rc::downgrade(pet));
        self
    }

    /// Get the affected pet of a trigger.
    /// # Example
    /// ```
    /// use saptest::effects::trigger::TRIGGER_START_BATTLE;
    /// // No single affected pet as affects every pet.
    /// assert!(TRIGGER_START_BATTLE.get_affected().is_none())
    /// ```
    pub fn get_affected(&self) -> Option<Weak<RefCell<Pet>>> {
        self.affected_pet.as_ref().cloned()
    }

    /// Get the afflicting pet of a trigger.
    /// # Example
    /// ```
    /// use saptest::effects::trigger::TRIGGER_START_BATTLE;
    /// // No single afflicting pet as no pet causes the start of battle.
    /// assert!(TRIGGER_START_BATTLE.get_afflicting().is_none())
    /// ```
    pub fn get_afflicting(&self) -> Option<Weak<RefCell<Pet>>> {
        self.afflicting_pet.as_ref().cloned()
    }
}
/// Status of [`Entity`](super::effect::Entity).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Status {
    /// Start of Turn.
    StartTurn,
    /// End of Turn.
    EndTurn,
    /// Shop tier upgraded.
    ShopTierUpgrade,
    /// Start of Battle.
    StartOfBattle,
    /// After start of battle, prior to first battle.
    BeforeFirstBattle,
    /// Won the Battle.
    WinBattle,
    /// Loss the battle.
    LoseBattle,
    /// Drew
    DrawBattle,
    /// Before pet attacks.
    BeforeAttack,
    /// Pet is attacking.
    Attack,
    /// A battle food effect. ex. Chili
    BattleFoodEffect,
    /// Any damage calculation
    AnyDmgCalc,
    /// Indirect dmg attack calculation.
    IndirectAttackDmgCalc,
    /// Direct dmg attack calculation.
    AttackDmgCalc,
    /// Pet levels up.
    Levelup,
    /// Food bought.
    BuyFood,
    /// Food eaten.
    AteFood,
    /// Specific food eaten.
    AteSpecificFood(FoodName),
    /// Pet bought.
    BuyPet,
    /// Pet sold.
    Sell,
    /// Shop rolled.
    Roll,
    /// Pet hurt.
    Hurt,
    /// Pet fainted.
    Faint,
    /// Pet knocks out another pet during an attack.
    /// * After [`attack`](crate::pets::combat::PetCombat::attack) or [`indirect_attack`](crate::pets::combat::PetCombat::indirect_attack)
    KnockOut,
    /// Pet summoned.
    Summoned,
    /// Pet pushed.
    Pushed,
    /// No status change.
    None,
}
