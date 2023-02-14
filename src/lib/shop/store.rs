use std::{
    cell::RefCell,
    fmt::Display,
    fmt::Write,
    ops::{Add, Range},
    rc::Weak,
};

use itertools::Itertools;
use rand::prelude::*;
use rand_chacha::ChaCha12Rng;

use crate::{
    battle::{effect::Entity, stats::Statistics},
    db::utils::setup_param_query,
    error::SAPTestError,
    foods::food::Food,
    pets::{names::PetName, pet::Pet},
    shop::viewer::ShopViewer,
    Position, SAPDB,
};

/// Sloth chance.
const SLOTH_CHANCE: f64 = 0.0001;
/// Default coins for player.
const DEFAULT_COIN_COUNT: usize = 10;
const MAX_SHOP_PETS: usize = 6;
const MAX_SHOP_FOODS: usize = 4;
const MIN_SHOP_TIER: usize = 1;
const MAX_SHOP_TIER: usize = 6;

/// State of item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemState {
    /// Frozen item.
    Frozen,
    /// Normal item.
    Normal,
}

/// Shop item slot.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemSlot {
    /// A shop pet.
    Pet(Pet),
    /// A shop food.
    Food(Food),
}

/// Shop Pet
#[derive(Debug, Clone, PartialEq)]
pub struct ShopItem {
    /// Shop item.
    pub(crate) item: ItemSlot,
    /// State of item in shop.
    pub(crate) state: ItemState,
    /// Current cost of item.
    pub(crate) cost: usize,
}

/// Sum the cost of shop items.
impl Add for &ShopItem {
    type Output = usize;

    fn add(self, rhs: Self) -> Self::Output {
        self.cost + rhs.cost
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
    /// Global permanent `Statistics` added to all `Pet`s.
    /// * Added via a `CannedFood`
    perm_stats: Statistics,
    /// Temporary stats that are removed on shop opening.
    temp_stats: Vec<(Weak<RefCell<Pet>>, Statistics)>,
    /// Free rolls.
    pub(crate) free_rolls: usize,
}

impl Default for Shop {
    fn default() -> Self {
        Self {
            seed: None,
            coins: DEFAULT_COIN_COUNT,
            tier: 1,
            perm_stats: Statistics::default(),
            temp_stats: vec![],
            pets: Vec::with_capacity(MAX_SHOP_PETS),
            foods: Vec::with_capacity(MAX_SHOP_FOODS),
            free_rolls: 0,
        }
    }
}

impl Display for Shop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "(Pets)")?;
        for pet_slot in self.pets.iter() {
            writeln!(f, "{}", pet_slot)?;
        }
        writeln!(f, "\n(Foods)")?;
        for food_slot in self.foods.iter() {
            writeln!(f, "{}", food_slot)?;
        }
        Ok(())
    }
}

impl Shop {
    /// Create a shop.
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

    /// Restock shop with foods and pets.
    /// * Frozen items are retained.
    /// * Restocking doesn't cost gold.
    pub fn restock(&mut self) -> Result<&mut Self, SAPTestError> {
        self.fill_pets()?.fill_foods()?;
        Ok(self)
    }

    /// Get coins available in `Shop`.
    /// # Example
    /// ```
    /// use saptest::Shop;
    /// let shop = Shop::default();
    /// assert_eq!(shop.coins(), 10);
    /// ```
    pub fn coins(&self) -> usize {
        self.coins
    }

