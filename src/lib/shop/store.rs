use std::{cell::RefCell, fmt::Display, fmt::Write, ops::Range, rc::Rc};

use itertools::Itertools;
use rand::prelude::*;
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};

use crate::{
    db::{pack::Pack, utils::setup_param_query},
    effects::{effect::Entity, stats::Statistics},
    error::SAPTestError,
    foods::food::Food,
    pets::{names::PetName, pet::Pet},
    shop::viewer::ShopViewer,
    Position, SAPDB,
};

/// Sloth chance.
const SLOTH_CHANCE: f64 = 0.0001;
/// Default coins for player.
pub const DEFAULT_COIN_COUNT: usize = 10;
const MAX_SHOP_PETS: usize = 6;
const MAX_SHOP_FOODS: usize = 4;
pub(crate) const MIN_SHOP_TIER: usize = 1;
pub(crate) const MAX_SHOP_TIER: usize = 6;

/// State of shop.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShopState {
    /// Open shop.
    Open,
    /// Closed shop.
    Closed,
}

/// State of item.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ItemState {
    /// Frozen item.
    Frozen,
    #[default]
    /// Normal item.
    Normal,
}

/// Item slot in [`Shop`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ItemSlot {
    /// A shop pet.
    Pet(Rc<RefCell<Pet>>),
    /// A shop food.
    Food(Rc<RefCell<Food>>),
}

/// A [`Shop`] item.
#[derive(Debug, Clone, PartialEq)]
pub struct ShopItem {
    /// Shop item.
    pub(crate) item: ItemSlot,
    /// State of item.
    pub(crate) state: ItemState,
    /// Current gold cost of item.
    pub(crate) cost: usize,
    /// Item position.
    pub(crate) pos: Option<usize>,
}

impl ShopItem {
    /// Create a [`Shop`] item.
    /// # Example
    /// ---
    /// Create a [`Coconut`](crate::FoodName::Coconut) that costs `5` gold in a [`Shop`].
    /// ```
    /// use saptest::{Shop, ShopItem, ShopItemViewer, EntityName, Food, FoodName};
    ///
    /// let new_shop_item = ShopItem::new(
    ///     Food::try_from(FoodName::Coconut).unwrap(),
    /// );
    /// assert!(new_shop_item.name() == EntityName::Food(FoodName::Coconut));
    /// ```
    pub fn new<I: Into<ShopItem>>(item: I) -> Self {
        item.into()
    }
}

impl From<Food> for ShopItem {
    fn from(value: Food) -> Self {
        let cost = value.cost;
        ShopItem {
            item: ItemSlot::Food(Rc::new(RefCell::new(value))),
            state: ItemState::Normal,
            cost,
            pos: None,
        }
    }
}

impl From<Pet> for ShopItem {
    fn from(value: Pet) -> Self {
        let cost = value.cost;
        ShopItem {
            item: ItemSlot::Pet(Rc::new(RefCell::new(value))),
            state: ItemState::Normal,
            cost,
            pos: None,
        }
    }
}

impl Display for ItemSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemSlot::Pet(pet) => write!(f, "{}", pet.borrow()),
            ItemSlot::Food(food) => write!(f, "{}", food.borrow()),
        }
    }
}

impl Display for ShopItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}) [${}] {}", self.state, self.cost, self.item)
    }
}

/// A Super Auto Pets shop.
#[derive(Debug, Clone)]
pub struct Shop {
    pub(crate) state: ShopState,
    /// Current tier of shop.
    tier: usize,
    /// Seed.
    pub seed: Option<u64>,
    /// Coins in shop.
    pub coins: usize,
    /// Pets in shop.
    pub pets: Vec<ShopItem>,
    /// Foods in shop.
    pub foods: Vec<ShopItem>,
    /// Packs shop should include.
    pub packs: Vec<Pack>,
    /// Global permanent `Statistics` added to all `Pet`s.
    /// * Added via a `CannedFood`
    pub perm_stats: Statistics,
    /// Temporary stats that are removed on shop opening.
    pub(crate) temp_stats: Vec<(String, Statistics)>,
    /// Free rolls.
    pub free_rolls: usize,
}

