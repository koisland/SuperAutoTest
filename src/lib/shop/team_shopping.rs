use std::{cell::RefCell, rc::Rc};

use itertools::Itertools;
use log::info;

use crate::{
    effects::{
        effect::Entity,
        state::{Status, Target},
        trigger::*,
    },
    error::SAPTestError,
    shop::{
        store::{ItemSlot, ShopState, DEFAULT_COIN_COUNT},
        trigger::*,
        viewer::ShopViewer,
    },
    teams::{
        combat::TeamCombat,
        effects::{EffectApplyHelpers, TeamEffects},
        viewer::TeamViewer,
    },
    Food, Pet, Position, Shop, Team,
};

use super::store::{MAX_SHOP_TIER, MIN_SHOP_TIER};

trait TeamShoppingHelpers {
    fn buy_food_behavior(
        &mut self,
        food: Rc<RefCell<Food>>,
        to_pos: &Position,
        empty_team: &mut Team,
    ) -> Result<(), SAPTestError>;
    fn buy_pet_behavior(
        &mut self,
        pet: Rc<RefCell<Pet>>,
        to_pos: &Position,
        empty_team: &mut Team,
    ) -> Result<(), SAPTestError>;
}

/// Implements Super Auto Pets [`Shop`](crate::Shop) behavior.
/// ```rust no run
/// use saptest::TeamShopping;
/// ```
pub trait TeamShopping {
    /// Buy a [`ShopItem`](crate::shop::store::ShopItem) from the [`Shop`](crate::Shop) and place it on the [`Team`](crate::Team).
    /// # Examples
    /// ---
    /// Buying a pet.
    /// ```
    /// use saptest::{Team, TeamShopping, Position, Entity};
    ///
    /// let mut team = Team::default();
    /// team.set_shop_seed(Some(42))
    ///     .open_shop().unwrap();
    /// // Buy the 1st pet in the shop and put it on/in front of the 1st pet slot on the team.
    /// let first_pet_purchase = team.buy(
    ///     &Position::First,
    ///     &Entity::Pet,
    ///     &Position::First
    /// );
    /// assert!(first_pet_purchase.is_ok());
    /// ```
    /// ---
    /// Buying a random food.
    /// ```
    /// use saptest::{Pet, PetName, Team, TeamShopping, Position, Entity, Condition};
    ///
    /// let mut team = Team::new(
    ///     &[Pet::try_from(PetName::Ant).unwrap()],
    ///     5
    /// ).unwrap();
    /// team.set_shop_seed(Some(42))
    ///     .open_shop().unwrap();
    /// // Buy a random food in the shop and put it on/in front of the 1st pet slot on the team.
    /// let first_random_item_purchase = team.buy(
    ///     &Position::Any(Condition::None),
    ///     &Entity::Food,
    ///     &Position::First
    /// );
    /// assert!(first_random_item_purchase.is_ok())
    /// ```
    fn buy(
        &mut self,
        from: &Position,
        item_type: &Entity,
        to: &Position,
    ) -> Result<&mut Self, SAPTestError>;
    /// Sell a [`Pet`](crate::Pet) on the [`Team`](crate::Team) for gold.
    /// # Example
    /// ```
    /// use saptest::{Team, TeamViewer, TeamShopping, Pet, PetName, Position};
    ///
    /// let mut team = Team::new(
    ///     &[Pet::try_from(PetName::Ant).unwrap()],
    ///     5
    /// ).unwrap();
    ///
    /// // Open shop with 10 coins. Sell and now at 11.
    /// team.open_shop().unwrap();
    /// assert_eq!(team.gold(), 10);
    /// assert!(
    ///     team.sell(&Position::First).is_ok() &&
    ///     team.gold() == 11
    /// );
    /// // Team is empty.
    /// assert!(team.all().is_empty());
    /// ```
    fn sell(&mut self, pos: &Position) -> Result<&mut Self, SAPTestError>;
    /// Roll the [`Shop`](crate::Shop) restocking it with new items at the cost of `1` gold.
    /// # Example
    /// ```
    /// use saptest::{Team, TeamShopping};
    /// let mut team = Team::default();
    ///
    /// // Open shop.
    /// team.open_shop().unwrap();
    ///
    /// assert_eq!(team.gold(), 10);
    /// assert!(
    ///     team.roll_shop().is_ok() &&
    ///     team.gold() == 9
    /// );
    /// ```
    fn roll_shop(&mut self) -> Result<&mut Self, SAPTestError>;
    /// Set the [`Shop`](crate::Shop)'s seed.
    /// * Setting the seed to [`None`] will randomize the rng.
    /// # Example
    /// ``` rust no run
    /// use saptest::{Team, TeamShopping};
    ///
    /// let mut team = Team::default();
    /// team.set_shop_seed(Some(42));
    /// ```
    fn set_shop_seed(&mut self, seed: Option<u64>) -> &mut Self;
    /// Set the [`Shop`](crate::Shop)'s tier.
    /// * Note: This adjusts the number of turns in the team's history to the minimum required to reach the given tier.
    /// # Example
    /// ```
    /// use saptest::{Team, TeamShopping};
    ///
    /// let mut team = Team::default();
    /// assert_eq!(team.shop_tier(), 1);
    ///
    /// let adj_to_tier_3 = team.set_shop_tier(3);
    /// assert!(adj_to_tier_3.is_ok());
    ///
    /// let adj_to_invalid_tier = team.set_shop_tier(12);
    /// assert!(adj_to_invalid_tier.is_err());
    /// ```
    fn set_shop_tier(&mut self, tier: usize) -> Result<&mut Self, SAPTestError>;
    /// Freeze an item in the [`Shop`](crate::Shop).
    /// # Example
    /// ```
    /// use saptest::{Team, TeamShopping, Position, Entity};
    /// let mut team = Team::default();
    /// team.open_shop().unwrap();
    /// assert!(
    ///     team.freeze_shop(Position::First, Entity::Pet).is_ok()
    /// );
    /// ```
    fn freeze_shop(&mut self, pos: Position, item_type: Entity) -> Result<&mut Self, SAPTestError>;
    /// Open the [`Shop`](crate::Shop) for a [`Team`](crate::Team).
    /// # Example
    /// ```
    /// use saptest::{Team, TeamShopping};
    ///
    /// let mut team = Team::default();
    /// assert!(team.open_shop().is_ok());
    /// ```
    fn open_shop(&mut self) -> Result<&mut Self, SAPTestError>;
    /// Close a [`Shop`](crate::Shop) for a [`Team`](crate::Team).
    /// * Enables [`Team`](crate::Team) fighting.
    /// # Example
    /// ```
    /// use saptest::{Team, TeamShopping};
    ///
    /// let mut team = Team::default();
    /// team.open_shop().unwrap();
    /// assert!(team.close_shop().is_ok());
    /// ```
    fn close_shop(&mut self) -> Result<&mut Self, SAPTestError>;
    /// Get [`Shop`](crate::Shop) gold available.
    /// # Example
    /// ```
    /// use saptest::{Team, TeamShopping};
    ///
    /// let mut team = Team::default();
    /// assert_eq!(team.gold(), 10);
    /// ```
    fn gold(&self) -> usize;
    /// Get the [`Shop`](crate::Shop) tier.
    /// ```
    /// use saptest::{Team, TeamShopping};
    ///
    /// let mut team = Team::default();
    /// assert_eq!(team.shop_tier(), 1);
    /// ```
    fn shop_tier(&self) -> usize;
    /// Get number of foods in the [`Shop`](crate::Shop).
    /// # Example
    /// ```
    /// use saptest::{Team, TeamShopping};
    ///
    /// // Defaults to tier 1 shop.
    /// let mut team = Team::default();
    /// team.open_shop().unwrap();
    /// assert_eq!(team.len_shop_foods(), 1);
    /// ```
    fn len_shop_foods(&self) -> usize;
    /// Get number of pets in the [`Shop`](crate::Shop).
    /// # Example
    /// ```
    /// use saptest::{Team, TeamShopping};
    ///
    /// // Defaults to tier 1 shop.
    /// let mut team = Team::default();
    /// team.open_shop().unwrap();
    /// assert_eq!(team.len_shop_pets(), 3);
    /// ```
    fn len_shop_pets(&self) -> usize;
    /// Replace [`Shop`](crate::Shop) of [`Team`](crate::Team).
    /// # Example
    /// ```
    /// use saptest::{
    ///     Shop, ShopItem, ShopViewer,
    ///     TeamShopping, Team,
    ///     EntityName, Food, FoodName,
    /// };
    ///
    /// // Create default team and empty shop.
    /// let mut team = Team::default();
    /// assert!(
    ///     team.len_shop_pets() == 0 &&
    ///     team.len_shop_foods() == 0 &&
    ///     team.shop_tier() == 1
    /// );
    ///
    /// // Create a custom shop.
    /// let mut custom_shop = Shop::new(5, Some(12)).unwrap();
    /// let coconut = ShopItem::new(Food::try_from(FoodName::Coconut).unwrap());
    /// custom_shop.add_item(coconut).unwrap();
    /// assert!(
    ///     custom_shop.pets.len() == 4 &&
    ///     custom_shop.foods.len() == 3 &&
    ///     custom_shop.tier() == 5
    /// );
    ///
    /// // Replace shop.
    /// assert!(team.replace_shop(custom_shop).is_ok());
    /// assert!(
    ///     team.len_shop_pets() == 4 &&
    ///     team.len_shop_foods() == 3 &&
    ///     team.shop_tier() == 5
    /// );
    /// ```
    fn replace_shop(&mut self, shop: Shop) -> Result<&mut Self, SAPTestError>;
    /// Prints the team's [`Shop`].
    /// # Example
    /// ```
    /// use saptest::{Team, TeamShopping};
    ///
    /// let mut team = Team::default();
    /// team.set_shop_seed(Some(1212)).open_shop().unwrap();
    /// team.print_shop();
    /// ```
    /// ---
    /// ```no run
    /// (Pets)
    /// (Normal) [Mosquito: (2,2) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    /// (Normal) [Beaver: (3,2) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    /// (Normal) [Horse: (2,1) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    ///
    /// (Foods)
    /// (Normal) [Apple: [Effect (Uses: None): (Food) - Trigger: [Status: None, Position: None, Affected: None, From: None] - Action: Add(StaticValue(Statistics { attack: 1, health: 1 })) on Friend (OnSelf) ]]
    /// ```
    fn print_shop(&self);
}

