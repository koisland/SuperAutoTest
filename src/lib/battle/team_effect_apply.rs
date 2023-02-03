use crate::{
    battle::{
        effect::{Effect, Entity, Modify},
        state::{Action, Condition, CopyAttr, Outcome, Position, Statistics, Target},
        team::Team,
        trigger::{self, *},
    },
    error::SAPTestError,
    pets::{
        names::PetName,
        pet::{MAX_PET_STATS, MIN_PET_STATS},
    },
    Pet, PetCombat,
};

use itertools::Itertools;
use log::{error, info};
use rand::{seq::IteratorRandom, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::{cell::RefCell, error::Error, rc::Rc};

const NONSPECIFIC_POSITIONS: [Position; 3] = [
    Position::None,
    Position::Any(Condition::None),
    Position::All(Condition::None),
];

/// Enable applying [`Effect`]s to multiple [`Team`]s.
/// # Examples
/// ```
/// use sapt::EffectApply;
/// ```
pub trait EffectApply {
    /// Apply [`Pet`](crate::pets::pet::Pet) [`Effect`]s based on a team's stored [`Outcome`] triggers.
    /// # Examples
    /// ```rust
    /// use sapt::{EffectApply, Team, Pet, PetName};
    ///
    /// let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// let mut team = Team::new(&vec![Some(mosquito); 5], 5).unwrap();
    /// let mut enemy_team = team.clone();
    ///
    /// // Start of battle triggers.
    /// assert_eq!(team.triggers.len(), 2);
    ///
    /// // Trigger effects.
    /// team.trigger_effects(&mut enemy_team);
    ///
    /// // Exhaust triggers.
    /// assert_eq!(team.triggers.len(), 0);
    /// ```
    fn trigger_effects(&mut self, opponent: &mut Team) -> &mut Self;
    /// Apply an [`Effect`] with an associated [`Outcome`] trigger and index to a [`Team`].
    /// * The `opponent` [`Team`] will get updated with additional [`Outcome`]s.
    /// * The `effect_pet_idx` is used if an [`Effect`] targets a position relative to itself.
    ///     * **Note**: This does not mutate the number of `uses` of the [`Effect`] for the [`Pet`](crate::pets::pet::Pet) at this index.
    /// # Examples
    /// ```rust
    /// use sapt::{EffectApply, Team, Pet, PetName, Statistics, battle::state::Status};
    ///
    /// // Get mosquito effect.
    /// let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// let mosquito_effect = mosquito.effect.first().unwrap().clone();
    /// // Init teams.
    /// let mut team = Team::new(&vec![Some(mosquito); 5], 5).unwrap();
    /// let mut enemy_team = team.clone();
    /// enemy_team.set_seed(0);
    ///
    /// // Get start of battle trigger.
    /// let start_of_battle_trigger = team.triggers.pop_back().unwrap();
    ///
    /// // Apply effect of mosquito at position 0 to a pet on team to enemy team.
    /// team.apply_effect(0, &start_of_battle_trigger, &mosquito_effect, &mut enemy_team);
    ///
    /// // Last enemy mosquito takes one damage and opponent triggers gets updated.
    /// assert_eq!(
    ///     enemy_team.friends[4].as_ref().unwrap().stats,
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
        opponent: &mut Team,
    ) -> Result<(), SAPTestError>;
}