impl Default for Shop {
    fn default() -> Self {
        Self {
            state: ShopState::Closed,
            seed: None,
            coins: DEFAULT_COIN_COUNT,
            tier: 1,
            perm_stats: Statistics::default(),
            temp_stats: vec![],
            pets: Vec::with_capacity(MAX_SHOP_PETS),
            foods: Vec::with_capacity(MAX_SHOP_FOODS),
            free_rolls: 0,
            packs: vec![Pack::Turtle],
        }
    }
}

impl Display for Shop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "(Pets)")?;
        for pet_slot in self.pets.iter() {
            writeln!(f, "{pet_slot}")?;
        }
        writeln!(f, "\n(Foods)")?;
        for food_slot in self.foods.iter() {
            writeln!(f, "{food_slot}")?;
        }
        Ok(())
    }
}

impl Shop {
    /// Create a `Shop`.
    /// # Examples
    /// ```
    /// use saptest::Shop;
    ///
    /// let seeded_shop = Shop::new(1, Some(1212));
    /// let random_shop = Shop::new(1, None);
    /// let invalid_shop = Shop::new(7, None);
    ///
    /// assert!(seeded_shop.is_ok() && random_shop.is_ok());
    /// assert!(invalid_shop.is_err());
    /// ```
    pub fn new(tier: usize, seed: Option<u64>) -> Result<Self, SAPTestError> {
        let mut default_shop = Shop {
            seed,
            ..Default::default()
        };
        default_shop.set_tier(tier)?;
        default_shop.fill_pets()?.fill_foods()?;

        Ok(default_shop)
    }

    /// Restock a shop with [`ShopItem`](crate::shop::store::ShopItem)s.
    /// * Frozen [`ShopItem`](crate::shop::store::ShopItem)s are retained.
    /// * Restocking doesn't cost gold.
    /// # Example
    /// ```
    /// use saptest::{Shop, ShopViewer};
    ///
    /// let mut shop = Shop::default();
    /// // Shop is empty and has maximum space available.
    /// assert!(
    ///     shop.available_food_slots() == shop.max_food_slots() &&
    ///     shop.available_pet_slots() == shop.max_pet_slots()
    /// );
    /// // Shop is restocked and no space left.
    /// assert!(shop.restock().is_ok());
    /// assert!(
    ///     shop.available_food_slots() == 0 &&
    ///     shop.available_pet_slots() == 0
    /// );
    /// ```
    pub fn restock(&mut self) -> Result<&mut Self, SAPTestError> {
        self.fill_pets()?.fill_foods()?;
        Ok(self)
    }

