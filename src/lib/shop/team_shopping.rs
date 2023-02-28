use std::{cell::RefCell, rc::Rc};

use itertools::Itertools;
use log::info;
use rand::{random, seq::IteratorRandom, SeedableRng};
use rand_chacha::ChaCha12Rng;

use crate::{
    db::pack::Pack,
    effects::{
        actions::Action,
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
    Food, FoodName, ItemCondition, Pet, PetName, Position, Shop, Team,
};

use super::store::{MAX_SHOP_TIER, MIN_SHOP_TIER};

pub(crate) trait TeamShoppingHelpers {
    fn merge_behavior(
        &mut self,
        from_pet: &Rc<RefCell<Pet>>,
        to_pet: &Rc<RefCell<Pet>>,
    ) -> Result<(), SAPTestError>;
    fn buy_food_behavior(
        &mut self,
        food: Rc<RefCell<Food>>,
        curr_pet: Option<Rc<RefCell<Pet>>>,
        to_pos: &Position,
        emit_buy_triggers: bool,
    ) -> Result<(), SAPTestError>;
    fn buy_pet_behavior(
        &mut self,
        pet: Rc<RefCell<Pet>>,
        curr_pet: Option<Rc<RefCell<Pet>>>,
        to_pos: &Position,
    ) -> Result<(), SAPTestError>;
}

/// Implements Super Auto Pets [`Shop`](crate::Shop) behavior.
/// ```rust no_run
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
    /// use saptest::{Pet, PetName, Team, TeamShopping, Position, Entity, ItemCondition};
    ///
    /// let mut team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Ant).unwrap())],
    ///     5
    /// ).unwrap();
    /// team.set_shop_seed(Some(42))
    ///     .open_shop().unwrap();
    /// // Buy a random food in the shop and put it on/in front of the 1st pet slot on the team.
    /// let first_random_item_purchase = team.buy(
    ///     &Position::Any(ItemCondition::None),
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
    ///     &[Some(Pet::try_from(PetName::Ant).unwrap())],
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
    /// ``` rust no_run
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

    /// Set the [`Shop`](crate::Shop) to only include [`PetName`](crate::PetName)s and [`FoodName`](crate::FoodName)s from these [`Pack`](crate::db::pack::Pack)s.
    /// # Example
    /// ```
    /// use saptest::{Team, TeamShopping, db::pack::Pack};
    ///
    /// let mut team = Team::default();
    /// team.set_shop_packs(&[Pack::Puppy]);
    /// let packs = team.get_shop_packs();
    ///
    /// assert_eq!(packs.len(), 1);
    /// assert_eq!(packs[0], Pack::Puppy);
    /// ```
    fn set_shop_packs(&mut self, packs: &[Pack]) -> &mut Self;

    /// Returns an immutable reference to the [`Shop`].
    /// # Example
    /// ```rust no_run
    /// use saptest::{Team, TeamShopping};
    ///
    /// let mut team = Team::default();
    /// team.get_shop();
    /// ```
    fn get_shop(&self) -> &Shop;

    /// Returns the [`Shop`]'s [`Pack`]s.
    /// # Example
    /// ```
    /// use saptest::{Team, TeamShopping, db::pack::Pack};
    ///
    /// // The Turtle pack is the default.
    /// let mut team = Team::default();
    /// let packs = team.get_shop_packs();
    ///
    /// assert_eq!(packs.len(), 1);
    /// assert_eq!(packs[0], Pack::Turtle);
    /// ```
    fn get_shop_packs(&mut self) -> &[Pack];

    /// Freeze an item in the [`Shop`](crate::Shop).
    /// # Example
    /// ```
    /// use saptest::{Team, TeamShopping, Position, Entity};
    /// let mut team = Team::default();
    /// team.open_shop().unwrap();
    /// assert!(
    ///     team.freeze_shop(&Position::First, &Entity::Pet).is_ok()
    /// );
    /// ```
    fn freeze_shop(
        &mut self,
        pos: &Position,
        item_type: &Entity,
    ) -> Result<&mut Self, SAPTestError>;

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

    /// Move [`Pet`]s around merging them if desired.
    /// # Example
    /// ```
    /// use saptest::{
    ///     Team, TeamViewer, TeamShopping,
    ///     Pet, PetName, Position
    /// };
    /// let mut team = Team::new(
    ///     &[
    ///         Some(Pet::try_from(PetName::Ant).unwrap()),
    ///         Some(Pet::try_from(PetName::Ant).unwrap()),
    ///         Some(Pet::try_from(PetName::Ant).unwrap()),
    ///     ],
    ///     5
    /// ).unwrap();
    /// let last_ant = team.last().unwrap();
    /// assert_eq!(last_ant.borrow().get_level(), 1);
    ///
    /// // Move first pet consecutively merging them into the last ant.
    /// team.move_pets(&Position::First, &Position::Relative(-2), true).unwrap();
    /// team.move_pets(&Position::First, &Position::Relative(-1), true).unwrap();
    ///
    /// assert_eq!(last_ant.borrow().get_level(), 2);
    /// ```
    fn move_pets(
        &mut self,
        from: &Position,
        to: &Position,
        merge: bool,
    ) -> Result<&mut Self, SAPTestError>;

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
    /// ```shell
    /// (Pets)
    /// (Normal) [$3] [Mosquito: (2,2) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    /// (Normal) [$3] [Beaver: (3,2) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    /// (Normal) [$3] [Horse: (2,1) (Level: 1 Exp: 0) (Pos: None) (Item: None)]
    ///
    /// (Foods)
    /// (Normal) [$3] [Apple: [Effect (Uses: None): (Food) - Trigger: [Status: None, Position: None, Affected: None, From: None] - Action: Add(StaticValue(Statistics { attack: 1, health: 1 })) on Friend (OnSelf) ]]
    /// ```
    fn print_shop(&self);
}

