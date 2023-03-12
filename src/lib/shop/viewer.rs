use itertools::Itertools;
use rand::{
    random,
    seq::{IteratorRandom, SliceRandom},
    SeedableRng,
};
use rand_chacha::ChaCha12Rng;

use super::store::{ItemSlot, ItemState, ShopItem};
use crate::{
    effects::{
        actions::{Action, StatChangeType},
        effect::EntityName,
        state::{EqualityCondition, ItemCondition, Status},
    },
    error::SAPTestError,
    Entity, Position, Shop,
};

/// Enables viewing [`ShopItem`]s and their state.
pub trait ShopViewer {
    /// Get [`ShopItem`](crate::shop::store::ShopItem)s by [`ItemCondition`](crate::ItemCondition).
    /// # Example
    /// ```
    /// use saptest::{Shop, ShopViewer, ShopItemViewer, Entity, ItemCondition, Position};
    ///
    /// let (cond, item_type) = (ItemCondition::Healthiest, Entity::Pet);
    /// let shop = Shop::new(1, Some(42)).unwrap();
    ///
    /// // Find highest health pet by searching through all pets manually.
    /// let all_items = shop.get_shop_items_by_pos(&Position::All(ItemCondition::None), &item_type).unwrap();
    /// let highest_health_pet = all_items.into_iter().max_by(|pet_1, pet_2| pet_1.health_stat().cmp(&pet_2.health_stat()));
    ///
    /// // Found directly using the condition.
    /// let found_items = shop.get_shop_items_by_cond(&cond, &item_type).unwrap();
    /// let first_pet = found_items.first().copied();
    /// assert_eq!(highest_health_pet, first_pet);
    /// ```
    fn get_shop_items_by_cond(
        &self,
        cond: &ItemCondition,
        item_type: &Entity,
    ) -> Result<Vec<&ShopItem>, SAPTestError>;

    /// Get [`ShopItem`](crate::shop::store::ShopItem)s by [`Position`](crate::Position).
    /// # Example
    /// ```
    /// use saptest::{Shop, ShopViewer, Entity, Position, ItemCondition};
    ///
    /// let (pos, item_type) = (Position::All(ItemCondition::None), Entity::Pet);
    /// let shop = Shop::new(1, Some(42)).unwrap();
    /// let found_items = shop.get_shop_items_by_pos(&pos, &item_type).unwrap();
    ///
    /// // Three pets in tier 1 shop.
    /// assert_eq!(found_items.len(), 3)
    /// ```
    fn get_shop_items_by_pos(
        &self,
        pos: &Position,
        item: &Entity,
    ) -> Result<Vec<&ShopItem>, SAPTestError>;

    /// Get the number of foods in the shop.
    fn len_foods(&self) -> usize;

    /// Get the number of pets in the shop.
    fn len_pets(&self) -> usize;

    /// Get the number of food [`ShopItem`]s at the shop's current tier.
    fn max_food_slots(&self) -> usize;

    /// Get the number of available food [`ShopItem`]s based on the the number of current pets.
    fn available_food_slots(&self) -> usize;

    /// Get the number of pet [`ShopItem`]s at the shop's current tier.
    fn max_pet_slots(&self) -> usize;

    /// Adjust number of pet [`ShopItem`]s based on the the number of current pets.
    fn available_pet_slots(&self) -> usize;
}

impl ShopViewer for Shop {
    fn len_foods(&self) -> usize {
        self.foods.len()
    }

    fn len_pets(&self) -> usize {
        self.pets.len()
    }

    fn max_food_slots(&self) -> usize {
        if self.tier() < 2 {
            1
        } else {
            2
        }
    }

    fn available_food_slots(&self) -> usize {
        self.max_food_slots().saturating_sub(self.len_foods())
    }

    fn max_pet_slots(&self) -> usize {
        if self.tier() < 3 {
            3
        } else if self.tier() < 5 {
            4
        } else {
            5
        }
    }

    fn available_pet_slots(&self) -> usize {
        self.max_pet_slots().saturating_sub(self.len_pets())
    }