    /// Add a [`ShopItem`] to the shop.
    /// * Foods added over the limit will remove pets on the **rightmost side**.
    /// * Pets can be added over the limit if the food limit allows the space.
    /// # Examples
    /// ---
    /// Add a food to the shop.
    /// ```
    /// use saptest::{Shop, ShopItem, EntityName, Pet, PetName};
    ///
    /// let mut shop = Shop::default();
    /// let pet = ShopItem::new(Pet::try_from(PetName::Ant).unwrap());
    /// assert!(shop.add_item(pet).is_ok());
    /// ```
    /// ---
    /// Add a pet to the shop.
    /// ```
    /// use saptest::{Shop, ShopItem, EntityName, Food, FoodName};
    ///
    /// let mut shop = Shop::default();
    /// let food = ShopItem::new(Food::try_from(FoodName::Coconut).unwrap());
    /// assert!(shop.add_item(food).is_ok());
    /// ```
    /// ---
    /// Override the rightmost pet with a food.
    /// ```
    /// use saptest::{Shop, ShopItem, ShopViewer, EntityName, Food, FoodName};
    ///
    /// let mut shop = Shop::new(5, Some(12)).unwrap();
    /// assert_eq!(shop.len_foods(), 2);
    /// assert_eq!(shop.len_pets(), 5);
    ///
    /// let food = ShopItem::new(Food::try_from(FoodName::Coconut).unwrap());
    /// shop.add_item(food);
    /// assert_eq!(shop.len_foods(), 3);
    /// assert_eq!(shop.len_pets(), 4);
    /// ```
    pub fn add_item(&mut self, item: ShopItem) -> Result<&mut Self, SAPTestError> {
        let max_shop_slots = self.max_food_slots() + self.max_pet_slots();
        match &item.item {
            ItemSlot::Pet(_) => {
                // Pets cannot exceed shop length (max pet and food slots).
                if self.available_pet_slots() + self.available_food_slots() != 0
                    && max_shop_slots > self.pets.len()
                {
                    self.pets.push(item)
                } else {
                    return Err(SAPTestError::InvalidShopAction {
                        subject: "Max Shop Pets".to_string(),
                        reason: format!("Insufficient space to add {item}."),
                    });
                }
                for (i, item) in self.pets.iter_mut().enumerate() {
                    item.pos = Some(i)
                }
            }
            ItemSlot::Food(_) => {
                // Need slots available so we remove the rightmost pet.
                if self.available_food_slots() == 0 {
                    self.pets.pop();
                }
                // Foods cannot exceed total shop length.
                if max_shop_slots > self.foods.len() + self.pets.len() {
                    self.foods.push(item)
                } else {
                    return Err(SAPTestError::InvalidShopAction {
                        subject: "Max Shop Foods".to_string(),
                        reason: format!("Insufficient space to add {item}."),
                    });
                }
                for (i, item) in self.foods.iter_mut().enumerate() {
                    item.pos = Some(i)
                }
            }
        };
        Ok(self)
    }

    /// Roll the `Shop`.
    /// * Frozen [`ShopItem`](crate::shop::store::ShopItem)s are retained.
    /// * Fails if invalid funds to reroll.
    ///     * Each roll costs `1` coin.
    /// # Examples
    /// ---
    /// Normal roll.
    /// ```
    /// use saptest::Shop;
    ///
    /// let mut shop = Shop::default();
    /// assert!(shop.roll().is_ok());
    /// ```
    /// ---
    /// Roll and retain frozen items.
    /// ```
    /// use saptest::{Shop, Entity, Position, ShopViewer, ShopItemViewer};
    ///
    /// // Freeze first pet.
    /// let mut shop = Shop::new(1, Some(12)).unwrap();
    /// let (pos, item_type) = (Position::First, Entity::Pet);
    /// shop.freeze(&pos, &item_type).unwrap();
    ///
    /// // Check first item is frozen.
    /// let found_items = shop.get_shop_items_by_pos(&pos, &item_type).unwrap();
    /// let first_item_no_roll = found_items.first().cloned().unwrap().clone();
    /// assert!(first_item_no_roll.is_frozen());
    ///
    /// // Roll the shop.
    /// shop.roll().unwrap();
    ///
    /// // Check items again.
    /// // First item was retained.
    /// let found_items_rolled = shop.get_shop_items_by_pos(&pos, &item_type).unwrap();
    /// let first_item_rolled = found_items_rolled.first().unwrap();
    /// assert_eq!(&&first_item_no_roll, first_item_rolled);
    /// ```
    /// ---
    /// Roll 10 times. On 11th roll, run out of money.
    /// ```
    /// use saptest::Shop;
    ///
    /// let mut shop = Shop::default();
    /// // Start with 10.
    /// assert_eq!(shop.coins, 10);
    ///
    /// // Roll your savings away.
    /// for i in 0..10 {
    ///     shop.roll().unwrap();
    /// }
    /// assert!(shop.roll().is_err());
    /// assert_eq!(shop.coins, 0);
    /// ```
    pub fn roll(&mut self) -> Result<&mut Self, SAPTestError> {
        // Decrement coin count if possible.
        if self.free_rolls != 0 {
            self.free_rolls = self.free_rolls.saturating_sub(1)
        } else if let Some(new_coins) = self.coins.checked_sub(1) {
            self.coins = new_coins;
        } else {
            return Err(SAPTestError::InvalidShopAction {
                subject: "Insufficient Coins (Roll)".to_string(),
                reason: "No coins to reroll".to_string(),
            });
        }
        // Only keep frozen pets/foods.
        self.foods.retain(|food| food.state == ItemState::Frozen);
        self.pets.retain(|pet| pet.state == ItemState::Frozen);

        self.restock()?;

        Ok(self)
    }