/// Helper methods for buy/sell behavior.
impl TeamShoppingHelpers for Team {
    fn buy_food_behavior(
        &mut self,
        food: Rc<RefCell<Food>>,
        to_pos: &Position,
        empty_team: &mut Team,
    ) -> Result<(), SAPTestError> {
        let trigger_any_food = TRIGGER_ANY_FOOD_BOUGHT;
        self.triggers.push_back(trigger_any_food);

        // Give food to a single pet.
        if food.borrow().holdable {
            let affected_pets =
                self.get_pets_by_pos(self.first(), &Target::Friend, to_pos, None, None)?;
            let (_, target_pet) = affected_pets
                .first()
                .ok_or(SAPTestError::InvalidTeamAction {
                    subject: "No Item Target".to_string(),
                    reason: "Holdable item must have a target".to_string(),
                })?;
            food.borrow_mut().ability.assign_owner(Some(target_pet));
            target_pet.borrow_mut().item = Some(food.borrow().clone());

            // Create trigger if food eaten.
            let mut trigger_self_food = TRIGGER_SELF_FOOD_EATEN;
            trigger_self_food.set_affected(target_pet);
            self.triggers.push_back(trigger_self_food);
        } else {
            let mut food_ability = food.borrow().ability.clone();
            let target_pos =
                Position::Multiple(vec![food_ability.position.clone(); food.borrow().n_targets]);
            let affected_pets =
                self.get_pets_by_pos(self.first(), &Target::Friend, &target_pos, None, None)?;
            // For each pet found by the effect of food bought, apply its effect.
            for (_, pet) in affected_pets {
                food_ability.assign_owner(Some(&pet));

                // Pet triggers for eating food.
                let mut trigger_self_food = TRIGGER_SELF_FOOD_EATEN;
                trigger_self_food.set_affected(&pet);

                self.apply_single_effect(pet, &food_ability, empty_team)?;
            }
        }
        Ok(())
    }

