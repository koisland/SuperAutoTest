use serde::{Deserialize, Serialize};
use std::{
    ops::RangeInclusive,
    sync::{Arc, RwLock, Weak},
};

use crate::{
    effects::{effect::EntityName, stats::Statistics},
    pets::pet::Pet,
    shop::store::ShopState,
    teams::team::TeamFightOutcome,
    Food, PetCombat, Team, TeamShopping, TeamViewer,
};

use super::actions::Action;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
/// Possible equality conditions to check.
pub enum EqualityCondition {
    /// Is same pet. Pet only.
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
    /// Is frozen. Shop only.
    Frozen,
    /// Has perk. Pet only.
    HasPerk,
}

impl EqualityCondition {
    pub(crate) fn matches_food(&self, food: &Food) -> bool {
        match self {
            EqualityCondition::Tier(tier) => food.tier == *tier,
            EqualityCondition::Name(EntityName::Food(food_name)) => food.name == *food_name,
            EqualityCondition::Action(action) => food.ability.action == **action,
            EqualityCondition::Trigger(trigger) => food.ability.trigger.status == *trigger,
            _ => false,
        }
    }

    pub(crate) fn matches_pet(&self, pet: &Pet) -> bool {
        match self {
            EqualityCondition::Tier(tier) => pet.tier == *tier,
            EqualityCondition::Name(EntityName::Pet(pet_name)) => pet.name == *pet_name,
            EqualityCondition::Action(action) => pet.has_effect_ability(action, false),
            EqualityCondition::Trigger(trigger) => pet.has_effect_trigger(trigger, false),
            EqualityCondition::HasPerk => pet.item.as_ref().map_or(false, |item| item.holdable),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
/// Orderings for Conditions.
pub enum CondOrdering {
    /// Less than or equal to inner value.
    LessEqual(usize),
    /// Less than inner value.
    Less(usize),
    /// Equasl inner value.
    Equal(usize),
    /// Greater than inner value.
    Greater(usize),
    /// Greater than or equal to inner value.
    GreaterEqual(usize),
}

impl CondOrdering {
    fn check_true(&self, comp: usize) -> bool {
        match self {
            CondOrdering::LessEqual(val) => comp <= *val,
            CondOrdering::Less(val) => comp < *val,
            CondOrdering::Equal(val) => comp == *val,
            CondOrdering::Greater(val) => comp > *val,
            CondOrdering::GreaterEqual(val) => comp >= *val,
        }
    }
}

/// Conversion can only be `Into<usize>`. Cannot determined [`CondOrdering``] by num.
#[allow(clippy::from_over_into)]
impl Into<usize> for &CondOrdering {
    fn into(self) -> usize {
        match self {
            CondOrdering::LessEqual(val)
            | CondOrdering::Less(val)
            | CondOrdering::Equal(val)
            | CondOrdering::Greater(val)
            | CondOrdering::GreaterEqual(val) => *val,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
/// Conditions a `Team` is in.
pub enum TeamCondition {
    /// Previous team fight was some outcome.
    /// * If used for [`Position::FrontToBack`], returns the number of previous [`TeamFightOutcome`] equal to the provided outcome.
    PreviousBattle(TeamFightOutcome),
    /// Number of open slots on team.
    /// * If used for [`Position::FrontToBack`] and value is [`None`], returns current number of open slots on team.
    /// * If used for [`Action::Conditional`], checks if current number of open slots meets [`CondOrdering`].
    OpenSpace(Option<CondOrdering>),
    /// Number of pets on team.
    /// * If used for [`Position::FrontToBack`] and value is [`None`], returns current number of pets on team.
    /// * If used for [`Action::Conditional`], checks if current number of pets meets [`CondOrdering`]e.
    NumberPets(Option<CondOrdering>),
    /// Number of fainted pets is a multiple of this value.
    NumberFaintedMultiple(usize),
    /// Counter.
    /// * If used for [`Position::FrontToBack`] and value is [`None`], returns current counter value.
    /// * If used for [`Action::Conditional`], checks if current counter value meets [`CondOrdering`].
    Counter(String, Option<CondOrdering>),
    /// Number of turns.
    /// * If used for [`Position::FrontToBack`] and value is [`None`], returns current team turn.
    /// * If used for [`Action::Conditional`], checks if current turn meets [`CondOrdering`].
    NumberTurns(Option<CondOrdering>),
    /// Number of pets with a perk.
    NumberPerkPets(Option<CondOrdering>),
    /// Check number of toys.
    NumberToys(Option<CondOrdering>),
}

impl TeamCondition {
    /// Count number of times a [`TeamCondition`] is met.
    pub(crate) fn to_num(&self, team: &Team) -> usize {
        let get_inner_num = |cond_num: &CondOrdering| cond_num.into();
        match self {
            TeamCondition::PreviousBattle(outcome) => team
                .history
                .fight_outcomes
                .iter()
                .filter(|prev_outcome| outcome == *prev_outcome)
                .count(),
            TeamCondition::OpenSpace(num_spaces) => num_spaces
                .as_ref()
                .map_or_else(|| team.open_slots(), get_inner_num),
            TeamCondition::NumberPets(num_pets) => num_pets
                .as_ref()
                .map_or_else(|| team.all().len(), get_inner_num),
            // Return divisor.
            TeamCondition::NumberFaintedMultiple(multiple) => team.fainted.len() / multiple,
            TeamCondition::Counter(counter, counter_num) => counter_num
                .as_ref()
                .map_or_else(|| *team.counters.get(counter).unwrap_or(&0), get_inner_num),
            TeamCondition::NumberTurns(turns) => {
                turns.as_ref().map_or(team.history.curr_turn, get_inner_num)
            }
            TeamCondition::NumberPerkPets(num_perk_pets) => num_perk_pets.as_ref().map_or_else(
                || {
                    team.all()
                        .into_iter()
                        .filter(|pet| EqualityCondition::HasPerk.matches_pet(&pet.read().unwrap()))
                        .count()
                },
                get_inner_num,
            ),
            TeamCondition::NumberToys(num_toys) => num_toys
                .as_ref()
                .map_or_else(|| team.toys.len(), get_inner_num),
        }
    }
    /// Check if [`TeamCondition`] is met.
    pub(crate) fn matches_team(&self, team: &Team) -> bool {
        match self {
            TeamCondition::PreviousBattle(outcome) => {
                // Get last battle outcome and if matches condition, apply effect.
                if let Some(last_outcome) = team.history.fight_outcomes.last() {
                    last_outcome == outcome
                } else {
                    false
                }
            }
            TeamCondition::OpenSpace(num_open) => num_open.as_ref().map_or(false, |cond_spaces| {
                cond_spaces.check_true(team.open_slots())
            }),
            TeamCondition::NumberPets(num_pets) => num_pets
                .as_ref()
                .map_or(false, |cond_pets| cond_pets.check_true(team.filled_slots())),
            TeamCondition::NumberFaintedMultiple(multiple) => team.fainted.len() % *multiple == 0,
            TeamCondition::Counter(counter_name, cond_counts) => team
                .counters
                .get(counter_name)
                .and_then(|count| cond_counts.as_ref().map(|cond| cond.check_true(*count)))
                .unwrap_or(false),
            TeamCondition::NumberTurns(cond_turns) => {
                cond_turns.as_ref().map_or(false, |cond_turns| {
                    cond_turns.check_true(team.history.curr_turn)
                })
            }
            TeamCondition::NumberPerkPets(cond_num_perk_pets) => {
                cond_num_perk_pets.as_ref().map_or(false, |num_perk_pets| {
                    num_perk_pets.check_true(
                        team.all()
                            .into_iter()
                            .filter(|pet| {
                                EqualityCondition::HasPerk.matches_pet(&pet.read().unwrap())
                            })
                            .count(),
                    )
                })
            }
            TeamCondition::NumberToys(cond_num_toys) => {
                cond_num_toys.as_ref().map_or(false, |cond_num_toys| {
                    cond_num_toys.check_true(team.toys.len())
                })
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
/// Conditions a `Shop` is in.
pub enum ShopCondition {
    /// Shop is in this state.
    InState(ShopState),
    /// Gold.
    /// * If used for [`Position::FrontToBack`] and value is [`None`], returns current shop gold.
    /// * If used for [`Action::Conditional`], checks if current shop gold is equal to provided value.
    Gold(Option<CondOrdering>),
    /// Shop tier.
    /// * If used for [`Position::FrontToBack`] and value is [`None`], returns current shop tier.
    /// * If used for [`Action::Conditional`], checks if current shop tier is equal to provided value.
    Tier(Option<CondOrdering>),
    /// Shop tier multiple of given size.
    TierMultiple(usize),
    /// Number of pets sold.
    NumberSoldMultiple(usize),
}

impl ShopCondition {
    pub(crate) fn to_num(&self, team: &Team) -> usize {
        match self {
            ShopCondition::Gold(gold_cond) => gold_cond
                .as_ref()
                .map_or(team.shop.coins, |cond| cond.into()),
            ShopCondition::Tier(tier_cond) => tier_cond
                .as_ref()
                .map_or_else(|| team.shop.tier(), |cond| cond.into()),
            // Return divisor. Num times multiple goes into tier.
            ShopCondition::TierMultiple(tier_multiple) => team.shop.tier() / tier_multiple,
            // Return divisor. Num times multiple goes into num sold pets.
            ShopCondition::NumberSoldMultiple(num_sold_mult) => team.sold.len() / num_sold_mult,
            _ => panic!("Can't convert {self:?} to num."),
        }
    }

    pub(crate) fn matches_shop(&self, team: &Team) -> bool {
        match self {
            ShopCondition::InState(state) => team.shop.state == *state,
            ShopCondition::Gold(gold) => gold
                .as_ref()
                .map_or(false, |gold_cond| gold_cond.check_true(team.gold())),
            // Default to false if tier is None.
            ShopCondition::Tier(tier) => tier
                .as_ref()
                .map_or(false, |tier_cond| tier_cond.check_true(team.shop_tier())),
            ShopCondition::TierMultiple(tier_multiple) => team.shop_tier() % tier_multiple == 0,
            ShopCondition::NumberSoldMultiple(sold_multiple) => {
                team.sold.len() % sold_multiple == 0
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
/// Conditions for [`Position::FrontToBack`]
pub enum FrontToBackCondition {
    /// Shop condition.
    Shop(ShopCondition),
    /// Team condition.
    Team(TeamCondition),
}

/// Conditions to select [`Pet`]s or [`ShopItem`](crate::ShopItem) by.
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
    ///
    /// Note: These positions are non-overlapping.
    N {
        /// [`ItemCondition`]s to select pets by.
        condition: ItemCondition,
        /// Number of pets to select.
        targets: usize,
        /// Shuffle any found pets.
        random: bool,
        /// Must be exact number of targets.
        exact_n_targets: bool,
    },
    /// Any [`Pet`] that matches a given [`ItemCondition`].
    Any(ItemCondition),
    /// All [`Pet`]s that match a given [`ItemCondition`].
    All(ItemCondition),
    /// Position of self.
    OnSelf,
    /// Pet affected in [`Outcome`] trigger.
    /// 1. Optional position relative to pet.
    ///
    /// Same pet.
    /// ```
    /// let curr_pet = saptest::Position::TriggerAffected(None);
    /// ```
    /// Two pets ahead of the trigger pet that are nearest.
    /// ```
    /// let curr_pet = saptest::Position::TriggerAfflicting(Some(Box::new(saptest::Position::Nearest(2))));
    /// ```
    TriggerAffected(Option<Box<Position>>),
    /// Pet causing in [`Outcome`] trigger.
    /// 1. Optional position relative to pet.
    ///
    /// Same pet.
    /// ```
    /// let curr_pet = saptest::Position::TriggerAfflicting(None);
    /// ```
    /// Pet behind trigger pet.
    /// ```
    /// let curr_pet = saptest::Position::TriggerAfflicting(Some(Box::new(saptest::Position::Relative(-1))));
    /// ```
    TriggerAfflicting(Option<Box<Position>>),
    /// First pet on [`Team`].
    First,
    /// Last pet on [`Team`].
    Last,
    /// Opposite team's pet at the current pet index.
    Opposite,
    /// All [`Pet`]s ahead of current pet.
    Ahead,
    /// A specified range on a [`Team`].
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
    /// Select pets front-to-back based on a given condition.
    FrontToBack(FrontToBackCondition),
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
                | Position::N { .. }
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
    pub(crate) affected_pet: Option<Weak<RwLock<Pet>>>,
    /// The affected team.
    pub affected_team: Target,
    #[serde(skip)]
    /// The pet causing the status update.
    pub(crate) afflicting_pet: Option<Weak<RwLock<Pet>>>,
    /// The team causing the status update.
    pub afflicting_team: Target,
    /// General position on `affected_team`.
    pub position: Position,
    /// Difference in [`Statistics`] after status update from initial state.
    pub(crate) stat_diff: Option<Statistics>,
    #[serde(skip)]
    /// The shop food causing the status update.
    pub(crate) afflicting_food: Option<Weak<RwLock<Food>>>,
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
            affected_pet: None,
            affected_team: Target::None,
            afflicting_pet: None,
            afflicting_team: Target::None,
            position: Position::None,
            stat_diff: None,
            afflicting_food: None,
        }
    }
}

impl Outcome {
    /// Attach the affected pet to this trigger.
    /// # Example.
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use saptest::{Pet, PetName, effects::trigger::TRIGGER_SELF_FAINT};
    ///
    /// let ant = Arc::new(RwLock::new(Pet::try_from(PetName::Ant).unwrap()));
    /// let mut faint_trigger = TRIGGER_SELF_FAINT.clone();
    /// // Set affected pet to be ant.
    /// faint_trigger.set_affected(&ant);
    ///
    /// let affected_pet = faint_trigger.get_affected().unwrap();
    /// assert!(affected_pet.ptr_eq(&Arc::downgrade(&ant)));
    /// ```
    pub fn set_affected(&mut self, pet: &Arc<RwLock<Pet>>) -> &mut Self {
        self.affected_pet = Some(Arc::downgrade(pet));
        self
    }

    /// Attach the afflicting food to this trigger.
    pub fn set_afflicting_food(&mut self, food: &Arc<RwLock<Food>>) -> &mut Self {
        self.afflicting_food = Some(Arc::downgrade(food));
        self
    }

    /// Attach the afflicting pet to this trigger.
    /// # Example.
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use saptest::{Pet, PetName, effects::trigger::TRIGGER_SELF_FAINT};
    ///
    /// let ant = Arc::new(RwLock::new(Pet::try_from(PetName::Ant).unwrap()));
    /// let mosquito = Arc::new(RwLock::new(Pet::try_from(PetName::Mosquito).unwrap()));
    /// let mut faint_trigger = TRIGGER_SELF_FAINT.clone();
    /// // Set affected pet to be ant and afflicting pet to be mosquito.
    /// faint_trigger.set_affected(&ant).set_afflicting(&mosquito);
    ///
    /// let afflicting_pet = faint_trigger.get_afflicting().unwrap();
    /// assert!(afflicting_pet.ptr_eq(&Arc::downgrade(&mosquito)));
    /// ```
    pub fn set_afflicting(&mut self, pet: &Arc<RwLock<Pet>>) -> &mut Self {
        self.afflicting_pet = Some(Arc::downgrade(pet));
        self
    }

    /// Get the affected pet of a trigger.
    /// # Example
    /// ```
    /// use saptest::effects::trigger::TRIGGER_START_BATTLE;
    /// // No single affected pet as affects every pet.
    /// assert!(TRIGGER_START_BATTLE.get_affected().is_none())
    /// ```
    pub fn get_affected(&self) -> Option<Weak<RwLock<Pet>>> {
        self.affected_pet.as_ref().cloned()
    }

    /// Get the afflicting pet of a trigger.
    /// # Example
    /// ```
    /// use saptest::effects::trigger::TRIGGER_START_BATTLE;
    /// // No single afflicting pet as no pet causes the start of battle.
    /// assert!(TRIGGER_START_BATTLE.get_afflicting().is_none())
    /// ```
    pub fn get_afflicting(&self) -> Option<Weak<RwLock<Pet>>> {
        self.afflicting_pet.as_ref().cloned()
    }
}

/// Status of [`Entity`](super::effect::Entity).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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
    /// Before pet attacks.
    BeforeAttack,
    /// Pet is attacking.
    Attack,
    /// After pet attacks
    AfterAttack,
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
    /// Pet gains an effect perk. ex. [`FoodName::Honey`](crate::FoodName::Honey)
    GainPerk,
    /// Pet gains an ailment. ex. [`FoodName::Ink`](crate::FoodName::Ink)
    GainAilment,
    /// Team State
    IsTeam(TeamCondition),
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
    /// Toy broke.
    BrokeToy,
    /// No status change.
    None,
}
