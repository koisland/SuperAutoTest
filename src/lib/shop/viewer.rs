use itertools::Itertools;
use rand::{
    random,
    seq::{IteratorRandom, SliceRandom},
    SeedableRng,
};
use rand_chacha::ChaCha12Rng;

use super::store::{ItemSlot, ItemState, ShopItem};
use crate::{
    battle::{
        actions::{Action, StatChangeType},
        effect::EntityName,
        state::{Condition, EqualityCondition, Status},
    },
    error::SAPTestError,
    Entity, Position, Shop,
};

/// Enables viewing [`ShopItem`]s and their state.
pub trait ShopViewer {
    /// Get [`ShopItem`](crate::shop::store::ShopItem)s by [`Condition`](crate::Condition).
    /// # Example
    /// ```
    /// use saptest::{Shop, ShopViewer, ShopItemViewer, Entity, Condition, Position};
    ///
    /// let (cond, item_type) = (Condition::Healthiest, Entity::Pet);
    /// let shop = Shop::new(1, Some(42)).unwrap();
    ///
    /// // Find highest health pet by searching through all pets manually.
    /// let all_items = shop.get_shop_items_by_pos(&Position::All(Condition::None), &item_type).unwrap();
    /// let highest_health_pet = all_items.into_iter().max_by(|pet_1, pet_2| pet_1.health_stat().cmp(&pet_2.health_stat()));
    ///
    /// // Found directly using the condition.
    /// let found_items = shop.get_shop_items_by_cond(&cond, &item_type).unwrap();
    /// let first_pet = found_items.first().copied();
    /// assert_eq!(highest_health_pet, first_pet);
    /// ```
    fn get_shop_items_by_cond(
        &self,
        cond: &Condition,
        item_type: &Entity,
    ) -> Result<Vec<&ShopItem>, SAPTestError>;
    /// Get [`ShopItem`](crate::shop::store::ShopItem)s by [`Position`](crate::Position).
    /// # Example
    /// ```
    /// use saptest::{Shop, ShopViewer, Entity, Position, Condition};
    ///
    /// let (pos, item_type) = (Position::All(Condition::None), Entity::Pet);
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
        cond: &Condition,
        item_type: &Entity,
    ) -> Result<Vec<&ShopItem>, SAPTestError> {
        let mut found_items = Vec::new();
        let all_items = match item_type {
            Entity::Pet => self.pets.iter(),
            Entity::Food => self.foods.iter(),
        };

        match cond {
            Condition::None => found_items.extend(all_items),
            Condition::Healthiest => {
                if let Some(highest_tier_item) = all_items
                    .max_by(|item_1, item_2| item_1.health_stat().cmp(&item_2.health_stat()))
                {
                    found_items.push(highest_tier_item)
                }
            }
            Condition::Illest => {
                if let Some(lowest_tier_item) = all_items
                    .min_by(|item_1, item_2| item_1.health_stat().cmp(&item_2.health_stat()))
                {
                    found_items.push(lowest_tier_item)
                }
            }
            Condition::Strongest => {
                if let Some(highest_tier_item) = all_items
                    .max_by(|item_1, item_2| item_1.attack_stat().cmp(&item_2.attack_stat()))
                {
                    found_items.push(highest_tier_item)
                }
            }
            Condition::Weakest => {
                if let Some(lowest_tier_item) = all_items
                    .min_by(|item_1, item_2| item_1.attack_stat().cmp(&item_2.attack_stat()))
                {
                    found_items.push(lowest_tier_item)
                }
            }
            Condition::HighestTier => {
                if let Some(highest_tier_item) =
                    all_items.max_by(|item_1, item_2| item_1.tier().cmp(&item_2.tier()))
                {
                    found_items.push(highest_tier_item)
                }
            }
            Condition::LowestTier => {
                if let Some(lowest_tier_item) =
                    all_items.min_by(|item_1, item_2| item_1.tier().cmp(&item_2.tier()))
                {
                    found_items.push(lowest_tier_item)
                }
            }
            Condition::Equal(eq_cond) => match eq_cond {
                EqualityCondition::Tier(tier) => {
                    found_items.extend(all_items.filter(|item| item.tier() == *tier))
                }
                EqualityCondition::Name(name) => {
                    found_items.extend(all_items.filter(|item| &item.name() == name))
                }
                EqualityCondition::Trigger(trigger) => found_items.extend(
                    all_items
                        .filter_map(|item| {
                            let item_triggers = item.triggers();
                            (item_triggers.contains(trigger)).then_some(item)
                        })
                        .into_iter(),
                ),
                _ => {
                    return Err(SAPTestError::InvalidShopAction {
                        subject: "Invalid Equality Condition".to_string(),
                        reason: format!("Cannot use {eq_cond:?} to search for items."),
                    })
                }
            },
            Condition::NotEqual(eq_cond) => {
                let eq_items =
                    self.get_shop_items_by_cond(&Condition::Equal(eq_cond.clone()), item_type)?;
                found_items.extend(all_items.filter(|item| !eq_items.contains(item)))
            }
            _ => {
                return Err(SAPTestError::InvalidShopAction {
                    subject: "Shop Items by Condition".to_string(),
                    reason: format!("Condition not implemented. {cond:?}"),
                })
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
}

impl<I: std::borrow::Borrow<ShopItem>> ShopItemViewer for I {
    fn name(&self) -> EntityName {
        match &self.borrow().item {
            ItemSlot::Pet(pet) => EntityName::Pet(pet.borrow().name.clone()),
            ItemSlot::Food(food) => EntityName::Food(food.borrow().name.clone()),
        }
    }
    fn is_frozen(&self) -> bool {
        self.borrow().state == ItemState::Frozen
    }
    /// Get health stat of item.
    fn health_stat(&self) -> Option<isize> {
        match &self.borrow().item {
            ItemSlot::Pet(pet) => Some(pet.borrow().stats.health),
            ItemSlot::Food(food) => match food.borrow().ability.action {
                Action::Add(StatChangeType::StaticValue(stats)) => Some(stats.health),
                Action::Remove(StatChangeType::StaticValue(stats)) => Some(stats.health),
                _ => None,
            },
        }
    }
    /// Get attack stat of item.
    fn attack_stat(&self) -> Option<isize> {
        match &self.borrow().item {
            ItemSlot::Pet(pet) => Some(pet.borrow().stats.attack),
            ItemSlot::Food(food) => match food.borrow().ability.action {
                Action::Add(StatChangeType::StaticValue(stats)) => Some(stats.attack),
                Action::Remove(StatChangeType::StaticValue(stats)) => Some(stats.attack),
                _ => None,
            },
        }
    }
    /// Get tier of item.
    fn tier(&self) -> usize {
        match &self.borrow().item {
            ItemSlot::Pet(pet) => pet.borrow().tier,
            ItemSlot::Food(food) => food.borrow().tier,
        }
    }
    /// Get effect triggers of item.
    fn triggers(&self) -> Vec<Status> {
        match &self.borrow().item {
            ItemSlot::Pet(pet) => pet
                .borrow()
                .effect
                .iter()
                .map(|effect| effect.trigger.status.clone())
                .collect_vec(),
            ItemSlot::Food(food) => vec![food.borrow().ability.trigger.status.clone()],
        }
    }
}