    fn buy_pet_behavior(
        &mut self,
        pet: Rc<RefCell<Pet>>,
        to_pos: &Position,
        empty_team: &mut Team,
    ) -> Result<(), SAPTestError> {
        let affected_pets =
            self.get_pets_by_pos(self.first(), &Target::Friend, to_pos, None, None)?;

        let purchased_pet = if let Some((_, affected_pet)) = affected_pets.first() {
            // If affected pet same as purchased pet.
            if affected_pet.borrow().name == pet.borrow().name {
                // Get previous level. This will increase for every levelup.
                let mut prev_lvl = affected_pet.borrow().lvl;

                // Stack pets.
                affected_pet.borrow_mut().merge(&pet.borrow())?;

                // Check if pet leveled up. For EACH levelup:
                // * Activate pet effects if trigger is a levelup.
                //      * Must be done at previous level otherwise will use effect at current level.
                //      * Ex. Fish levelup must use lvl. 1 effect not its current effect at lvl. 2.
                // * Add shop pet on level
                // * Add team levelup triggers.
                for _ in 0..(affected_pet.borrow().lvl - prev_lvl) {
                    let mut levelup_trigger = TRIGGER_SELF_LEVELUP;
                    levelup_trigger.set_affected(affected_pet);

                    // For pet effect of leveled up pet.
                    for mut effect in affected_pet.borrow().get_effect(prev_lvl)? {
                        effect.assign_owner(Some(affected_pet));
                        if effect.trigger.status == Status::Levelup {
                            // Apply pet effect directly here if trigger is levelup.
                            self.apply_effect(&levelup_trigger, &effect, empty_team)?;
                        }
                    }
                    // Increment level.
                    prev_lvl += 1;

                    // If pet levels, add a pet (tier above current tier) to shop.
                    if self.shop.add_levelup_pet().is_err() {
                        info!(target: "run", "Maximum pet capacity reached. No levelup pet added.")
                    };

                    // Add triggers for effects that trigger on any levelup.
                    let mut levelup_any_trigger = TRIGGER_ANY_LEVELUP;
                    levelup_any_trigger.set_affected(affected_pet);
                    self.triggers.push_back(levelup_any_trigger);
                }
                Some(affected_pet.clone())
            } else {
                // Otherwise, add pet to position.
                let adj_idx = if let Position::Last = to_pos { 1 } else { 0 };
                let pos = affected_pet.borrow().pos.unwrap_or(0) + adj_idx;
                self.add_pet(pet.borrow().clone(), pos, None)?;
                self.nth(pos)
            }
        } else {
            // No pets at all, check if specific position.
            self.add_pet(pet.borrow().clone(), 0, None)?;
            self.nth(0)
        };

        if let Some(pet) = purchased_pet {
            let mut buy_trigger = TRIGGER_SELF_PET_BOUGHT;
            buy_trigger.set_affected(&pet);
            self.triggers.push_back(buy_trigger)
        }

        Ok(())
    }
}

