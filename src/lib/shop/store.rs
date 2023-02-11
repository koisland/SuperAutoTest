use std::{
    cell::RefCell,
    error::Error,
    ops::Range,
    rc::Weak, collections::VecDeque,
};

use itertools::Itertools;
use petgraph::visit::Dfs;
use rand::prelude::*;
use rand_chacha::ChaCha12Rng;

use crate::{
    battle::{
        effect::Entity,
        state::{Outcome, Target},
        stats::Statistics,
        team::Team,
        trigger::*,
    },
    db::utils::setup_param_query,
    foods::food::Food,
    pets::{names::PetName, pet::Pet},
    TeamEffects, SAPDB, error::SAPTestError,
    shop::trigger::*
};

/// Sloth chance.
const SLOTH_CHANCE: f64 = 0.01;
/// Default coins for player.
const DEFAULT_COIN_COUNT: usize = 10;

/// State of item.
#[derive(Debug, Clone)]
enum ItemState {
    /// Frozen item.
    Frozen,
    /// Discounted item. Inner value represents counts discounted.
    Discounted(usize),
    /// Normal item.
    Normal,
}

/// Shop Pet
#[derive(Debug, Clone)]
struct ShopPet {
    /// Pet
    pet: Pet,
    /// State of pet in shop.
    state: ItemState,
}

/// Shop food.
#[derive(Debug, Clone)]
struct ShopFood {
    /// Food
    food: Food,
    /// State of food in shop.
    state: ItemState,
}

/// A Super Auto Pets shop.
#[derive(Debug)]
pub struct Shop<'a> {
    /// Pets in shop.
    pets: Vec<ShopPet>,
    /// Foods in shop.
    foods: Vec<ShopFood>,
    /// Coins in shop.
    coins: usize,
    /// Current tier of shop.
    tier: usize,
    /// Pointer to `Team`.
    team: Option<&'a mut Team>,
    /// Empty team.
    _empty_team: Team,
    /// Global permanent `Statistics` added to all `Pet`s.
    add_stats: Statistics,
    /// Temporary stats that are removed on shop init.
    temp_stats: Vec<(Weak<RefCell<Pet>>, Statistics)>,
    /// Free rolls.
    free_rolls: usize,

    /// Shop triggers
    triggers: VecDeque<Outcome>
}

impl<'a> Default for Shop<'a> {
    fn default() -> Self {
        Self {
            coins: DEFAULT_COIN_COUNT,
            tier: 1,
            team: None,
            add_stats: Statistics::default(),
            temp_stats: vec![],
            pets: Vec::with_capacity(6),
            foods: Vec::with_capacity(4),
            free_rolls: 0,
            _empty_team: Team::default(),
            triggers: VecDeque::new()
        }
    }
}

impl<'a> Shop<'a> {
    /// Create a new `Shop`.
    /// ```
    /// use sapdb::common::{shop::store::Shop, battle::team::Team};
    ///
    /// let mut starting_team = Team::default();
    /// let shop = Shop::new(&mut starting_team);
    ///
    /// assert_eq!(true, shop.is_ok());
    /// ```
    pub fn new(team: &'a mut Team) -> Result<Self, Box<dyn Error>> {
        let mut default_shop = Shop {
            team: Some(team),
            ..Default::default()
        };
        default_shop
            .set_tier()
            .emit_trigger(TRIGGER_START_TURN)
            .trigger_effects(&[])
            .fill_pets()?
            .fill_foods()?;

        Ok(default_shop)
    }

    /// Get the `Shop` tier.
    /// ```
    /// use sapdb::common::shop::store::Shop;
    ///
    /// let mut shop = Shop::default();
    ///
    /// assert_eq!(shop.tier(), 1)
    /// ```
    pub fn tier(&self) -> usize {
        self.tier
    }

    pub fn view(&self) {}

    pub fn roll(&mut self) -> &mut Self {
        // self.emit_trigger(TRIGGER_ROLL);
        self
    }

    /// Freeze the contents of a `Shop`
    /// ```
    /// use sapdb::common::{shop::store::Shop, battle::team::Team};
    ///
    /// let mut starting_team = Team::default();
    /// let mut shop = Shop::new(&mut starting_team).unwrap();
    ///
    ///
    pub fn freeze(&mut self, item: Entity, pos: usize) -> Result<&mut Self, Box<dyn Error>> {
        match item {
            Entity::Pet => {}
            Entity::Food => {}
        }
        Ok(self)
    }

