use crate::{
    db::record::{FoodRecord, PetRecord},
    effects::{
        actions::{
            Action, ConditionType, CopyType, GainType, LogicType, RandomizeType, StatChangeType,
            SummonType,
        },
        effect::{Effect, Entity, Modify},
        state::{ItemCondition, Outcome, Position, ShopCondition, Status, Target, TeamCondition},
        stats::Statistics,
        trigger::*,
    },
    error::SAPTestError,
    pets::{
        names::PetName,
        pet::{MAX_PET_STATS, MIN_PET_STATS},
    },
    shop::{store::ShopState, trigger::*},
    teams::{
        team::Team,
        viewer::{TargetPets, TeamViewer},
    },
    Food, Pet, PetCombat, ShopItem, ShopViewer, TeamShopping, SAPDB,
};

use itertools::Itertools;
use log::info;
use rand::{
    random,
    seq::{IteratorRandom, SliceRandom},
    SeedableRng,
};
use rand_chacha::ChaCha12Rng;
use std::{cell::RefCell, collections::VecDeque, fmt::Write, rc::Rc};

const NON_BATTLE_TRIGGERS: [Outcome; 11] = [
    TRIGGER_ANY_LEVELUP,
    TRIGGER_START_TURN,
    TRIGGER_START_BATTLE,
    TRIGGER_BEFORE_FIRST_BATTLE,
    TRIGGER_END_TURN,
    TRIGGER_ANY_FOOD_BOUGHT,
    TRIGGER_ANY_FOOD_EATEN,
    TRIGGER_ANY_PET_BOUGHT,
    TRIGGER_ANY_PET_SOLD,
    TRIGGER_ROLL,
    TRIGGER_SHOP_TIER_UPGRADED,
];

fn is_nonspecific_position(pos: &Position) -> bool {
    matches!(
        pos,
        Position::Any(_) | Position::All(_) | Position::None | Position::Relative(_)
    )
}

/// Used to check if pet Effect is triggered.
/// * Exact match of trigger.
/// * Nonspecific position match as well as other equalities.
/// * Not out of uses
fn trigger_activates_effect(effect: &Effect, trigger: &Outcome) -> bool {
    (&effect.trigger == trigger
        // This bottom condition allows triggers for effects that activate on any position/positions. ex. Horse.
        || (is_nonspecific_position(&effect.trigger.position)
            && effect.trigger.position == trigger.position
            && effect.trigger.affected_team == trigger.affected_team
            && effect.trigger.status == trigger.status))
        && effect.uses != Some(0)
}

/// Used to ignore an effect if its trigger fits a set of conditions.
///  * Trigger for an [`Effect`] with an [`Action::Summon`] is a [`ZombieFly`](crate::PetName::ZombieFly).
///  * Trigger for an [`Effect`] with an [`Action::Summon`] is a [`Fly`](crate::PetName::Fly) and is also the current pet is that [`Fly`](crate::PetName::Fly).
///  * The pet causing the trigger is the same as the pet being checked for effects and the triggers targets [`Position::Any`](crate::Position::Any).
fn is_pet_effect_exception(
    trigger: &Outcome,
    trigger_petname: Option<&PetName>,
    pet_effect: &Effect,
    same_pet_as_trigger: bool,
) -> bool {
    if let Some(trigger_pet_name) = trigger_petname {
        if let Action::Summon(_) = pet_effect.action {
            *trigger_pet_name == PetName::ZombieFly
                || (*trigger_pet_name == PetName::Fly && same_pet_as_trigger)
        } else if let Action::Add(_) = pet_effect.action {
            trigger.position == Position::Any(ItemCondition::None) && same_pet_as_trigger
        } else {
            false
        }
    } else {
        false
    }
}

/// Enable applying [`Effect`]s to multiple [`Team`]s.
/// ```rust no run
/// use saptest::TeamEffects;
/// ```
pub trait TeamEffects {
    /// Trigger the start of battle for two [`Team`]s.
    /// * Invocation order does not matter.
    ///     * `team.trigger_start_battle_effects(&mut enemy_team)` or its reverse will not alter the outcome.
    /// * This takes all [`Pet`]s into consideration unlike [`trigger_effects`](TeamEffects::trigger_effects) which only activates effects from a single [`Team`].
    /// * Triggers are not exhausted after executing the start of battle.
    /// # Example
    /// ```
    /// use saptest::{
    ///     Team, TeamEffects, TeamViewer,
    ///     Pet, PetName, Statistics
    /// };
    ///
    /// // Dolphin at base (4,3).
    /// let dolphin = Pet::try_from(PetName::Dolphin).unwrap();
    /// // Caterpillar at level 3 with (3,4). Activates after dolphin.
    /// let mut caterpillar = Pet::try_from(PetName::Caterpillar).unwrap();
    /// caterpillar.set_level(3).unwrap();
    /// caterpillar.stats = Statistics::new(3,4).unwrap();
    ///
    /// let mut team = Team::new(&[Some(caterpillar)], 5).unwrap();
    /// let mut enemy_team = Team::new(&[Some(dolphin)], 5).unwrap();
    /// team.trigger_start_battle_effects(&mut enemy_team).unwrap();
    ///
    /// let butterfly = team.first().unwrap();
    /// assert!(
    ///     butterfly.borrow().stats == Statistics {attack: 1, health: 1} &&
    ///     butterfly.borrow().name == PetName::Butterfly
    /// )
    /// ```
    fn trigger_start_battle_effects(
        &mut self,
        opponent: &mut Team,
    ) -> Result<&mut Self, SAPTestError>;

    /// Apply [`Pet`](crate::pets::pet::Pet) [`Effect`]s based on a team's stored [`Outcome`] triggers.
    /// * **Note**: This only applies effects on a **single** [`Team`].
    /// * Start of battle effects should be handled by [`trigger_start_battle_effects`](TeamEffects::trigger_start_battle_effects).
    ///     * Non-opponent affecting effects can be activated by adding a [`TRIGGER_START_BATTLE`](crate::effects::trigger::TRIGGER_START_BATTLE) to the team.
    /// * Pet effects activate first followed by food effects.
    /// * This exhausts all effect [`Outcome`] triggers.
    /// # Example
    /// ```rust
    /// use saptest::{
    ///     TeamEffects, Team, TeamViewer,
    ///     Pet, PetName,
    ///     effects::{state::Status, trigger::{TRIGGER_START_BATTLE, TRIGGER_SELF_HURT}}
    /// };
    ///
    /// let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// let mut team = Team::new(&vec![Some(mosquito); 5], 5).unwrap();
    /// let mut enemy_team = team.clone();
    ///
    /// // Add a start of battle trigger.
    /// team.triggers.push_front(TRIGGER_START_BATTLE);
    /// // Trigger effects.
    /// team.trigger_effects(Some(&mut enemy_team)).unwrap();
    ///
    /// // Triggers exhausted.
    /// // Enemy team hurt by mosquito barrage.
    /// assert_eq!(team.triggers.len(), 0);
    /// assert!(enemy_team.triggers.iter().any(|trigger| matches!(trigger.status, Status::Hurt)));
    /// ```
    fn trigger_effects(&mut self, opponent: Option<&mut Team>) -> Result<&mut Self, SAPTestError>;

