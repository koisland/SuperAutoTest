use crate::{
    effects::{
        effect::Effect,
        state::{ItemCondition, Position, ShopCondition, Target, TeamCondition},
        stats::Statistics,
    },
    error::SAPTestError,
    foods::{food::Food, names::FoodName},
    pets::pet::Pet,
    Entity, PetName,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// [`Pet`] attribute used for [`Action::Copy`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum CopyType {
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

/// Types of [`Statistics`] changes for [`Action::Remove`] or [`Action::Add`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum StatChangeType {
    /// Change to a static [`Statistics`] value.
    Static(Statistics),
    /// Change stats by a multiplier.
    Multiplier(Statistics),
    /// Change only [`Statistics`] attack.
    StaticAttack(isize),
    /// Change only [`Statistics`] health.
    StaticHealth(isize),
    /// Change both stats to current attack.
    CurrentAttack,
    /// Change both stats to current health.
    CurrentHealth,
    /// Set statistics based on a given team counter.
    TeamCounter(String),
}

impl StatChangeType {
    /// Convert [`StatChangeType`] into [`Statistics`].
    pub(crate) fn to_stats(
        &self,
        pet_stats: Option<Statistics>,
        team_counters: Option<&HashMap<String, usize>>,
    ) -> Result<Statistics, SAPTestError> {
        Ok(match self {
            StatChangeType::Static(stats) => *stats,
            StatChangeType::Multiplier(multiplier) => pet_stats
                .map(|pet_stats| pet_stats.mult_perc(multiplier))
                .ok_or(SAPTestError::InvalidTeamAction {
                    subject: "No Pet Stats".to_owned(),
                    reason: "Multiplier needs pet stats.".to_owned(),
                })?,
            StatChangeType::StaticAttack(atk) => Statistics {
                attack: *atk,
                health: 0,
            },
            StatChangeType::StaticHealth(health) => Statistics {
                attack: 0,
                health: *health,
            },
            StatChangeType::CurrentAttack => pet_stats
                .map(|pet_stats| {
                    let mut new_stats = pet_stats;
                    new_stats.health = pet_stats.attack;
                    new_stats
                })
                .ok_or(SAPTestError::InvalidTeamAction {
                    subject: "No Pet Stats".to_owned(),
                    reason: "Needs pet stats current attack.".to_owned(),
                })?,
            StatChangeType::CurrentHealth => pet_stats
                .map(|pet_stats| {
                    let mut new_stats = pet_stats;
                    new_stats.attack = pet_stats.health;
                    new_stats
                })
                .ok_or(SAPTestError::InvalidTeamAction {
                    subject: "No Pet Stats".to_owned(),
                    reason: "Needs pet stats current health.".to_owned(),
                })?,
            StatChangeType::TeamCounter(counter_key) => {
                let counter_value = team_counters
                    .and_then(|counters| counters.get(counter_key))
                    .ok_or(SAPTestError::InvalidTeamAction {
                        subject: "Invalid Stat Change".to_owned(),
                        reason: format!("No such counter key: {counter_key}"),
                    })?;

                let counter_value = TryInto::<isize>::try_into(*counter_value)?;
                Statistics {
                    attack: counter_value,
                    health: counter_value,
                }
            }
        })
    }
}