impl TeamShopping for Team {
    fn gold(&self) -> usize {
        self.shop.coins
    }

    fn shop_tier(&self) -> usize {
        self.shop.tier()
    }

    fn buy(
        &mut self,
        from: &Position,
        item_type: &Entity,
        to: &Position,
    ) -> Result<&mut Self, SAPTestError> {
        if self.shop.state == ShopState::Closed {
            return Err(SAPTestError::InvalidShopAction {
                subject: "Closed Shop (Buy)".to_string(),
                reason: "Cannot perform action on closed shop.".to_string(),
            });
        }

        let selected_items = self
            .shop
            .get_shop_items_by_pos(from, item_type)?
            .into_iter()
            .cloned()
            .collect_vec();

        if selected_items.is_empty() {
            return Err(SAPTestError::InvalidShopAction {
                subject: "No Items Selected".to_string(),
                reason: format!("No {item_type:?} items selected with {from:?} position."),
            });
        }

        // Check for sufficient funds.
        let total_cost = selected_items
            .iter()
            .fold(0, |total_cost, item| total_cost + item.cost);
        if total_cost > self.shop.coins {
            return Err(SAPTestError::InvalidShopAction {
                subject: format!("Insufficient Coins (Buy {item_type:?})"),
                reason: format!(
                    "Insufficient coins to purchase items {total_cost} > {}",
                    self.shop.coins
                ),
            });
        }

        // TODO: Not great. Need to find way to make Team effect apply take optional opponent.
        // Creates new empty team each time an item is bought.
        let mut empty_team = Team::default();

        // Remove sold items.
        match item_type {
            Entity::Pet => self.shop.pets.retain(|pet| !selected_items.contains(pet)),
            Entity::Food => self
                .shop
                .foods
                .retain(|food| !selected_items.contains(food)),
        }

        // Buy the item and check if sufficient funds.
        for item in selected_items.into_iter() {
            // Decrement coins.
            self.shop.coins -= item.cost;

            match item.item {
                ItemSlot::Pet(pet) => self.buy_pet_behavior(pet, to, &mut empty_team)?,
                ItemSlot::Food(food) => self.buy_food_behavior(food, to, &mut empty_team)?,
            };
        }

        self.trigger_effects(&mut empty_team)?;
        self.clear_team();
        Ok(self)
    }