    /// Apply an [`Effect`] with an associated [`Outcome`] trigger to a [`Team`].
    /// * The `opponent` [`Team`] will get updated with additional [`Outcome`]s.
    /// * Effects and triggers should contain a Weak reference to the owning/affecting pet.
    /// # Examples
    /// ```rust
    /// use saptest::{
    ///     TeamEffects, Team, TeamViewer,
    ///     Pet, PetName,
    ///     Statistics, effects::{state::Status, trigger::*}
    /// };
    /// // Get mosquito effect.
    /// let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// // Get effect with no reference.
    /// let no_ref_mosquito_effect = mosquito.effect.first().cloned().unwrap();
    ///
    /// // Init teams.
    /// let mut team = Team::new(&vec![Some(mosquito.clone()); 5], 5).unwrap();
    /// let mut enemy_team = Team::new(&vec![Some(mosquito); 5], 5).unwrap();
    /// enemy_team.set_seed(Some(0));
    ///
    /// // Without a reference to the pet owning the effect, this will fail.
    /// assert!(team.apply_effect(&TRIGGER_START_BATTLE, &no_ref_mosquito_effect, Some(&mut enemy_team)).is_err());
    ///
    /// // Get mosquito_effect with reference.
    /// // Apply effect of mosquito at position 0 to a pet on team to enemy team.
    /// let mosquito_effect = team.first().unwrap().borrow().effect.first().cloned().unwrap();
    /// team.apply_effect(&TRIGGER_START_BATTLE, &mosquito_effect, Some(&mut enemy_team)).unwrap();
    ///
    /// // Last enemy mosquito takes one damage and opponent triggers gets updated.
    /// assert_eq!(
    ///     enemy_team.nth(4).unwrap().borrow().stats,
    ///     Statistics::new(2, 1).unwrap()
    /// );
    /// assert!(
    ///     enemy_team.triggers
    ///     .iter()
    ///     .find(|trigger| trigger.status == Status::Hurt)
    ///     .is_some()
    /// );
    /// ```
    fn apply_effect(
        &mut self,
        trigger: &Outcome,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError>;

    /// Get effect order for a single [`Team`].
    /// * Order is found by going from highest to lowest [`Pet`] attack.
    /// * If in battle:
    ///     * The first pet on the team is always first in effect priority.
    fn get_pet_effect_order(&self, in_battle: bool) -> Vec<Rc<RefCell<Pet>>>;
}

impl TeamEffects for Team {
    fn get_pet_effect_order(&self, in_battle: bool) -> Vec<Rc<RefCell<Pet>>> {
        let mut ordered_pets = self
            .friends
            .iter()
            .flatten()
            .sorted_by(|pet_1, pet_2| {
                pet_1
                    .borrow()
                    .stats
                    .attack
                    .cmp(&pet_2.borrow().stats.attack)
            })
            .rev()
            .cloned()
            .collect_vec();

        let curr_pet = self.curr_pet.as_ref().map(|pet| pet.upgrade());

        if let Some(Some(first_pet)) = curr_pet {
            if in_battle {
                // Remove first pet from ordered pets.
                ordered_pets.retain(|pet| !Rc::ptr_eq(pet, &first_pet));
                // And insert as first. Current pet always activates effect first.
                ordered_pets.insert(0, first_pet);
            }
        }
        ordered_pets
    }

    fn trigger_start_battle_effects(
        &mut self,
        opponent: &mut Team,
    ) -> Result<&mut Self, SAPTestError> {
        let self_pets = self
            .friends
            .iter()
            .flatten()
            .map(|pet| (Target::Friend, pet));
        let opponent_pets = opponent
            .friends
            .iter()
            .flatten()
            .map(|pet| (Target::Enemy, pet));

        let mut activated_effects: Vec<(Target, Effect)> = vec![];
        for (team, pet) in self_pets
            .chain(opponent_pets)
            .sorted_by(|(_, pet_1), (_, pet_2)| {
                pet_1
                    .borrow()
                    .stats
                    .attack
                    .cmp(&pet_2.borrow().stats.attack)
            })
            .rev()
        {
            let effect_pet_idx = pet.borrow().pos.ok_or(SAPTestError::InvalidTeamAction {
                subject: "No Pet Position Index.".to_string(),
                reason: format!("Pet {} must have an index set at this point.", pet.borrow()),
            })?;

            // Do not need to mutate to reduce uses as start of battle should only occur once.
            let start_of_battle_effects = pet
                .borrow()
                .effect
                .iter()
                .filter_map(|effect| {
                    if effect.trigger.status == Status::StartOfBattle {
                        Some((team, effect.clone()))
                    } else {
                        None
                    }
                })
                .collect_vec();

            // Check for tiger effects.
            let tiger_effects = match team {
                Target::Friend => self.repeat_effects_if_tiger(
                    effect_pet_idx,
                    pet,
                    &TRIGGER_START_BATTLE,
                    None,
                    false,
                )?,
                Target::Enemy => opponent.repeat_effects_if_tiger(
                    effect_pet_idx,
                    pet,
                    &TRIGGER_START_BATTLE,
                    None,
                    false,
                )?,
                _ => unreachable!("Not possible to get other targets."),
            };

            activated_effects.extend(start_of_battle_effects);
            activated_effects.extend(tiger_effects.into_iter().map(|effect| (team, effect)))
        }
        for (team, effect) in activated_effects.iter() {
            match team {
                Target::Friend => {
                    self.apply_effect(&TRIGGER_START_BATTLE, effect, Some(opponent))?
                }
                Target::Enemy => {
                    opponent.apply_effect(&TRIGGER_START_BATTLE, effect, Some(self))?
                }
                _ => unreachable!("Not possible to have other targets."),
            }
        }

        Ok(self)
    }