/// Types of summons for [`Action::Summon`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum SummonType {
    /// Summon a [`Pet`] via a `SQL` query.
    /// 1. `SQL` statement where params are listed as (`?`).
    ///     * All fields must be kept in the `SELECT` statment with the `*`.
    /// 2. Parameters.
    /// 3. [`Statistics`] if provided.
    ///
    /// # Example
    /// ```rust no_run
    /// use saptest::effects::actions::SummonType;
    /// // Summon a dog at default stats.
    /// let sql = "SELECT * FROM pets WHERE name = ?";
    /// let params = vec!["Dog".to_string()];
    /// let summon_type = SummonType::QueryPet(sql.to_owned(), params, None);
    /// ```
    QueryPet(String, Vec<String>, Option<Statistics>),
    /// Summon a stored [`Pet`].
    StoredPet(Box<Pet>),
    /// Summon a [`Pet`] with default `stats` and `level`.
    DefaultPet(PetName),
    /// Summon a custom [`Pet`] with `stats` from [`StatChangeType`]. Used for [`Rooster`](crate::PetName::Rooster)
    /// 1. Pet to summon.
    /// 2. Pet [`Statistics`] type.
    /// 3. The `level` the summon should be.
    CustomPet(PetName, StatChangeType, usize),
    /// Summon the [`Pet`] owning the [`Effect`] of this [`Action::Summon`].
    /// 1. Pet [`Statistics`]. Defaults to current [`Pet`] if omitted.
    /// 2. Pet `level`. Defaults to current [`Pet`] if omitted.
    /// 3. Keep item of current [`Pet`].
    SelfPet(Option<Statistics>, Option<usize>, bool),
    /// Summon a random pet at the same `tier` as the current [`Pet`]. Used for [`Popcorns`](crate::FoodName::Popcorns).
    /// 1. Pet [`Statistics`]. Defaults to current [`Pet`] if omitted.
    /// 2. Pet `level`. Defaults to current [`Pet`] if omitted.
    SelfTierPet(Option<Statistics>, Option<usize>),
    /// Summon a random [`Pet`] from the same [`Team`](crate::Team). Used for [`Tapir`](crate::PetName::Tapir).
    /// 1. Pet [`Statistics`]. Defaults to current [`Pet`] if omitted.
    /// 2. Pet `level`. Defaults to current [`Pet`] if omitted.
    /// 3. Ignore any [`Pet`]s on the [`Team`](crate::Team) with this [`PetName`].
    SelfTeamPet(Option<Statistics>, Option<usize>, PetName),
}

/// Types of item gains for [`Action::Gain`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum GainType {
    /// Gain the [`Food`] of the pet owning the [`Effect`] of this [`Action`].
    SelfItem,
    /// Gain the default [`Food`] item.
    DefaultItem(FoodName),
    /// Gain a [`Food`] item from a `SQL` query.
    /// 1. `SQL` statement where params are listed as (`?`).
    ///     * All fields must be kept in the `SELECT` statment with the `*`.
    /// 2. Parameters.
    /// # Example
    /// ```rust no_run
    /// use saptest::effects::actions::GainType;
    /// // Gain Garlic.
    /// let sql = "SELECT * FROM foods WHERE name = ?";
    /// let params = vec!["Garlic".to_string()];
    /// let summon_type = GainType::QueryItem(sql.to_owned(), params);
    /// ```
    QueryItem(String, Vec<String>),
    /// Random [`Shop`](crate::Shop) [`Food`].
    RandomShopItem,
    /// Gain the stored [`Food`].
    StoredItem(Box<Food>),
    /// Remove [`Food`].
    NoItem,
}

/// Types of ways [`Action::Swap`] or [`Action::Shuffle`] can randomize pets.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum RandomizeType {
    /// Alter positions.
    Positions,
    /// Alter [`Statistics`].
    Stats,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
/// Conditions for [`LogicType`].
pub enum ConditionType {
    /// Pet condition.
    Pet(Target, ItemCondition),
    /// Team condition.
    Team(Target, TeamCondition),
    /// Shop condition.
    Shop(ShopCondition),
}

/// Conditional logic for [`Action::Conditional`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum LogicType {
    /// Do multiple `Action`s based on number of times [`ConditionType`] met.
    ForEach(ConditionType),
    /// If [`ConditionType`] met.
    If(ConditionType),
    /// If [`ConditionType`] not met.
    IfNot(ConditionType),
    /// If any [`ConditionType`] met.
    IfAny(ConditionType),
}