    /// Roll the shop.
    /// * Frozen items are retained.
    /// * Fails if invalid funds to reroll.
    ///     * Each roll costs `1` coin.
    /// # Examples
    /// ---
    /// Normal roll.
    /// ```
    /// use saptest::Shop;
    /// let mut shop = Shop::default();
    /// assert!(shop.roll().is_ok());
    /// ```
    /// 
    /// ---
    /// Roll 5 times.
    /// ```
    /// use saptest::{Shop, Entity, Position};
    /// let mut shop = Shop::default();
    /// shop.freeze(&Position::First, Entity::Pet).unwrap();
    /// ```
    /// ---
    /// Roll 10 times. On 11th roll, run out of money.
    /// ```
    /// use saptest::Shop;
    /// let mut shop = Shop::default();
    /// // Start with 10.
    /// assert_eq!(shop.coins(), 10);
    /// // Roll your savings away. :/
    /// for i in 0..10 {
    ///     shop.roll().unwrap();
    /// }
    /// assert!(shop.roll().is_err())
    /// ```
    pub fn roll(&mut self) -> Result<&mut Self, SAPTestError> {
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
    pub fn freeze(&mut self, pos: &Position, item_type: &Entity) -> Result<&mut Self, SAPTestError> {
        // Get indices of items that should be frozen.
        // Need sep block as as getter function returns immutable refs.
        let selected_idx: Vec<usize> = {
            let selected_items = self.get_shop_items_by_pos(pos, item_type)?;
            let items = match item_type {
                Entity::Pet => self.pets.iter(),
                Entity::Food => self.foods.iter(),
            };
            items.enumerate().filter_map(|(i, item)| selected_items.contains(&item).then_some(i)).collect_vec()
        };

        // Then mutate items setting item state to frozen.
        let items = if let Entity::Pet = item_type {&mut self.pets} else {&mut self.foods};
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

    /// Get the `Shop` tier.
    /// # Example
    /// ```
    /// use saptest::Shop;
    ///
    /// let shop_default = Shop::default();
    /// let shop_tier_3 = Shop::new(3, None);
    ///
    /// assert_eq!(shop.tier(), 1);
    /// assert_eq!(shop.tier(), 3);
    /// ```
    pub fn tier(&self) -> usize {
        self.tier
    }

    /// Set the tier of a `Shop`.
    /// * Use in combination with [`restock`](Shop@restock)
    /// # Example
    /// ```
    /// use saptest::Shop;
    /// let mut shop_default = Shop::default();
    /// // Default is tier 1.
    /// assert_eq!(shop_default.tier(), 1)
    /// // Now is tier 2.
    /// shop_default.set_tier(2);
    /// assert_eq!(shop_default.tier(), 2)
    /// 
    /// // 
    /// ```
    pub fn set_tier(&mut self, tier: usize) -> Result<&mut Self, SAPTestError> {
        if !(MIN_SHOP_TIER..=MAX_SHOP_TIER).contains(&tier) {
            return Err(
                SAPTestError::ShopError {
                    reason: format!("Tier provided ({tier}) is invalid. ({MIN_SHOP_TIER} <= tier <= {MAX_SHOP_TIER})")
                }
            );
        }
        self.tier = tier;
        Ok(self)
    }

    /// Get the number of `ShopItem` slots for `Food`s at the shop's current tier.
    fn food_slots(&self) -> usize {
        let n_foods: usize = if self.tier < 2 { 1 } else { 2 };
        n_foods.saturating_sub(self.foods.len())
    }

    /// Get the number of `ShopItem` slots for `Pet`s at the shop's current tier.
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
    /// * By default, shops are randomly seeded.
    /// ```
    /// use saptest::Shop;
    ///
    /// let shop = Shop::default();
    /// // Set to 42.
    /// shop.set_seed(Some(42));
    /// // Use random seed.
    /// shop.set_seed(None);
    /// ```
    pub fn set_seed(&mut self, seed: Option<u64>) -> &mut Self {
        self.seed = seed;
        self
    }

    pub(crate) fn get_rng(&self) -> ChaCha12Rng {
        let seed = self.seed.unwrap_or_else(random);
        ChaCha12Rng::seed_from_u64(seed)
    }

    /// Fill pets based on current tier of shop.
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
                        reason: format!(
                            "SQL ({sql}) with params ({params:?}) yielded no pet records."
                        ),
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

    /// Fill the shop with foods based on current tier of shop.
    fn fill_foods(&mut self) -> Result<&mut Self, SAPTestError> {
        let (sql, params) = self.shop_query(Entity::Food, 1..self.tier + 1);
        let possible_foods = SAPDB.execute_food_query(&sql, &params)?;
        let mut rng = self.get_rng();

        // Iterate through slots choose a random food.
        for _ in 0..self.food_slots() {
            let food_record =
                possible_foods
                    .choose(&mut rng)
                    .ok_or(SAPTestError::QueryFailure {
                        subject: "Empty Shop Query".to_string(),
                        reason: format!(
                            "SQL ({sql}) with params ({params:?}) yielded no food records."
                        ),
                    })?;
            self.foods.push(ShopItem {
                item: ItemSlot::Food(Food::try_from(food_record.name.clone())?),
                state: ItemState::Normal,
                cost: food_record.cost,
            });
        }

        Ok(self)
    }
}