    fn trigger_effects(
        &mut self,
        mut opponent: Option<&mut Team>,
    ) -> Result<&mut Self, SAPTestError> {
        info!(target: "run", "(\"{}\")\nTriggers:\n{}", self.name, self.triggers.iter().join("\n"));

        let mut item_effect_triggers: VecDeque<Outcome> = VecDeque::new();
        // Continue iterating until all triggers consumed activating pet effects.
        // Each trigger produced is run again but through item effects.
        while let Some(trigger) = self.triggers.pop_front() {
            let mut applied_effects: Vec<Effect> = vec![];

            // Get petname and position of trigger.
            let (trigger_pet_name, trigger_pet_pos) = if let Some(Some(trigger_pet)) =
                trigger.affected_pet.as_ref().map(|pet| pet.upgrade())
            {
                (
                    Some(trigger_pet.borrow().name.clone()),
                    trigger_pet.borrow().pos,
                )
            } else {
                (None, None)
            };

            // Determine pet order of effects on team.
            let ordered_pets = self.get_pet_effect_order(!NON_BATTLE_TRIGGERS.contains(&trigger));

            // Iterate through pets in descending order by attack strength to collect valid effects.
            for pet in ordered_pets.iter() {
                let effect_pet_idx = pet.borrow().pos.ok_or(SAPTestError::InvalidTeamAction {
                    subject: "No Pet Position Index.".to_string(),
                    reason: format!("Pet {} must have an index set at this point.", pet.borrow()),
                })?;
                let same_pet_as_trigger = trigger
                    .clone()
                    .affected_pet
                    .map_or(false, |trigger_pet| trigger_pet.ptr_eq(&Rc::downgrade(pet)));

                let valid_effects = pet
                    .borrow_mut()
                    .effect
                    .iter_mut()
                    .filter(|effect| trigger_activates_effect(effect, &trigger))
                    .filter_map(|effect| {
                        if is_pet_effect_exception(
                            &trigger,
                            trigger_pet_name.as_ref(),
                            effect,
                            same_pet_as_trigger,
                        ) {
                            None
                        } else {
                            // Drop uses by one if possible.
                            effect.remove_uses(1);
                            Some(effect.clone())
                        }
                    })
                    .collect_vec();

                // Check if tiger should activate.
                let tiger_effects = self.repeat_effects_if_tiger(
                    effect_pet_idx,
                    pet,
                    &trigger,
                    trigger_pet_name.as_ref(),
                    same_pet_as_trigger,
                )?;

                applied_effects.extend(valid_effects);
                applied_effects.extend(tiger_effects);
            }

            // Pet sold. Remove pet from friends.
            // Keep reference alive until rest of effects activated.
            let _sold_pet = if (&trigger.status, &trigger.position, &trigger.affected_team)
                == (&Status::Sell, &Position::OnSelf, &Target::Friend)
            {
                if let Some(pet_pos) = trigger_pet_pos {
                    let pet = self.friends.remove(pet_pos);
                    self.friends.insert(pet_pos, None);
                    Some(pet)
                } else {
                    None
                }
            } else {
                None
            };

            for effect in applied_effects.into_iter() {
                // Add node here for activated effect.
                let node_idx = self.history.effect_graph.add_node(trigger.clone());
                self.history.curr_node = Some(node_idx);
                if let Some(opponent) = opponent.as_mut() {
                    self.apply_effect(&trigger, &effect, Some(opponent))?
                } else {
                    self.apply_effect(&trigger, &effect, None)?
                }
            }

            // Set curr node to previous.
            self.history.prev_node = self.history.curr_node;
            item_effect_triggers.push_front(trigger)
        }

        // Need to iterate twice as items need to be last to activate.
        while let Some(trigger) = item_effect_triggers.pop_front() {
            let mut applied_effects: Vec<Effect> = vec![];
            let ordered_pets = self.get_pet_effect_order(!NON_BATTLE_TRIGGERS.contains(&trigger));

            for pet in ordered_pets.iter() {
                // Get food and pet effect based on if its trigger is equal to current trigger, if any.
                if let Some(food) = pet
                    .borrow_mut()
                    .item
                    .as_mut()
                    .filter(|food| trigger_activates_effect(&food.ability, &trigger))
                {
                    // Drop uses by one if possible.
                    food.ability.remove_uses(1);
                    applied_effects.push(food.ability.clone())
                };
            }

            for effect in applied_effects.into_iter() {
                // Add node here for activated effect.
                let node_idx = self.history.effect_graph.add_node(trigger.clone());
                self.history.curr_node = Some(node_idx);
                if let Some(opponent) = opponent.as_mut() {
                    self.apply_effect(&trigger, &effect, Some(opponent))?
                } else {
                    self.apply_effect(&trigger, &effect, None)?
                }
            }

            // Set curr node to previous.
            self.history.prev_node = self.history.curr_node;
        }

        Ok(self)
    }

