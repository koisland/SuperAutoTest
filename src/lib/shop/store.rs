use std::{
    cell::RefCell,
    collections::VecDeque,
    fmt::Display,
    fmt::Write,
    ops::Range,
    rc::{Rc, Weak},
};

use itertools::Itertools;
use petgraph::visit::Dfs;
use rand::prelude::*;
use rand_chacha::ChaCha12Rng;

use crate::{
    battle::{
        actions::{Action, StatChangeType},
        effect::Entity,
        state::{Outcome, Status, Target},
        stats::Statistics,
        team::Team,
        trigger::*,
    },
    db::utils::setup_param_query,
    error::SAPTestError,
    foods::food::Food,
    pets::{names::PetName, pet::Pet},
    shop::trigger::*,
    TeamEffects, SAPDB,
};

/// Sloth chance.
const SLOTH_CHANCE: f64 = 0.0001;
/// Default coins for player.
const DEFAULT_COIN_COUNT: usize = 10;
const MAX_SHOP_PETS: usize = 6;
const MAX_SHOP_FOODS: usize = 4;

/// State of item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemState {
    /// Frozen item.
    Frozen,
    /// Normal item.
    Normal,
    /// Sold item.
    Sold,
}

#[derive(Debug, Clone)]
pub enum ItemSlot {
    Pet(Pet),
    Food(Food),
}

/// Shop Pet
#[derive(Debug, Clone)]
pub struct ShopItem {
    /// Shop item.
    pub(crate) item: ItemSlot,
    /// State of item in shop.
    pub(crate) state: ItemState,
    /// Current cost of item.
    pub(crate) cost: usize,
}

impl ShopItem {
    pub fn health(&self) -> Option<isize> {
        match &self.item {
            ItemSlot::Pet(pet) => Some(pet.stats.health),
            ItemSlot::Food(food) => match food.ability.action {
                Action::Add(StatChangeType::StaticValue(stats)) => Some(stats.health),
                Action::Remove(StatChangeType::StaticValue(stats)) => Some(stats.health),
                _ => None,
            },
        }
    }
    pub fn attack(&self) -> Option<isize> {
        match &self.item {
            ItemSlot::Pet(pet) => Some(pet.stats.attack),
            ItemSlot::Food(food) => match food.ability.action {
                Action::Add(StatChangeType::StaticValue(stats)) => Some(stats.attack),
                Action::Remove(StatChangeType::StaticValue(stats)) => Some(stats.attack),
                _ => None,
            },
        }
    }
    pub fn tier(&self) -> Option<usize> {
        match &self.item {
            ItemSlot::Pet(pet) => Some(pet.tier),
            ItemSlot::Food(food) => Some(food.tier),
        }
    }
    pub fn triggers(&self) -> Vec<&Status> {
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

impl Display for ItemSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemSlot::Pet(pet) => write!(f, "{pet}"),
            ItemSlot::Food(food) => write!(f, "{food}"),
        }
    }
}

impl Display for ShopItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}) {}", self.state, self.item)
    }
}

/// Check state of Shop.
pub trait CheckShopState {
    /// Check if item in Shop is frozen.
    fn is_frozen(&self) -> bool;
}

impl CheckShopState for &ShopItem {
    fn is_frozen(&self) -> bool {
        self.state == ItemState::Frozen
    }
}

/// A Super Auto Pets shop.
#[derive(Debug, Clone)]
pub struct Shop {
    /// Current tier of shop.
    tier: usize,
    /// Seed.
    seed: Option<u64>,
    /// Coins in shop.
    pub(crate) coins: usize,
    /// Pets in shop.
    pub(crate) pets: Vec<ShopItem>,
    /// Foods in shop.
    pub(crate) foods: Vec<ShopItem>,
    /// Sold pets.
    sold_pets: Vec<Rc<RefCell<Pet>>>,
    /// Global permanent `Statistics` added to all `Pet`s.
    add_stats: Statistics,
    /// Temporary stats that are removed on shop init.
    temp_stats: Vec<(Weak<RefCell<Pet>>, Statistics)>,
    /// Free rolls.
    free_rolls: usize,
    /// Shop triggers
    triggers: VecDeque<Outcome>,
}

