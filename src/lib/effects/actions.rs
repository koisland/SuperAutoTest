use crate::{
    effects::{
        effect::Effect,
        state::{ItemCondition, Position, ShopCondition, Target, TeamCondition},
        stats::Statistics,
    },
    foods::{food::Food, names::FoodName},
    pets::pet::Pet,
    Entity, PetName,
};
use itertools::Itertools;
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
    /// Summon a [`Pet`] via a `SQL` query.
    /// * 1st and 2nd argument are the `SQL` statement with params listed as (`?`).
    /// * All fields must be kept in the `SELECT` statment with the `*`.
    /// # Example
    /// ```rust no_run
    /// use saptest::effects::actions::SummonType;
    /// // Summon a dog at default stats.
    /// let sql = "SELECT * FROM pets WHERE name = ?";
    /// let params = vec!['Dog'.to_string()];
    /// let summon_type = SummonType::QueryPet(sql.to_owned(), params, None)
    /// ```
    /// * 3rd argument sets [`Statistics`] if provided.
    QueryPet(String, Vec<String>, Option<Statistics>),
    /// Summon a stored [`Pet`].
    StoredPet(Box<Pet>),
    /// Summon a [`Pet`] with default `stats` and `level`.
    DefaultPet(PetName),
    /// Summon a custom [`Pet`] with `stats` from [`StatChangeType`]. Used for [`Rooster`](crate::PetName::Rooster)
    /// * 3rd argument lists the `level` the summon should be.
    CustomPet(PetName, StatChangeType, usize),
    /// Summon the [`Pet`] owning the [`Effect`] of this [`Action::Summon`].
    /// * 1st argument sets `stats`. Defaults to current [`Pet`] if omitted.
    /// * 2nd argument sets `level`. Defaults to current [`Pet`] if omitted.
    /// * 3rd argument specifies keeping item of current [`Pet`].
    SelfPet(Option<Statistics>, Option<usize>, bool),
    /// Summon a random pet at the same `tier` as the current [`Pet`]. Used for [`Popcorns`](crate::FoodName::Popcorns).
    /// * 1st argument sets `stats`. Defaults to current [`Pet`] if omitted.
    /// * 2nd argument sets `level`. Defaults to current [`Pet`] if omitted.
    SelfTierPet(Option<Statistics>, Option<usize>),
    /// Summon a random [`Pet`] from the same [`Team`](crate::Team). Used for [`Tapir`](crate::PetName::Tapir).
    /// * 1st argument sets `stats`. Defaults to current [`Pet`] if omitted.
    /// * 2nd argument sets `level`. Defaults to current [`Pet`] if omitted.
    /// * 3rd argument will ignore any [`Pet`]s on the [`Team`](crate::Team) with this [`PetName`].
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
    /// let summon_type = GainType::QueryItem(sql.to_owned(), params, None);
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
    /// Alter positions.
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
    /// Remove [`Statistics`] from a [`Pet`].
    /// * Altering stats this way creates hurt trigger [`Outcome`](crate::effects::state::Outcome)s.
    Remove(StatChangeType),
    /// Debuff a [`Pet`] by subtracting some **percent** of [`Statistics`] from it.
    Debuff(Statistics),
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
    /// **Note: This does not emit a summon trigger.**
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
    ///     Box::new(Action::Remove(StatChangeType::StaticValue(effect_stats))),
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
    ///     Box::new(Action::Add(StatChangeType::StaticValue(effect_stats))),
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
    #[default]
    /// No action to take.
    None,
}

impl std::fmt::Display for StatChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatChangeType::StaticValue(stats) => write!(f, "{stats}"),
            StatChangeType::SelfMultValue(stats) => {
                write!(f, "({}%, {}%) of Self Stats", stats.attack, stats.health)
            }
        }
    }
}