    fn apply_effect(
        &mut self,
        trigger: &Outcome,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError> {
        // Set current pet.
        self.curr_pet = effect.owner.clone();

        match (&effect.target, &effect.action) {
            (_, Action::Swap(swap_type)) => {
                let target_pets = if let Some(opponent) = opponent.as_ref() {
                    self.get_pets_by_effect(trigger, effect, Some(opponent))?
                } else {
                    self.get_pets_by_effect(trigger, effect, None)?
                };

                if target_pets.len() != 2 {
                    return Err(SAPTestError::InvalidTeamAction {
                        subject: format!("Swap {swap_type:?}"),
                        reason: format!(
                            "Only two friends allowed for swapping. Given: {}",
                            target_pets.len()
                        ),
                    });
                }

                // Safe to unwrap.
                let ((_, pet_1), (_, pet_2)) =
                    (target_pets.first().unwrap(), target_pets.get(1).unwrap());
                match swap_type {
                    RandomizeType::Positions => {
                        pet_1.borrow_mut().swap(&mut pet_2.borrow_mut());
                        self
                    }
                    RandomizeType::Stats => {
                        pet_1.borrow_mut().swap_stats(&mut pet_2.borrow_mut());
                        self
                    }
                };
            }
            // Must be here to only activate once.
            (target_team, Action::Shuffle(shuffle_by)) => {
                let teams = match target_team {
                    Target::Friend => vec![Some(self)],
                    Target::Enemy => vec![opponent],
                    Target::Either => vec![Some(self), opponent],
                    _ => unimplemented!("Cannot shuffle on given target."),
                };
                for team in teams.into_iter().flatten() {
                    let mut rng = ChaCha12Rng::seed_from_u64(team.seed.unwrap_or_else(random));
                    match shuffle_by {
                        RandomizeType::Positions => {
                            team.friends.shuffle(&mut rng);
                            team.set_indices();
                        }
                        RandomizeType::Stats => {
                            for pet in team.friends.iter().flatten() {
                                pet.borrow_mut().stats.invert();
                            }
                        }
                    }
                }
            }
            // All shop actions go here.
            (Target::Shop, _) => self.apply_shop_effect(effect)?,
            _ => {
                let target_pets = if let Some(opponent) = opponent.as_ref() {
                    self.get_pets_by_effect(trigger, effect, Some(opponent))?
                } else {
                    self.get_pets_by_effect(trigger, effect, None)?
                };

                if let Some(opponent) = opponent {
                    for (team, pet) in target_pets.into_iter() {
                        match team {
                            Target::Friend => {
                                self.apply_single_effect(pet, effect, Some(opponent))?
                            }
                            Target::Enemy => {
                                opponent.apply_single_effect(pet, effect, Some(self))?
                            }
                            _ => unreachable!("Should never reach this branch as only team and enemy team allowed."),
                        };
                    }
                } else {
                    for (_, pet) in target_pets.into_iter() {
                        self.apply_single_effect(pet, effect, None)?
                    }
                }
            }
        }

        Ok(())
    }
}
pub(crate) trait EffectApplyHelpers {
    /// Apply an `Action` to a target idx on a `Team`.
    fn apply_single_effect(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError>;

    /// Apply shop effects.
    fn apply_shop_effect(&mut self, effect: &Effect) -> Result<(), SAPTestError>;

    fn copy_effect(
        &self,
        attr_to_copy: &CopyType,
        targets: TargetPets,
        receiving_pet: &Rc<RefCell<Pet>>,
    ) -> Result<(), SAPTestError>;

    fn convert_gain_type_to_food(
        &self,
        gain_type: &GainType,
        target_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Option<Food>, SAPTestError>;

    fn convert_summon_type_to_pet(
        &self,
        summon_type: &SummonType,
        target_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Pet, SAPTestError>;

    fn summon_pet(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        summon_type: &SummonType,
        opponent: Option<&mut Team>,
    ) -> Result<Option<String>, SAPTestError>;

    fn get_matching_pets(
        &self,
        target: &Target,
        condition: &ItemCondition,
        opponent: &Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError>;

    fn check_condition(
        &self,
        condition_type: &ConditionType,
        target_pet: &Rc<RefCell<Pet>>,
        opponent: &Option<&mut Team>,
    ) -> Result<bool, SAPTestError>;

    fn apply_conditional_pet_action(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        condition_type: &LogicType,
        effect: &Effect,
        action: &Action,
        opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError>;

    /// Hard-coded [`Tiger`](crate::PetName::Tiger) behavior.
    /// * Checks that pet behind current pet is a tiger.
    /// * Determines if [`Effect`] is valid by same methods in [`trigger_effects`](TeamEffects::trigger_effects).
    /// * Creates effects of `pet` at given tiger level.
    fn repeat_effects_if_tiger(
        &self,
        pet_idx: usize,
        pet: &Rc<RefCell<Pet>>,
        trigger: &Outcome,
        trigger_petname: Option<&PetName>,
        same_pet_as_trigger: bool,
    ) -> Result<Vec<Effect>, SAPTestError>;

    /// Hard-coded [`Whale`](crate::PetName::Whale) behavior.
    fn evolve_pet(
        &mut self,
        lvl: usize,
        targets: TargetPets,
        target_pet: Rc<RefCell<Pet>>,
        opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError>;

    /// Calculates an adjusted index based on the current index and a relative index.
    /// * `:param curr_idx:` The current index.
    /// * `:param rel_idx:` Number of positions relative to the current index.
    ///     * If *negative*, the index is **behind** the current index.
    ///     * If *positive*, the index is **ahead** of the current index.
    ///
    /// Output:
    /// * Value of the new index on a team represented by a variant in the enum `Target`.
    fn cvt_rel_idx_to_adj_idx(
        &self,
        curr_idx: usize,
        rel_idx: isize,
    ) -> Result<(Target, usize), SAPTestError>;
    fn cvt_pos_to_idx(&self, pos: &Position) -> Option<usize>;
}

impl EffectApplyHelpers for Team {
    fn repeat_effects_if_tiger(
        &self,
        pet_idx: usize,
        pet: &Rc<RefCell<Pet>>,
        trigger: &Outcome,
        trigger_petname: Option<&PetName>,
        same_pet_as_trigger: bool,
    ) -> Result<Vec<Effect>, SAPTestError> {
        let mut tiger_doubled_effects = vec![];
        // For Tiger. Check if behind.
        if let Some(Some(pet_behind)) = self.friends.get(pet_idx + 1) {
            if pet_behind.borrow().name == PetName::Tiger && self.shop.state == ShopState::Closed {
                // Get effect at level of tiger and repeat it.
                for mut effect in pet
                    .borrow()
                    .get_effect(pet_behind.borrow().lvl)?
                    .into_iter()
                    .filter(|effect| {
                        trigger_activates_effect(effect, trigger)
                            && !is_pet_effect_exception(
                                trigger,
                                trigger_petname,
                                effect,
                                same_pet_as_trigger,
                            )
                    })
                {
                    effect.assign_owner(Some(pet));
                    tiger_doubled_effects.push(effect)
                }
            }
        };
        Ok(tiger_doubled_effects)
    }
    fn convert_gain_type_to_food(
        &self,
        gain_type: &GainType,
        target_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Option<Food>, SAPTestError> {
        Ok(match gain_type {
            GainType::SelfItem => target_pet.borrow().item.clone(),
            GainType::DefaultItem(food_name) => Some(Food::try_from(food_name)?),
            GainType::StoredItem(food) => Some(*food.clone()),
            GainType::RandomShopItem => {
                let (sql, params) = self.shop.shop_query(Entity::Food, 1..self.shop.tier() + 1);
                self.convert_gain_type_to_food(&GainType::QueryItem(sql, params), target_pet)?
            }
            GainType::QueryItem(query, params) => {
                let food_records: Vec<FoodRecord> = SAPDB.execute_food_query(query, params)?;
                let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));
                // Only select one pet.
                let food_record =
                    food_records
                        .choose(&mut rng)
                        .ok_or(SAPTestError::QueryFailure {
                            subject: "Food Query".to_string(),
                            reason: format!("No record found for query: {query} with {params:?}"),
                        })?;
                Some(Food::try_from(food_record.name.clone())?)
            }
            GainType::NoItem => None,
        })
    }
    fn convert_summon_type_to_pet(
        &self,
        summon_type: &SummonType,
        target_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Pet, SAPTestError> {
        match summon_type {
            SummonType::QueryPet(sql, params, stats) => {
                let pet_records: Vec<PetRecord> = SAPDB.execute_pet_query(sql, params)?;
                let mut rng =
                    ChaCha12Rng::seed_from_u64(target_pet.borrow().seed.unwrap_or_else(random));
                // Only select one pet.
                let pet_record =
                    pet_records
                        .choose(&mut rng)
                        .ok_or(SAPTestError::QueryFailure {
                            subject: "Summon Query".to_string(),
                            reason: format!("No record found for query: {sql} with {params:?}"),
                        })?;
                let mut pet = Pet::try_from(pet_record.clone())?;
                // Set stats if some value provided.
                if let Some(set_stats) = stats {
                    pet.stats = *set_stats;
                }
                Ok(pet)
            }
            SummonType::StoredPet(box_pet) => Ok(*box_pet.clone()),
            SummonType::DefaultPet(default_pet) => Pet::try_from(default_pet.clone()),
            SummonType::CustomPet(name, stat_types, lvl) => {
                let mut stats = match stat_types {
                    StatChangeType::StaticValue(stats) => *stats,
                    StatChangeType::SelfMultValue(stats) => {
                        target_pet.borrow().stats.mult_perc(stats)
                    }
                };
                Pet::new(
                    name.clone(),
                    None,
                    Some(stats.clamp(1, MAX_PET_STATS).to_owned()),
                    *lvl,
                )
            }
            SummonType::SelfPet(stats) => {
                // Current pet. Remove item
                let mut pet = target_pet.borrow().clone();
                if let Some(id) = pet.id.as_mut() {
                    write!(id, "_copy").unwrap();
                }
                pet.item = None;
                pet.stats = *stats;
                Ok(pet)
            }
            SummonType::SelfTierPet => {
                let summon_query_type = SummonType::QueryPet(
                    "SELECT * FROM pets where tier = ?".to_string(),
                    vec![target_pet.borrow().tier.to_string()],
                    None,
                );
                self.convert_summon_type_to_pet(&summon_query_type, target_pet)
            }
        }
    }
    fn summon_pet(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        summon_type: &SummonType,
        opponent: Option<&mut Team>,
    ) -> Result<Option<String>, SAPTestError> {
        // Can't impl TryFrom because requires target pet.
        let pet = self.convert_summon_type_to_pet(summon_type, &target_pet)?;

        let target_idx = target_pet
            .borrow()
            .pos
            .ok_or(SAPTestError::InvalidTeamAction {
                subject: "Missing Summon Position".to_string(),
                reason: format!("Target pet {} has no position.", target_pet.borrow()),
            })?;

        // Handle case where pet in front faints and vector is empty.
        // Would panic attempting to insert at any position not at 0.
        // Also update position to be correct.
        let adj_target_idx = if target_idx > self.friends.len() {
            0
        } else {
            target_idx
        };

        self.add_pet(pet, adj_target_idx, opponent)?;
        let added_pet = self.friends.get(adj_target_idx);
        let new_pet_id = added_pet
            .unwrap()
            .as_ref()
            .and_then(|item| item.borrow().id.clone());
        Ok(new_pet_id)
    }

    fn evolve_pet(
        &mut self,
        lvl: usize,
        targets: TargetPets,
        target_pet: Rc<RefCell<Pet>>,
        opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError> {
        let (_, chosen_pet) = targets.first().ok_or(SAPTestError::InvalidTeamAction {
            subject: "Evolve Pet".to_string(),
            reason: "No pet found to evolve.".to_string(),
        })?;

        // Clone the pet, upgrade the chosen pet's abilities, and remove its item.
        let mut leveled_pet = chosen_pet.borrow().clone();
        leveled_pet.set_level(lvl)?;
        leveled_pet.item = None;

        // Kill the original pet.
        chosen_pet.borrow_mut().stats.health = 0;
        info!(target: "run", "(\"{}\")\nKilled pet {}.", self.name, chosen_pet.borrow());

        // Add death triggers.
        let mut self_faint_triggers = get_self_faint_triggers(&None);
        let mut enemy_self_faint_triggers = get_self_enemy_faint_triggers(&None);
        for trigger in self_faint_triggers
            .iter_mut()
            .chain(enemy_self_faint_triggers.iter_mut())
        {
            trigger.set_affected(chosen_pet);
        }
        self.triggers.extend(self_faint_triggers);
        if let Some(opponent) = opponent {
            opponent.triggers.extend(enemy_self_faint_triggers);
        }

        // Set the target's pet ability to summon the pet.
        let target_pet_ref = Rc::downgrade(&target_pet);
        let mut target_pet_trigger = TRIGGER_SELF_FAINT;
        target_pet_trigger.affected_pet = Some(target_pet_ref.clone());
        target_pet.borrow_mut().effect = vec![Effect {
            owner: Some(target_pet_ref),
            entity: Entity::Pet,
            trigger: target_pet_trigger,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Summon(SummonType::StoredPet(Box::new(leveled_pet.clone()))),
            uses: Some(1),
            temp: true,
        }];
        info!(target: "run", "(\"{}\")\nEvolving {}.", self.name, leveled_pet);
        info!(target: "run", "(\"{}\")\nSet pet {} to summon evolved pet on faint.", self.name, target_pet.borrow());
        Ok(())
    }

    fn copy_effect(
        &self,
        attr_to_copy: &CopyType,
        targets: TargetPets,
        receiving_pet: &Rc<RefCell<Pet>>,
    ) -> Result<(), SAPTestError> {
        // Choose the first pet.
        let copied_attr = if let Some((_, pet_to_copy)) = targets.first() {
            match attr_to_copy.clone() {
                CopyType::Stats(replacement_stats) => Some(CopyType::Stats(
                    replacement_stats.map_or(Some(pet_to_copy.borrow().stats), Some),
                )),
                CopyType::PercentStats(perc_stats_mult) => {
                    // Multiply the stats of a chosen pet by some multiplier
                    let mut new_stats = pet_to_copy.borrow().stats.mult_perc(&perc_stats_mult);
                    new_stats.clamp(MIN_PET_STATS, MAX_PET_STATS);
                    info!(
                        target: "run", "(\"{}\")\nCopied {}% atk and {}% health from {}.",
                        self.name,
                        perc_stats_mult.attack,
                        perc_stats_mult.health,
                        receiving_pet.borrow()
                    );
                    Some(CopyType::Stats(Some(new_stats)))
                }
                CopyType::Effect(_, lvl) => Some(CopyType::Effect(
                    pet_to_copy.borrow().get_effect(lvl.unwrap_or(1))?,
                    lvl,
                )),
                CopyType::Item(_) => pet_to_copy
                    .borrow()
                    .item
                    .as_ref()
                    .map(|food| CopyType::Item(Some(Box::new(food.clone())))),
                _ => None,
            }
        } else {
            None
        };

        // Chose the target of recipient of copied pet stats/effect.
        // Calculate stats or set ability.
        match copied_attr.unwrap_or(CopyType::None) {
            CopyType::Stats(new_stats) => {
                // If some stats given use those as base.
                let new_stats = if let Some(mut new_stats) = new_stats {
                    // If any stat value is 0, use the target's original stats, otherwise, use the new stats.
                    *new_stats.comp_set_value(&receiving_pet.borrow().stats, 0)
                } else {
                    // Otherwise, copy stats from target.
                    receiving_pet.borrow().stats
                };

                receiving_pet.borrow_mut().stats = new_stats;

                info!(
                    target: "run", "(\"{}\")\nSet stats for {} to {}.",
                    self.name,
                    receiving_pet.borrow(),
                    receiving_pet.borrow().stats
                );
            }
            CopyType::Effect(mut effects, _) => {
                for effect in effects.iter_mut() {
                    effect.assign_owner(Some(receiving_pet));
                }
                receiving_pet.borrow_mut().effect = effects;

                info!(
                    target: "run", "(\"{}\")\nSet effect for {} to {:?}.",
                    self.name,
                    receiving_pet.borrow(),
                    receiving_pet.borrow().effect
                );
            }
            CopyType::Item(item) => {
                if let Some(mut food) = item {
                    // Assign ability owner to target_pet.
                    food.ability.assign_owner(Some(receiving_pet));

                    receiving_pet.borrow_mut().item = Some(*food);
                    info!(
                        target: "run", "(\"{}\")\nCopyied item for {} to {:?}.",
                        self.name,
                        receiving_pet.borrow(),
                        receiving_pet.borrow().item
                    );
                }
            }
            CopyType::None => {}
            CopyType::PercentStats(_) => {}
        }
        Ok(())
    }

    fn check_condition(
        &self,
        condition_type: &ConditionType,
        target_pet: &Rc<RefCell<Pet>>,
        opponent: &Option<&mut Team>,
    ) -> Result<bool, SAPTestError> {
        match condition_type {
            ConditionType::Pet(target, cond) => Ok(self
                .get_matching_pets(target, cond, opponent)?
                .contains(target_pet)),
            ConditionType::Team(cond) => {
                match cond {
                    TeamCondition::PreviousWon
                    | TeamCondition::PreviousDraw
                    | TeamCondition::PreviousLoss => {
                        // Get last battle outcome and if matches condition, apply effect.
                        if let Some(Some(last_outcome)) = self
                            .history
                            .last_outcome
                            .map(|idx| self.history.effect_graph.node_weight(idx))
                        {
                            Ok(last_outcome.status == cond.try_into()?)
                        } else {
                            Ok(false)
                        }
                    }
                    TeamCondition::OpenSpaceEqual(des_num_open) => {
                        // Number of spaces open.
                        Ok(*des_num_open == self.open_slots())
                    }
                    TeamCondition::NumberPetsEqual(num_pets) => {
                        Ok(*num_pets == self.filled_slots())
                    }
                    TeamCondition::NumberPetsGreaterEqual(num_pets) => {
                        Ok(*num_pets <= self.filled_slots())
                    }
                }
            }
            ConditionType::Shop(cond) => match cond {
                ShopCondition::InState(state) => Ok(self.shop.state == *state),
                ShopCondition::GoldEqual(gold) => Ok(self.gold() == *gold),
                ShopCondition::GoldGreaterEqual(gold) => Ok(self.gold() >= *gold),
            },
        }
    }

    fn get_matching_pets(
        &self,
        target: &Target,
        condition: &ItemCondition,
        opponent: &Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError> {
        if *target == Target::Friend {
            Ok(self.get_pets_by_cond(condition))
        } else {
            Ok(opponent
                .as_ref()
                .ok_or(SAPTestError::InvalidTeamAction {
                    subject: format!("Missing Opponent for {:?}", &condition),
                    reason: "Opponent must be known for this action.".to_string(),
                })?
                .get_pets_by_cond(condition))
        }
    }

    fn apply_conditional_pet_action(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        condition_type: &LogicType,
        effect: &Effect,
        action: &Action,
        opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError> {
        // Create new effect with action.
        let mut effect_copy = effect.clone();
        effect_copy.action = action.clone();

        match condition_type {
            LogicType::ForEach(cond_type) => {
                let num_actions = match cond_type {
                    ConditionType::Pet(target, cond) => {
                        // Get number of pets matching condition
                        self.get_matching_pets(target, cond, &opponent)?.len()
                    }
                    ConditionType::Team(cond) => match cond {
                        TeamCondition::PreviousWon
                        | TeamCondition::PreviousDraw
                        | TeamCondition::PreviousLoss => {
                            let battle_status: Status = cond.try_into()?;
                            self.history
                                .effect_graph
                                .raw_nodes()
                                .iter()
                                .filter(|node| node.weight.status == battle_status)
                                .count()
                        }
                        _ => unimplemented!("Team condition not implemented for logic type."),
                    },
                    _ => unimplemented!("ConditionType not implemented for logic type."),
                };

                // For each condition met, execute the action.
                if let Some(opponent) = opponent {
                    for _ in 0..num_actions {
                        self.apply_single_effect(target_pet.clone(), &effect_copy, Some(opponent))?
                    }
                } else {
                    for _ in 0..num_actions {
                        self.apply_single_effect(target_pet.clone(), &effect_copy, None)?
                    }
                }
            }
            LogicType::If(cond_type) => {
                if self.check_condition(cond_type, &target_pet, &opponent)? {
                    self.apply_single_effect(target_pet, &effect_copy, opponent)?
                }
            }
            LogicType::IfNot(cond_type) => {
                if !self.check_condition(cond_type, &target_pet, &opponent)? {
                    self.apply_single_effect(target_pet, &effect_copy, opponent)?
                }
            }
            LogicType::IfAny(cond_type) => match cond_type {
                ConditionType::Pet(target, cond) => {
                    // If any pet matches condition, run action.
                    if !self.get_matching_pets(target, cond, &opponent)?.is_empty() {
                        self.apply_single_effect(target_pet, &effect_copy, opponent)?;
                    }
                }
                _ => unimplemented!("ConditionType not implemented for logic type."),
            },
        }
        Ok(())
    }

    fn apply_shop_effect(&mut self, effect: &Effect) -> Result<(), SAPTestError> {
        let effect_owner: Rc<RefCell<Pet>> = effect.try_into()?;

        match &effect.action {
            Action::AddShopStats(stats) => {
                self.shop.perm_stats += *stats;
                info!(target: "run", "(\"{}\")\nAdded permanent shop {}.", self.name, stats)
            }
            Action::AddShopFood(gain_food_type) => {
                let new_shop_food = self
                    .convert_gain_type_to_food(gain_food_type, &effect_owner)?
                    .map(ShopItem::from);
                info!(target: "run", "(\"{}\")\nAdding shop item {:?}.", self.name, new_shop_food.as_ref());

                if let Some(Err(err)) = new_shop_food.map(|item| self.shop.add_item(item)) {
                    info!(target: "run", "(\"{}\")\n{err}.", self.name)
                }
            }
            Action::AddShopPet(summon_type) => {
                let new_shop_pet =
                    ShopItem::from(self.convert_summon_type_to_pet(summon_type, &effect_owner)?);
                info!(target: "run", "(\"{}\")\nAdding shop item {:?}.", self.name, &new_shop_pet);

                if let Err(err) = self.shop.add_item(new_shop_pet) {
                    info!(target: "run", "(\"{}\")\n{err}.", self.name)
                }
            }
            Action::ClearShop(item_type) => {
                match item_type {
                    Entity::Pet => self.shop.pets.clear(),
                    Entity::Food => self.shop.foods.clear(),
                }
                info!(target: "run", "(\"{}\")\nCleared shop {item_type:?}.", self.name)
            }
            Action::Profit => {
                self.shop.coins += 1;
                info!(target: "run", "(\"{}\")\nIncreased shop coins by 1. New coin count: {}", self.name, self.shop.coins)
            }
            Action::Discount(entity, discount) => {
                // Method only gets immutable refs.
                let affected_items_copy = self
                    .shop
                    .get_shop_items_by_pos(&effect.position, entity)?
                    .into_iter()
                    .cloned()
                    .collect_vec();
                let shop_items = match entity {
                    Entity::Pet => self.shop.pets.iter_mut(),
                    Entity::Food => self.shop.foods.iter_mut(),
                };
                for item in shop_items.filter(|item| affected_items_copy.contains(item)) {
                    item.cost = item.cost.saturating_sub(*discount)
                }
            }
            Action::FreeRoll => {
                self.shop.free_rolls += 1;
                info!(target: "run", "(\"{}\")\nIncreased free rolls by 1. New free rolls: {}", self.name, self.shop.free_rolls)
            }
            Action::Multiple(actions) => {
                let mut effect_copy = effect.clone();
                for action in actions {
                    effect_copy.action = action.clone();
                    self.apply_shop_effect(&effect_copy)?;
                }
            }
            _ => {
                for (_, pet) in self.get_pets_by_effect(&TRIGGER_NONE, effect, None)? {
                    self.apply_single_effect(pet, effect, None)?
                }
            }
        }

        Ok(())
    }
    fn apply_single_effect(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        effect: &Effect,
        mut opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError> {
        let mut target_ids: Vec<Option<String>> = vec![];
        let effect_owner: Rc<RefCell<Pet>> = effect.try_into()?;

        match &effect.action {
            Action::Add(stat_change) => {
                let added_stats = match stat_change {
                    StatChangeType::StaticValue(stats) => *stats,
                    StatChangeType::SelfMultValue(stats) => {
                        effect_owner.borrow().stats.mult_perc(stats)
                    }
                };
                // If effect is temporary, store stats to be removed from referenced pet on reopening shop.
                if effect.temp
                    && effect.target == Target::Friend
                    && self.shop.state == ShopState::Open
                {
                    self.shop.temp_stats.push((
                        target_pet.borrow().id.as_ref().unwrap().clone(),
                        added_stats,
                    ));
                }
                target_pet.borrow_mut().stats += added_stats;
                info!(target: "run", "(\"{}\")\nAdded {} to {}.", self.name, added_stats, target_pet.borrow());
                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Remove(stat_change) => {
                let mut remove_stats = match stat_change {
                    StatChangeType::StaticValue(stats) => *stats,
                    StatChangeType::SelfMultValue(stats) => {
                        effect_owner.borrow().stats.mult_perc(stats)
                    }
                };
                // Check for food. Add any effect dmg modifiers.
                if let Some(item) = effect_owner
                    .borrow()
                    .item
                    .as_ref()
                    .filter(|item| Status::IndirectAttackDmgCalc == item.ability.trigger.status)
                {
                    if let Action::Add(modifier) = &item.ability.action {
                        remove_stats = match modifier {
                            StatChangeType::StaticValue(stats) => remove_stats + *stats,
                            StatChangeType::SelfMultValue(stats_mult) => {
                                remove_stats.mult_perc(stats_mult)
                            }
                        }
                    }
                }
                let mut atk_outcome = target_pet.borrow_mut().indirect_attack(&remove_stats);

                // Update triggers from where they came from.
                for trigger in atk_outcome
                    .friends
                    .iter_mut()
                    .chain(atk_outcome.opponents.iter_mut())
                {
                    trigger.set_affected(&target_pet);
                    trigger.afflicting_pet = effect.owner.clone();
                }
                // Collect triggers for both teams.
                info!(target: "run", "(\"{}\")\nRemoved {} health from {}.", self.name, remove_stats.attack, target_pet.borrow());
                self.triggers.extend(atk_outcome.friends);
                if let Some(opponent) = opponent {
                    opponent.triggers.extend(atk_outcome.opponents);
                }

                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Vulture(stats) => {
                let opponent = opponent.ok_or(SAPTestError::InvalidTeamAction {
                    subject: format!("Missing Opponent for {:?}", &effect.action),
                    reason: "Opponent must be known for this action.".to_string(),
                })?;
                // Should only target enemies for this to work as we are only checking the opposing team the effect comes from.
                // Add 1 because faint triggers and only after does pet move into fainted.
                let num_fainted = opponent.fainted.len() + 1;
                // If num fainted pets is even, do damage.
                if num_fainted % 2 == 0 {
                    info!(target: "run", "(\"{}\")\nTwo pets fainted.", self.name);
                    let mut remove_effect = effect.clone();
                    remove_effect.action = Action::Remove(StatChangeType::StaticValue(*stats));
                    self.apply_single_effect(target_pet, &remove_effect, Some(opponent))?;
                }
            }
            Action::Gain(gain_food_type) => {
                let food = self.convert_gain_type_to_food(gain_food_type, &effect_owner)?;

                if food.is_none() {
                    info!(target: "run", "(\"{}\")\nRemoved food from {}.", self.name, target_pet.borrow());
                    target_pet.borrow_mut().item = None;
                    target_ids.push(target_pet.borrow().id.clone())
                } else if let Some(mut food) = food {
                    info!(target: "run", "(\"{}\")\nGave {} to {}.", self.name, food, target_pet.borrow());
                    food.ability.assign_owner(Some(&target_pet));
                    target_pet.borrow_mut().item = Some(food);
                    target_ids.push(target_pet.borrow().id.clone())
                }
            }
            Action::Experience => {
                let prev_target_lvl = target_pet.borrow().lvl;
                target_pet.borrow_mut().add_experience(1)?;
                info!(target: "run", "(\"{}\")\nGave experience point to {}.", self.name, target_pet.borrow());

                // Target leveled up. Create trigger.
                let pet_leveled_up = if target_pet.borrow().lvl != prev_target_lvl {
                    info!(target: "run", "(\"{}\")\nPet {} leveled up.", self.name, target_pet.borrow());
                    let mut lvl_trigger = TRIGGER_ANY_LEVELUP;
                    lvl_trigger.affected_pet = Some(Rc::downgrade(&target_pet));
                    Some(lvl_trigger)
                } else {
                    None
                };

                if let Some(level_trigger) = pet_leveled_up {
                    self.triggers.push_back(level_trigger)
                }
            }
            Action::Push(position) => {
                let pos_change = match position {
                    Position::First => {
                        let friends_len: isize = self.max_size.try_into()?;
                        friends_len
                    }
                    Position::Last => {
                        let friends_len: isize = self.max_size.try_into()?;
                        -friends_len
                    }
                    Position::Relative(rel_idx) => *rel_idx,
                    _ => unimplemented!("Position not implemented for push."),
                };
                if let Some(position) = target_pet.borrow().pos {
                    info!(target: "run", "(\"{}\")\nPushed pet at position {} by {}.", self.name, position, pos_change);
                    self.push_pet(position, pos_change, opponent)?;
                }
            }
            Action::Transform(pet_name, stats, lvl) => {
                if let Some(target_idx) = target_pet.borrow().pos {
                    let mut transformed_pet = Pet::new(pet_name.clone(), None, *stats, *lvl)?;
                    transformed_pet.set_pos(target_idx);

                    if (0..self.friends.len()).contains(&target_idx) {
                        self.friends.remove(target_idx);
                        info!(target: "run", "(\"{}\")\nTransformed pet at position {} to {}.", self.name, target_idx, &transformed_pet);
                        let rc_transformed_pet = Rc::new(RefCell::new(transformed_pet));

                        for effect in rc_transformed_pet.borrow_mut().effect.iter_mut() {
                            effect.assign_owner(Some(&rc_transformed_pet));
                        }
                        self.friends.insert(target_idx, Some(rc_transformed_pet));
                    }
                }
            }
            Action::Summon(summon_type) => {
                let summon_result = self.summon_pet(target_pet, summon_type, opponent);
                if let Err(err) = summon_result {
                    // Fallible error. Attempted to add too many pets.
                    if let SAPTestError::InvalidPetAction { .. } = err {
                    } else {
                        // Otherwise, something actually went wrong.
                        return Err(err);
                    }
                } else if let Ok(summoned_pet_id) = summon_result {
                    target_ids.push(summoned_pet_id)
                };
            }
            Action::Multiple(actions) => {
                // Create new effect with single action.
                let mut effect_copy = effect.clone();
                if let Some(opponent) = opponent {
                    for action in actions {
                        effect_copy.action = action.clone();
                        self.apply_single_effect(target_pet.clone(), &effect_copy, Some(opponent))?
                    }
                } else {
                    for action in actions {
                        effect_copy.action = action.clone();
                        self.apply_single_effect(target_pet.clone(), &effect_copy, None)?
                    }
                }
            }
            Action::Conditional(condition_type, action) => {
                self.apply_conditional_pet_action(
                    target_pet,
                    condition_type,
                    effect,
                    action,
                    opponent,
                )?;
            }
            Action::Kill => {
                target_pet.borrow_mut().stats.health = 0;
                info!(target: "run", "(\"{}\")\nKilled pet {}.", self.name, target_pet.borrow());

                let mut self_faint_triggers = get_self_faint_triggers(&None);
                let mut enemy_faint_triggers = get_self_enemy_faint_triggers(&None);

                for trigger in self_faint_triggers
                    .iter_mut()
                    .chain(enemy_faint_triggers.iter_mut())
                {
                    trigger.set_affected(&target_pet);
                }
                // Add death triggers.
                self.triggers.extend(self_faint_triggers);
                if let Some(opponent) = opponent {
                    opponent.triggers.extend(enemy_faint_triggers);
                }
            }
            Action::Rhino(stats) => {
                // Double damage against tier 1 pets.
                let tier_spec_stats = if target_pet.borrow().tier == 1 {
                    Statistics {
                        attack: stats.attack * 2,
                        health: stats.health,
                    }
                } else {
                    *stats
                };
                let mut atk_outcome = target_pet.borrow_mut().indirect_attack(&tier_spec_stats);

                // If kill by indirect, still counts as knockout.
                if target_pet.borrow().stats.health == 0 {
                    let mut knockout_trigger = TRIGGER_KNOCKOUT;
                    knockout_trigger.set_afflicting(&target_pet);
                    knockout_trigger.affected_pet = effect.owner.clone();
                    if let Some(opponent) = opponent.as_mut() {
                        opponent.triggers.push_front(knockout_trigger);
                    }
                }

                for trigger in atk_outcome
                    .friends
                    .iter_mut()
                    .chain(atk_outcome.opponents.iter_mut())
                {
                    trigger.set_affected(&target_pet);
                    trigger.afflicting_pet = effect.owner.clone();
                }

                // Collect triggers for both teams.
                self.triggers.extend(atk_outcome.friends);
                if let Some(opponent) = opponent {
                    opponent.triggers.extend(atk_outcome.opponents);
                }

                info!(target: "run", "(\"{}\")\nRemoved {} health from {}.", self.name, tier_spec_stats.attack, target_pet.borrow());
                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Debuff(perc_stats) => {
                let debuff_stats = target_pet.borrow().stats.mult_perc(perc_stats);
                target_pet.borrow_mut().stats -= debuff_stats;
                info!(target: "run", "(\"{}\")\nMultiplied stats of {} by {}.", self.name, target_pet.borrow(), perc_stats)
            }
            Action::Tapir => {
                let mut rng =
                    ChaCha12Rng::seed_from_u64(effect_owner.borrow().seed.unwrap_or_else(random));
                // Choose a pet on the current team that isn't a tapir.
                let chosen_friend = self
                    .friends
                    .iter()
                    .flatten()
                    .filter_map(|pet| {
                        let pet_name = pet.borrow().name.clone();
                        (pet_name != PetName::Tapir).then_some(pet_name)
                    })
                    .choose(&mut rng);

                if let Some(pet_name) = chosen_friend {
                    let summon =
                        Box::new(Pet::new(pet_name, None, None, effect_owner.borrow().lvl)?);
                    self.summon_pet(target_pet, &SummonType::StoredPet(summon), opponent)?;
                }
            }
            Action::Lynx => {
                let opponent_lvls: usize = opponent
                    .as_ref()
                    .ok_or(SAPTestError::InvalidTeamAction {
                        subject: format!("Missing Opponent for {:?}", &effect.action),
                        reason: "Opponent must be known for this action.".to_string(),
                    })?
                    .all()
                    .iter()
                    .map(|pet| pet.borrow().lvl)
                    .sum();
                let lvl_dmg_action = Action::Remove(StatChangeType::StaticValue(Statistics::new(
                    opponent_lvls,
                    0,
                )?));
                let mut effect_copy = effect.clone();
                effect_copy.action = lvl_dmg_action;

                self.apply_single_effect(target_pet, &effect_copy, opponent)?
            }
            Action::Whale(lvl, rel_pos) => {
                let mut copy_effect = effect.clone();
                // Based on a specific relative position, select the pet to 'swallow' and remove.
                copy_effect.position = rel_pos.clone();
                copy_effect.target = Target::Friend;

                let targets = self.get_pets_by_effect(&TRIGGER_NONE, &copy_effect, None)?;

                self.evolve_pet(*lvl, targets, target_pet, opponent)?;
            }
            Action::Stegosaurus(stats) => {
                let mut turn_mult_stats = *stats;
                // Multiply by turn number. Need to multiply raw values since mult op treats as percent.
                let turn_multiplier = TryInto::<isize>::try_into(self.history.curr_turn)?;
                turn_mult_stats.attack *= turn_multiplier;
                turn_mult_stats.health *= turn_multiplier;

                // Modify action to add turn-multiplied stats and apply effect.
                let mut effect_copy = effect.clone();
                effect_copy.action = Action::Add(StatChangeType::StaticValue(turn_mult_stats));
                self.apply_single_effect(target_pet, &effect_copy, opponent)?;
            }
            Action::Cockroach => {
                let mut pet_stats = target_pet.borrow().stats;
                pet_stats.attack = TryInto::<isize>::try_into(self.shop.tier())? + 1;

                target_pet.borrow_mut().stats = pet_stats;
                info!(target: "run", "(\"{}\")\nSet stats of {} by {}.", self.name, target_pet.borrow(), pet_stats)
            }
            Action::Copy(attr, target, pos) => {
                // Create effect to select a pet.
                let mut copy_effect = effect.clone();
                copy_effect.position = pos.clone();
                copy_effect.target = *target;

                let targets = if let Some(opponent) = opponent {
                    self.get_pets_by_effect(&TRIGGER_NONE, &copy_effect, Some(opponent))?
                } else {
                    self.get_pets_by_effect(&TRIGGER_NONE, &copy_effect, None)?
                };
                self.copy_effect(attr, targets, &target_pet)?;

                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Swap(swap_type) => {
                // Only allow stat swap here.
                if let RandomizeType::Stats = swap_type {
                    target_pet.borrow_mut().stats.invert();
                }
            }
            Action::None => {}
            _ => {
                return Err(SAPTestError::InvalidTeamAction {
                    subject: "Action Not Implemented".to_string(),
                    reason: format!("Single action ({:?}) not implemented yet.", &effect.action),
                })
            }
        }
        // Create edge by iterating over all targets affected.
        if let (Some(prev_node), Some(curr_node)) = (self.history.prev_node, self.history.curr_node)
        {
            for target_id in target_ids {
                self.history.effect_graph.add_edge(
                    prev_node,
                    curr_node,
                    (
                        effect.target,
                        effect.position.clone(),
                        effect.action.clone(),
                        // If added, may not have and id. Default to 'None'.
                        target_id.unwrap_or_else(|| "None".to_string()),
                    ),
                );
            }
        };
        Ok(())
    }

    fn cvt_pos_to_idx(&self, pos: &Position) -> Option<usize> {
        match pos {
            Position::Any(cond) => {
                let pets = self.get_pets_by_cond(cond);
                let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));
                pets.choose(&mut rng).and_then(|pet| pet.borrow().pos)
            }
            Position::First => (!self.friends.is_empty()).then_some(0),
            Position::Last => Some(self.friends.len().saturating_sub(1)),
            Position::Relative(idx) => {
                let Some((Target::Friend, adj_idx)) = self.cvt_rel_idx_to_adj_idx(0, *idx).ok() else {
                    return None;
                };
                Some(adj_idx)
            }
            _ => None,
        }
    }
    fn cvt_rel_idx_to_adj_idx(
        &self,
        curr_idx: usize,
        rel_idx: isize,
    ) -> Result<(Target, usize), SAPTestError> {
        let effect_pet_idx = isize::try_from(curr_idx)?;
        // Negative idx means behind.
        // Positive idx mean ahead.
        // We adjust so within bounds of team.
        let adj_idx = if rel_idx.is_negative() {
            -rel_idx + effect_pet_idx
        } else {
            let new_idx = effect_pet_idx - rel_idx;
            // On the other team.
            if new_idx.is_negative() {
                return Ok((Target::Enemy, (-new_idx - 1).try_into()?));
            } else {
                new_idx
            }
        };
        Ok((
            Target::Friend,
            adj_idx.clamp(0, self.max_size.try_into()?).try_into()?,
        ))
    }
}
