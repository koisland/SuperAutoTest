use itertools::Itertools;

use crate::{
    battle::{
        effect::Entity,
        state::Target,
        team_effect_apply::{EffectApplyHelpers, TeamEffects},
        trigger::{TRIGGER_ANY_LEVELUP, TRIGGER_END_TURN, TRIGGER_START_TURN},
    },
    error::SAPTestError,
    Food, Pet, Position, Team,
};

use super::{
    store::{ItemSlot, ShopState},
    trigger::{
        TRIGGER_ANY_PET_SOLD, TRIGGER_FOOD_BOUGHT, TRIGGER_ROLL, TRIGGER_SELF_PET_BOUGHT,
        TRIGGER_SELF_PET_SOLD,
    },
    viewer::ShopViewer,
};

trait ShoppingHelpers {
    fn buy_food_behavior(
        &mut self,
        food: Food,
        to_pos: &Position,
        empty_team: &mut Team,
    ) -> Result<(), SAPTestError>;
    fn buy_pet_behavior(&mut self, pet: Pet, to_pos: &Position) -> Result<(), SAPTestError>;
}

/// Implements Super Auto Pets shop behavior.
pub trait Shopping {
    /// Buy a [`ShopItem`](crate::shop::store::ShopItem) from the [`Shop`](crate::Shop) and place it on the [`Team`](crate::Team).
    /// # Examples
    /// ---
    /// Buying Pets
    /// ```
    /// use saptest::{Team, Shopping, Position, Entity};
    ///
    /// let mut team = Team::default();
    /// team.set_shop_seed(Some(42))
    ///     .open_shop().unwrap();
    /// // Buy the 1st pet in the shop and put it on/in front of the 1st pet slot on the team.
    /// let first_pet_purchase = team.buy(
    ///     Position::First,
    ///     Entity::Pet,
    ///     Position::First
    /// );
    /// assert!(first_pet_purchase.is_ok())
    /// ```
    /// ---
    /// Buying a random food.
    /// ```
    /// use saptest::{Pet, PetName, Team, Shopping, Position, Entity};
    ///
    /// let mut team = Team::new(
    ///     &[Pet::try_from(PetName::Ant).unwrap()],
    ///     5
    /// ).unwrap();
    /// team.set_shop_seed(Some(42))
    ///     .open_shop().unwrap();
    /// // Buy a random food in the shop and put it on/in front of the 1st pet slot on the team.
    /// let first_random_item_purchase = team.buy(
    ///     Position::Any(Condition::None),
    ///     Entity::Food,
    ///     Position::First
    /// );
    /// assert!(first_random_item_purchase.is_ok())
    /// ```
    fn buy(
        &mut self,
        from: Position,
        item_type: Entity,
        to: Position,
    ) -> Result<&mut Self, SAPTestError>;
    /// Sell a [`Pet`](crate::Pet) on the [`Team`](crate::Team) for gold.
    fn sell(&mut self, pos: Position) -> Result<&mut Self, SAPTestError>;
    /// Roll the shop restocking it with new items.
    fn roll(&mut self) -> Result<&mut Self, SAPTestError>;
    /// Set the shop's seed.
    fn set_shop_seed(&mut self, seed: Option<u64>) -> &mut Self;
    /// Freeze an item in the Shop.
    fn freeze(&mut self, pos: Position, item_type: Entity) -> Result<&mut Self, SAPTestError>;
    /// Open a [`Shop`](crate::Shop) for a [`Team`](crate::Team).
    /// # Example
    /// ```
    /// use saptest::{Team, Shopping};
    ///
    /// let mut team = Team::default();
    /// assert!(team.open_shop().is_ok());
    /// ```
    fn open_shop(&mut self) -> Result<&mut Self, SAPTestError>;
    /// Close a [`Shop`](crate::Shop) for a [`Team`](crate::Team).
    /// * This will create triggers for ending the turn.
    /// * Allows battling.
    /// # Example
    /// ```
    /// use saptest::{Team, Shopping};
    ///
    /// let mut team = Team::default();
    /// team.open_shop().unwrap();
    /// assert!(team.close_shop().is_ok();
    /// ```
    fn close_shop(&mut self) -> Result<&mut Self, SAPTestError>;
}

