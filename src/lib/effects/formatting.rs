use itertools::Itertools;

use crate::{Effect, ItemCondition};

use super::{
    actions::{Action, ConditionType, CopyType, GainType, LogicType, StatChangeType, SummonType},
    state::{EqualityCondition, Outcome, Status, TeamCondition},
};

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Effect (Uses: {:?}): Action: {} on {:?} ({:?}), Trigger: {}]",
            self.uses, self.action, self.target, self.position, self.trigger
        )
    }
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Status: {:?}, Position: {:?}, Affected: {:?}, From: {:?}]",
            self.status, self.position, self.affected_pet, self.afflicting_pet
        )
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::fmt::Display for StatChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatChangeType::Static(stats) => write!(f, "{stats}"),
            StatChangeType::Multiplier(stats) => {
                write!(f, "({}%, {}%) of Stats", stats.attack, stats.health)
            }
            StatChangeType::StaticAttack(atk) => write!(f, "({atk}, 0)"),
            StatChangeType::StaticHealth(health) => write!(f, "(0, {health})"),
            StatChangeType::CurrentAttack => write!(f, "To Current Attack"),
            StatChangeType::CurrentHealth => write!(f, "To Current Health"),
            StatChangeType::TeamCounter(counter_key) => write!(f, "Based on {counter_key}"),
        }
    }
}

impl std::fmt::Display for EqualityCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::fmt::Display for CopyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CopyType::PercentStats(stats) => write!(f, "({}%, {}%)", stats.attack, stats.health),
            CopyType::Stats(stats) => {
                let stats = stats.map_or("Self Stats".to_owned(), |stats| stats.to_string());
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
            CopyType::None => write!(f, "Nothing"),
        }
    }
}