impl Default for Shop {
    fn default() -> Self {
        Self {
            seed: None,
            coins: DEFAULT_COIN_COUNT,
            tier: 1,
            add_stats: Statistics::default(),
            temp_stats: vec![],
            pets: Vec::with_capacity(MAX_SHOP_PETS),
            foods: Vec::with_capacity(MAX_SHOP_FOODS),
            sold_pets: vec![],
            free_rolls: 0,
            triggers: VecDeque::new(),
        }
    }
}

impl Display for Shop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "(Pets)")?;
        for pet_slot in self.pets.iter() {
            writeln!(f, "{} ({:?})", pet_slot, pet_slot.state)?;
        }
        writeln!(f, "\n(Foods)")?;
        for food_slot in self.foods.iter() {
            writeln!(f, "{} ({:?})", food_slot, food_slot.state)?;
        }
        Ok(())
    }
}

impl Shop {
    /// Create a new shop with a given seed.
    /// # Example
    /// ```
    /// use saptest::{Shop, Team};
    ///
    /// let (mut team_1, mut team_2) = (Team::default(), Team::default());
    /// let seeded_shop = Shop::new(&mut team_1, Some(1212));
    /// let random_shop = Shop::new(&mut team_2, None);
    ///
    /// assert!(seeded_shop.is_ok() && random_shop.is_ok());
    /// ```
    pub fn new(team: &mut Team, seed: Option<u64>) -> Result<Self, SAPTestError> {
        let mut default_shop = Shop {
            seed,
            ..Default::default()
        };
        default_shop.tier = (team.history.curr_turn / 2 + team.history.curr_turn % 2).clamp(1, 6);
        default_shop.triggers.push_back(TRIGGER_START_TURN);
        default_shop
            .trigger_effects(team)?
            .fill_pets()?
            .fill_foods()?;

        Ok(default_shop)
    }

    /// Create an example shop.
    pub fn example(tier: usize, seed: Option<u64>) -> Result<Self, SAPTestError> {
        let mut default_shop = Shop {
            tier,
            seed,
            ..Default::default()
        };
        default_shop.triggers.push_back(TRIGGER_START_TURN);
        default_shop.fill_pets()?.fill_foods()?;

        Ok(default_shop)
    }

    /// Setup shop.
    pub fn setup(&mut self) -> Result<&mut Self, SAPTestError> {
        self.triggers.push_back(TRIGGER_START_TURN);
        self.fill_pets()?.fill_foods()?;

        Ok(self)
    }

    /// Get the `Shop` tier.
    /// ```
    /// use saptest::Shop;
    ///
    /// let mut shop = Shop::default();
    ///
    /// assert_eq!(shop.tier(), 1)
    /// ```
    pub fn tier(&self) -> usize {
        self.tier
    }

    /// Roll the shop.
    pub fn roll(&mut self, team: &mut Team) -> Result<&mut Self, SAPTestError> {
        // Decrement coin count if possible.
        if self.free_rolls != 0 {
            self.free_rolls = self.free_rolls.saturating_sub(1)
        } else if let Some(new_coins) = self.coins.checked_sub(1) {
            self.coins = new_coins;
        } else {
            return Err(SAPTestError::ShopError {
                reason: "No coins to reroll".to_string(),
            });
        }
        // Only keep frozen pets/foods.
        self.foods.retain(|food| food.state == ItemState::Frozen);
        self.pets.retain(|pet| pet.state == ItemState::Frozen);

        self.fill_pets()?;
        self.fill_foods()?;

        // TODO: Squirrel effect must be added here.
        self.triggers.push_back(TRIGGER_ROLL);
        self.trigger_effects(team)?;
        Ok(self)
    }

    /// Freeze the contents of a `Shop`
    /// ```
    /// use saptest::common::{shop::store::Shop, battle::team::Team};
    ///
    /// let mut starting_team = Team::default();
    /// let mut shop = Shop::new(&mut starting_team).unwrap();
    ///
    ///
    pub fn freeze_pet(&mut self, pos: usize) -> &mut Self {
        if let Some(pet) = self.pets.get_mut(pos) {
            pet.state = if pet.state == ItemState::Normal {
                ItemState::Frozen
            } else {
                ItemState::Normal
            }
        }
        self
    }