impl std::fmt::Display for CopyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CopyType::PercentStats(stats) => write!(f, "{stats}%"),
            CopyType::Stats(stats) => {
                let stats = stats.map_or("Target's Stats".to_owned(), |stats| stats.to_string());
                write!(f, "{stats}")
            }
            CopyType::Effect(effects, lvl) => {
                let effect_str = effects
                    .iter()
                    .map(|effect| effect.to_string())
                    .join(" And ");
                write!(f, "{effect_str} at Lvl {}", lvl.unwrap_or(1))
            }
            CopyType::Item(item) => match item {
                Some(food) => write!(f, "Copied {food}"),
                None => write!(f, "Target's Item"),
            },
            CopyType::None => write!(f, "None"),
        }
    }
}

impl std::fmt::Display for GainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GainType::SelfItem => write!(f, "Target Item"),
            GainType::DefaultItem(food_name) => write!(f, "{food_name}"),
            GainType::QueryItem(query, params) => write!(f, "Query Item ({query}) {params:?}"),
            GainType::RandomShopItem => write!(f, "Random Shop item"),
            GainType::StoredItem(item) => write!(f, "{item}"),
            GainType::NoItem => write!(f, "No Item"),
        }
    }
}

impl std::fmt::Display for SummonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SummonType::QueryPet(sql, params, stats) => {
                let stats_str =
                    stats.map_or_else(|| "(Default Stats)".to_string(), |stats| stats.to_string());
                write!(f, "Query Pet ({sql}) {params:?} {stats_str}")
            }
            SummonType::StoredPet(pet) => write!(f, "{pet}"),
            SummonType::DefaultPet(pet_name) => write!(f, "Default {pet_name}"),
            SummonType::CustomPet(pet_name, stats, lvl) => {
                write!(f, "Custom {pet_name} {stats} at Level {lvl}")
            }
            SummonType::SelfPet(stats, lvl, keep_item) => {
                let stats_str =
                    stats.map_or_else(|| "(Same Stats)".to_string(), |stats| stats.to_string());
                let lvl = lvl.unwrap_or(1);
                write!(
                    f,
                    "Self Pet {stats_str} at Lvl {lvl} (Keep Item: {keep_item})"
                )
            }
            SummonType::SelfTierPet(stats, lvl) => {
                let stats_str =
                    stats.map_or_else(|| "(Same Stats)".to_string(), |stats| stats.to_string());
                let lvl = lvl.unwrap_or(1);
                write!(f, "Pet {stats_str} at Self Tier at Level {lvl}")
            }
            SummonType::SelfTeamPet(stats, lvl, ignore) => {
                let stats_str =
                    stats.map_or_else(|| "(Owner Stats)".to_string(), |stats| stats.to_string());
                let lvl = lvl.unwrap_or(1);
                write!(
                    f,
                    "Any (Ignoring {ignore}) Pet {stats_str} at Self Tier at Level {lvl}"
                )
            }
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Add(stat_change) => write!(f, "Add {stat_change}"),
            Action::Remove(stat_change) => write!(f, "Damage {stat_change}"),
            Action::Debuff(stat_debuff) => write!(
                f,
                "Debuff ({}%, {}%)",
                stat_debuff.attack, stat_debuff.health
            ),
            Action::Shuffle(shuffle_type) => write!(f, "Shuffle {shuffle_type:?}"),
            Action::Swap(swap_type) => write!(f, "Swap {swap_type:?}"),
            Action::Push(pos) => write!(f, "Push from Current to {pos:?} Position"),
            Action::Copy(copy_type, target, pos) => {
                write!(f, "Copy {copy_type} to {pos:?} {target:?}")
            }
            Action::Negate(stats) => write!(f, "Negate {stats}"),
            Action::Critical(percentage) => write!(f, "Critical Chance {percentage}%"),
            Action::Whale(lvl, pos) => write!(f, "Evolve {pos:?} to {lvl}"),
            Action::Transform(petname, stats, lvl) => {
                let stats_str =
                    stats.map_or_else(|| "Current Stats".to_string(), |stats| stats.to_string());
                write!(f, "Transform into Level {lvl} {petname} at {stats_str}")
            }
            Action::Kill => write!(f, "Faint"),
            Action::Invincible => write!(f, "Invincibility"),
            Action::Gain(gain_type) => write!(f, "Gain {gain_type}"),
            Action::AddShopStats(stats) => write!(f, "Add Shop {stats}"),
            Action::AddShopFood(food) => write!(f, "Add {food} to Shop"),
            Action::AddShopPet(pet) => write!(f, "Add {pet} to Shop"),
            Action::ClearShop(item_type) => write!(f, "Clear Shop {item_type:?}"),
            Action::Profit(gold) => write!(f, "Gain {gold} Gold"),
            Action::Discount(item_type, gold) => {
                write!(f, "Discount {gold} Gold from {item_type:?}")
            }
            Action::FreeRoll(rolls) => write!(f, "Gain {rolls} Free Rolls"),
            Action::Summon(summon_type) => write!(f, "Summon {summon_type}"),
            Action::Multiple(actions) => {
                let action_str = actions
                    .iter()
                    .map(|action| action.to_string())
                    .join(" And ");
                write!(f, "Do {action_str}.")
            }
            Action::Lynx => write!(f, "Lynx (Dmg Sum Levels)"),
            Action::Stegosaurus(stats) => write!(f, "Stegosaurus (Add {stats} x Turns)"),
            Action::Cockroach => write!(f, "Cockroach"),
            Action::Moose(stats) => write!(f, "Moose {stats}"),
            Action::Fox(item_type, multiplier) => {
                write!(f, "Fox (Steal {item_type:?}) ({multiplier}x Stats)")
            }
            Action::Experience(exp) => write!(f, "Add Experience ({exp})"),
            Action::Endure => write!(f, "Endure (Pepper)"),
            Action::None => write!(f, "None"),
            Action::Conditional(logic_type, if_action, else_action) => {
                // ForEach does multiplea actions per condition met. Only one else action.
                if let LogicType::ForEach(_) = logic_type {
                    write!(f, "{logic_type} {if_action}. Otherwise, {else_action}.")
                } else {
                    write!(
                        f,
                        "{logic_type} Then {if_action}. Otherwise, {else_action}."
                    )
                }
            }
        }
    }
}

