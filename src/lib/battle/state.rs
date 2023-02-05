use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    fmt::Display,
    ops::RangeInclusive,
    rc::{Rc, Weak},
};

use crate::{
    battle::{effect::Effect, stats::Statistics},
    foods::{food::Food, names::FoodName},
    pets::pet::Pet,
    PetName,
};

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
    HasFood(FoodName),
    /// Choose all pet that have an [`Effect`] triggered by some [`Status`].
    TriggeredBy(Status),
    /// Multiple conditions.
    Multiple(Vec<Condition>),
    /// Multiple conditions. All must be met to be included.
    MultipleAll(Vec<Condition>),
    /// Ignore self.
    IgnoreSelf,
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
    /// Position of [`Outcome`] trigger.
    Trigger,
    /// First pet on [`Team`](crate::battle::team::Team).
    First,
    /// Last pet on [`Team`](crate::battle::team::Team).
    Last,
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

/// The outcome of any [`Pet`] action. Serve as [`Effect`] triggers in battle.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Outcome {
    /// Status of a [`Pet`].
    pub status: Status,
    // TODO: https://serde.rs/field-attrs.html. Replace with serde(serialize_with).
    #[serde(skip)]
    /// The affected pet.
    pub affected_pet: Option<Weak<RefCell<Pet>>>,
    /// The affected team.
    pub affected_team: Target,
    #[serde(skip)]
    /// The pet causing the status_update.
    pub afflicting_pet: Option<Weak<RefCell<Pet>>>,
    /// The team causing the status update.
    pub afflicting_team: Target,
    /// General position on `target`.
    pub position: Position,
    /// Difference in [`Statistics`] after status update from initial state.
    pub stat_diff: Option<Statistics>,
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
    /// Attach the affected pet reference to an Outcome.
    pub fn set_affected(&mut self, pet: &Rc<RefCell<Pet>>) -> &mut Self {
        self.affected_pet = Some(Rc::downgrade(pet));
        self
    }

    /// Attach the afflicting pet reference to an Outcome.
    pub fn set_afflicted(&mut self, pet: &Rc<RefCell<Pet>>) -> &mut Self {
        self.afflicting_pet = Some(Rc::downgrade(pet));
        self
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

/// General Pet attribute use for [`Action::Copy`].
///
/// [`Statistics`] for `health` or `attack` are a set percentage.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum CopyAttr {
    /// Percent pet stats to copy.
    PercentStats(Statistics),
    /// Pet stats to copy.
    Stats(Option<Statistics>),
    /// Effects at a specific level to copy.
    Effect(Vec<Effect>, Option<usize>),
    /// Food item to copy.
    Item(Option<Box<Food>>),
    /// Nothing to copy.
    None,
}

/// Pet actions.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub enum Action {
    /// Add some amount of `Statistics` to a `Pet`.
    Add(Statistics),
    /// Remove some amount of `Statistics` from a `Pet`.
    Remove(Statistics),
    /// Debuff a `Pet` by subtracting some **percent** of `Statistics` from it.
    Debuff(Statistics),
    /// Swap positions of `Pet`s
    SwapPositions,
    /// Swap `Statistics` of `Pet`s.
    SwapStats,
    /// Push a `Pet` to some new position from its original position. The following positions are implemented.
    /// * [`Position::Relative`]
    /// * [`Position::First`]
    /// * [`Position::Last`]
    Push(Position),
    /// Copy some attribute from a `Pet` to a given `Position`.
    Copy(CopyAttr, Target, Position),
    /// Negate some amount of `Statistics` damage.
    Negate(Statistics),
    /// Do a critical attack with a percent probability dealing double damage.
    Critical(usize),
    /// Swallow a `Pet` at a specified index, level it, and spawn it on faint.
    Whale(usize, Position),
    /// Transform into another `Pet`.
    /// * Note: This does not emit a summon trigger.
    Transform(PetName, Option<Statistics>, usize),
    /// Instantly kill a `Pet`.
    Kill,
    /// Take no damage. Action of `Coconut`.
    Invincible,
    /// Gain a `Food` item.
    Gain(Option<Box<Food>>),
    /// WIP: Get gold.
    Profit,
    /// Summon a `Pet` with an optional `Statistics` arg to replace store `Pet`.
    Summon(Option<Box<Pet>>, Option<Statistics>),
    /// Do multiple `Action`s.
    Multiple(Vec<Action>),
    /// Do multiple `Action`s based on number of `Pet`s matching a `Condition`.
    ForEachCondition(Box<Action>, Target, Condition),
    /// If target meets condition, do `Action`.
    IfTargetCondition(Box<Action>, Condition),
    /// Hardcoded Rhino ability.
    Rhino(Statistics),
    /// Hardcoded lynx ability.
    Lynx,
    /// Return damage back to pet that triggered effect.
    Thorns(Statistics),
    /// Gain one experience point.
    Experience,
    /// WIP: Endure damage so health doesn't go below one.
    Endure,
    #[default]
    /// No action to take.
    None,
}