    fn get_shop_items_by_cond(
        &self,
        cond: &ItemCondition,
        item_type: &Entity,
    ) -> Result<Vec<&ShopItem>, SAPTestError> {
        let mut found_items = Vec::new();
        let all_items = match item_type {
            Entity::Pet => self.pets.iter(),
            Entity::Food => self.foods.iter(),
        };

        match cond {
            ItemCondition::None => found_items.extend(all_items),
            ItemCondition::Healthiest => {
                if let Some(highest_tier_item) = all_items
                    .max_by(|item_1, item_2| item_1.health_stat().cmp(&item_2.health_stat()))
                {
                    found_items.push(highest_tier_item)
                }
            }
            ItemCondition::Illest => {
                if let Some(lowest_tier_item) = all_items
                    .min_by(|item_1, item_2| item_1.health_stat().cmp(&item_2.health_stat()))
                {
                    found_items.push(lowest_tier_item)
                }
            }
            ItemCondition::Strongest => {
                if let Some(highest_tier_item) = all_items
                    .max_by(|item_1, item_2| item_1.attack_stat().cmp(&item_2.attack_stat()))
                {
                    found_items.push(highest_tier_item)
                }
            }
            ItemCondition::Weakest => {
                if let Some(lowest_tier_item) = all_items
                    .min_by(|item_1, item_2| item_1.attack_stat().cmp(&item_2.attack_stat()))
                {
                    found_items.push(lowest_tier_item)
                }
            }
            ItemCondition::HighestTier | ItemCondition::LowestTier => {
                let all_tiers = all_items.clone().map(|item| item.tier());
                let des_tier = if cond == &ItemCondition::HighestTier {
                    all_tiers.max()
                } else {
                    all_tiers.min()
                };

                if let Some(found_tier_items) = des_tier
                    .as_ref()
                    .map(|tier| all_items.into_iter().filter(|item| item.tier() == *tier))
                {
                    found_items.extend(found_tier_items)
                }
            }
            ItemCondition::Equal(eq_cond) => match eq_cond {
                EqualityCondition::Tier(tier) => {
                    found_items.extend(all_items.filter(|item| item.tier() == *tier))
                }
                EqualityCondition::Name(name) => {
                    found_items.extend(all_items.filter(|item| &item.name() == name))
                }
                EqualityCondition::Trigger(trigger) => {
                    found_items.extend(all_items.filter(|item| item.triggers().contains(trigger)))
                }
                EqualityCondition::Action(action) => {
                    found_items.extend(all_items.filter(|item| item.actions().contains(action)))
                }
                EqualityCondition::Frozen => {
                    found_items.extend(all_items.filter(|item| item.is_frozen()))
                }
                _ => {
                    return Err(SAPTestError::InvalidShopAction {
                        subject: "Invalid Equality ItemCondition".to_string(),
                        reason: format!("Cannot use {eq_cond:?} to search for items."),
                    })
                }
            },
            ItemCondition::NotEqual(eq_cond) => {
                let eq_items =
                    self.get_shop_items_by_cond(&ItemCondition::Equal(eq_cond.clone()), item_type)?;
                found_items.extend(all_items.filter(|item| !eq_items.contains(item)))
            }
            ItemCondition::Multiple(conditions) => {
                let all_cond_items: Vec<&ShopItem> = conditions
                    .iter()
                    .filter_map(|condition| self.get_shop_items_by_cond(condition, item_type).ok())
                    .flatten()
                    .collect_vec();
                found_items.extend(all_cond_items)
            }
            ItemCondition::MultipleAll(conditions) => {
                let mut matching_pets = vec![];
                let all_matches = conditions
                    .iter()
                    .filter_map(|cond| self.get_shop_items_by_cond(cond, item_type).ok())
                    .collect_vec();
                // Take smallest set of matches.
                if let Some(mut first_matching_pets) = all_matches
                    .iter()
                    .min_by(|matches_1, matches_2| matches_1.len().cmp(&matches_2.len()))
                    .cloned()
                {
                    // Remove any pets not within.
                    for matches in all_matches.iter() {
                        first_matching_pets.retain(|pet| matches.contains(pet))
                    }
                    matching_pets.extend(first_matching_pets.iter().cloned())
                }

                found_items.extend(matching_pets)
            }
        };
        Ok(found_items)
    }

