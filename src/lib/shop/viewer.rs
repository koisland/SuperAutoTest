use itertools::Itertools;
use rand::seq::IteratorRandom;

use super::store::{ItemSlot, ItemState, ShopItem};
use crate::{battle::{
    actions::{Action, StatChangeType},
    state::{Status, Condition},
}, Entity, Position, error::SAPTestError, Shop};

/// View shop items.
pub trait ShopViewer: {
    /// Get shop items by condition.
    /// ```
    /// use saptest::{Shop, ShopViewer};
    /// 
    /// let shop = Shop::new(1, Some(42));
    /// 
    /// ```
    fn get_shop_items_by_cond(
        &self,
        cond: &Condition,
        item_type: &Entity,
    ) -> Vec<&ShopItem>;
    /// Get shop items by position.
    fn get_shop_items_by_pos(
        &self,
        pos: &Position,
        item: &Entity,
    ) -> Result<Vec<&ShopItem>, SAPTestError>;
}

impl ShopViewer for Shop {
    fn get_shop_items_by_cond(
        &self,
        cond: &Condition,
        item_type: &Entity,
    ) -> Vec<&ShopItem> {
        let mut found_foods = Vec::new();
        let all_items = match item_type {
            Entity::Pet => self.pets.iter(),
            Entity::Food => self.foods.iter(),
        };

        match cond {
            Condition::None => found_foods.extend(all_items),
            Condition::Healthiest => {
                if let Some(highest_tier_food) = all_items
                    .max_by(|item_1, item_2| item_1.health_stat().cmp(&item_2.health_stat()))
                {
                    found_foods.push(highest_tier_food)
                }
            }
            Condition::Illest => {
                if let Some(lowest_tier_food) = all_items
                    .min_by(|food_1, food_2| food_1.health_stat().cmp(&food_2.health_stat()))
                {
                    found_foods.push(lowest_tier_food)
                }
            }
            Condition::Strongest => {
                if let Some(highest_tier_food) = all_items
                    .max_by(|item_1, item_2| item_1.attack_stat().cmp(&item_2.attack_stat()))
                {
                    found_foods.push(highest_tier_food)
                }
            }
            Condition::Weakest => {
                if let Some(lowest_tier_food) = all_items
                    .min_by(|food_1, food_2| food_1.attack_stat().cmp(&food_2.attack_stat()))
                {
                    found_foods.push(lowest_tier_food)
                }
            }
            Condition::HighestTier => {
                if let Some(highest_tier_food) =
                    all_items.max_by(|item_1, item_2| item_1.tier().cmp(&item_2.tier()))
                {
                    found_foods.push(highest_tier_food)
                }
            }
            Condition::LowestTier => {
                if let Some(lowest_tier_food) =
                    all_items.min_by(|food_1, food_2| food_1.tier().cmp(&food_2.tier()))
                {
                    found_foods.push(lowest_tier_food)
                }
            }
            Condition::TriggeredBy(trigger_status) => found_foods.extend(
                all_items
                    .filter_map(|item| {
                        let item_triggers = item.triggers();
                        (item_triggers.contains(&&trigger_status)).then_some(item)
                    })
                    .into_iter(),
            ),
            _ => unimplemented!(),
        };
        found_foods
    }

    fn get_shop_items_by_pos(
        &self,
        pos: &Position,
        item: &Entity,
    ) -> Result<Vec<&ShopItem>, SAPTestError> {
        let mut found_items = vec![];

        match pos {
            Position::Any(condition) => {
                let mut rng = self.get_rng();
                let found_found_items = self
                    .get_shop_items_by_cond(condition, item)
                    .into_iter()
                    .choose(&mut rng);
                if let Some(any_item) = found_found_items {
                    found_items.push(any_item)
                }
            }
            Position::All(condition) => {
                let found_found_items = self.get_shop_items_by_cond(condition, item);
                found_items.extend(found_found_items)
            }
            Position::First => {
                let item = if let Entity::Food = item {
                    self.foods.first()
                } else {
                    self.pets.first()
                };

                if let Some(item) = item {
                    found_items.push(item)
                };
            }
            Position::Last => {
                let item = if let Entity::Food = item {
                    self.foods.last()
                } else {
                    self.pets.last()
                };

                if let Some(item) = item {
                    found_items.push(item)
                };
            }
            Position::Range(range_idx) => {
                let end_idx = range_idx.clone()
                    .into_iter()
                    .filter_map(|idx| TryInto::<usize>::try_into(idx).ok())
                    .max();
                let found_found_items = if let Entity::Food = item {
                    end_idx.map(|idx| self.foods.get(0..idx))
                } else {
                    end_idx.map(|idx| self.pets.get(0..idx))
                };
                if let Some(Some(found_found_items)) = found_found_items {
                    found_items.extend(found_found_items)
                }
            }
            Position::Relative(idx) => {
                let converted_idx =
                    TryInto::<usize>::try_into(-idx)
                        .ok()
                        .ok_or(SAPTestError::ShopError {
                            reason: format!("Invalid relative index. {idx}"),
                        })?;
                if let Some(found_item) = if let Entity::Food = item {
                    self.foods.get(converted_idx)
                } else {
                    self.pets.get(converted_idx)
                } {
                    found_items.push(found_item)
                }
            }
            _ => unimplemented!(),
        };

        Ok(found_items)
    }
}

/// View the state of a single shop item.
pub trait ShopItemViewer {
    /// Check if item in Shop is frozen.
    fn is_frozen(&self) -> bool;
    /// Get health stat of ShopItem.
    fn health_stat(&self) -> Option<isize>;
    /// Get attack stat of ShopItem.
    fn attack_stat(&self) -> Option<isize>;
    /// Get tier of ShopItem.
    fn tier(&self) -> usize;
    /// Get effect triggers of ShopItem.
    fn triggers(&self) -> Vec<&Status>;
}

impl ShopItemViewer for &ShopItem {
    fn is_frozen(&self) -> bool {
        self.state == ItemState::Frozen
    }
    /// Get health stat of item.
    fn health_stat(&self) -> Option<isize> {
        match &self.item {
            ItemSlot::Pet(pet) => Some(pet.stats.health),
            ItemSlot::Food(food) => match food.ability.action {
                Action::Add(StatChangeType::StaticValue(stats)) => Some(stats.health),
                Action::Remove(StatChangeType::StaticValue(stats)) => Some(stats.health),
                _ => None,
            },
        }
    }
    /// Get attack stat of item.
    fn attack_stat(&self) -> Option<isize> {
        match &self.item {
            ItemSlot::Pet(pet) => Some(pet.stats.attack),
            ItemSlot::Food(food) => match food.ability.action {
                Action::Add(StatChangeType::StaticValue(stats)) => Some(stats.attack),
                Action::Remove(StatChangeType::StaticValue(stats)) => Some(stats.attack),
                _ => None,
            },
        }
    }
    /// Get tier of item.
    fn tier(&self) -> usize {
        match &self.item {
            ItemSlot::Pet(pet) => pet.tier,
            ItemSlot::Food(food) => food.tier,
        }
    }
    /// Get effect triggers of item.
    fn triggers(&self) -> Vec<&Status> {
        match &self.item {
            ItemSlot::Pet(pet) => pet
                .effect
                .iter()
                .map(|effect| &effect.trigger.status)
                .collect_vec(),
            ItemSlot::Food(food) => vec![&food.ability.trigger.status],
        }
    }
}