/// Helper methods for buy/sell behavior.
impl TeamShoppingHelpers for Team {
    fn merge_behavior(
        &mut self,
        from_pet: &Rc<RefCell<Pet>>,
        to_pet: &Rc<RefCell<Pet>>,
    ) -> Result<(), SAPTestError> {
        // Get previous level. This will increase for every levelup.
        let mut prev_lvl = to_pet.borrow().lvl;

        // Stack pets.
        to_pet.borrow_mut().merge(&from_pet.borrow())?;

        // Check if pet leveled up. For EACH levelup:
        // * Activate pet effects if trigger is a levelup.
        //      * Must be done at previous level otherwise will use effect at current level.
        //      * Ex. Fish levelup must use lvl. 1 effect not its current effect at lvl. 2.
        // * Add shop pet on level
        // * Add team levelup triggers.
        for _ in 0..(to_pet.borrow().lvl - prev_lvl) {
            let mut levelup_trigger = TRIGGER_SELF_LEVELUP;
            levelup_trigger.set_affected(to_pet);

            // For pet effect of leveled up pet.
            for mut effect in to_pet.borrow().get_effect(prev_lvl)? {
                effect.assign_owner(Some(to_pet));
                if effect.trigger.status == Status::Levelup {
                    // Apply pet effect directly here if trigger is levelup.
                    self.apply_effect(&levelup_trigger, &effect, None)?;
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
            levelup_any_trigger.set_affected(to_pet);
            self.triggers.push_back(levelup_any_trigger);
        }
        Ok(())
    }
    fn buy_food_behavior(
        &mut self,
        food: Rc<RefCell<Food>>,
        curr_pet: Option<Rc<RefCell<Pet>>>,
        to_pos: &Position,
        emit_buy_triggers: bool,
    ) -> Result<(), SAPTestError> {
        // Emit buy any food trigger.
        if emit_buy_triggers {
            let trigger_any_food = TRIGGER_ANY_FOOD_BOUGHT;
            self.triggers.push_back(trigger_any_food);
        }

        // Give food to a single pet.
        if food.borrow().holdable {
            let affected_pets =
                self.get_pets_by_pos(curr_pet, &Target::Friend, to_pos, None, None)?;

            for (_, pet) in affected_pets {
                food.borrow_mut().ability.assign_owner(Some(&pet));
                pet.borrow_mut().item = Some(food.borrow().clone());

                // Create trigger if food eaten.
                let mut trigger_self_food = TRIGGER_SELF_FOOD_EATEN;
                let mut trigger_any_food = TRIGGER_ANY_FOOD_EATEN;
                let mut trigger_self_food_name =
                    trigger_self_food_ate_name(food.borrow().name.clone());
                trigger_self_food.set_affected(&pet);
                trigger_any_food.set_affected(&pet);
                trigger_self_food_name.set_affected(&pet);

                self.triggers
                    .extend([trigger_self_food, trigger_any_food, trigger_self_food_name]);
            }
        } else if food.borrow().name == FoodName::CannedFood {
            // Applying any effect requires an owner so assign current pet.
            food.borrow_mut().ability.assign_owner(curr_pet.as_ref());
            self.apply_shop_effect(&food.borrow().ability)?;
        } else {
            let mut food_ability = food.borrow().ability.clone();
            // If only one position (ex. apple), use target position, otherwise, use the food.ability positions.
            let target_pos = if food.borrow().n_targets == 1 {
                to_pos.clone()
            } else {
                food_ability.position.clone()
            };
            let affected_pets =
                self.get_pets_by_pos(curr_pet, &food_ability.target, &target_pos, None, None)?;

            // Hard-coded cat multiplier.
            // Repeat applying effect if action is to add stats.
            let cat_multiplier = if matches!(food_ability.action, Action::Add(_)) {
                self.friends
                    .iter()
                    .flatten()
                    .find_map(|pet| {
                        if pet.borrow().name == PetName::Cat {
                            Some(pet.borrow().lvl)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0)
            } else {
                0
            };

            // For each pet found by the effect of food bought, apply its effect.
            for (_, pet) in affected_pets {
                food_ability.assign_owner(Some(&pet));

                // Pet triggers for eating food.
                let mut trigger_self_food = TRIGGER_SELF_FOOD_EATEN;
                let mut trigger_any_food = TRIGGER_ANY_FOOD_EATEN;
                let mut trigger_self_food_name =
                    trigger_self_food_ate_name(food.borrow().name.clone());
                trigger_self_food.set_affected(&pet);
                trigger_any_food.set_affected(&pet);
                trigger_self_food_name.set_affected(&pet);

                self.triggers
                    .extend([trigger_self_food, trigger_any_food, trigger_self_food_name]);

                for _ in 0..(1 + cat_multiplier) {
                    self.apply_single_effect(&pet, &food_ability, None)?;
                }
            }
        }
        Ok(())
    }

    fn buy_pet_behavior(
        &mut self,
        pet: Rc<RefCell<Pet>>,
        curr_pet: Option<Rc<RefCell<Pet>>>,
        to_pos: &Position,
    ) -> Result<(), SAPTestError> {
        let affected_pets = self.get_pets_by_pos(curr_pet, &Target::Friend, to_pos, None, None)?;

        let purchased_pet = if let Some((_, affected_pet)) = affected_pets.first() {
            // If affected pet same as purchased pet.
            if affected_pet.borrow().name == pet.borrow().name {
                self.merge_behavior(affected_pet, &pet)?;
                Some(affected_pet.clone())
            } else {
                // Pet target exists. If position is last, make sure put after pet.
                // Otherwise, add pet in front of position.
                let adj_idx = if let Position::Last = to_pos { 1 } else { 0 };
                let pos = affected_pet.borrow().pos.unwrap_or(0) + adj_idx;
                self.add_pet(pet.borrow().clone(), pos, None)?;
                self.nth(pos)
            }
        } else {
            // No pets at all, summon at specific position, defaulting to 0 if not specific.
            let idx = self.cvt_pos_to_idx(to_pos).unwrap_or(0);
            self.add_pet(pet.borrow().clone(), idx, None)?;
            self.nth(idx)
        };

        if let Some(pet) = purchased_pet {
            let mut buy_trigger = TRIGGER_SELF_PET_BOUGHT;
            let mut buy_any_trigger = TRIGGER_ANY_PET_BOUGHT;
            let mut buy_any_tier_trigger = trigger_any_pet_bought_tier(pet.borrow().tier);
            buy_trigger.set_affected(&pet);
            buy_any_trigger.set_affected(&pet);
            buy_any_tier_trigger.set_affected(&pet);
            self.triggers
                .extend([buy_trigger, buy_any_trigger, buy_any_tier_trigger]);

            // For each effect a pet has create a buy trigger to show that a pet with this status purchased.
            // Needed for salamander.
            for effect in pet.borrow().effect.iter() {
                let mut buy_any_trigger =
                    trigger_any_pet_bought_status(effect.trigger.status.clone());
                buy_any_trigger.set_affected(&pet);
                self.triggers.push_back(buy_any_trigger)
            }
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
            let items_empty = match item_type {
                Entity::Pet => self.shop.pets.is_empty(),
                Entity::Food => self.shop.foods.is_empty(),
            };
            let err_msg = if items_empty {
                format!("No {item_type:?} items left to purchase.")
            } else {
                format!("No {item_type:?} items selected with {from:?} position.")
            };
            return Err(SAPTestError::InvalidShopAction {
                subject: "No Items Selectable".to_string(),
                reason: err_msg,
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
                ItemSlot::Pet(pet) => self.buy_pet_behavior(pet, self.first(), to)?,
                ItemSlot::Food(food) => self.buy_food_behavior(food, self.first(), to, true)?,
            };
        }

        self.trigger_effects(None)?;
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
                sell_trigger.set_affected(&pet);
                let mut sell_any_trigger = TRIGGER_ANY_PET_SOLD;
                sell_any_trigger.set_affected(&pet);

                // First sell trigger must be self trigger to remove it from friends list.
                // Otherwise, will remain in friends as target for effects.
                self.triggers.push_back(sell_trigger);

                // Check for an any trigger that shouldn't activate.
                let mut any_trigger_on_self = false;
                for effect in pet.borrow().effect.iter() {
                    let mut sell_w_status_trigger =
                        trigger_any_pet_sold_status(effect.trigger.status.clone());
                    sell_w_status_trigger.set_affected(&pet);
                    self.triggers.push_back(sell_w_status_trigger);

                    // If effect triggers on any pet sold, and target position is any pet. Ignore any trigger.
                    // Ex. Shrimp
                    if effect.trigger == sell_any_trigger
                        && effect.position == Position::Any(ItemCondition::None)
                    {
                        any_trigger_on_self = true
                    }
                }

                if !any_trigger_on_self {
                    self.triggers.push_back(sell_any_trigger);
                }
            }
        } else {
            return Err(SAPTestError::InvalidShopAction {
                subject: "No Sell-able Pet".to_string(),
                reason: format!("No pet to sell at position: {pos:?}."),
            });
        }

        // Trigger effects here.
        self.trigger_effects(None)?;
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
        self.trigger_effects(None)?;
        Ok(self)
    }

    fn len_shop_foods(&self) -> usize {
        self.shop.len_foods()
    }

    fn len_shop_pets(&self) -> usize {
        self.shop.len_pets()
    }

    fn get_shop(&self) -> &Shop {
        &self.shop
    }

    fn get_shop_packs(&mut self) -> &[Pack] {
        &self.shop.packs
    }

    fn set_shop_packs(&mut self, packs: &[Pack]) -> &mut Self {
        self.shop.packs = packs.to_vec();
        self
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
        self.trigger_effects(None)?;

        Ok(self)
    }

    fn set_shop_seed(&mut self, seed: Option<u64>) -> &mut Self {
        self.shop.seed = seed;
        self
    }

    fn freeze_shop(
        &mut self,
        pos: &Position,
        item_type: &Entity,
    ) -> Result<&mut Self, SAPTestError> {
        if self.shop.state == ShopState::Closed {
            return Err(SAPTestError::InvalidShopAction {
                subject: "Closed Shop (Freeze)".to_string(),
                reason: "Cannot perform action on closed shop.".to_string(),
            });
        }

        self.shop.freeze(pos, item_type)?;
        Ok(self)
    }

    fn open_shop(&mut self) -> Result<&mut Self, SAPTestError> {
        if self.shop.state == ShopState::Open {
            return Err(SAPTestError::InvalidShopAction {
                subject: "Opened Shop (Open)".to_string(),
                reason: "Cannot open an open shop.".to_string(),
            });
        }

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
                .flatten()
                .find(|pet| pet.borrow().id.as_ref() == Some(pet_id))
            {
                pet.borrow_mut().stats -= *stats
            }
        }
        // Trigger start of turn.
        self.triggers.push_front(TRIGGER_START_TURN);
        self.shop.restock()?;
        self.trigger_effects(None)?;
        self.clear_team();

        Ok(self)
    }

    fn close_shop(&mut self) -> Result<&mut Self, SAPTestError> {
        if self.shop.state == ShopState::Closed {
            return Err(SAPTestError::InvalidShopAction {
                subject: "Closed Shop (Close)".to_string(),
                reason: "Cannot close a closed shop.".to_string(),
            });
        }

        // Reset effects.
        for friend in self.friends.iter().flatten() {
            let mut reset_effect = friend.borrow().get_effect(friend.borrow().lvl)?;
            for effect in reset_effect.iter_mut() {
                effect.assign_owner(Some(friend));
            }
            friend.borrow_mut().effect = reset_effect;
        }

        // Trigger end of turn.
        self.triggers.push_front(TRIGGER_END_TURN);
        self.trigger_effects(None)?;
        self.clear_team();

        // Store friends.
        self.stored_friends = self
            .friends
            .iter()
            .map(|slot| slot.as_ref().map(|pet| pet.borrow().clone()))
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

    fn move_pets(
        &mut self,
        from: &Position,
        to: &Position,
        merge: bool,
    ) -> Result<&mut Self, SAPTestError> {
        let mut pets = Vec::with_capacity(2);

        for pos in [from, to].into_iter() {
            let pet = match pos {
                Position::Any(condition) => {
                    let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));
                    self.get_pets_by_cond(condition)
                        .into_iter()
                        .choose(&mut rng)
                }
                Position::First => self.first(),
                Position::Last => self
                    .friends
                    .len()
                    .checked_sub(1)
                    .and_then(|idx| self.nth(idx)),
                Position::Relative(idx) => {
                    let idx = *idx;
                    let adj_idx = TryInto::<usize>::try_into(-idx)?;
                    self.nth(adj_idx)
                }
                _ => {
                    return Err(SAPTestError::InvalidTeamAction {
                        subject: "Move Position Not Implemented".to_string(),
                        reason: format!("Position ({pos:?}) not implemented for move."),
                    })
                }
            };
            pets.push(pet)
        }
        if let (Some(from_pet), Some(to_pet)) = (&pets[0], &pets[1]) {
            // Same pet so just return.
            if Rc::ptr_eq(from_pet, to_pet) {
                return Ok(self);
            }
            let from_pos = from_pet
                .borrow()
                .pos
                .ok_or(SAPTestError::InvalidTeamAction {
                    subject: "Missing Position".to_string(),
                    reason: format!("Pet {from_pet:?} has no position."),
                })?;
            let to_pos = to_pet.borrow().pos.ok_or(SAPTestError::InvalidTeamAction {
                subject: "Missing Position".to_string(),
                reason: format!("Pet {to_pet:?} has no position."),
            })?;

            // If same pet name, merge.
            // Otherwise move from_pet to to_pet position.
            if from_pet.borrow().name == to_pet.borrow().name && merge {
                self.merge_behavior(from_pet, to_pet)?;
                // Remove pet.
                self.friends.remove(from_pos);
            } else {
                let from_pet = self.friends.remove(from_pos);
                self.friends.insert(to_pos, from_pet)
            }

            self.trigger_effects(None)?;
            self.clear_team();
        }
        Ok(self)
    }

    fn print_shop(&self) {
        println!("{}", self.shop)
    }
}