impl std::fmt::Display for GainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GainType::SelfItem => write!(f, "Self Item"),
            GainType::DefaultItem(food_name) => write!(f, "{food_name}"),
            GainType::QueryItem(query, params) => write!(f, "Query Item ({query}) {params:?}"),
            GainType::RandomShopItem => write!(f, "Random Shop Item"),
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
                write!(f, "Pet {stats_str} from Self Tier at Level {lvl}")
            }
            SummonType::SelfTeamPet(stats, lvl, ignore) => {
                let stats_str =
                    stats.map_or_else(|| "(Owner Stats)".to_string(), |stats| stats.to_string());
                let lvl = lvl.unwrap_or(1);
                write!(
                    f,
                    "Any (Ignoring {ignore}) Team Pet {stats_str} at Level {lvl}"
                )
            }
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Add(stat_change) => write!(f, "Add {stat_change}"),
            Action::Set(stat_change) => write!(f, "Set {stat_change}"),
            Action::Remove(stat_change) => write!(f, "Damage {stat_change}"),
            Action::Debuff(stat_change) => write!(f, "Debuff {stat_change}"),
            Action::Shuffle(shuffle_type) => write!(f, "Shuffle {shuffle_type:?}"),
            Action::Swap(swap_type) => write!(f, "Swap {swap_type:?}"),
            Action::Push(pos) => write!(f, "Push from Current to {pos:?} Position"),
            Action::Copy(copy_type, target, pos) => {
                write!(f, "Copy {copy_type} to {pos:?} Pet(s) on {target:?} Team.")
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
            Action::AlterGold(gold_change) => write!(f, "Alter gold by {gold_change}"),
            Action::Discount(item_type, gold) => {
                write!(f, "Discount {gold} Gold from {item_type:?}")
            }
            Action::SaveGold { limit } => write!(f, "Save Remaining Gold up to {limit} Gold"),
            Action::FreeRoll(rolls) => write!(f, "Gain {rolls} Free Rolls"),
            Action::Summon(summon_type) => write!(f, "Summon {summon_type}"),
            Action::Multiple(actions) => {
                let action_str = actions
                    .iter()
                    .map(|action| action.to_string())
                    .join(" And ");
                write!(f, "Do {action_str}.")
            }
            Action::Lynx => write!(f, "Lynx (Damage Equal Sum Levels)"),
            Action::Stegosaurus(stats) => write!(f, "Stegosaurus (Add {stats} x Turns)"),
            Action::Cockroach => write!(f, "Cockroach (Set Attack Equal Shop Tier + Level)"),
            Action::Moose(stats) => write!(f, "Moose (Unfreeze And Add {stats} x Lowest Tier)"),
            Action::Fox(item_type, multiplier) => {
                write!(f, "Fox (Steal {item_type:?} With {multiplier}x Stats)")
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
            Action::AddToCounter(counter, count_change) => {
                write!(f, "Adjust {counter} by {count_change}")
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
            actions::{
                Action, ConditionType, CopyType, GainType, LogicType, RandomizeType,
                StatChangeType, SummonType,
            },
            state::{EqualityCondition, Status, Target, TeamCondition},
        },
        teams::team::TeamFightOutcome,
        Entity, EntityName, Food, FoodName, ItemCondition, Pet, PetName, Position, Statistics,
    };

    #[test]
    fn test_stat_change_type_formatting() {
        let add_action = Action::Add(StatChangeType::Multiplier(Statistics {
            attack: 50,
            health: 0,
        }));
        assert_eq!("Add (50%, 0%) of Stats", format!("{add_action}"));

        let remove_action = Action::Remove(StatChangeType::Static(Statistics {
            attack: 10,
            health: 0,
        }));
        assert_eq!("Damage (10, 0)", format!("{remove_action}"));

        let debuff_action = Action::Debuff(StatChangeType::Multiplier(Statistics {
            attack: 50,
            health: 50,
        }));
        assert_eq!("Debuff (50%, 50%) of Stats", format!("{debuff_action}"));
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

        assert_eq!("Faint", format!("{}", Action::Kill));

        assert_eq!("Invincibility", format!("{}", Action::Invincible));

        let multi_action = Action::Multiple(vec![
            Action::ClearShop(Entity::Food),
            Action::AddShopStats(Statistics {
                attack: 1,
                health: 1,
            }),
        ]);
        assert_eq!(
            "Do Clear Shop Food And Add Shop (1, 1).",
            format!("{multi_action}")
        );

        let transform_action = Action::Transform(
            PetName::Butterfly,
            Some(Statistics {
                attack: 1,
                health: 1,
            }),
            1,
        );
        assert_eq!(
            "Transform into Level 1 Butterfly at (1, 1)",
            format!("{transform_action}")
        );

        let lynx_action = Action::Lynx;
        assert_eq!("Lynx (Damage Equal Sum Levels)", format!("{lynx_action}"));

        let stego_action = Action::Stegosaurus(Statistics {
            attack: 1,
            health: 1,
        });
        assert_eq!(
            "Stegosaurus (Add (1, 1) x Turns)",
            format!("{stego_action}")
        );

        let cockroach_action = Action::Cockroach;
        assert_eq!(
            "Cockroach (Set Attack Equal Shop Tier + Level)",
            format!("{cockroach_action}")
        );

        let moose_action = Action::Moose(Statistics {
            attack: 1,
            health: 1,
        });
        assert_eq!(
            "Moose (Unfreeze And Add (1, 1) x Lowest Tier)",
            format!("{moose_action}")
        );

        let fox_action = Action::Fox(Entity::Food, 2);
        assert_eq!("Fox (Steal Food With 2x Stats)", format!("{fox_action}"));

        let add_experience_action = Action::Experience(2);
        assert_eq!("Add Experience (2)", format!("{add_experience_action}"));

        let endure_action = Action::Endure;
        assert_eq!("Endure (Pepper)", format!("{endure_action}"));

        let no_action = Action::None;
        assert_eq!("None", format!("{no_action}"));
    }

    #[test]
    fn test_shop_action_formatting() {
        let add_shop_pet = Action::AddShopPet(SummonType::QueryPet(
            "SELECT * FROM pets".to_string(),
            vec![],
            None,
        ));
        assert_eq!(
            "Add Query Pet (SELECT * FROM pets) [] (Default Stats) to Shop",
            format!("{add_shop_pet}")
        );

        let add_shop_food_action = Action::AddShopFood(GainType::RandomShopItem);
        assert_eq!(
            "Add Random Shop Item to Shop",
            format!("{add_shop_food_action}")
        );

        let shop_add_stats_action = Action::AddShopStats(Statistics {
            attack: 1,
            health: 1,
        });
        assert_eq!("Add Shop (1, 1)", format!("{shop_add_stats_action}"));

        let shop_clear_action = Action::ClearShop(Entity::Food);
        assert_eq!("Clear Shop Food", format!("{shop_clear_action}"));

        let shop_profit_action = Action::AlterGold(3);
        assert_eq!("Alter gold by 3", format!("{shop_profit_action}"));

        let shop_discount_action = Action::Discount(Entity::Food, 3);
        assert_eq!(
            "Discount 3 Gold from Food",
            format!("{shop_discount_action}")
        );

        let shop_free_roll_action = Action::FreeRoll(3);
        assert_eq!("Gain 3 Free Rolls", format!("{shop_free_roll_action}"));
    }

    #[test]
    fn test_gain_type_formatting() {
        let gain_self_item_action = Action::Gain(GainType::SelfItem);
        assert_eq!("Gain Self Item", format!("{gain_self_item_action}"));

        let gain_def_item_action = Action::Gain(GainType::DefaultItem(FoodName::Carrot));
        assert_eq!("Gain Carrot", format!("{gain_def_item_action}"));

        let gain_query_item_action = Action::Gain(GainType::QueryItem(
            "SELECT * FROM foods WHERE name = ?".to_string(),
            vec!["Garlic".to_string()],
        ));
        assert_eq!(
            "Gain Query Item (SELECT * FROM foods WHERE name = ?) [\"Garlic\"]",
            format!("{gain_query_item_action}"),
        );

        let gain_random_shop_item_action = Action::Gain(GainType::RandomShopItem);
        assert_eq!(
            "Gain Random Shop Item",
            format!("{gain_random_shop_item_action}")
        );

        let gain_stored_item_action = Action::Gain(GainType::StoredItem(Box::new(
            Food::try_from(FoodName::Chocolate).unwrap(),
        )));
        assert_eq!(
            "Gain [Chocolate: [Effect (Uses: None): Action: Add Experience (1) on Friend (OnSelf), Trigger: [Status: AteFood, Position: OnSelf, Affected: None, From: None]]]",
            format!("{gain_stored_item_action}")
        );

        let gain_no_item_action = Action::Gain(GainType::NoItem);
        assert_eq!("Gain No Item", format!("{gain_no_item_action}"));
    }

    #[test]
    fn test_summon_type_formatting() {
        let summon_query_pet_action = Action::Summon(SummonType::QueryPet(
            "SELECT * FROM pets WHERE name = ?".to_string(),
            vec!["Dog".to_owned()],
            Some(Statistics {
                attack: 50,
                health: 50,
            }),
        ));
        assert_eq!(
            "Summon Query Pet (SELECT * FROM pets WHERE name = ?) [\"Dog\"] (50, 50)",
            format!("{summon_query_pet_action}")
        );

        let summon_stored_pet_action = Action::Summon(SummonType::StoredPet(Box::new(
            Pet::try_from(PetName::Ant).unwrap(),
        )));
        assert_eq!(
            "Summon [Ant: (2,2) (Level: 1 Exp: 0) (Pos: None) (Item: None)]",
            format!("{summon_stored_pet_action}")
        );

        let summon_default_pet_action = Action::Summon(SummonType::DefaultPet(PetName::Ant));
        assert_eq!("Summon Default Ant", format!("{summon_default_pet_action}"));

        let summon_custom_pet = Action::Summon(SummonType::CustomPet(
            PetName::Chick,
            StatChangeType::Static(Statistics {
                attack: 12,
                health: 1,
            }),
            1,
        ));
        assert_eq!(
            "Summon Custom Chick (12, 1) at Level 1",
            format!("{summon_custom_pet}")
        );

        let summon_self_pet = Action::Summon(SummonType::SelfPet(
            Some(Statistics {
                attack: 1,
                health: 1,
            }),
            Some(3),
            false,
        ));
        assert_eq!(
            "Summon Self Pet (1, 1) at Lvl 3 (Keep Item: false)",
            format!("{summon_self_pet}")
        );

        let summon_self_tier_pet = Action::Summon(SummonType::SelfTierPet(
            Some(Statistics {
                attack: 1,
                health: 1,
            }),
            Some(1),
        ));
        assert_eq!(
            "Summon Pet (1, 1) from Self Tier at Level 1",
            format!("{summon_self_tier_pet}")
        );

        let summon_self_team_pet =
            Action::Summon(SummonType::SelfTeamPet(None, None, PetName::Tapir));
        assert_eq!(
            "Summon Any (Ignoring Tapir) Team Pet (Owner Stats) at Level 1",
            format!("{summon_self_team_pet}")
        );
    }

    #[test]
    fn test_copy_type_formatting() {
        let pet = Pet::try_from(PetName::Badger).unwrap();

        let copy_effect_action = Action::Copy(
            CopyType::Effect(pet.effect, Some(2)),
            Target::Friend,
            Position::All(ItemCondition::None),
        );
        assert_eq!(
            "Copy [Effect (Uses: Some(1)): Action: Damage (50%, 0%) of Stats on Either (Multiple([Relative(1), Relative(-1)])), Trigger: [Status: Faint, Position: OnSelf, Affected: None, From: None]] at Lvl 2 to All(None) Pet(s) on Friend Team.",
            format!("{copy_effect_action}")
        );

        let copy_item_action = Action::Copy(CopyType::Item(None), Target::Enemy, Position::First);
        assert_eq!(
            "Copy Target's Item to First Pet(s) on Enemy Team.",
            format!("{copy_item_action}")
        );

        let copy_perc_stats_action = Action::Copy(
            CopyType::PercentStats(Statistics {
                attack: 15,
                health: 15,
            }),
            Target::Either,
            Position::Any(ItemCondition::Equal(EqualityCondition::Tier(2))),
        );
        assert_eq!(
            "Copy (15%, 15%) to Any(Equal(Tier(2))) Pet(s) on Either Team.",
            format!("{copy_perc_stats_action}")
        );

        let copy_stats_action = Action::Copy(
            CopyType::Stats(Some(Statistics {
                attack: 15,
                health: 15,
            })),
            Target::Enemy,
            Position::Last,
        );
        assert_eq!(
            "Copy (15, 15) to Last Pet(s) on Enemy Team.",
            format!("{copy_stats_action}")
        );

        let no_copy_action = Action::Copy(CopyType::None, Target::Friend, Position::First);
        assert_eq!(
            "Copy Nothing to First Pet(s) on Friend Team.",
            format!("{no_copy_action}")
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
            Box::new(Action::Remove(StatChangeType::Static(Statistics {
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
            Box::new(Action::Add(StatChangeType::Static(Statistics {
                attack: 1,
                health: 1,
            }))),
            Box::new(Action::None),
        );

        assert_eq!(
            "For Each Pet (Shop) Equal to Tier(5) Or Tier(6) Add (1, 1). Otherwise, None.",
            format!("{conditional_for_each_action}")
        );

        let conditional_if_any_action = Action::Conditional(
            LogicType::IfAny(ConditionType::Pet(
                Target::Either,
                ItemCondition::Equal(EqualityCondition::Tier(2)),
            )),
            Box::new(Action::Remove(StatChangeType::Static(Statistics {
                attack: 2,
                health: 2,
            }))),
            Box::new(Action::Add(StatChangeType::Static(Statistics {
                attack: 2,
                health: 2,
            }))),
        );
        assert_eq!(
            "If Any Pet (Either) Tier(2) Then Damage (2, 2). Otherwise, Add (2, 2).",
            format!("{conditional_if_any_action}")
        );

        let conditional_if_not_action = Action::Conditional(
            LogicType::IfNot(ConditionType::Team(
                Target::Friend,
                TeamCondition::PreviousBattle(TeamFightOutcome::Loss),
            )),
            Box::new(Action::Add(StatChangeType::Static(Statistics {
                attack: 2,
                health: 2,
            }))),
            Box::new(Action::None),
        );
        assert_eq!(
            "If Not Friend Team PreviousBattle(Loss) Then Add (2, 2). Otherwise, None.",
            format!("{conditional_if_not_action}")
        );

        let conditional_if_multi_all_action = Action::Conditional(
            LogicType::If(ConditionType::Pet(
                Target::Friend,
                ItemCondition::MultipleAll(vec![
                    ItemCondition::Equal(EqualityCondition::Tier(3)),
                    ItemCondition::Equal(EqualityCondition::Trigger(Status::Faint)),
                ]),
            )),
            Box::new(Action::Add(StatChangeType::Static(Statistics {
                attack: 2,
                health: 2,
            }))),
            Box::new(Action::None),
        );
        assert_eq!(
            "If Pet (Friend) Equal to Tier(3) And Trigger(Faint) Then Add (2, 2). Otherwise, None.",
            format!("{conditional_if_multi_all_action}"),
        )
    }
}