/// Pet actions.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub enum Action {
    /// Add [`Statistics`] to a [`Pet`].
    Add(StatChangeType),
    /// Set [`Statistics`] of a [`Pet`].
    Set(StatChangeType),
    /// Remove [`Statistics`] from a [`Pet`].
    /// * Altering stats this way creates hurt trigger [`Outcome`](crate::effects::state::Outcome)s.
    Remove(StatChangeType),
    /// Debuff a [`Pet`] by subtracting some **percent** of [`Statistics`] from it.
    Debuff(StatChangeType),
    /// Shuffle all pets on a specific [`RandomizeType`].
    Shuffle(RandomizeType),
    /// Swap two pets on a specific [`RandomizeType`].
    Swap(RandomizeType),
    /// Push a [`Pet`] to some new position from its original position.
    ///
    /// The following positions are implemented.
    /// * [`Position::Relative`]
    /// * [`Position::First`]
    /// * [`Position::Last`]
    Push(Position),
    /// Copy some attribute from [`CopyType`] to the owner from the [`Pet`] at a given [`Position`].
    /// 1. Attribute to copy.
    /// 2. [`Target`] [`Team`](crate::Team) to copy from.
    /// 3. Position of copy from.
    ///     * If multiple pets are targeted, only the first is taken.
    Copy(CopyType, Target, Position),
    /// Negate some amount of [`Statistics`] damage.
    /// * An item-only [`Action`].
    /// * Used for [`Garlic`](crate::FoodName::Garlic) and [`Melon`](crate::FoodName::Melon)
    /// * The [`Statistics`](crate::Statistics) **`attack`** field represents the negated damage.
    Negate(Statistics),
    /// Do a critical attack with a percent probability to deal double damage.
    /// * An item-only [`Action`].
    /// * Used by [`Cheese`](crate::FoodName::Cheese) and [`Fortune Cookie`](crate::FoodName::FortuneCookie)
    Critical(usize),
    /// Swallow a [`Pet`], summon it, and level it on faint.
    /// 1. Specified `level` after spawning.
    /// 2. Pet [`Position`] of the pet to swallow.
    ///     * If this targets multiple [`Pet`]s, the first is taken.
    Whale(usize, Position),
    /// Transform owner into another [`Pet`].
    /// 1. [`PetName`] to summon as.
    /// 2. [`Statistics`] of transformed pet.
    /// 3. Level after transformation.
    ///
    /// **Note: This does not emit a summon trigger or faint trigger.**
    ///
    /// <https://superautopets.fandom.com/wiki/Whale>
    Transform(PetName, Option<Statistics>, usize),
    /// Instantly kill a [`Pet`].
    Kill,
    /// Take no damage.
    /// * An item-only [`Action`].
    /// * Action of [`Coconut`](crate::FoodName::Coconut).
    Invincible,
    /// Gain a [`Food`] item specified by [`GainType`].
    Gain(GainType),
    /// Add permanent [`Statistics`] to shop.
    /// * The action of the [`Canned Food`](crate::FoodName::CannedFood).
    /// * Also, immediately buffs the current [`Pet`]s in the [`Shop`](crate::Shop)
    AddShopStats(Statistics),
    /// Add a [`Shop`](crate::Shop) food as a [`ShopItem`](crate::shop::store::ShopItem).
    AddShopFood(GainType),
    /// Add a [`Shop`](crate::Shop) pet as a [`ShopItem`](crate::shop::store::ShopItem).
    AddShopPet(SummonType),
    /// Clear [`Shop`](crate::Shop) items of a specified [type](crate::effects::effect::Entity).
    ClearShop(Entity),
    /// Get gold for the [`Shop`](crate::Shop).
    Profit(usize),
    /// Reduce cost of [`Shop`](crate::Shop) a [`ShopItem`](crate::shop::store::ShopItem).
    /// 1. Item type to discount.
    /// 2. Gold to discount by.
    Discount(Entity, usize),
    /// Free roll(s) for the [`Shop`](crate::Shop).
    FreeRoll(usize),
    /// Save remaining gold up to a given limit. This gold is then available on the next turn.
    SaveGold {
        /// Gold limit.
        limit: usize,
    },
    /// Summon a [`Pet`] of a [`SummonType`].
    Summon(SummonType),
    /// Do multiple [`Action`]s.
    Multiple(Vec<Action>),
    /// Perform a conditional [`Action`] defaulting to the second [`Action`] if the [`LogicType`] condition not met.
    /// 1. Condition as [`LogicType`].
    /// 2. `If` [`Action`]
    /// 3. `Else` [`Action`]
    ///     * With [`ForEach`](crate::effects::actions::LogicType::ForEach), this action will only execute once.
    ///     * Set this to [`Action::None`] to cause nothing to activate.
    ///
    /// **Note**: If the condition fails, the effect will **still** deplete a use.
    /// * Any custom actions built should be restricted to:
    ///     * [`Effect`]s with `uses` set to [`None`] (unlimited).
    ///     * [`Effect`]s that trigger once and are restored at the end of a turn.
    /// # Examples
    /// ---
    /// ### Vulture
    /// **Two friends faint** → Deal 4 damage to one random enemy.
    /// ```rust compile_fail
    /// let vulture_action = Action::Conditional(
    ///     LogicType::If(ConditionType::Team(
    ///         Target::Enemy,
    ///         TeamCondition::NumberFaintedMultiple(2),
    ///     )),
    ///     Box::new(Action::Remove(StatChangeType::Static(effect_stats))),
    ///     Box::new(Action::None),
    /// );
    /// ```
    /// ---
    /// ### Ostrich
    /// **End turn** → Gain +2 attack and +2 health for every tier 5 pet or higher in the shop.
    /// ```rust compile_fail
    /// let ostrich_action = Action::Conditional(
    ///     LogicType::ForEach(ConditionType::Pet(
    ///         Target::Shop,
    ///         ItemCondition::Multiple(vec![
    ///             ItemCondition::Equal(EqualityCondition::Tier(5)),
    ///             ItemCondition::Equal(EqualityCondition::Tier(6)),
    ///         ]),
    ///     )),
    ///     Box::new(Action::Add(StatChangeType::Static(effect_stats))),
    ///     Box::new(Action::None),
    /// );
    /// ```
    Conditional(LogicType, Box<Action>, Box<Action>),
    /// Hardcoded Lynx ability.
    ///
    /// <https://superautopets.fandom.com/wiki/Lynx>
    Lynx,
    /// Hardcoded Stegosaurus ability
    /// 1. [`Statistics`] buff.
    ///
    /// <https://superautopets.fandom.com/wiki/Stegosaurus>
    Stegosaurus(Statistics),
    /// Hardcoded Cockroach ability.
    ///
    /// <https://superautopets.fandom.com/wiki/Cockroach>
    Cockroach,
    /// Hardcoded moose ability
    /// 1. [`Statistics`] buff.
    ///
    /// <https://superautopets.fandom.com/wiki/Moose>
    Moose(Statistics),
    /// Hardcoded Fox ability.
    /// 1. Item [type](crate::Entity) to steal.
    /// 2. Buff multiplier.
    ///     * Fox set to `2x`
    ///
    /// <https://superautopets.fandom.com/wiki/Fox>
    Fox(Entity, usize),
    /// Gain experiences point.
    /// 1. Number of points to gain.
    Experience(usize),
    /// Endure damage so health doesn't go below one.
    /// * An item-only [`Action`].
    /// * Used for the [`Pepper`](crate::FoodName::Pepper).
    Endure,
    /// Adjust counter for a team.
    /// 1. Counter name to modify.
    ///     * If this counter does not exist, a new entry is created.
    /// 2. Amount to modify counter by.
    ///     * Positive values increment, while negative values decrement the count.
    ///
    /// **NOTE**: When creating an effect with this action, a [`Position`] must be set that targets an existing pet.
    /// * This is because internally, all effects target a pet.
    /// ```
    /// use saptest::{
    ///     Effect, Position,
    ///     effects::{trigger::*, state::Target, actions::Action}
    /// };
    /// let add_trumpets_effect = Effect {
    ///     owner: None,
    ///     trigger: TRIGGER_SELF_FAINT,
    ///     target: Target::Friend,
    ///     // Doesn't target pet but is required.
    ///     position: Position::TriggerAffected,
    ///     action: Action::AddToCounter(String::from("Trumpets"), 2),
    ///     uses: Some(1),
    ///     temp: true,
    /// }
    /// ```
    AddToCounter(String, isize),
    #[default]
    /// No action to take.
    None,
}
