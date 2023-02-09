use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    fmt::Display,
    ops::RangeInclusive,
    rc::{Rc, Weak},
};

use crate::{battle::stats::Statistics, foods::names::FoodName, pets::pet::Pet, PetName};

/// The outcome of a [`Team`](crate::battle::team::Team) fight.
///
/// # Examples
/// This can be used as an exit condition in a fight.
/// ```rust
/// use sapt::{Team, Pet, PetName, Statistics, battle::state::TeamFightOutcome};
///
/// let pet = Pet::try_from(PetName::Blowfish).unwrap();
/// let mut team = Team::new(&vec![pet.clone(); 5], 5).unwrap();
/// let mut enemy_team = Team::clone(&team);
///
/// // Continue fighting while the winner of a fight is None.
/// let mut winner = team.fight(&mut enemy_team);
/// while let TeamFightOutcome::None = winner {
///     winner = team.fight(&mut enemy_team);
/// }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum TeamFightOutcome {
    /// Outcome of fight is a win.
    Win,
    /// Outcome of fight is a loss.
    Loss,
    /// Outcome of fight is a draw.
    Draw,
    /// No outcome for fight.
    None,
}

/// Conditions to select [`Pet`]s by.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Condition {
    /// Choose the healthiest (highest health) pet.
    Healthiest,
    /// Choose the illest (lowest health) pet.
    Illest,
    /// Choose the stronges (highest attack) pet.
    Strongest,
    /// Choose the weakest (lowest attack) pet.
    Weakest,
    /// Highest tier pet.
    HighestTier,
    /// Lowest tier pet.
    LowestTier,
    /// Choose all pets that have an item with a given [`FoodName`].
    HasFood(Option<FoodName>),
    /// Choose all pet that have an [`Effect`](crate::Effect) triggered by some [`Status`].
    TriggeredBy(Status),
    /// Multiple conditions.
    Multiple(Vec<Condition>),
    /// Multiple conditions. All must be met to be included.
    MultipleAll(Vec<Condition>),
    /// Ignore self.
    NotSelf,
    /// Not a specific [`PetName`].
    NotPetName(PetName),
    /// No condition.
    None,
}

/// Positions to select pets by.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub enum Position {
    ///Some number of [`Pet`]s based on a given [`Condition`].
    N(Condition, usize),
    /// Any [`Pet`] that matches a given [`Condition`].
    Any(Condition),
    /// All [`Pet`]s that match a given [`Condition`].
    All(Condition),
    /// Position of self.
    OnSelf,
    /// Pet affected in [`Outcome`] trigger.
    TriggerAffected,
    /// Pet causing in [`Outcome`] trigger.
    TriggerAfflicting,
    /// First pet on [`Team`](crate::battle::team::Team).
    First,
    /// Last pet on [`Team`](crate::battle::team::Team).
    Last,
    /// Opposite team's pet at the current pet index.
    Opposite,
    /// A specified range on a [`Team`](crate::battle::team::Team).
    Range(RangeInclusive<isize>),
    /// A [`Pet`] relative to current [`Pet`].
    Relative(isize),
    /// Multiple [`Position`]s.
    Multiple(Vec<Position>),
    /// All [`Pet`]'s adjacent to current index.
    Adjacent,
    #[default]
    /// No position.
    None,
}

/// Target team for an effect.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
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
    // TODO: https://serde.rs/field-attrs.html. Replace with serde(serialize_with).
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
impl Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Status: {:?}, Position: {:?}, Affected: {:?}, From: {:?}]",
            self.status, self.position, self.affected_pet, self.afflicting_pet
        )
    }
}

impl Outcome {
    /// Attach the affected pet to this trigger.
    /// # Example.
    /// ```
    /// use std::{rc::Rc, cell::RefCell};
    /// use sapt::{Pet, PetName, battle::trigger::TRIGGER_SELF_FAINT};
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
    /// use sapt::{Pet, PetName, battle::trigger::TRIGGER_SELF_FAINT};
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
    /// use sapt::battle::trigger::TRIGGER_START_BATTLE;
    /// // No single affected pet as affects every pet.
    /// assert!(TRIGGER_START_BATTLE.get_affected().is_none())
    /// ```
    pub fn get_affected(&self) -> Option<Weak<RefCell<Pet>>> {
        self.affected_pet.as_ref().cloned()
    }

    /// Get the afflicting pet of a trigger.
    /// # Example
    /// ```
    /// use sapt::battle::trigger::TRIGGER_START_BATTLE;
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
    /// Start of Battle.
    StartOfBattle,
    /// After start of battle, prior to first battle.
    BeforeFirstBattle,
    /// End of Battle.
    EndOfBattle,
    /// Before pet attacks.
    BeforeAttack,
    /// Pet is attacking.
    Attack,
    /// Pet levels up.
    Levelup,
    /// Food bought.
    BuyFood,
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
    /// Pet knocked out during an attack.
    /// * After [`attack`](crate::pets::combat::PetCombat::attack) or [`indirect_attack`](crate::pets::combat::PetCombat::indirect_attack)
    KnockOut,
    /// Pet summoned.
    Summoned,
    /// Pet pushed.
    Pushed,
    /// No status change.
    None,
}