    fn sell(&mut self, pos: &Position) -> Result<&mut Self, SAPTestError> {
        if self.shop.state == ShopState::Closed {
            return Err(SAPTestError::InvalidShopAction {
                subject: "Closed Shop (Sell)".to_string(),
                reason: "Cannot perform action on closed shop.".to_string(),
            });
        }

        let affected_pets = self.get_pets_by_pos(self.first(), &Target::Friend, pos, None, None)?;

        if !affected_pets.is_empty() {
            for (_, pet) in affected_pets {
                // Add coins for sold pet.
                self.shop.coins += pet.borrow().lvl;

                let mut sell_trigger = TRIGGER_SELF_PET_SOLD;
                let mut sell_any_trigger = TRIGGER_ANY_PET_SOLD;
                sell_trigger.set_affected(&pet);
                sell_any_trigger.set_affected(&pet);

                self.triggers.extend([sell_trigger, sell_any_trigger]);
            }
        } else {
            return Err(SAPTestError::InvalidShopAction {
                subject: "No Sell-able Pet".to_string(),
                reason: format!("No pet to sell at position: {pos:?}."),
            });
        }

        // Trigger effects here.
        let mut empty_team = Team::default();
        self.trigger_effects(&mut empty_team)?;
        self.clear_team();

        Ok(self)
    }

    fn roll_shop(&mut self) -> Result<&mut Self, SAPTestError> {
        if self.shop.state == ShopState::Closed {
            return Err(SAPTestError::InvalidShopAction {
                subject: "Closed Shop (Roll)".to_string(),
                reason: "Cannot perform action on closed shop.".to_string(),
            });
        }

        self.shop.roll()?;
        self.triggers.push_back(TRIGGER_ROLL);
        Ok(self)
    }

    fn len_shop_foods(&self) -> usize {
        self.shop.len_foods()
    }

    fn len_shop_pets(&self) -> usize {
        self.shop.len_pets()
    }