impl ShoppingHelpers for Team {
    fn buy_food_behavior(
        &mut self,
        mut food: Food,
        to_pos: &Position,
        empty_team: &mut Team,
    ) -> Result<(), SAPTestError> {
        // Give food to a single pet.
        if food.holdable {
            let affected_pets =
                self.get_pets_by_pos(self.first(), Target::Friend, to_pos, None, None)?;
            let (_, target_pet) = affected_pets
                .first()
                .ok_or(SAPTestError::InvalidTeamAction {
                    subject: "No Item Target".to_string(),
                    reason: "Holdable item must have a target".to_string(),
                })?;
            food.ability.assign_owner(Some(target_pet));
            target_pet.borrow_mut().item = Some(food);

            // Create trigger if food bought.
            let mut trigger = TRIGGER_FOOD_BOUGHT;
            trigger.set_affected(target_pet);
            self.triggers.push_back(trigger)
        } else {
            let mut food_ability = food.ability.clone();
            let target_pos =
                Position::Multiple(vec![food_ability.position.clone(); food.n_targets]);
            // For each pet found by the effect of food bought, apply its effect.
            for (_, pet) in
                self.get_pets_by_pos(self.first(), Target::Friend, &target_pos, None, None)?
            {
                food_ability.assign_owner(Some(&pet));
                self.apply_single_effect(pet, &food_ability, empty_team)?;
            }
        }
        Ok(())
    }