    /// Freeze the contents of a `Shop`
    /// # Example
    /// ```
    /// use saptest::{Shop, Position, Entity, ShopViewer, ShopItemViewer};
    ///
    /// let mut shop = Shop::new(1, None).unwrap();
    /// // Freeze the first pet in the shop.
    /// let (pos, item_type) = (Position::First, Entity::Pet);
    /// assert!(shop.freeze(&pos, &item_type).is_ok());
    /// // Check first pet.
    /// let found_pets = shop.get_shop_items_by_pos(&pos, &item_type).unwrap();
    /// assert!(found_pets.first().unwrap().is_frozen())
    ///
    pub fn freeze(
        &mut self,
        pos: &Position,
        item_type: &Entity,
    ) -> Result<&mut Self, SAPTestError> {
        // Get indices of items that should be frozen.
        // Need sep block as as getter function returns immutable refs.
        let selected_idx: Vec<usize> = {
            let selected_items = self.get_shop_items_by_pos(pos, item_type)?;
            let items = match item_type {
                Entity::Pet => self.pets.iter(),
                Entity::Food => self.foods.iter(),
            };
            items
                .enumerate()
                .filter_map(|(i, item)| selected_items.contains(&item).then_some(i))
                .collect_vec()
        };

        // Then mutate items setting item state to frozen.
        let items = if let Entity::Pet = item_type {
            &mut self.pets
        } else {
            &mut self.foods
        };
        for idx in selected_idx {
            if let Some(item) = items.get_mut(idx) {
                item.state = if let ItemState::Normal = item.state {
                    ItemState::Frozen
                } else {
                    ItemState::Normal
                }
            }
        }
        Ok(self)
    }

    /// Add a new pet to the end of the shop.
    /// * Used on any pet levelup in shop.
    /// * Added pet will be a tier higher than the current shop tier.
    pub(crate) fn add_levelup_pet(&mut self) -> Result<&mut Self, SAPTestError> {
        // No space so do nothing.
        if self.pets.len() == MAX_SHOP_PETS {
            return Ok(self);
        }
        let records = SAPDB.execute_pet_query(
            "SELECT * FROM pets where tier = ? and lvl = ?",
            &[(self.tier + 1).clamp(1, 6).to_string(), 1.to_string()],
        )?;

        if let Some(added_pet) = records.first().cloned() {
            let pet: Pet = added_pet.try_into()?;
            self.add_item(pet.into())?;
        }
        Ok(self)
    }

    /// Get the `Shop` tier.
    /// # Example
    /// ```
    /// use saptest::Shop;
    ///
    /// let shop_default = Shop::default();
    /// let shop_tier_3 = Shop::new(3, None).unwrap();
    ///
    /// assert_eq!(shop_default.tier(), 1);
    /// assert_eq!(shop_tier_3.tier(), 3);
    /// ```
    pub fn tier(&self) -> usize {
        self.tier
    }

    /// Check if valid shop tier.
    pub(crate) fn is_valid_shop_tier(tier: usize) -> Result<(), SAPTestError> {
        if !(MIN_SHOP_TIER..=MAX_SHOP_TIER).contains(&tier) {
            return Err(
                SAPTestError::InvalidShopAction {
                    subject: "Shop Tier".to_string(),
                    reason: format!("Tier provided ({tier}) is invalid. ({MIN_SHOP_TIER} <= tier <= {MAX_SHOP_TIER})")
                }
            );
        }
        Ok(())
    }

    /// Convert tier to num_turns.
    pub(crate) fn tier_to_num_turns(tier: usize) -> Result<usize, SAPTestError> {
        Shop::is_valid_shop_tier(tier)?;
        Ok(1 + (2 * tier.saturating_sub(1)))
    }

