use std::fmt::Display;

use itertools::Itertools;
use rand::{
    seq::{IteratorRandom, SliceRandom},
    SeedableRng,
};
use rand_chacha::ChaCha12Rng;

use crate::{
    battle::{
        effect::Entity,
        state::{Condition, Target},
        team_effect_apply::EffectApplyHelpers,
    },
    error::SAPTestError,
    Position, Team, TeamEffects,
};

use super::{
    store::{ItemSlot, ItemState, ShopItem},
    trigger::TRIGGER_FOOD_BOUGHT,
};

trait Shopping {
    fn buy(
        &mut self,
        from: Position,
        to: Position,
        item_type: Entity,
    ) -> Result<&mut Self, SAPTestError>;
    fn sell(&mut self, pos: Position) -> Result<&mut Self, SAPTestError>;
    fn roll(&mut self) -> Result<&mut Self, SAPTestError>;
    fn set_shop_seed(&mut self, seed: Option<u64>) -> &mut Self;
    fn freeze(&mut self, pos: Position) -> Result<&mut Self, SAPTestError>;
    fn open_shop(&mut self) -> Result<&mut Self, SAPTestError>;
    fn close_shop(&mut self) -> Result<&mut Self, SAPTestError>;
    fn get_shop_items_by_pos(&mut self, pos: Position, item: Entity) -> Vec<&mut ShopItem>;
    fn get_shop_items_by_cond(&mut self, cond: Condition, item_type: Entity) -> Vec<&mut ShopItem>;
}

impl Shopping for Team {
    fn buy(
        &mut self,
        from: Position,
        to: Position,
        item_type: Entity,
    ) -> Result<&mut Self, SAPTestError> {
        let mut curr_coins = self.shop.coins;
        let selected_items = self.get_shop_items_by_pos(from, item_type);
        let mut purchased_items = vec![];

        // Buy the item. Set to sold state. Clone item.
        for item in selected_items.into_iter() {
            if let Some(new_coins) = curr_coins.checked_sub(item.cost) {
                curr_coins = new_coins;
                item.state = ItemState::Sold;
                purchased_items.push(item.clone());
            } else {
                // Not enough to buy so re-add to shop.
                return Err(SAPTestError::ShopError {
                    reason: format!("Not enough coins ({}) to buy pet.", self.shop.coins),
                });
            }
        }

        let affected_pets = self.get_pets_by_pos(self.first(), Target::Friend, to, None, None)?;
        for item in purchased_items.into_iter() {
            if let ItemSlot::Food(food) = &item.item {
                // Give food to a single pet.
                if food.holdable {
                    let (_, target_pet) =
                        affected_pets
                            .first()
                            .ok_or(SAPTestError::InvalidTeamAction {
                                subject: "No Item Target".to_string(),
                                reason: "Holdable item must have a target".to_string(),
                            })?;
                    let mut new_food = food.clone();
                    new_food.ability.assign_owner(Some(target_pet));
                    target_pet.borrow_mut().item = Some(new_food);

                    // Create trigger if food bought.
                    let mut trigger = TRIGGER_FOOD_BOUGHT;
                    trigger.set_affected(target_pet);
                    self.triggers.push_back(trigger)
                } else {
                    for i in 0..food.n_targets {
                        if let Some((_, target_pet)) = affected_pets.first() {
                            // self.apply_single_effect(target_pet.clone(), &food.ability);
                        } else {
                            break;
                        }
                    }
                }
            } else if let ItemSlot::Pet(pet) = &item.item {
            }
        }
        Ok(self)
    }

    fn sell(&mut self, pos: Position) -> Result<&mut Self, SAPTestError> {
        todo!()
    }

    fn roll(&mut self) -> Result<&mut Self, SAPTestError> {
        todo!()
    }

    fn set_shop_seed(&mut self, seed: Option<u64>) -> &mut Self {
        self.shop.set_seed(seed);
        self
    }

    fn freeze(&mut self, pos: Position) -> Result<&mut Self, SAPTestError> {
        todo!()
    }

    fn open_shop(&mut self) -> Result<&mut Self, SAPTestError> {
        self.shop.setup()?;
        Ok(self)
    }

    fn close_shop(&mut self) -> Result<&mut Self, SAPTestError> {
        todo!()
    }

    fn get_shop_items_by_cond(&mut self, cond: Condition, item_type: Entity) -> Vec<&mut ShopItem> {
        let mut found_foods = Vec::with_capacity(self.shop.foods.len());
        let all_items = match item_type {
            Entity::Pet => self.shop.pets.iter_mut(),
            Entity::Food => self.shop.foods.iter_mut(),
        };

        match cond {
            Condition::None => found_foods.extend(all_items),
            Condition::Healthiest => {
                if let Some(highest_tier_food) =
                    all_items.max_by(|item_1, item_2| item_1.health().cmp(&item_2.health()))
                {
                    found_foods.push(highest_tier_food)
                }
            }
            Condition::Illest => {
                if let Some(lowest_tier_food) =
                    all_items.min_by(|food_1, food_2| food_1.health().cmp(&food_2.health()))
                {
                    found_foods.push(lowest_tier_food)
                }
            }
            Condition::Strongest => {
                if let Some(highest_tier_food) =
                    all_items.max_by(|item_1, item_2| item_1.attack().cmp(&item_2.attack()))
                {
                    found_foods.push(highest_tier_food)
                }
            }
            Condition::Weakest => {
                if let Some(lowest_tier_food) =
                    all_items.min_by(|food_1, food_2| food_1.attack().cmp(&food_2.attack()))
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

    fn get_shop_items_by_pos(&mut self, pos: Position, item: Entity) -> Vec<&mut ShopItem> {
        let mut found_items = Vec::with_capacity(self.shop.pets.len());

        match pos {
            Position::Any(condition) => {
                let mut rng = ChaCha12Rng::seed_from_u64(self.seed);
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
                    self.shop.foods.first_mut()
                } else {
                    self.shop.pets.first_mut()
                };

                if let Some(item) = item {
                    found_items.push(item)
                };
            }
            Position::Last => {
                let item = if let Entity::Food = item {
                    self.shop.foods.last_mut()
                } else {
                    self.shop.pets.last_mut()
                };

                if let Some(item) = item {
                    found_items.push(item)
                };
            }
            Position::Range(range_idx) => {
                let end_idx = range_idx
                    .into_iter()
                    .filter_map(|idx| TryInto::<usize>::try_into(idx).ok())
                    .max();
                let found_found_items = if let Entity::Food = item {
                    end_idx.map(|idx| self.shop.foods.get_mut(0..idx))
                } else {
                    end_idx.map(|idx| self.shop.pets.get_mut(0..idx))
                };
                if let Some(Some(found_found_items)) = found_found_items {
                    found_items.extend(found_found_items)
                }
            }
            Position::Relative(idx) => {
                if let Ok(Some(item)) = idx
                    .try_into()
                    .map(|idx: usize| self.shop.foods.get_mut(idx))
                {
                    found_items.push(item)
                }
            }
            _ => unimplemented!(),
        };

        found_items
    }
}

#[cfg(test)]
mod tests {
    use super::Shopping;
    use crate::{
        battle::{effect::Entity, state::Condition},
        Position, Team,
    };

    #[test]
    fn test_team_shop() {
        let mut team = Team::default();

        println!("{}", team.shop);
        team.set_shop_seed(Some(12)).open_shop().unwrap();
        println!("{}", team.shop);

        let items = team.get_shop_items_by_pos(Position::All(Condition::None), Entity::Pet);
        for item in items {
            println!("{item}")
        }
    }
}