    /// Freeze a `Food` from the shop.
    /// # Example
    /// ```
    /// use saptest::{Shop, Team};
    /// let shop = Shop::new(&mut Team::default(), Some(1111)).unwrap();
    /// shop.freeze_food(0);
    /// assert!(shop.get_food(0).unwrap().is_frozen());
    /// ```
    pub fn freeze_food(&mut self, pos: usize) -> &mut Self {
        if let Some(food) = self.foods.get_mut(pos) {
            food.state = if food.state == ItemState::Normal {
                ItemState::Frozen
            } else {
                ItemState::Normal
            }
        }
        self
    }

    // /// Buy a food.
    // pub fn buy_food(
    //     &mut self,
    //     from: usize,
    //     to: Option<usize>,
    //     team: &mut Team,
    // ) -> Result<&mut Self, SAPTestError> {
    //     if from < self.foods.len() {
    //         // Check if can purchase food.
    //         let shop_food = self.foods.get(from).unwrap();
    //         if let Some(new_coins) = self.coins.checked_sub(shop_food.food.cost) {
    //             self.coins = new_coins;
    //         } else {
    //             return Err(SAPTestError::ShopError {
    //                 reason: format!("Not enough coins ({}) to buy item.", self.coins,),
    //             });
    //         }

    //         // Give item to pet if holdable.
    //         if shop_food.food.holdable {
    //             if let Some(pet_idx) = to {
    //                 if let Some(pet) = team.nth(pet_idx) {
    //                     let mut item = self.foods.remove(from).food;
    //                     item.ability.assign_owner(Some(&pet));
    //                     pet.borrow_mut().item = Some(item);

    //                     // Create trigger if food bought.
    //                     let mut trigger = TRIGGER_FOOD_BOUGHT;
    //                     trigger.set_affected(&pet);
    //                     self.triggers.push_back(trigger)
    //                 } else {
    //                     return Err(SAPTestError::ShopError {
    //                         reason: format!("Not a valid index for team ({pet_idx})."),
    //                     });
    //                 }
    //             }
    //         } else {
    //             // Otherwise, take effect and send trigger.
    //             let item = self.foods.remove(from).food;
    //             let item_effect = item.ability;

    //             let prev_node = team.history.curr_node;

    //             // TODO: Find some better way to this.
    //             let mut empty_team = Team::default();
    //             team.apply_effect(&TRIGGER_NONE, &item_effect, &mut empty_team)?;

    //             // Search through nodes.
    //             if let Some(start_node) = prev_node {
    //                 let effect_graph = &team.history.effect_graph;
    //                 let mut dfs = Dfs::new(effect_graph, start_node);

    //                 // See affected pets.
    //                 while let Some(Some(node)) = dfs
    //                     .next(effect_graph)
    //                     .map(|node| effect_graph.node_weight(node))
    //                 {
    //                     let mut trigger = TRIGGER_FOOD_BOUGHT;
    //                     trigger.affected_pet = node.affected_pet.clone();
    //                     self.triggers.push_back(trigger);
    //                 }
    //             }
    //         }
    //     } else {
    //         return Err(SAPTestError::ShopError {
    //             reason: format!("Not valid indices for shop ({from})."),
    //         });
    //     }

    //     self.trigger_effects(team)?;

    //     Ok(self)
    // }

    // /// Buy a pet.
    // pub fn buy_pet(
    //     &mut self,
    //     from: usize,
    //     to: usize,
    //     team: &mut Team,
    // ) -> Result<&mut Self, SAPTestError> {
    //     // From index and To index is valid.
    //     if from < self.pets.len() && to <= team.friends.len() {
    //         // Get pet from shop.
    //         let shop_pet = self.pets.get(from).unwrap();
    //         // Buy the item.
    //         if let Some(new_coins) = self.coins.checked_sub(shop_pet.pet.cost) {
    //             self.coins = new_coins;
    //         } else {
    //             // Not enough to buy so re-add to shop.
    //             return Err(SAPTestError::ShopError {
    //                 reason: format!("Not enough coins ({}) to buy pet.", self.coins,),
    //             });
    //         }
    //         // If pet in slot.
    //         let purchased_pet = if let Some(slot_pet) = team
    //             .friends
    //             .get(to)
    //             .filter(|pet| pet.borrow().name == shop_pet.pet.name)
    //         {
    //             // Get previous level.
    //             let prev_lvl = slot_pet.borrow().lvl;