    /// Set the tier of a `Shop`.
    /// * Use in combination with [`restock`](crate::Shop::restock)
    /// # Example
    /// ```
    /// use saptest::Shop;
    /// let mut shop_default = Shop::default();
    /// // Default is tier 1.
    /// assert_eq!(shop_default.tier(), 1);
    /// // Now is tier 2.
    /// shop_default.set_tier(2);
    /// assert_eq!(shop_default.tier(), 2);
    /// ```
    pub fn set_tier(&mut self, tier: usize) -> Result<&mut Self, SAPTestError> {
        Shop::is_valid_shop_tier(tier)?;
        self.tier = tier;
        Ok(self)
    }

    /// Build shop query.
    pub(crate) fn shop_query(&self, entity: Entity, tiers: Range<usize>) -> (String, Vec<String>) {
        let tiers = tiers.into_iter().map(|tier| tier.to_string()).collect_vec();
        let params: [Vec<String>; 3] = [
            tiers,
            self.packs.iter().map(|pack| pack.to_string()).collect_vec(),
            vec![1.to_string()],
        ];
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

    pub(crate) fn get_rng(&self) -> ChaCha12Rng {
        let seed = self.seed.unwrap_or_else(random);
        ChaCha12Rng::seed_from_u64(seed)
    }

    /// Fill pets based on current tier of shop.
    pub(crate) fn fill_pets(&mut self) -> Result<&mut Self, SAPTestError> {
        let (sql, params) = self.shop_query(Entity::Pet, 1..self.tier + 1);
        let possible_pets = SAPDB.execute_pet_query(&sql, &params)?;
        let mut rng = self.get_rng();

        // Iterate through slots choose a random pet or sloth.
        let total_items = self.len_foods() + self.len_pets();
        let total_allowed_items = self.max_food_slots() + self.max_pet_slots();
        let n_slots = if total_items == total_allowed_items {
            0
        } else {
            self.available_pet_slots()
        };
        for i in 0..n_slots {
            let (cost, mut pet) = if rng.gen_bool(SLOTH_CHANCE) {
                (3, Pet::try_from(PetName::Sloth)?)
            } else {
                let record = possible_pets
                    .choose(&mut rng)
                    .ok_or(SAPTestError::QueryFailure {
                        subject: "Empty Shop Query".to_string(),
                        reason: format!(
                            "SQL ({sql}) with params ({params:?}) yielded no pet records."
                        ),
                    })?;
                (record.cost, Pet::try_from(record.name.clone())?)
            };
            // Add permanent pet stats.
            pet.stats += self.perm_stats;
            self.pets.push(ShopItem {
                item: ItemSlot::Pet(Rc::new(RefCell::new(pet))),
                state: ItemState::Normal,
                cost,
                pos: Some(i),
            });
        }

        Ok(self)
    }

    /// Fill the shop with foods based on current tier of shop.
    pub(crate) fn fill_foods(&mut self) -> Result<&mut Self, SAPTestError> {
        let (sql, params) = self.shop_query(Entity::Food, 1..self.tier + 1);
        let possible_foods = SAPDB.execute_food_query(&sql, &params)?;
        let mut rng = self.get_rng();

        // Iterate through slots choose a random food.
        let total_items = self.len_foods() + self.len_pets();
        let total_allowed_items = self.max_food_slots() + self.max_pet_slots();
        let n_slots = if total_items == total_allowed_items {
            0
        } else {
            self.available_food_slots()
        };
        for i in 0..n_slots {
            let food_record =
                possible_foods
                    .choose(&mut rng)
                    .ok_or(SAPTestError::QueryFailure {
                        subject: "Empty Shop Query".to_string(),
                        reason: format!(
                            "SQL ({sql}) with params ({params:?}) yielded no food records."
                        ),
                    })?;
            let food = Food::try_from(food_record.name.clone())?;
            self.foods.push(ShopItem {
                item: ItemSlot::Food(Rc::new(RefCell::new(food))),
                state: ItemState::Normal,
                cost: food_record.cost,
                pos: Some(i),
            });
        }

        Ok(self)
    }
}