    fn get_shop_items_by_pos(
        &self,
        pos: &Position,
        item: &Entity,
    ) -> Result<Vec<&ShopItem>, SAPTestError> {
        let mut found_items = vec![];

        match pos {
            Position::N(condition, number_items, randomize) => {
                let mut found_shop_items = self.get_shop_items_by_cond(condition, item)?;
                if *randomize {
                    let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));
                    found_shop_items.shuffle(&mut rng)
                }
                let mut found_shop_items = found_shop_items.into_iter();
                for _ in 0..*number_items {
                    if let Some(item) = found_shop_items.next() {
                        found_items.push(item)
                    }
                }
            }
            Position::Any(condition) => {
                let mut rng = self.get_rng();
                let found_found_items = self
                    .get_shop_items_by_cond(condition, item)?
                    .into_iter()
                    .choose(&mut rng);
                if let Some(any_item) = found_found_items {
                    found_items.push(any_item)
                }
            }
            Position::All(condition) => {
                let found_found_items = self.get_shop_items_by_cond(condition, item)?;
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
                let end_idx = range_idx
                    .clone()
                    .filter_map(|idx| TryInto::<usize>::try_into(idx).ok())
                    .max();
                let found_found_items = if let Entity::Food = item {
                    end_idx.map(|idx| self.foods.get(0..=idx))
                } else {
                    end_idx.map(|idx| self.pets.get(0..=idx))
                };
                if let Some(Some(found_found_items)) = found_found_items {
                    found_items.extend(found_found_items)
                }
            }
            Position::Relative(idx) => {
                let converted_idx = TryInto::<usize>::try_into(-idx).ok().ok_or(
                    SAPTestError::InvalidShopAction {
                        subject: "Shop Items by Position".to_string(),
                        reason: format!("Invalid relative index. {idx}"),
                    },
                )?;
                if let Some(found_item) = if let Entity::Food = item {
                    self.foods.get(converted_idx)
                } else {
                    self.pets.get(converted_idx)
                } {
                    found_items.push(found_item)
                }
            }
            Position::Multiple(positions) => {
                for pos_found_items in positions
                    .iter()
                    .flat_map(|pos| self.get_shop_items_by_pos(pos, item).ok())
                {
                    found_items.extend(pos_found_items.iter())
                }
            }
            Position::None => {}
            _ => {
                return Err(SAPTestError::InvalidShopAction {
                    subject: "Shop Items by Position".to_string(),
                    reason: format!("Position not implemented. {pos:?}"),
                })
            }
        };

        Ok(found_items)
    }
}

/// View attributes of a single [`ShopItem`].
pub trait ShopItemViewer: std::borrow::Borrow<ShopItem> {
    /// Get [`ShopItem`] name.
    fn name(&self) -> EntityName;
    /// Get [`ShopItem`] cost.
    fn cost(&self) -> usize;
    /// Check if [`ShopItem`] in [`Shop`] is frozen.
    fn is_frozen(&self) -> bool;
    /// Get health stat of [`ShopItem`].
    fn health_stat(&self) -> Option<isize>;
    /// Get attack stat of [`ShopItem`].
    fn attack_stat(&self) -> Option<isize>;
    /// Get tier of [`ShopItem`].
    fn tier(&self) -> usize;
    /// Get effect triggers of [`ShopItem`].
    fn triggers(&self) -> Vec<Status>;
    /// Get effect actions of [`ShopItem`]
    fn actions(&self) -> Vec<Action>;
}

impl ShopItemViewer for ShopItem {
    fn name(&self) -> EntityName {
        match &self.item {
            ItemSlot::Pet(pet) => EntityName::Pet(pet.read().unwrap().name.clone()),
            ItemSlot::Food(food) => EntityName::Food(food.read().unwrap().name.clone()),
        }
    }
    fn cost(&self) -> usize {
        self.cost
    }
    fn is_frozen(&self) -> bool {
        self.state == ItemState::Frozen
    }
    fn health_stat(&self) -> Option<isize> {
        match &self.item {
            ItemSlot::Pet(pet) => Some(pet.read().unwrap().stats.health),
            ItemSlot::Food(food) => match food.read().unwrap().ability.action {
                Action::Add(StatChangeType::StaticValue(stats)) => Some(stats.health),
                Action::Remove(StatChangeType::StaticValue(stats)) => Some(stats.health),
                _ => None,
            },
        }
    }
    fn attack_stat(&self) -> Option<isize> {
        match &self.item {
            ItemSlot::Pet(pet) => Some(pet.read().unwrap().stats.attack),
            ItemSlot::Food(food) => match food.read().unwrap().ability.action {
                Action::Add(StatChangeType::StaticValue(stats)) => Some(stats.attack),
                Action::Remove(StatChangeType::StaticValue(stats)) => Some(stats.attack),
                Action::Negate(stats) => Some(stats.attack),
                _ => None,
            },
        }
    }
    fn tier(&self) -> usize {
        match &self.item {
            ItemSlot::Pet(pet) => pet.read().unwrap().tier,
            ItemSlot::Food(food) => food.read().unwrap().tier,
        }
    }
    fn triggers(&self) -> Vec<Status> {
        match &self.item {
            ItemSlot::Pet(pet) => pet
                .read()
                .unwrap()
                .effect
                .iter()
                .map(|effect| effect.trigger.status.clone())
                .collect_vec(),
            ItemSlot::Food(food) => vec![food.read().unwrap().ability.trigger.status.clone()],
        }
    }
    fn actions(&self) -> Vec<Action> {
        match &self.item {
            ItemSlot::Pet(pet) => pet
                .read()
                .unwrap()
                .effect
                .iter()
                .map(|effect| effect.action.clone())
                .collect_vec(),
            ItemSlot::Food(food) => vec![food.read().unwrap().ability.action.clone()],
        }
    }
}
