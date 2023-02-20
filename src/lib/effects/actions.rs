use crate::{
    effects::{
        effect::Effect,
        state::{Condition, Position, Target},
        stats::Statistics,
    },
    foods::{food::Food, names::FoodName},
    pets::pet::Pet,
    Entity, PetName,
};
use serde::{Deserialize, Serialize};

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
    /// Change stats by a static value.
    StaticValue(Statistics),
    /// Change `Statistics` by a given percentage of the stats of the owner of this `Action`.
    /// * Example: `Lion` or `Leopard`
    SelfMultValue(Statistics),
}

/// Types of summons for [`Action::Summon`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum SummonType {
    /// Summon a pet via a SQL query.
    /// * 3rd arg sets Statistics if provided.
    /// * Not easy to modify values within query.
    QueryPet(String, Vec<String>, Option<Statistics>),
    /// Summon a stored pet.
    StoredPet(Box<Pet>),
    /// Summon a default pet.
    DefaultPet(PetName),
    /// Summon a custom pet with stats from [`StatChangeType`].
    CustomPet(PetName, StatChangeType, usize),
    /// Summon the pet owning the effect of this action.
    SelfPet(Statistics),
    /// Summon a pet at the same tier as this pet with default stats.
    SelfTierPet,
}

/// Types of item gains for [`Action::Gain`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum GainType {
    /// Gain the item of the pet owning the effect of this action.
    SelfItem,
    /// Gain the default food item.
    DefaultItem(FoodName),
    /// Gain the stored item.
    StoredItem(Box<Food>),
    /// Remove item.
    NoItem,
}

/// Types of ways [`Action::Swap`] or [`Action::Shuffle`] can alter pets.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum RandomizeType {
    /// Alter positions.
    Positions,
    /// Alter positions.
    Stats,
}

/// Conditional logic for [`Action::Conditional`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ConditionType {
    /// Do multiple `Action`s based on number of `Pet`s matching a `Condition`.
    ForEach(Target, Condition),
    /// If target `Pet` meets condition, do `Action`.
    If(Target, Condition),
}

/// Pet actions.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub enum Action {
    /// Add some amount of `Statistics` to a `Pet`.
    Add(StatChangeType),
    /// Remove some amount of `Statistics` from a `Pet`.
    Remove(StatChangeType),
    /// Debuff a `Pet` by subtracting some **percent** of `Statistics` from it.
    Debuff(Statistics),
    /// Shuffle all pets on a specific RandomizeType.
    Shuffle(RandomizeType),
    /// Swap two pets on a specific RandomizeType.
    Swap(RandomizeType),
    /// Push a `Pet` to some new position from its original position. The following positions are implemented.
    /// * [`Position::Relative`]
    /// * [`Position::First`]
    /// * [`Position::Last`]
    Push(Position),
    /// Copy some attribute from a `Pet` to a given `Position`.
    Copy(CopyType, Target, Position),
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
    Gain(GainType),
    /// Add permanent stats to shop.
    AddShopStats(Statistics),
    /// Add a shop food.
    AddShopFood(GainType),
    /// Add a shop pet.
    AddShopPet(SummonType),
    /// Clear shop items.
    ClearShop(Entity),
    /// Get gold.
    Profit,
    /// Free roll.
    FreeRoll,
    /// Summon a `Pet` with an optional `Statistics` arg to replace store `Pet`.
    Summon(SummonType),
    /// Do multiple `Action`s.
    Multiple(Vec<Action>),
    /// Perform a conditional `Action`.
    Conditional(ConditionType, Box<Action>),
    /// Hardcoded rhino ability.
    Rhino(Statistics),
    /// Hardcoded lynx ability.
    Lynx,
    /// Hardcoded vulture ability.
    Vulture(Statistics),
    /// Hardcoded stegosaurus ability
    Stegosaurus(Statistics),
    /// Hardcoded tapir ability.
    Tapir,
    /// Hardcoded cockroach ability.
    Cockroach,
    /// Gain one experience point.
    Experience,
    /// WIP: Endure damage so health doesn't go below one.
    Endure,
    #[default]
    /// No action to take.
    None,
}