    fn buy_pet_behavior(&mut self, pet: Pet, to_pos: &Position) -> Result<(), SAPTestError> {
        let affected_pets =
            self.get_pets_by_pos(self.first(), Target::Friend, to_pos, None, None)?;

        let purchased_pet = if let Some((_, affected_pet)) = affected_pets.first() {
            // If affected pet same as purchased pet.
            if affected_pet.borrow().name == pet.name {
                // Get previous level.
                let prev_lvl = affected_pet.borrow().lvl;

                // Stack pet. Take max attack and health from pet.
                let max_attack = affected_pet.borrow().stats.attack.max(pet.stats.attack);
                let max_health = affected_pet.borrow().stats.health.max(pet.stats.health);

                affected_pet.borrow_mut().stats.attack = max_attack;
                affected_pet.borrow_mut().stats.health = max_health;
                affected_pet.borrow_mut().add_experience(1)?;

                // Pet leveled up. Add shop and team levelup triggers.
                if affected_pet.borrow().lvl == prev_lvl + 1 {
                    let mut levelup_trigger = TRIGGER_ANY_LEVELUP;
                    levelup_trigger.set_affected(affected_pet);

                    // If pet levels, add a pet (tier above current tier) to shop.
                    self.shop.add_levelup_pet()?;

                    self.triggers.push_back(levelup_trigger.clone());
                }
                Some(affected_pet.clone())
            } else {
                // Otherwise, add pet to position.
                let pos = affected_pet.borrow().pos.unwrap_or(0);
                self.add_pet(pet, pos, None)?;
                self.nth(pos)
            }
        } else {
            // No pets at all, just add at first position.
            self.add_pet(pet, 0, None)?;
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

impl Shopping for Team {
    fn buy(
        &mut self,
        from: Position,
        item_type: Entity,
        to: Position,
    ) -> Result<&mut Self, SAPTestError> {
        let selected_items = self
            .shop
            .get_shop_items_by_pos(&from, &item_type)?
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
                ItemSlot::Pet(pet) => self.buy_pet_behavior(pet, &to)?,
                ItemSlot::Food(food) => self.buy_food_behavior(food, &to, &mut empty_team)?,
            };
        }

        self.trigger_effects(&mut empty_team)?;
        Ok(self)
    }

    fn sell(&mut self, pos: Position) -> Result<&mut Self, SAPTestError> {
        let affected_pets = self.get_pets_by_pos(None, Target::Friend, &pos, None, None)?;

        if !affected_pets.is_empty() {
            // Remove pets.
            self.friends
                .retain(|pet| affected_pets.contains(&(Target::Friend, pet.clone())));

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

        Ok(self)
    }

    fn roll(&mut self) -> Result<&mut Self, SAPTestError> {
        self.shop.roll()?;
        self.triggers.push_back(TRIGGER_ROLL);
        Ok(self)
    }

    fn set_shop_seed(&mut self, seed: Option<u64>) -> &mut Self {
        self.shop.set_seed(seed);
        self
    }

    fn freeze(&mut self, pos: Position, item_type: Entity) -> Result<&mut Self, SAPTestError> {
        self.shop.freeze(&pos, &item_type)?;
        Ok(self)
    }

    fn open_shop(&mut self) -> Result<&mut Self, SAPTestError> {
        self.shop.state = ShopState::Open;
        // Restore team to previous state.
        self.restore();
        // Trigger start of turn.
        self.triggers.push_front(TRIGGER_START_TURN);
        self.shop.restock()?;
        Ok(self)
    }

    fn close_shop(&mut self) -> Result<&mut Self, SAPTestError> {
        self.shop.state = ShopState::Closed;
        // Trigger end of turn.
        self.triggers.push_front(TRIGGER_END_TURN);
        // Clear any pets that fainted during shop phase.
        self.fainted.clear();
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::Shopping;
    use crate::{battle::effect::Entity, shop::store::ShopState, Pet, PetName, Position, Team};

    #[test]
    fn test_team_shop_setup() {
        let mut team = Team::new(
            &[
                Pet::try_from(PetName::Beaver).unwrap(),
                Pet::try_from(PetName::Mosquito).unwrap(),
            ],
            5,
        )
        .unwrap();

        team.shop.set_tier(3).unwrap();
        team.open_shop().unwrap();

        println!("{}", team.shop)
    }
    #[test]
    fn test_team_shop_buy() {
        let mut team = Team::new(
            &[
                Pet::try_from(PetName::Beaver).unwrap(),
                Pet::try_from(PetName::Mosquito).unwrap(),
            ],
            5,
        )
        .unwrap();
        let sep = "=".repeat(50);
        println!("{}", team.shop);
        team.set_shop_seed(Some(1212)).open_shop().unwrap();
        println!("{sep}");
        println!("{}", team.shop);
        println!("{sep}");
        println!("Coins: {}", team.shop.coins);
        team.buy(Position::Relative(-1), Entity::Pet, Position::First)
            .unwrap();
        println!("{team}");
        println!("Coins: {}", team.shop.coins);
        team.buy(Position::First, Entity::Food, Position::First)
            .unwrap();
        println!("{team}");
        println!("Coins: {}", team.shop.coins);
    }

    #[test]
    fn test_shop_() {
        let mut team = Team::default();
        team.set_shop_seed(Some(42)).open_shop().unwrap();

        // Buy the 1st pet in the shop and put it on/in front of the 1st pet on the team.
        let first_pet_purchase = team.buy(Position::First, Entity::Pet, Position::First);
        assert!(first_pet_purchase.is_ok())
    }
    #[test]
    fn test_team_shop_sell() {
        let mut team = Team::new(
            &[
                Pet::try_from(PetName::Beaver).unwrap(),
                Pet::try_from(PetName::Mosquito).unwrap(),
            ],
            5,
        )
        .unwrap();

        println!("{}", team.shop.coins);
        println!("{}", team);
        team.sell(Position::First).unwrap();
        println!("{}", team.shop.coins);
        println!("{}", team);
    }

    #[test]
    fn test_team_shop_state_battle() {
        let mut team = Team::new(
            &[
                Pet::try_from(PetName::Beaver).unwrap(),
                Pet::try_from(PetName::Mosquito).unwrap(),
            ],
            5,
        )
        .unwrap();
        let mut opponent = team.clone();

        // Cannot fight while shop is open.
        team.shop.state = ShopState::Open;
        assert!(team.fight(&mut opponent).is_err());

        team.shop.state = ShopState::Closed;
        assert!(team.fight(&mut opponent).is_ok())
    }
}