    //             // Stack pet. Take max attack and health from pet.
    //             let max_attack = slot_pet
    //                 .borrow()
    //                 .stats
    //                 .attack
    //                 .max(shop_pet.pet.stats.attack);
    //             let max_health = slot_pet
    //                 .borrow()
    //                 .stats
    //                 .health
    //                 .max(shop_pet.pet.stats.health);

    //             slot_pet.borrow_mut().stats.attack = max_attack;
    //             slot_pet.borrow_mut().stats.health = max_health;
    //             slot_pet.borrow_mut().add_experience(1)?;
    //             // Remove pet from shop.
    //             self.pets.remove(from);

    //             // Pet leveled up. Add shop and team levelup triggers.
    //             if slot_pet.borrow().lvl == prev_lvl + 1 {
    //                 let mut levelup_trigger = TRIGGER_ANY_LEVELUP;
    //                 levelup_trigger.set_affected(slot_pet);

    //                 self.triggers.push_back(levelup_trigger.clone());
    //                 team.triggers.push_back(levelup_trigger);
    //             }
    //             Some(slot_pet.clone())
    //         } else {
    //             // Otherwise buying.
    //             let pet = self.pets.remove(from).pet;
    //             team.add_pet(pet, to, None)?;
    //             team.nth(to)
    //         };

    //         if let Some(pet) = purchased_pet {
    //             let mut buy_trigger = TRIGGER_PET_BOUGHT;
    //             buy_trigger.set_affected(&pet);
    //             self.triggers.push_back(buy_trigger)
    //         }
    //     } else {
    //         return Err(SAPTestError::ShopError {
    //             reason: format!("Not valid indices for shop ({from}) or team ({to:?})."),
    //         });
    //     }

    //     self.trigger_effects(team);

    //     Ok(self)
    // }

    /// Add a new pet a tier higher than the current tier on pet levelup.
    fn add_levelup_pet(&mut self) -> Result<&mut Self, SAPTestError> {
        if self.pets.len() == MAX_SHOP_PETS {
            return Err(SAPTestError::ShopError {
                reason: "Reached max number of pets.".to_string(),
            });
        }
        // TODO: Check for levelup.
        let records = SAPDB.execute_pet_query(
            "SELECT * FROM pets where tier = ? and level = ?",
            &[(self.tier + 1).clamp(1, 6).to_string(), 1.to_string()],
        )?;

        if let Some(added_pet) = records.first().cloned() {
            let cost = added_pet.cost;
            self.pets.push(ShopItem {
                item: ItemSlot::Pet(added_pet.try_into()?),
                state: ItemState::Normal,
                cost,
            });
        }
        Ok(self)
    }

    /// Sell a pet on the shop's team.
    pub fn sell(&mut self, pos: usize, team: &mut Team) -> Result<&mut Self, SAPTestError> {
        if (0..team.friends.len()).contains(&pos) {
            let pet = team.friends.remove(pos);

            // Add coins for sold pet.
            self.coins += pet.borrow().lvl;

            let mut sell_trigger = TRIGGER_PET_SOLD;
            let mut sell_any_trigger = TRIGGER_ANY_PET_SOLD;
            sell_trigger.set_affected(&pet);
            sell_any_trigger.set_affected(&pet);

            // Add pet to sold pets.
            self.sold_pets.push(pet);

            self.triggers.extend([sell_trigger, sell_any_trigger]);
            self.trigger_effects(team)?;
        } else {
            return Err(SAPTestError::ShopError {
                reason: format!("No pet to sell at position: {pos}."),
            });
        }
        Ok(self)
    }