    fn set_shop_tier(&mut self, tier: usize) -> Result<&mut Self, SAPTestError> {
        // Calculate tier change.
        let tier_diff =
            TryInto::<isize>::try_into(tier)? - TryInto::<isize>::try_into(self.shop_tier())?;

        // If increasing in tier, for each tier crate shop tier upgrade trigger.
        if tier_diff.is_positive() {
            for _ in 0..tier_diff {
                self.triggers.push_back(TRIGGER_SHOP_TIER_UPGRADED)
            }
        }
        self.shop.set_tier(tier)?;
        // Adjust history of team so curr turn reflects tier.
        let min_turn_to_tier = Shop::tier_to_num_turns(tier)?;

        self.history.curr_turn = min_turn_to_tier;
        // Update trigger effects.
        let mut empty_team = Team::default();
        self.trigger_effects(&mut empty_team)?;

        Ok(self)
    }

    fn set_shop_seed(&mut self, seed: Option<u64>) -> &mut Self {
        self.shop.seed = seed;
        self
    }

    fn freeze_shop(&mut self, pos: Position, item_type: Entity) -> Result<&mut Self, SAPTestError> {
        if self.shop.state == ShopState::Closed {
            return Err(SAPTestError::InvalidShopAction {
                subject: "Closed Shop (Freeze)".to_string(),
                reason: "Cannot perform action on closed shop.".to_string(),
            });
        }

        self.shop.freeze(&pos, &item_type)?;
        Ok(self)
    }

    fn open_shop(&mut self) -> Result<&mut Self, SAPTestError> {
        if self.shop.state == ShopState::Open {
            return Err(SAPTestError::InvalidShopAction {
                subject: "Opened Shop (Open)".to_string(),
                reason: "Cannot open an open shop.".to_string(),
            });
        }

        let mut empty_team = Team::default();

        self.shop.state = ShopState::Open;
        /*
        (1 / 2 = 0) + (1 % 2 = 0) = 1
        (2 / 2 = 1) + (2 % 2 = 0) = 1
        (3 / 2 = 1) + (3 % 2 = 1) = 2
        (4 / 2 = 2) + (4 % 2 = 0) = 2
        (5 / 2 = 2) + (5 % 2 = 1) = 3
        */
        let calc_tier = ((self.history.curr_turn / 2) + (self.history.curr_turn % 2))
            .clamp(MIN_SHOP_TIER, MAX_SHOP_TIER);

        // Shop tier upgraded.
        if self.shop.tier() + 1 == calc_tier {
            self.triggers.push_back(TRIGGER_SHOP_TIER_UPGRADED)
        }
        self.shop.set_tier(calc_tier)?;
        // Restore team to previous state.
        self.restore();

        // Remove temporary stats.
        for (pet_id, stats) in self.shop.temp_stats.iter() {
            if let Some(pet) = self
                .friends
                .iter()
                .find(|pet| pet.borrow().id.as_ref() == Some(pet_id))
            {
                pet.borrow_mut().stats -= *stats
            }
        }
        // Trigger start of turn.
        self.triggers.push_front(TRIGGER_START_TURN);
        self.shop.restock()?;
        self.trigger_effects(&mut empty_team)?;

        Ok(self)
    }

    fn close_shop(&mut self) -> Result<&mut Self, SAPTestError> {
        if self.shop.state == ShopState::Closed {
            return Err(SAPTestError::InvalidShopAction {
                subject: "Closed Shop (Close)".to_string(),
                reason: "Cannot close a closed shop.".to_string(),
            });
        }

        let mut empty_team = Team::default();

        // Trigger end of turn.
        self.triggers.push_front(TRIGGER_END_TURN);
        self.trigger_effects(&mut empty_team)?;
        self.clear_team();

        // Store friends.
        self.stored_friends = self
            .all()
            .into_iter()
            .map(|pet| pet.borrow().clone())
            .collect_vec();

        // Reset coins.
        self.shop.coins = DEFAULT_COIN_COUNT;
        self.shop.state = ShopState::Closed;

        Ok(self)
    }
    fn replace_shop(&mut self, shop: Shop) -> Result<&mut Self, SAPTestError> {
        // If shop has invalid tier, return err.
        let adj_turn = Shop::tier_to_num_turns(shop.tier())?;
        // Adjust turns to reflect tier.
        self.history.curr_turn = adj_turn;
        self.shop = shop;
        Ok(self)
    }

    fn print_shop(&self) {
        println!("{}", self.shop)
    }
}