    pub fn buy_food(&'a mut self, from: usize, to: Option<usize>) -> Result<&mut Shop, SAPTestError> {
        let Some(team_ref) = self.team.as_mut() else {
            return Err(SAPTestError::ShopError { reason: "No team".to_string() });
        };
        let mut buy_triggers = vec![];

        if from < self.foods.len() {
            // Check if can purchase food.
            let shop_food = self.foods.get(from).unwrap();
            if let Some(new_coins) = self.coins.checked_sub(shop_food.food.cost) {
                self.coins = new_coins;
            } else {
                return Err(SAPTestError::ShopError {
                    reason: format!("Not enough coins ({}) to buy item.", self.coins,),
                });
            }

            // Give item to pet if holdable.
            if shop_food.food.holdable {
                if let Some(pet_idx) = to {
                    if let Some(pet) = team_ref.nth(pet_idx) {
                        let mut item = self.foods.remove(from).food;
                        item.ability.assign_owner(Some(&pet));
                        pet.borrow_mut().item = Some(item);
                        
                        // Create trigger if food bought.
                        let mut trigger = TRIGGER_FOOD_BOUGHT;
                        trigger.set_affected(&pet);
                        buy_triggers.push(trigger)
                    } else {
                        return Err(SAPTestError::ShopError {
                            reason: format!("Not a valid index for team ({}).", pet_idx),
                        });
                    }
                }
            } else {
                // Otherwise, take effect and send trigger.
                let item = self.foods.remove(from).food;
                let item_effect = item.ability;

                let prev_node = team_ref.history.curr_node;
                team_ref.apply_effect(
                    &TRIGGER_NONE,
                    &item_effect,
                    &mut self._empty_team,
                )?;

                // Search through nodes.
                if let Some(start_node) = prev_node {
                    let effect_graph = &team_ref.history.effect_graph;
                    let mut dfs = Dfs::new(effect_graph, start_node);

                    // See affected pets.
                    while let Some(Some(node)) = dfs.next(effect_graph).map(|node| effect_graph.node_weight(node)) {
                        let mut trigger = TRIGGER_FOOD_BOUGHT; 
                        trigger.affected_pet = node.affected_pet.clone();
                        buy_triggers.push(trigger);
                    }
                }
                
             
            }
        } else {
            return Err(SAPTestError::ShopError {
                reason: format!("Not valid indices for shop ({}).", from),
            });
        }

        self.trigger_effects(&buy_triggers);

        Ok(self)
    }
    pub fn buy_pet(
        &mut self,
        from: usize,
        to: usize,
    ) -> Result<&mut Self, SAPTestError> {
        let Some(team) = self.team.as_mut() else {
            return Err(SAPTestError::ShopError { reason: "No team".to_string() });
        };

        let mut buy_triggers: Vec<Outcome> = vec![];
        // From index and To index is valid.
        if from < self.pets.len() && to <= team.friends.len()
        {
            // Get pet from shop.
            let shop_pet = self.pets.get(from).unwrap();
            // Buy the item.
            if let Some(new_coins) = self.coins.checked_sub(shop_pet.pet.cost) {
                self.coins = new_coins;
            } else {
                // Not enough to buy so re-add to shop.
                return Err(SAPTestError::ShopError {
                    reason: format!("Not enough coins ({}) to buy pet.", self.coins,),
                });
            }
            // If pet in slot.
            let purchased_pet = if let Some(slot_pet) = team.friends.get(to)
            {
                if slot_pet.borrow().name == shop_pet.pet.name {
                    // Stack pet. Take max attack and health from pet.
                    slot_pet.borrow_mut().stats.attack = slot_pet.borrow()
                        .stats
                        .attack
                        .max(shop_pet.pet.stats.attack);
                    slot_pet.borrow_mut().stats.health = slot_pet.borrow()
                        .stats
                        .health
                        .max(shop_pet.pet.stats.health);
                    slot_pet.borrow_mut().add_experience(1)?;
                    // Remove pet from shop.
                    self.pets.remove(from);
                    Some(slot_pet.clone())
                } else {
                    return Err(SAPTestError::ShopError {
                        reason: format!(
                            "Shop pet at pos {} not the same as pos {:?} on team.",
                            from, to
                        ),
                    });
                }
            } else {
                // Otherwise buying.
                let pet = self.pets.remove(from).pet;
                team.add_pet(pet, to, None)?;
                team.nth(to)
            };

            if let Some(pet) = purchased_pet {
                let mut buy_trigger = TRIGGER_PET_BOUGHT;
                buy_trigger.set_affected(&pet);
                buy_triggers.push(buy_trigger)
            }
        } else {
            return Err(SAPTestError::ShopError {
                reason: format!(
                    "Not valid indices for shop ({}) or team ({:?}).",
                    from, to
                ),
            });
        }
        
        self.trigger_effects(&buy_triggers);

        Ok(self)
    }