    /// Close the shop and trigger the end of the turn.
    pub fn close<'a>(&mut self, team: &'a mut Team) -> Result<&'a mut Team, SAPTestError> {
        // Emit end of turn
        self.triggers.push_back(TRIGGER_END_TURN);

        self.trigger_effects(team)?;

        // restore money.
        self.coins = DEFAULT_COIN_COUNT;

        Ok(team)
    }

    /// Set the tier of a shop.
    pub fn set_tier(&mut self, tier: usize) -> &mut Self {
        self.tier = tier;
        self
    }

    fn food_slots(&self) -> usize {
        let n_foods: usize = if self.tier < 2 { 1 } else { 2 };
        n_foods.saturating_sub(self.foods.len())
    }

    /// Get the number of `Pet` slots for a shop.
    fn pet_slots(&self) -> usize {
        let n_pets: usize = if self.tier < 3 {
            3
        } else if self.tier < 5 {
            4
        } else {
            5
        };
        n_pets.saturating_sub(self.pets.len())
    }

    fn trigger_effects(&mut self, team: &mut Team) -> Result<&mut Self, SAPTestError> {
        let mut empty_team = Team::default();

        // Activate shop triggers.
        while let Some(shop_trigger) = self.triggers.pop_front() {
            if let Some(Some(affected_pet)) =
                shop_trigger.affected_pet.as_ref().map(|pet| pet.upgrade())
            {
                println!("{}", affected_pet.borrow())
            }

            // Check team pet effects.
            for pet in team.all().into_iter() {
                for effect in pet
                    .borrow()
                    .effect
                    .iter()
                    .filter(|effect| effect.trigger == shop_trigger)
                {
                    match &effect.target {
                        Target::Friend => {
                            team.apply_effect(&shop_trigger, effect, &mut empty_team)?;
                        }
                        Target::Shop => {
                            println!("{effect}")
                        }
                        _ => {}
                    }
                }
            }
        }
        // team.trigger_effects();
        // TODO: Check history for temp effect targets and get reference to those pets.
        Ok(self)
    }

    /// Build shop query.
    fn shop_query(&self, entity: Entity, tiers: Range<usize>) -> (String, Vec<String>) {
        let tiers = tiers.into_iter().map(|tier| tier.to_string()).collect_vec();
        let params: [Vec<String>; 3] = [tiers, vec!["Turtle".to_string()], vec![1.to_string()]];
        let mut flat_param: Vec<String> = params.iter().flatten().cloned().collect_vec();

        let stmt = match entity {
            Entity::Pet => {
                let named_params = [
                    ("tier", &params[0]),
                    ("pack", &params[1]),
                    ("lvl", &params[2]),
                ];
                let mut sql = setup_param_query("pets", &named_params);
                write!(sql, "AND name != 'Sloth'").unwrap();
                sql
            }
            Entity::Food => {
                let named_params = [("tier", &params[0]), ("pack", &params[1])];
                flat_param.pop();
                setup_param_query("foods", &named_params)
            }
        };

        (stmt, flat_param)
    }

    /// Set the shop seed.
    pub fn set_seed(&mut self, seed: Option<u64>) -> &mut Self {
        self.seed = seed;
        self
    }

    fn get_rng(&self) -> ChaCha12Rng {
        let seed = self.seed.unwrap_or_else(random);
        ChaCha12Rng::seed_from_u64(seed)
    }

    fn fill_pets(&mut self) -> Result<&mut Self, SAPTestError> {
        let (sql, params) = self.shop_query(Entity::Pet, 1..self.tier + 1);
        let possible_pets = SAPDB.execute_pet_query(&sql, &params)?;
        let mut rng = self.get_rng();

        // Iterate through slots choose a random pet or sloth.
        for _ in 0..self.pet_slots() {
            let pet = if rng.gen_bool(SLOTH_CHANCE) {
                Pet::try_from(PetName::Sloth)?
            } else {
                let record = possible_pets
                    .choose(&mut rng)
                    .ok_or(SAPTestError::QueryFailure {
                        subject: "Empty Shop Query".to_string(),
                        reason: format!("SQL ({sql}) with params ({params:?}) yielded no records."),
                    })?;
                Pet::try_from(record.name.clone())?
            };
            let cost = pet.cost;
            self.pets.push(ShopItem {
                item: ItemSlot::Pet(pet),
                state: ItemState::Normal,
                cost,
            });
        }

        Ok(self)
    }

    /// Fill the shop with foods.
    fn fill_foods(&mut self) -> Result<&mut Self, SAPTestError> {
        let (sql, params) = self.shop_query(Entity::Food, 1..self.tier + 1);
        let possible_foods = SAPDB.execute_food_query(&sql, &params)?;
        let mut rng = self.get_rng();

        // Iterate through slots choose a random food.
        for _ in 0..self.food_slots() {
            if let Some(food_record) = possible_foods.choose(&mut rng) {
                self.foods.push(ShopItem {
                    item: ItemSlot::Food(Food::try_from(food_record.name.clone())?),
                    state: ItemState::Normal,
                    cost: food_record.cost,
                });
            }
        }

        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        shop::store::{CheckShopState, ItemSlot},
        Pet, PetName, Position, Shop, Statistics, Team,
    };

    #[test]
    fn test_shop_freeze() {
        let mut team = Team::default();
        let mut shop = Shop::new(&mut team, Some(1111)).unwrap();
        shop.freeze_food(0).freeze_pet(0);

        assert!(shop.foods.first().unwrap().is_frozen());
        assert!(shop.pets.first().unwrap().is_frozen());
    }

    #[test]
    fn test_shop_roll_frozen() {
        let mut team = Team::default();
        let mut shop = Shop::new(&mut team, None).unwrap();

        shop.freeze_food(0).freeze_pet(0);

        assert!(shop.foods.first().unwrap().is_frozen());
        assert!(shop.pets.first().unwrap().is_frozen());

        shop.roll(&mut team).unwrap();

        // Frozen items are retained.
        assert!(shop.foods.first().unwrap().is_frozen());
        assert!(shop.pets.first().unwrap().is_frozen());
    }

    #[test]
    fn test_shop_stack_pet() {
        let mut team = Team::new(
            &[
                Pet::try_from(PetName::Ant).unwrap(),
                Pet::try_from(PetName::Mosquito).unwrap(),
            ],
            5,
        )
        .unwrap();

        let mosquito = team.nth(1).unwrap();
        // Mosquito at pos 1 has base stats and zero experience.
        assert_eq!(mosquito.borrow().stats, Statistics::new(2, 2).unwrap());
        assert_eq!(mosquito.borrow().exp, 0);

        let mut shop = Shop::new(&mut team, Some(1212)).unwrap();

        // Mosquito at first position of shop pets.
        let first_pet_slot = &shop.pets.first().unwrap().item;
        let ItemSlot::Pet(pet) = first_pet_slot else {
            panic!("Not a pet.")
        };
        assert_eq!(pet.name, PetName::Mosquito);
        // Stack shop pet onto team.
        // shop.buy_pet(0, 1, &mut team).unwrap();

        assert_eq!(mosquito.borrow().stats, Statistics::new(3, 3).unwrap());
    }

    #[test]
    fn test_shop_sell_pet() {
        let mut team = Team::new(
            &[
                Pet::try_from(PetName::Ant).unwrap(),
                Pet::try_from(PetName::Mosquito).unwrap(),
            ],
            5,
        )
        .unwrap();

        {
            let mut shop = Shop::new(&mut team, Some(1212)).unwrap();
            assert_eq!(shop.coins, 10);
            // Sell ant.
            shop.sell(0, &mut team).unwrap();
            // Get one coin.
            assert_eq!(shop.coins, 11);
        }

        // Levelup mosquito.
        team.set_level(Position::First, 2).unwrap();

        {
            let mut shop = Shop::new(&mut team, Some(1212)).unwrap();
            assert_eq!(shop.coins, 10);
            // Sell lvl. 2 mosquito.
            shop.sell(0, &mut team).unwrap();
            // Get two coins.
            assert_eq!(shop.coins, 12);
        }
    }
}