impl EffectApply for Team {
    fn trigger_effects(&mut self, opponent: &mut Team) -> &mut Self {
        info!(target: "dev", "(\"{}\")\nTriggers:\n{}", self.name, self.triggers.iter().join("\n"));

        // Continue iterating until all triggers consumed.
        while let Some(trigger) = self.triggers.pop_front() {
            let mut applied_effects: Vec<(Outcome, Effect)> = vec![];

            // Get petname of trigger.
            let trigger_pet_name = if let Some(Some(trigger_pet)) =
                trigger.affected_pet.as_ref().map(|pet| pet.upgrade())
            {
                Some(trigger_pet.borrow().name.clone())
            } else {
                None
            };

            // Iterate through pets in descending order by attack strength to collect valid effects.
            for (effect_pet_idx, pet) in self
                .friends
                .iter()
                .enumerate()
                .sorted_by(|(_, pet_1), (_, pet_2)| {
                    pet_1
                        .borrow()
                        .stats
                        .attack
                        .cmp(&pet_2.borrow().stats.attack)
                })
                .rev()
            {
                let same_pet_as_trigger = trigger
                    .clone()
                    .affected_pet
                    .map_or(false, |trigger_pet| trigger_pet.ptr_eq(&Rc::downgrade(pet)));

                // Get food and pet effect based on if its trigger is equal to current trigger, if any.
                if let Some(food) = pet.borrow_mut().item.as_mut().filter(|food| {
                    (food.ability.trigger == trigger
                        // This bottom condition allows triggers for effects that activate on any position/positions
                        || (NONSPECIFIC_POSITIONS.contains(&food.ability.trigger.position)
                            && food.ability.trigger.position == trigger.position
                            && food.ability.trigger.affected_team == trigger.affected_team
                            && food.ability.trigger.status == trigger.status))
                        && food.ability.uses != Some(0)
                }) {
                    // Drop uses by one if possible.
                    food.ability.remove_uses(1);
                    applied_effects.push((trigger.clone(), food.ability.clone()))
                };
                for pet_effect in pet.borrow_mut().effect.iter_mut().filter(|effect| {
                    (effect.trigger == trigger
                        // This bottom condition allows triggers for effects that activate on any position/positions. ex. Horse.
                        || (NONSPECIFIC_POSITIONS.contains(&effect.trigger.position)
                            && effect.trigger.position == trigger.position
                            && effect.trigger.affected_team == trigger.affected_team
                            && effect.trigger.status == trigger.status))
                        && effect.uses != Some(0)
                }) {
                    // Check the trigger name as final check before adding effect.
                    // Specific check for:
                    //  * If trigger for a summon action is a Zombie Fly, ignore it.
                    //  * If trigger for a summon action is a Fly and is also the current pet is that fly, ignore it.
                    if let Some(trigger_pet_name) = trigger_pet_name.as_ref() {
                        if let Action::Summon(_, _) = pet_effect.action {
                            if *trigger_pet_name == PetName::ZombieFly
                                || (*trigger_pet_name == PetName::Fly && same_pet_as_trigger)
                            {
                                continue;
                            }
                        } else if let Action::Add(_) = pet_effect.action {
                            // On self trigger and position any, ignore effect.
                            if trigger.position == Position::Any(Condition::None)
                                && same_pet_as_trigger
                            {
                                continue;
                            }
                        }
                    }
                    // For Tiger. Check if behind. Determines number of times effect applied.
                    let num_times_applied = self
                        .friends
                        .iter()
                        .nth(effect_pet_idx + 1)
                        .map(|pet| {
                            if pet.borrow().name == PetName::Tiger {
                                2
                            } else {
                                1
                            }
                        })
                        .unwrap_or(1);

                    // Drop uses by one if possible.
                    pet_effect.remove_uses(1);

                    for _ in 0..num_times_applied {
                        applied_effects.push((trigger.clone(), pet_effect.clone()))
                    }
                }
            }
            // Apply effects in reverse so proper order followed.
            for (trigger, effect) in applied_effects.into_iter().rev() {
                // Add node here for activated effect.
                let node_idx = self.history.effect_graph.add_node(trigger.clone());
                self.history.curr_node = Some(node_idx);

                if let Err(err) = self.apply_effect(&trigger, &effect, opponent) {
                    error!(target: "dev", "(\"{}\")\nSomething went wrong. {:?}", self.name, err)
                };
            }

            // Set curr node to previous.
            self.history.prev_node = self.history.curr_node;
        }

        self
    }
    fn apply_effect(
        &mut self,
        trigger: &Outcome,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), SAPTestError> {
        // Set current pet.
        self.curr_pet = effect.owner.clone();

        match effect.target {
            Target::Friend => {
                let target_pets = self._match_position_one_team(trigger, &effect.position)?;
                match effect.action {
                    Action::SwapPositions => {
                        if target_pets.len() != 2 {
                            return Err(SAPTestError::InvalidTeamAction {
                                subject: "Swap Positions".to_string(),
                                indices: vec![],
                                reason: format!(
                                    "Only two friends allowed for swapping positions. Given: {}",
                                    target_pets.len()
                                ),
                            });
                        }
                        if let (Some(Some(pet_1_pos)), Some(Some(pet_2_pos))) = (
                            target_pets.first().map(|pet| pet.borrow().pos),
                            target_pets.get(1).map(|pet| pet.borrow().pos),
                        ) {
                            self.swap_pets(pet_1_pos, pet_2_pos)?;
                        } else {
                            return Err(SAPTestError::InvalidTeamAction {
                                subject: "Swap Positions".to_string(),
                                indices: vec![],
                                reason: "Cannot access pets.".to_string(),
                            });
                        }
                    }
                    Action::SwapStats => {
                        if target_pets.len() != 2 {
                            return Err(SAPTestError::InvalidTeamAction {
                                subject: "Swap Stats".to_string(),
                                indices: vec![],
                                reason: format!(
                                    "Only two friends allowed for swapping stats. Given: {}",
                                    target_pets.len()
                                ),
                            });
                        }
                        if let (Some(Some(pet_1_pos)), Some(Some(pet_2_pos))) = (
                            target_pets.first().map(|pet| pet.borrow().pos),
                            target_pets.get(1).map(|pet| pet.borrow().pos),
                        ) {
                            self.swap_pet_stats(pet_1_pos, pet_2_pos)?;
                        } else {
                            return Err(SAPTestError::InvalidTeamAction {
                                subject: "Swap Stats".to_string(),
                                indices: vec![],
                                reason: "Cannot access pets.".to_string(),
                            });
                        }
                    }
                    _ => {
                        for pet in target_pets.into_iter() {
                            self._target_effect_idx(pet, effect, opponent);
                        }
                    }
                }
            }
            Target::Enemy => {
                let target_pets = opponent._match_position_one_team(trigger, &effect.position)?;
                match effect.action {
                    Action::SwapPositions => {
                        if target_pets.len() != 2 {
                            return Err(SAPTestError::InvalidTeamAction {
                                subject: "Swap Positions".to_string(),
                                indices: vec![],
                                reason: format!(
                                    "Only two friends allowed for swapping positions. Given: {}",
                                    target_pets.len()
                                ),
                            });
                        }
                        if let (Some(Some(pet_1_pos)), Some(Some(pet_2_pos))) = (
                            target_pets.first().map(|pet| pet.borrow().pos),
                            target_pets.get(1).map(|pet| pet.borrow().pos),
                        ) {
                            opponent.swap_pets(pet_1_pos, pet_2_pos)?;
                        } else {
                            return Err(SAPTestError::InvalidTeamAction {
                                subject: "Swap Positions".to_string(),
                                indices: vec![],
                                reason: "Cannot access pets.".to_string(),
                            });
                        }
                    }
                    Action::SwapStats => {
                        if target_pets.len() != 2 {
                            return Err(SAPTestError::InvalidTeamAction {
                                subject: "Swap Stats".to_string(),
                                indices: vec![],
                                reason: format!(
                                    "Only two friends allowed for swapping stats. Given: {}",
                                    target_pets.len()
                                ),
                            });
                        }
                        if let (Some(Some(pet_1_pos)), Some(Some(pet_2_pos))) = (
                            target_pets.first().map(|pet| pet.borrow().pos),
                            target_pets.get(1).map(|pet| pet.borrow().pos),
                        ) {
                            opponent.swap_pet_stats(pet_1_pos, pet_2_pos)?;
                        } else {
                            return Err(SAPTestError::InvalidTeamAction {
                                subject: "Swap Stats".to_string(),
                                indices: vec![],
                                reason: "Cannot access pets.".to_string(),
                            });
                        }
                    }
                    _ => {
                        for pet in target_pets.into_iter() {
                            opponent._target_effect_idx(pet, effect, self);
                        }
                    }
                }
            }
            Target::Either => {
                // let target_pos = self._match_position_either_team(trigger, effect, opponent)?
            }
            _ => {}
        }

        Ok(())
    }
}
pub(crate) trait EffectApplyHelpers {
    /// Apply an `Action` to a target idx on a `Team`.
    fn _target_effect_idx(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>>;
    /// Match statement applying effect to exclusively one `Team`.
    fn _match_position_one_team(
        &mut self,
        trigger: &Outcome,
        position: &Position,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError>;
    // /// Match statement applying effect to either self or opponent `Team`.
    // fn _match_position_either_team(
    //     &mut self,
    //     effect_pet_idx: usize,
    //     trigger: &Outcome,
    //     effect: &Effect,
    //     opponent: &mut Team,
    // ) -> Result<Vec<Rc<RefCell<Pet>>>, Box<dyn Error>>;
    fn _choose_pet(&mut self, pos: &Position, curr_idx: usize) -> Option<Rc<RefCell<Pet>>>;
    /// Calculates an adjusted index based on the current index and a relative index.
    /// * `:param curr_idx:` The current index.
    /// * `:param rel_idx:` Number of positions relative to the current index.
    ///     * If *negative*, the index is **behind** the current index.
    ///     * If *positive*, the index is **ahead** of the current index.
    ///
    /// Output:
    /// * Value of the new index on a team represented by a variant in the enum `Target`.
    fn _cvt_rel_idx_to_adj_idx(
        &mut self,
        curr_idx: usize,
        rel_idx: isize,
    ) -> Result<(Target, usize), Box<dyn Error>>;
}

impl EffectApplyHelpers for Team {
    fn _choose_pet(&mut self, pos: &Position, curr_idx: usize) -> Option<Rc<RefCell<Pet>>> {
        match pos {
            Position::Any(condition) => {
                let mut rng = ChaCha12Rng::seed_from_u64(self.seed);
                self.get_pets_by_cond(condition)
                    .into_iter()
                    .choose(&mut rng)
            }
            Position::Relative(rel_pos) => {
                if let Ok((team, adj_idx)) = self._cvt_rel_idx_to_adj_idx(curr_idx, *rel_pos) {
                    if team == Target::Friend {
                        self.nth(adj_idx)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Position::First => self.first(),
            Position::Last => self.last(),
            Position::N(condition, _) => self.get_pets_by_cond(condition).into_iter().next(),
            _ => None,
        }
    }

    fn _target_effect_idx(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>> {
        let name = self.name.clone();
        let mut target_ids: Vec<Option<String>> = vec![];

        match &effect.action {
            Action::Add(stats) => {
                target_pet.borrow_mut().stats += *stats;
                info!(target: "dev", "(\"{}\")\nAdded {} to {}.", name, stats, target_pet.borrow());
                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Remove(stats) => {
                let mut atk_outcome = target_pet.borrow_mut().indirect_attack(stats);

                // TODO: Update triggers from where they came from.
                for trigger in atk_outcome
                    .friends
                    .iter_mut()
                    .chain(atk_outcome.opponents.iter_mut())
                {
                    trigger.affected_pet = Some(Rc::downgrade(&target_pet));
                    trigger.afflicting_pet = effect.owner.clone();
                }
                // Collect triggers for both teams.
                info!(target: "dev", "(\"{}\")\nRemoved {} health from {}.", name, stats.attack, target_pet.borrow());
                self.triggers.extend(atk_outcome.friends);
                opponent.triggers.extend(atk_outcome.opponents);

                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Gain(food) => {
                if let Some(food) = food {
                    target_pet.borrow_mut().item = Some(*food.clone());
                    info!(target: "dev", "(\"{}\")\nGave {} to {}.", name, food, target_pet.borrow());
                    target_ids.push(target_pet.borrow().id.clone())
                }
            }
            Action::Experience => {
                let prev_target_lvl = target_pet.borrow().lvl;
                target_pet.borrow_mut().add_experience(1)?;
                info!(target: "dev", "(\"{}\")\nGave experience point to {}.", name, target_pet.borrow());

                // Target leveled up. Create trigger.
                let pet_leveled_up = if target_pet.borrow().lvl != prev_target_lvl {
                    info!(target: "dev", "(\"{}\")\nPet {} leveled up.", name, target_pet.borrow());
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
                // Helper methods used in applying effects only consider pets that are:
                // * Alive
                // If friends still has fainted pets, the idx provided will not match.
                // * We have to map this provided position to the actual idx we want
                // let valid_idx = self
                //     .friends
                //     .iter()
                //     .enumerate()
                //     .filter_map(|(i, pet)| {
                //         if pet.as_ref().filter(|pet| pet.stats.health != 0).is_some() {
                //             Some(i)
                //         } else {
                //             None
                //         }
                //     })
                //     .nth(target_idx);
                if let Some(position) = target_pet.borrow().pos {
                    info!(target: "dev", "(\"{}\")\nPushed pet at position {} by {}.", name, position, pos_change);
                    self.push_pet(position, pos_change, Some(opponent))?;
                }

                // if let Some(adj_target_idx) = valid_idx {
                //     info!(target: "dev", "(\"{}\")\nPushed pet at position {} by {}.", name, adj_target_idx, pos_change);
                //     self.push_pet(adj_target_idx, pos_change, Some(opponent))?;
                // }
            }
            Action::Transform(pet_name, stats, lvl) => {
                if let Some(target_idx) = target_pet.borrow().pos {
                    let mut transformed_pet = Pet::new(pet_name.clone(), None, *stats, *lvl)?;
                    transformed_pet.set_pos(target_idx);

                    if (0..self.friends.len()).contains(&target_idx) {
                        self.friends.remove(target_idx);
                        info!(target: "dev", "(\"{}\")\nTransformed pet at position {} to {}.", name, target_idx, &transformed_pet);
                        self.friends
                            .insert(target_idx, Rc::new(RefCell::new(transformed_pet)));
                    }
                }
            }
            Action::Summon(stored_pet, stats) => {
                // If stored pet is None, assume is summoning self.
                let stored_box_pet = if stored_pet.is_none() {
                    if let Some(summon_stats) = stats {
                        // Copy the pet, set its stats to opt_summon_stats
                        let mut one_up_pet = target_pet.borrow().clone();
                        one_up_pet.stats = *summon_stats;
                        // Remove the item.
                        one_up_pet.item = None;

                        Some(Box::new(one_up_pet))
                    } else {
                        None
                    }
                } else {
                    // Otherwise use stored pet.
                    stored_pet.clone()
                };

                if let (Some(summoned_pet), Some(target_idx)) =
                    (stored_box_pet, target_pet.borrow().pos)
                {
                    // Handle case where pet in front faints and vector is empty.
                    // Would panic attempting to insert at any position not at 0.
                    // Also update position to be correct.
                    let adj_target_idx = if target_idx > self.friends.len() {
                        0
                    } else {
                        target_idx
                    };

                    self.add_pet(*summoned_pet, adj_target_idx, Some(opponent))?;

                    // Added pet so id is safe to unwrap.
                    let new_pet_id = &self.friends.get(adj_target_idx).unwrap().borrow().id;
                    target_ids.push(new_pet_id.clone())
                }
            }
            Action::Multiple(actions) => {
                for action in actions {
                    // Create new effect with single action.
                    let mut effect_copy = effect.clone();
                    effect_copy.action = action.clone();
                    self._target_effect_idx(target_pet.clone(), &effect_copy, opponent)?
                }
            }
            Action::IfTargetCondition(action, condition) => {
                let matching_pets = self.get_pets_by_cond(condition);

                // If a pet matches condition, run action.
                if matching_pets.contains(&target_pet) {
                    let mut effect_copy = effect.clone();
                    effect_copy.action = *action.clone();
                    self._target_effect_idx(target_pet, &effect_copy, opponent)?;
                }
            }
            Action::ForEachCondition(action, target, condition) => {
                let num_matches = if *target == Target::Friend {
                    self.get_pets_by_cond(condition).len()
                } else {
                    opponent.get_pets_by_cond(condition).len()
                };
                // Create new effect with action.
                let mut effect_copy = effect.clone();
                effect_copy.action = *action.clone();
                // For each pet that matches the condition, execute the action.
                for _ in 0..num_matches {
                    self._target_effect_idx(target_pet.clone(), &effect_copy, opponent)?
                }
            }
            Action::Kill => {
                target_pet.borrow_mut().stats.health = 0;
                info!(target: "dev", "(\"{}\")\nKilled pet {}.", name, target_pet.borrow());

                let mut self_faint_triggers = get_self_faint_triggers(&None);
                let mut enemy_faint_triggers = get_self_enemy_faint_triggers(&None);

                for trigger in self_faint_triggers
                    .iter_mut()
                    .chain(enemy_faint_triggers.iter_mut())
                {
                    trigger.affected_pet = Some(Rc::downgrade(&target_pet));
                }
                // Add death triggers.
                self.triggers.extend(self_faint_triggers);
                opponent.triggers.extend(enemy_faint_triggers);
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
                    knockout_trigger.affected_pet = Some(Rc::downgrade(&target_pet));
                    knockout_trigger.afflicting_pet = effect.owner.clone();
                    opponent.triggers.push_front(knockout_trigger)
                }

                for trigger in atk_outcome
                    .friends
                    .iter_mut()
                    .chain(atk_outcome.opponents.iter_mut())
                {
                    trigger.affected_pet = Some(Rc::downgrade(&target_pet));
                    trigger.afflicting_pet = effect.owner.clone();
                }

                // Collect triggers for both teams.
                self.triggers.extend(atk_outcome.friends);
                opponent.triggers.extend(atk_outcome.opponents);

                info!(target: "dev", "(\"{}\")\nRemoved {} health from {}.", name, tier_spec_stats.attack, target_pet.borrow());
                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Debuff(perc_stats) => {
                let debuff_stats = target_pet.borrow().stats * *perc_stats;
                target_pet.borrow_mut().stats -= debuff_stats;
                info!(target: "dev", "(\"{}\")\nMultiplied stats of {} by {}.", name, target_pet.borrow(), perc_stats)
            }
            Action::Lynx => {
                let opponent_lvls: usize = opponent.all().iter().map(|pet| pet.borrow().lvl).sum();
                let lvl_dmg_action = Action::Remove(Statistics::new(opponent_lvls, 0).unwrap());
                let mut effect_copy = effect.clone();
                effect_copy.action = lvl_dmg_action;

                self._target_effect_idx(target_pet, &effect_copy, opponent)?
            }
            Action::Whale(lvl, rel_pos) => {
                // Based on a specific relative position, select the pet to 'swallow' and remove.
                let chosen_pet = if let (Position::Relative(rel_pos), Some(target_pos)) =
                    (rel_pos, target_pet.borrow().pos)
                {
                    // Do in new scope so mut ref to pet is dropped.
                    let (_, adj_idx) = self._cvt_rel_idx_to_adj_idx(target_pos, *rel_pos)?;
                    let chosen_pet = self.nth(adj_idx);
                    let evolved_pet = chosen_pet.as_ref().map(|pet| {
                        if let Ok(leveled_pet) = pet.borrow().clone().set_level(*lvl) {
                            pet.borrow_mut().stats.health = 0;
                            info!(target: "dev", "(\"{}\")\nKilled pet {}.", name, pet.borrow());
                            // Clone the pet and remove its item.
                            let mut leveled_pet = leveled_pet.clone();
                            leveled_pet.item = None;
                            Some(leveled_pet)
                        } else {
                            None
                        }
                    });
                    if let (Some(Some(evolved_pet)), Some(dead_evolved_pet)) =
                        (evolved_pet, chosen_pet.as_ref())
                    {
                        let mut self_faint_triggers = get_self_faint_triggers(&None);
                        let mut enemy_self_faint_triggers = get_self_enemy_faint_triggers(&None);
                        for trigger in self_faint_triggers
                            .iter_mut()
                            .chain(enemy_self_faint_triggers.iter_mut())
                        {
                            trigger.affected_pet = Some(Rc::downgrade(&dead_evolved_pet));
                        }
                        // Add death triggers.
                        self.triggers.extend(self_faint_triggers);
                        opponent.triggers.extend(enemy_self_faint_triggers);
                        Some(evolved_pet)
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Upgrade the chosen pet's abilities. And get the target pet.
                if let Some(leveled_pet) = chosen_pet {
                    // Set the target's pet ability to summon the pet.
                    target_pet.borrow_mut().effect = vec![Effect {
                        owner: Some(Rc::downgrade(&target_pet)),
                        entity: Entity::Pet,
                        trigger: TRIGGER_SELF_FAINT,
                        target: Target::Friend,
                        position: Position::OnSelf,
                        action: Action::Summon(Some(Box::new(leveled_pet.clone())), None),
                        uses: Some(1),
                        temp: true,
                    }];
                    info!(target: "dev", "(\"{}\")\nEvolving {}.", name, leveled_pet);
                    info!(target: "dev", "(\"{}\")\nSet pet {} to summon evolved pet on faint.", name, target_pet.borrow());
                } else {
                    return Err(Box::new(SAPTestError::InvalidTeamAction {
                        subject: "Evolve Pet".to_string(),
                        indices: vec![],
                        reason: format!("Cannot access position {rel_pos:?} or targeted pet."),
                    }));
                }
            }
            // TODO: May need to also choose to copy from an enemy pet at some point.
            Action::Copy(attr, target, pos) => {
                let copied_attr = if let Some(target_idx) = target_pet.borrow().pos {
                    // Based on position, select the pet to copy.
                    let chosen_pet = if *target == Target::Friend {
                        self._choose_pet(pos, target_idx)
                    } else {
                        opponent._choose_pet(pos, target_idx)
                    };
                    if let Some(pet_to_copy) = chosen_pet {
                        match attr.clone() {
                            CopyAttr::Stats(replacement_stats) => Some(CopyAttr::Stats(
                                replacement_stats.map_or(Some(pet_to_copy.borrow().stats), Some),
                            )),
                            CopyAttr::PercentStats(perc_stats_mult) => {
                                // Multiply the stats of a chosen pet by some multiplier
                                let mut new_stats = pet_to_copy.borrow().stats * perc_stats_mult;
                                new_stats.clamp(MIN_PET_STATS, MAX_PET_STATS);
                                info!(
                                    target: "dev", "(\"{}\")\nCopied {}% atk and {}% health from {}.",
                                    name,
                                    perc_stats_mult.attack,
                                    perc_stats_mult.health,
                                    target_pet.borrow()
                                );
                                Some(CopyAttr::Stats(Some(new_stats)))
                            }
                            CopyAttr::Effect(_, lvl) => Some(CopyAttr::Effect(
                                pet_to_copy.borrow().get_effect(lvl.unwrap_or(1))?,
                                lvl,
                            )),
                            CopyAttr::Item(_) => pet_to_copy
                                .borrow()
                                .item
                                .as_ref()
                                .map(|food| CopyAttr::Item(Some(Box::new(food.clone())))),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Chose the target of recipient of copied pet stats/effect.
                // Calculate stats or set ability.
                match copied_attr.unwrap_or(CopyAttr::None) {
                    CopyAttr::Stats(new_stats) => {
                        // If some stats given use those as base.
                        let new_stats = if let Some(mut new_stats) = new_stats {
                            // If any stat value is 0, use the target's original stats, otherwise, use the new stats.
                            *new_stats.comp_set_value(&target_pet.borrow().stats, 0)
                        } else {
                            // Otherwise, copy stats from target.
                            target_pet.borrow().stats
                        };

                        target_pet.borrow_mut().stats = new_stats;

                        info!(
                            target: "dev", "(\"{}\")\nSet stats for {} to {}.",
                            name,
                            target_pet.borrow(),
                            target_pet.borrow().stats
                        );
                    }
                    CopyAttr::Effect(effect, _) => {
                        target_pet.borrow_mut().effect = effect;
                        info!(
                            target: "dev", "(\"{}\")\nSet effect for {} to {:?}.",
                            name,
                            target_pet.borrow(),
                            target_pet.borrow().effect
                        );
                    }
                    CopyAttr::Item(item) => {
                        if let Some(food) = item {
                            target_pet.borrow_mut().item = Some(*food);
                            info!(
                                target: "dev", "(\"{}\")\nCopyied item for {} to {:?}.",
                                name,
                                target_pet.borrow(),
                                target_pet.borrow().item
                            );
                        }
                    }
                    CopyAttr::None => {}
                    CopyAttr::PercentStats(_) => {}
                }
                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Thorns(stats) => {
                let mut thorn_effect = effect.clone();
                thorn_effect.action = Action::Remove(*stats);
                self._target_effect_idx(target_pet, &thorn_effect, opponent)?;
            }
            Action::None => {}
            _ => unimplemented!("Action not implemented"),
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

    fn _cvt_rel_idx_to_adj_idx(
        &mut self,
        curr_idx: usize,
        rel_idx: isize,
    ) -> Result<(Target, usize), Box<dyn Error>> {
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

    fn _match_position_one_team(
        &mut self,
        trigger: &Outcome,
        position: &Position,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError> {
        let curr_pet = if let Some(effect_pet) = &self.curr_pet {
            effect_pet.upgrade()
        } else {
            return Err(SAPTestError::InvalidTeamAction {
                subject: "Pet Reference".to_string(),
                indices: vec![],
                reason: "Doesn't exist".to_string(),
            });
        };

        let mut pets = vec![];
        match position {
            Position::Any(condition) => {
                let mut rng = ChaCha12Rng::seed_from_u64(self.seed);
                if let Some(random_pet) = self
                    .get_pets_by_cond(condition)
                    .into_iter()
                    .choose(&mut rng)
                {
                    pets.push(random_pet)
                }
            }
            Position::All(condition) => {
                pets.extend(self.get_pets_by_cond(condition).into_iter());
            }
            Position::OnSelf => {
                if let Some(self_pet) = &curr_pet {
                    pets.push(self_pet.clone())
                }
            }
            Position::Trigger => {
                if let Some(Some(affected_pet)) = trigger
                    .affected_pet
                    .as_ref()
                    .map(|pet_ref| pet_ref.upgrade())
                {
                    pets.push(affected_pet)
                }
            }
            Position::Relative(rel_pos) => {
                if let Some(Some(effect_pet_idx)) = &curr_pet.as_ref().map(|pet| pet.borrow().pos) {
                    let (team, adj_idx) = self
                        ._cvt_rel_idx_to_adj_idx(*effect_pet_idx, *rel_pos)
                        .unwrap();
                    if team == Target::Friend {
                        if let Some(rel_pet) = self.friends.get(adj_idx) {
                            pets.push(rel_pet.clone())
                        }
                    }
                }
            }
            Position::Range(effect_range) => {
                for idx in effect_range.clone().into_iter() {
                    if let Some(Some(effect_pet_idx)) =
                        curr_pet.as_ref().map(|pet| pet.borrow().pos)
                    {
                        let (team, adj_idx) =
                            self._cvt_rel_idx_to_adj_idx(effect_pet_idx, idx).unwrap();
                        if team == Target::Friend {
                            if let Some(rel_pet) = self.friends.get(adj_idx) {
                                pets.push(rel_pet.clone())
                            }
                        }
                    }
                }
            }
            Position::First => {
                if let Some(first_pet) = self.all().first() {
                    pets.push(first_pet.clone())
                }
            }
            Position::Last => {
                if let Some(last_pet) = self.all().last() {
                    pets.push(last_pet.clone())
                }
            }
            Position::Multiple(positions) => {
                for pos in positions {
                    pets.extend(self._match_position_one_team(trigger, pos)?)
                }
            }
            Position::N(condition, n) => {
                let mut found_pets = self.get_pets_by_cond(condition).into_iter();
                // Get n values of indices.
                for _ in 0..*n {
                    if let Some(pet) = found_pets.next() {
                        pets.push(pet)
                    }
                }
            }
            Position::Adjacent => {
                // Get pet ahead and behind.
                for rel_pos in [-1, 1].into_iter() {
                    pets.extend(
                        self._match_position_one_team(trigger, &Position::Relative(rel_pos))?,
                    )
                }

                // if let Action::SwapPositions = effect.action {
                //     self.swap_pets(pos_1, pos_2)?;
                //     info!(target: "dev", "(\"{}\")\nSwapped positions for pets at positions {} and {}.", self.name, pos_1, pos_2);
                // } else if let Action::SwapStats = effect.action {
                //     self.swap_pet_stats(pos_1, pos_2)?;
                //     info!(target: "dev", "(\"{}\")\nSwapped stats for pets at positions {} and {}.", self.name, pos_1, pos_2);
                // } else {
                //     for pos in [pos_1, pos_2].into_iter() {
                //         self._target_effect_idx(pos, effect, opponent)?;
                //     }
                // }
            }
            _ => {}
        }

        Ok(pets)
    }

    // fn _match_position_either_team(
    //     &mut self,
    //     effect_pet_idx: usize,
    //     trigger: &Outcome,
    //     effect: &Effect,
    //     opponent: &mut Team,
    // ) -> Result<(), Box<dyn Error>> {
    //     match &effect.position {
    //         Position::Relative(rel_pos) => {
    //             let (team, adj_idx) = self._cvt_rel_idx_to_adj_idx(effect_pet_idx, *rel_pos)?;
    //             match team {
    //                 Target::Enemy => opponent._target_effect_idx(adj_idx, effect, self)?,
    //                 Target::Friend => self._target_effect_idx(adj_idx, effect, opponent)?,
    //                 _ => unreachable!("Cannot return other types."),
    //             }
    //         }
    //         Position::All(condition) => {
    //             let pos = self
    //                 .get_pets_by_cond(condition)
    //                 .into_iter()
    //                 .filter_map(|pet| pet.pos)
    //                 .collect_vec();
    //             let enemy_pos = opponent
    //                 .get_pets_by_cond(condition)
    //                 .into_iter()
    //                 .filter_map(|pet| pet.pos)
    //                 .collect_vec();
    //             for pet_idx in pos {
    //                 self._target_effect_idx(pet_idx, effect, opponent)?
    //             }
    //             for pet_idx in enemy_pos {
    //                 opponent._target_effect_idx(pet_idx, effect, self)?
    //             }
    //         }
    //         Position::Multiple(positions) => {
    //             for pos in positions {
    //                 // For each position:
    //                 // * Make a copy of the effect
    //                 // * Set the position to the desired position
    //                 // * Reduce the uses to 1 to not double the number of times an effect is activated.
    //                 let mut effect_copy = effect.clone();
    //                 effect_copy.position = pos.clone();
    //                 effect_copy.uses = Some(1);

    //                 // Add outcome to outcomes.
    //                 self.apply_effect(effect_pet_idx, trigger, &effect_copy, opponent)?
    //             }
    //         }
    //         Position::Range(effect_range) => {
    //             let adj_idxs = effect_range
    //                 .clone()
    //                 .into_iter()
    //                 .filter_map(|rel_idx| {
    //                     self._cvt_rel_idx_to_adj_idx(effect_pet_idx, rel_idx).ok()
    //                 })
    //                 .collect_vec();
    //             for (team, adj_idx) in adj_idxs {
    //                 if team == Target::Friend {
    //                     self._target_effect_idx(adj_idx, effect, opponent)?
    //                 } else {
    //                     opponent._target_effect_idx(adj_idx, effect, self)?
    //                 }
    //             }
    //         }
    //         Position::Trigger => {
    //             if let Target::Friend = trigger.from_target {
    //                 let trigger_pos = trigger
    //                     .to_idx
    //                     .ok_or("No idx position given to apply effect.")?;
    //                 self._target_effect_idx(trigger_pos, effect, opponent)?;
    //             } else if let Target::Enemy = trigger.from_target {
    //                 let trigger_pos = trigger
    //                     .from_idx
    //                     .ok_or("No idx position given to apply effect.")?;
    //                 opponent._target_effect_idx(trigger_pos, effect, self)?;
    //             } else {
    //                 unimplemented!("Trigger cannot come from both teams.")
    //             }
    //         }
    //         Position::None => {}
    //         _ => unimplemented!("Position not implemented."),
    //     };
    //     Ok(())
    // }
}