    pub fn sell(&mut self, pos: usize) -> &mut Self {
        self
    }

    pub fn close(&mut self) {
        // Emit end of turn
        self.emit_trigger(TRIGGER_END_TURN);

        // restore money.
        self.coins = DEFAULT_COIN_COUNT;
    }

    /// Set the tier of a shop.
    fn set_tier(&mut self) -> &mut Self {
        if let Some(team) = &self.team {
            let n_turns = team.history.curr_turn;
            self.tier = (n_turns / 2 + n_turns % 2).clamp(1, 6);
        }
        self
    }

    fn get_rng(&self) -> ChaCha12Rng {
        let seed = self.team.as_ref().map_or_else(|| random::<u64>(), |team| team.seed);
        ChaCha12Rng::seed_from_u64(seed)
    }

    fn food_slots(&self) -> usize {
        if self.tier < 2 {
            1
        } else {
            2
        }
    }
    /// Get the number of `Pet` slots for a shop.
    fn pet_slots(&self) -> usize {
        if self.tier < 3 {
            3
        } else if self.tier < 5 {
            4
        } else {
            5
        }
    }

    fn match_position() {}

    fn emit_trigger(&mut self, trigger: Outcome) -> &mut Self {
        if let Some(team) = self.team.as_mut() {
            team.triggers.push_back(trigger);
        }

        self
    }

    fn trigger_effects(&mut self, buy_triggers: &[Outcome]) -> &mut Self {
        if let Some(team) = self.team.as_mut() {

            let all_effects = team.get_effects();
            // Activate shop triggers.
            for trigger in buy_triggers.iter() {
                // Check team pet effects.
                for effects in all_effects.iter() {
                    for effect in effects {
                        if trigger == &effect.trigger && effect.target == Target::Shop {
                            team.apply_effect(trigger, effect, &mut self._empty_team).unwrap();
                        }
                    }
                }
            }

            team.trigger_effects(&mut self._empty_team);
            // TODO: Check history for temp effect targets and get reference to those pets.
        }
        self
    }

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
                setup_param_query("pets", &named_params)
            }
            Entity::Food => {
                let named_params = [("tier", &params[0]), ("pack", &params[1])];
                flat_param.pop();
                setup_param_query("foods", &named_params)
            }
        };

        (stmt, flat_param)
    }

    fn fill_pets(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        let (sql, params) = self.shop_query(Entity::Pet, 1..self.tier + 1);
        let possible_pets = SAPDB.execute_pet_query(&sql, &params)?;
        let mut rng = self.get_rng();

        // Iterate through slots choose a random pet or sloth.
        for i in 0..self.pet_slots() {
            if rng.gen_bool(SLOTH_CHANCE) {
                let sloth = Pet::try_from(PetName::Sloth)?;
                self.pets.insert(
                    i,
                    ShopPet {
                        pet: sloth,
                        state: ItemState::Normal,
                    },
                );
            } else if let Some(pet_record) = possible_pets.choose(&mut rng) {
                self.pets.insert(
                    i,
                    ShopPet {
                        pet: Pet::try_from(pet_record.name.clone())?,
                        state: ItemState::Normal,
                    },
                );
            }
        }

        Ok(self)
    }

    fn fill_foods(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        let (sql, params) = self.shop_query(Entity::Food, 1..self.tier + 1);
        let possible_foods = SAPDB.execute_food_query(&sql, &params)?;
        let mut rng = self.get_rng();

        // Iterate through slots choose a random food.
        for i in 0..self.food_slots() {
            if let Some(food_record) = possible_foods.choose(&mut rng) {
                self.foods.insert(
                    i,
                    ShopFood {
                        food: Food::try_from(food_record.name.clone())?,
                        state: ItemState::Normal,
                    },
                );
            }
        }

        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use super::Shop;
    use crate::{battle::effect::Entity, Team};
    #[test]
    fn test_shop() {
        let mut team = Team::default();
        let mut shop = Shop::new(&mut team).unwrap();
        shop.buy_pet(0, 2)
            .unwrap()
            .buy_pet(1, 0)
            .unwrap()
            // .roll()
            // .sell(0)
            .close();
    }
}