impl std::fmt::Display for ItemCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemCondition::Multiple(conds) => {
                write!(
                    f,
                    "Equal to {}",
                    conds.iter().map(|cond| cond.to_string()).join(" Or ")
                )
            }
            ItemCondition::MultipleAll(conds) => {
                write!(
                    f,
                    "Equal to {}",
                    conds.iter().map(|cond| cond.to_string()).join(" And ")
                )
            }
            ItemCondition::Equal(cond) | ItemCondition::NotEqual(cond) => write!(f, "{cond}"),
            _ => write!(f, "{self:?}"),
        }
    }
}

impl std::fmt::Display for TeamCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::fmt::Display for ConditionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionType::Pet(target, item_cond) => write!(f, "Pet ({target:?}) {item_cond}"),
            ConditionType::Team(target, team_cond) => write!(f, "{target:?} Team {team_cond}"),
            ConditionType::Shop(shop_cond) => write!(f, "Shop {shop_cond:?}"),
        }
    }
}

impl std::fmt::Display for LogicType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicType::ForEach(cond) => write!(f, "For Each {cond}"),
            LogicType::If(cond) => write!(f, "If {cond}"),
            LogicType::IfNot(cond) => write!(f, "If Not {cond}"),
            LogicType::IfAny(cond) => write!(f, "If Any {cond}"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        effects::{
            actions::{ConditionType, CopyType, GainType, LogicType, SummonType},
            state::{EqualityCondition, Target},
        },
        EntityName, FoodName, ItemCondition, Pet, PetName, Position, Statistics,
    };

    use super::{Action, RandomizeType, StatChangeType};

    #[test]
    fn test_stat_change_type_formatting() {
        let add_action = Action::Add(StatChangeType::SelfMultValue(Statistics {
            attack: 50,
            health: 0,
        }));
        assert_eq!("Add (50%, 0%) of Self Stats", format!("{add_action}"));

        let remove_action = Action::Remove(StatChangeType::StaticValue(Statistics {
            attack: 10,
            health: 0,
        }));
        assert_eq!("Damage (10, 0)", format!("{remove_action}"));

        let debuff_action = Action::Debuff(Statistics {
            attack: 50,
            health: 50,
        });
        assert_eq!("Debuff (50%, 50%)", format!("{debuff_action}"));
    }

    #[test]
    fn test_randomize_type_formatting() {
        let shuffle_pos_action = Action::Shuffle(RandomizeType::Positions);
        assert_eq!("Shuffle Positions", format!("{shuffle_pos_action}"));

        let shuffle_stats_action = Action::Shuffle(RandomizeType::Stats);
        assert_eq!("Shuffle Stats", format!("{shuffle_stats_action}"));

        let swap_pos_action = Action::Swap(RandomizeType::Positions);
        assert_eq!("Swap Positions", format!("{swap_pos_action}"));

        let swap_stats_action = Action::Swap(RandomizeType::Stats);
        assert_eq!("Swap Stats", format!("{swap_stats_action}"));
    }

    #[test]
    fn test_other_action_formatting() {
        let push_action = Action::Push(Position::First);
        assert_eq!(
            "Push from Current to First Position",
            format!("{push_action}")
        );

        let negate_action = Action::Negate(Statistics {
            attack: 2,
            health: 0,
        });
        assert_eq!("Negate (2, 0)", format!("{negate_action}"));

        let critical_action = Action::Critical(25);
        assert_eq!("Critical Chance 25%", format!("{critical_action}"));

        let whale_action = Action::Whale(2, Position::Nearest(1));
        assert_eq!("Evolve Nearest(1) to 2", format!("{whale_action}"));
    }

    #[test]
    fn test_gain_type_formatting() {}

    #[test]
    fn test_summon_type_formatting() {
        let summon_action = Action::Summon(SummonType::QueryPet(
            "SELECT * FROM pets WHERE name = ?".to_string(),
            vec!["Dog".to_owned()],
            Some(Statistics {
                attack: 50,
                health: 50,
            }),
        ));
        assert_eq!(
            "Summon Query Pet (SELECT * FROM pets WHERE name = ?) [\"Dog\"] (50, 50)",
            format!("{summon_action}")
        );
    }

    #[test]
    fn test_copy_type_formatting() {
        let pet = Pet::try_from(PetName::Badger).unwrap();

        let copy_action = Action::Copy(
            CopyType::Effect(pet.effect.clone(), Some(2)),
            Target::Friend,
            Position::All(ItemCondition::None),
        );
        assert_eq!(
            "Copy [Pet Effect (Uses: Some(1)): Action: Damage (50%, 0%) of Self Stats on Either (Multiple([Relative(1), Relative(-1)])), Trigger: [Status: Faint, Position: OnSelf, Affected: None, From: None]] at Lvl 2 to All(None) Friend",
            format!("{copy_action}")
        );
    }

    #[test]
    fn test_logic_action_formatting() {
        let conditional_if_action = Action::Conditional(
            LogicType::If(ConditionType::Pet(
                Target::Friend,
                ItemCondition::Equal(EqualityCondition::Name(EntityName::Food(FoodName::Garlic))),
            )),
            Box::new(Action::Gain(GainType::DefaultItem(FoodName::Weak))),
            Box::new(Action::Remove(StatChangeType::StaticValue(Statistics {
                attack: 10,
                health: 10,
            }))),
        );
        assert_eq!(
            "If Pet (Friend) Name(Food(Garlic)) Then Gain Weak. Otherwise, Damage (10, 10).",
            format!("{conditional_if_action}")
        );

        let conditional_for_each_action = Action::Conditional(
            LogicType::ForEach(ConditionType::Pet(
                Target::Shop,
                ItemCondition::Multiple(vec![
                    ItemCondition::Equal(EqualityCondition::Tier(5)),
                    ItemCondition::Equal(EqualityCondition::Tier(6)),
                ]),
            )),
            Box::new(Action::Add(StatChangeType::StaticValue(Statistics {
                attack: 1,
                health: 1,
            }))),
            Box::new(Action::None),
        );

        assert_eq!(
            "For Each Pet (Shop) Equal to Tier(5) Or Tier(6) Add (1, 1). Otherwise, None.",
            format!("{conditional_for_each_action}")
        )
    }
}
