use crate::{
    battle::{
        effect::{Effect, Entity, Modify},
        state::{Action, Condition, CopyAttr, Outcome, Position, Statistics, Target},
        team::Team,
        trigger::*,
    },
    error::SAPTestError,
    pets::{
        combat::PetCombat,
        names::PetName,
        pet::{MAX_PET_STATS, MIN_PET_STATS},
    },
    Pet,
};

use itertools::Itertools;
use log::{error, info};
use rand::{seq::IteratorRandom, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::error::Error;

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
        effect_pet_idx: usize,
        trigger: &Outcome,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>>;
}

impl EffectApply for Team {
    fn trigger_effects(&mut self, opponent: &mut Team) -> &mut Self {
        info!(target: "dev", "(\"{}\")\nTriggers:\n{}", self.name, self.triggers.iter().join("\n"));

        // Continue iterating until all triggers consumed.
        while let Some(trigger) = self.triggers.pop_front() {
            let mut applied_effects: Vec<(usize, Outcome, Effect)> = vec![];

            // Get petname of trigger.
            let trigger_pet_name = match trigger.to_target {
                Target::Friend => trigger.to_idx.map(|idx| {
                    if let Some(Some(pet)) = self.friends.get(idx) {
                        Some(pet.name.clone())
                    } else {
                        None
                    }
                }),
                Target::Enemy => trigger.to_idx.map(|idx| {
                    if let Some(Some(pet)) = opponent.friends.get(idx) {
                        Some(pet.name.clone())
                    } else {
                        None
                    }
                }),
                _ => None,
            };

            // Iterate through pets in descending order by attack strength to collect valid effects.
            for (effect_pet_idx, pet) in self
                .friends
                .iter_mut()
                .filter_map(|slot| {
                    // Extract pets and its pos.
                    if let Some(pet) = slot.as_mut() {
                        pet.pos.map(|pet_pos| (pet_pos, pet))
                    } else {
                        None
                    }
                })
                .sorted_by(|(_, pet_1), (_, pet_2)| pet_1.stats.attack.cmp(&pet_2.stats.attack))
                .rev()
            {
                // This checks whether or not a trigger should cause a pet's effect to activate.
                // * Effects that trigger on Any position are automatically allowed.
                // * Tests trigger idx so that multiple triggers aren't all activated.
                //     * For pets with Position::OnSelf and idx 0 like Crickets.
                if trigger.position != Position::Any(Condition::None)
                    && trigger.to_idx.is_some()
                    && trigger.to_idx != Some(effect_pet_idx)
                {
                    continue;
                }

                // Get food and pet effect based on if its trigger is equal to current trigger, if any.
                if let Some(food) = pet
                    .item
                    .as_mut()
                    .filter(|food| food.ability.trigger == trigger && food.ability.uses != Some(0))
                {
                    // Drop uses by one if possible.
                    food.ability.remove_uses(1);
                    applied_effects.push((effect_pet_idx, trigger.clone(), food.ability.clone()))
                };
                for pet_effect in pet
                    .effect
                    .iter_mut()
                    .filter(|effect| effect.trigger == trigger && effect.uses != Some(0))
                {
                    // Check the trigger name as final check before adding effect.
                    // Specific check for:
                    //  * If trigger for a summon action is a Zombie Fly, ignore it.
                    //  * If trigger for a summon action is a Fly and is also the current pet is that fly, ignore it.
                    if let Some(Some(trigger_name)) = trigger_pet_name.clone() {
                        if let Action::Summon(_, _) = pet_effect.action {
                            if trigger_name == PetName::ZombieFly
                                || (trigger_name == PetName::Fly
                                    && trigger.to_idx == Some(effect_pet_idx))
                            {
                                continue;
                            }
                        } else if let Action::Add(_) = pet_effect.action {
                            // On self trigger and position any, ignore effect.
                            if trigger.position == Position::Any(Condition::None)
                                && trigger.to_target == Target::Friend
                                && trigger.to_idx == Some(effect_pet_idx)
                            {
                                continue;
                            }
                        }
                    }
                    // Drop uses by one if possible.
                    pet_effect.remove_uses(1);
                    applied_effects.push((effect_pet_idx, trigger.clone(), pet_effect.clone()))
                }
            }
            // Apply effects in reverse so proper order followed.
            for (effect_pet_idx, trigger, effect) in applied_effects.into_iter().rev() {
                // For Tiger. Check if behind. Determines number of times effect applied.
                let num_times_applied = self
                    .nth(effect_pet_idx + 1)
                    .map(|pet| if pet.name == PetName::Tiger { 2 } else { 1 })
                    .unwrap_or(1);

                // Set current effect pet.
                self.effect_idx = Some(effect_pet_idx);
                for _ in 0..num_times_applied {
                    // Add node here for activated effect.
                    let node_idx = self.history.effect_graph.add_node(trigger.clone());
                    self.history.curr_node = Some(node_idx);

                    if let Err(err) = self.apply_effect(effect_pet_idx, &trigger, &effect, opponent)
                    {
                        error!(target: "dev", "(\"{}\")\nSomething went wrong. {:?}", self.name, err)
                    };
                }
            }

            // Set curr node to previous.
            self.history.prev_node = self.history.curr_node;
        }

        self
    }
    fn apply_effect(
        &mut self,
        effect_pet_idx: usize,
        trigger: &Outcome,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>> {
        match effect.target {
            Target::Friend => {
                self._match_position_one_team(effect_pet_idx, trigger, effect, opponent)?
            }
            Target::Enemy => {
                opponent._match_position_one_team(effect_pet_idx, trigger, effect, self)?
            }
            Target::Either => {
                self._match_position_either_team(effect_pet_idx, trigger, effect, opponent)?
            }
            _ => {}
        }

        Ok(())
    }
}
trait EffectApplyHelpers {
    /// Apply an `Action` to a target idx on a `Team`.
    fn _target_effect_idx(
        &mut self,
        target_idx: usize,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>>;
    /// Match statement applying effect to exclusively one `Team`.
    fn _match_position_one_team(
        &mut self,
        effect_pet_idx: usize,
        trigger: &Outcome,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>>;
    /// Match statement applying effect to either self or opponent `Team`.
    fn _match_position_either_team(
        &mut self,
        effect_pet_idx: usize,
        trigger: &Outcome,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>>;
    fn _choose_pet(&mut self, pos: &Position, curr_idx: usize) -> Option<&mut Pet>;
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
    fn _choose_pet(&mut self, pos: &Position, curr_idx: usize) -> Option<&mut Pet> {
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
        target_idx: usize,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>> {
        let name = self.name.clone();
        let mut target_ids: Vec<Option<String>> = vec![];

        match &effect.action {
            Action::Add(stats) => {
                if let Some(target) = self.nth(target_idx) {
                    target.stats += *stats;
                    info!(target: "dev", "(\"{}\")\nAdded {} to {}.", name, stats, target);
                    target_ids.push(target.id.clone())
                }
            }
            Action::Remove(stats) => {
                let mut outcomes: Vec<Outcome> = vec![];
                let target_id = if let Some(target) = self.nth(target_idx) {
                    let mut atk_outcome = target.indirect_attack(stats);
                    atk_outcome.update_opponents_pos(Some(effect.target), effect.owner_idx);
                    atk_outcome.update_friends_pos(Some(effect.target), effect.owner_idx);

                    // Collect triggers for both teams.
                    outcomes.extend(atk_outcome.friends);
                    opponent.triggers.extend(atk_outcome.opponents);

                    info!(target: "dev", "(\"{}\")\nRemoved {} health from {}.", name, stats.attack, target);
                    target.id.clone()
                } else {
                    None
                };
                self.triggers.extend(outcomes);
                target_ids.push(target_id)
            }
            Action::Gain(food) => {
                if let (Some(target), Some(food)) = (self.nth(target_idx), food) {
                    target.item = Some(*food.clone());
                    info!(target: "dev", "(\"{}\")\nGave {} to {}.", name, food, target);
                    target_ids.push(target.id.clone())
                }
            }
            Action::Experience => {
                let pet_leveled_up = if let Some(target) = self.nth(target_idx) {
                    let prev_target_lvl = target.lvl;
                    target.add_experience(1)?;
                    info!(target: "dev", "(\"{}\")\nGave experience point to {}.", name, target);
                    // Target leveled up. Create trigger.
                    if target.lvl != prev_target_lvl {
                        info!(target: "dev", "(\"{}\")\nPet {} leveled up.", name, target);
                        let mut lvl_trigger = TRIGGER_ANY_LEVELUP;
                        lvl_trigger.to_idx = target.pos;
                        Some(lvl_trigger)
                    } else {
                        None
                    }
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
                // * Not a slot.
                // If friends still has empty slots or fainted pets, the idx provided will not match.
                // * We have to map this provided position to the actual idx we want
                let valid_idx = self
                    .friends
                    .iter()
                    .enumerate()
                    .filter_map(|(i, pet)| {
                        if pet.as_ref().filter(|pet| pet.stats.health != 0).is_some() {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .nth(target_idx);
                if let Some(adj_target_idx) = valid_idx {
                    info!(target: "dev", "(\"{}\")\nPushed pet at position {} by {}.", name, adj_target_idx, pos_change);
                    self.push_pet(adj_target_idx, pos_change, Some(opponent))?;
                }
            }
            Action::Transform(pet_name, stats, lvl) => {
                let mut transformed_pet = Pet::new(pet_name.clone(), None, *stats, *lvl)?;
                transformed_pet.set_pos(target_idx);

                if (0..self.friends.len()).contains(&target_idx) {
                    self.friends.remove(target_idx);
                    info!(target: "dev", "(\"{}\")\nTransformed pet at position {} to {}.", name, target_idx, &transformed_pet);
                    self.friends.insert(target_idx, Some(transformed_pet));
                }
            }
            Action::Summon(stored_pet, stats) => {
                // If stored pet is None, assume is summoning self.
                let stored_box_pet = if stored_pet.is_none() {
                    if let (Some(Some(pet)), Some(summon_stats)) = (self.friends.get(0), stats) {
                        // Copy the pet, set its stats to opt_summon_stats
                        let mut one_up_pet = pet.clone();
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

                if let Some(summoned_pet) = stored_box_pet {
                    // Handle case where pet in front faints and vector is empty.
                    // Would panic attempting to insert at any position not at 0.
                    // Also update position to be correct.
                    let adj_target_idx = if target_idx > self.friends.len() {
                        0
                    } else {
                        target_idx
                    };

                    if self
                        .add_pet(*summoned_pet, adj_target_idx, Some(opponent))
                        .is_ok()
                    {
                        // Added pet so safe to unwrap.
                        let id = self
                            .friends
                            .get(adj_target_idx)
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .id
                            .clone();
                        target_ids.push(id)
                    }
                }
            }
            Action::Multiple(actions) => {
                for action in actions {
                    // Create new effect with single action.
                    let mut effect_copy = effect.clone();
                    effect_copy.action = action.clone();
                    self._target_effect_idx(target_idx, &effect_copy, opponent)?
                }
            }
            Action::IfTargetCondition(action, condition) => {
                let target_pet = self
                    .get_pets_by_cond(condition)
                    .into_iter()
                    .find(|pet| pet.pos == Some(target_idx));
                // If a pet matches condition, run action.
                if target_pet.is_some() {
                    let mut effect_copy = effect.clone();
                    effect_copy.action = *action.clone();
                    self._target_effect_idx(target_idx, &effect_copy, opponent)?;
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
                    self._target_effect_idx(target_idx, &effect_copy, opponent)?
                }
            }
            Action::Kill => {
                let dead_pet_pos = self.nth(target_idx).map(|pet| {
                    pet.stats.health = 0;
                    info!(target: "dev", "(\"{}\")\nKilled pet {}.", name, pet);
                    pet.pos
                });
                if let Some(trigger_pos) = dead_pet_pos {
                    // Add death triggers.
                    self.triggers
                        .extend(get_self_faint_triggers(trigger_pos, &None));

                    opponent
                        .triggers
                        .extend(get_self_enemy_faint_triggers(trigger_pos, &None));
                }
            }
            Action::Rhino(stats) => {
                let mut outcomes: Vec<Outcome> = vec![];
                let target_id = if let Some(Some(target)) = self.friends.get_mut(target_idx) {
                    // Double damage against tier 1 pets.
                    let tier_spec_stats = if target.tier == 1 {
                        Statistics {
                            attack: stats.attack * 2,
                            health: stats.health,
                        }
                    } else {
                        *stats
                    };
                    let atk_outcome = target.indirect_attack(&tier_spec_stats);

                    // If kill by indirect, still counts as knockout.
                    if target.stats.health == 0 {
                        opponent.triggers.push_front(TRIGGER_KNOCKOUT)
                    }

                    // Collect triggers for both teams.
                    outcomes.extend(atk_outcome.friends);
                    opponent.triggers.extend(atk_outcome.opponents);

                    info!(target: "dev", "(\"{}\")\nRemoved {} health from {}.", name, tier_spec_stats.attack, target);
                    target.id.clone()
                } else {
                    None
                };
                self.triggers.extend(outcomes);
                target_ids.push(target_id)
            }
            Action::Debuff(perc_stats) => {
                if let Some(pet) = self.nth(target_idx) {
                    let debuff_stats = pet.stats * *perc_stats;
                    pet.stats -= debuff_stats;
                    info!(target: "dev", "(\"{}\")\nMultiplied stats of {} by {}.", name, pet, perc_stats)
                }
            }
            Action::Lynx => {
                let opponent_lvls: usize = opponent.all().iter().map(|pet| pet.lvl).sum();
                let lvl_dmg_action = Action::Remove(Statistics::new(opponent_lvls, 0).unwrap());
                let mut effect_copy = effect.clone();
                effect_copy.action = lvl_dmg_action;

                self._target_effect_idx(target_idx, &effect_copy, opponent)?
            }
            Action::Whale(lvl, rel_pos) => {
                // Based on a specific relative position, select the pet to 'swallow' and remove.
                let chosen_pet = if let Position::Relative(rel_pos) = rel_pos {
                    // Do in new scope so mut ref to pet is dropped.
                    let (_, adj_idx) = self._cvt_rel_idx_to_adj_idx(target_idx, *rel_pos)?;
                    let evolved_pet = self.nth(adj_idx).map(|pet| {
                        if let Ok(leveled_pet) = pet.clone().set_level(*lvl) {
                            pet.stats.health = 0;
                            info!(target: "dev", "(\"{}\")\nKilled pet {}.", name, pet);
                            // Clone the pet and remove its item.
                            let mut leveled_pet = leveled_pet.clone();
                            leveled_pet.item = None;
                            Some(leveled_pet)
                        } else {
                            None
                        }
                    });
                    if let Some(Some(evolved_pet)) = evolved_pet {
                        // Add death triggers.
                        self.triggers
                            .extend(get_self_faint_triggers(evolved_pet.pos, &None));
                        opponent
                            .triggers
                            .extend(get_self_enemy_faint_triggers(evolved_pet.pos, &None));
                        Some(evolved_pet)
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Upgrade the chosen pet's abilities. And get the target pet.
                if let (Some(leveled_pet), Some(Some(target_pet))) =
                    (chosen_pet, self.friends.get_mut(target_idx))
                {
                    // Set the target's pet ability to summon the pet.
                    target_pet.effect = vec![Effect {
                        owner_target: None,
                        entity: Entity::Pet,
                        owner_idx: target_pet.pos,
                        trigger: TRIGGER_SELF_FAINT,
                        target: Target::Friend,
                        position: Position::OnSelf,
                        action: Action::Summon(Some(Box::new(leveled_pet.clone())), None),
                        uses: Some(1),
                        temp: true,
                    }];
                    info!(target: "dev", "(\"{}\")\nEvolving {}.", name, leveled_pet);
                    info!(target: "dev", "(\"{}\")\nSet pet {} to summon evolved pet on faint.", name, target_pet);
                } else {
                    return Err(Box::new(SAPTestError::InvalidTeamAction {
                        subject: "Evolve Pet".to_string(),
                        indices: vec![target_idx],
                        reason: format!("Cannot access position {rel_pos:?} or targeted pet."),
                    }));
                }
            }
            // TODO: May need to also choose to copy from an enemy pet at some point.
            Action::Copy(attr, target, pos) => {
                // Based on position, select the pet to copy.
                let chosen_pet = if *target == Target::Friend {
                    self._choose_pet(pos, target_idx)
                } else {
                    opponent._choose_pet(pos, target_idx)
                };
                let copied_attr = if let Some(pet) = chosen_pet {
                    match attr.clone() {
                        CopyAttr::Stats(replacement_stats) => Some(CopyAttr::Stats(
                            replacement_stats.map_or(Some(pet.stats), Some),
                        )),
                        CopyAttr::PercentStats(perc_stats_mult) => {
                            // Multiply the stats of a chosen pet by some multiplier
                            let mut new_stats = pet.stats * perc_stats_mult;
                            new_stats.clamp(MIN_PET_STATS, MAX_PET_STATS);
                            info!(
                                target: "dev", "(\"{}\")\nCopied {}% atk and {}% health from {}.",
                                name,
                                perc_stats_mult.attack,
                                perc_stats_mult.health,
                                pet
                            );
                            Some(CopyAttr::Stats(Some(new_stats)))
                        }
                        CopyAttr::Effect(_, lvl) => {
                            Some(CopyAttr::Effect(pet.get_effect(lvl.unwrap_or(1))?, lvl))
                        }
                        CopyAttr::Item(_) => pet
                            .item
                            .as_ref()
                            .map(|food| CopyAttr::Item(Some(Box::new(food.clone())))),
                        _ => None,
                    }
                } else {
                    None
                };

                // Chose the target of recipient of copied pet stats/effect.
                if let Some(target) = self.nth(target_idx) {
                    // Calculate stats or set ability.
                    match copied_attr.unwrap_or(CopyAttr::None) {
                        CopyAttr::Stats(new_stats) => {
                            // If some stats given use those as base.
                            let new_stats = if let Some(mut new_stats) = new_stats {
                                // If any stat value is 0, use the target's original stats, otherwise, use the new stats.
                                *new_stats.comp_set_value(&target.stats, 0)
                            } else {
                                // Otherwise, copy stats from target.
                                target.stats
                            };

                            target.stats = new_stats;

                            info!(
                                target: "dev", "(\"{}\")\nSet stats for {} to {}.",
                                name,
                                target,
                                target.stats
                            );
                        }
                        CopyAttr::Effect(effect, _) => {
                            target.effect = effect;
                            info!(
                                target: "dev", "(\"{}\")\nSet effect for {} to {:?}.",
                                name,
                                target,
                                target.effect
                            );
                        }
                        CopyAttr::Item(item) => {
                            if let Some(food) = item {
                                target.item = Some(*food);
                                info!(
                                    target: "dev", "(\"{}\")\nCopyied item for {} to {:?}.",
                                    name,
                                    target,
                                    target.item
                                );
                            }
                        }
                        CopyAttr::None => {}
                        CopyAttr::PercentStats(_) => {}
                    }
                    target_ids.push(target.id.clone())
                }
            }
            Action::Thorns(stats) => {
                let mut thorn_effect = effect.clone();
                thorn_effect.action = Action::Remove(*stats);
                self._target_effect_idx(target_idx, &thorn_effect, opponent)?;
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
        effect_pet_idx: usize,
        trigger: &Outcome,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>> {
        match &effect.position {
            Position::Any(condition) => {
                let mut rng = ChaCha12Rng::seed_from_u64(self.seed);
                let random_pet_pos = self
                    .get_pets_by_cond(condition)
                    .into_iter()
                    .filter_map(|pet| pet.pos)
                    .choose(&mut rng);
                if let Some(target_idx) = random_pet_pos {
                    self._target_effect_idx(target_idx, effect, opponent)?
                }
            }
            Position::All(condition) => {
                let pos = self
                    .get_pets_by_cond(condition)
                    .into_iter()
                    .filter_map(|pet| pet.pos)
                    .collect_vec();

                for pet_idx in pos {
                    self._target_effect_idx(pet_idx, effect, opponent)?
                }
            }
            Position::OnSelf => self._target_effect_idx(effect_pet_idx, effect, opponent)?,
            Position::Trigger => {
                let trigger_pos = trigger
                    .to_idx
                    .ok_or("No idx position given to apply effect.")?;
                self._target_effect_idx(trigger_pos, effect, opponent)?
            }
            Position::Relative(rel_pos) => {
                let (team, adj_idx) = self._cvt_rel_idx_to_adj_idx(effect_pet_idx, *rel_pos)?;
                // One team so should only target self team.
                if team == Target::Friend {
                    self._target_effect_idx(adj_idx, effect, opponent)?
                }
            }
            Position::Range(effect_range) => {
                let adj_idxs = effect_range
                    .clone()
                    .into_iter()
                    .filter_map(|rel_idx| {
                        self._cvt_rel_idx_to_adj_idx(effect_pet_idx, rel_idx).ok()
                    })
                    .collect_vec();
                for (team, adj_idx) in adj_idxs {
                    if team == Target::Friend {
                        self._target_effect_idx(adj_idx, effect, opponent)?
                    }
                }
            }
            Position::First => {
                if let Some(Some(first_index)) = self.all().first().map(|pet| pet.pos) {
                    self._target_effect_idx(first_index, effect, opponent)?
                }
            }
            Position::Last => {
                if let Some(last_index) = self.all().len().checked_sub(1) {
                    self._target_effect_idx(last_index, effect, opponent)?
                }
            }
            Position::Multiple(positions) => {
                for pos in positions {
                    // For each position:
                    // * Make a copy of the effect
                    // * Set the position to the desired position
                    // * Reduce the uses to 1 to not increase the number of times an effect is activated.
                    let mut effect_copy = effect.clone();
                    effect_copy.position = pos.clone();
                    effect_copy.uses = Some(1);

                    self._match_position_one_team(effect_pet_idx, trigger, &effect_copy, opponent)?
                }
            }
            Position::N(condition, n) => {
                let indices = self
                    .get_pets_by_cond(condition)
                    .into_iter()
                    .filter_map(|pet| pet.pos)
                    .collect_vec();
                // Get n values of indices.
                for idx in (0..*n).filter_map(|i| indices.get(i)) {
                    self._target_effect_idx(*idx, effect, opponent)?
                }
            }
            Position::Adjacent => {
                let (pos_1, pos_2) = (
                    effect_pet_idx.saturating_sub(1),
                    (effect_pet_idx + 1).clamp(0, self.friends.len()),
                );

                if pos_1 == effect_pet_idx || pos_2 == effect_pet_idx {
                    return Err("No adjacent pets".into());
                }
                if let Action::SwapPositions = effect.action {
                    self.swap_pets(pos_1, pos_2)?;
                    info!(target: "dev", "(\"{}\")\nSwapped positions for pets at positions {} and {}.", self.name, pos_1, pos_2);
                } else if let Action::SwapStats = effect.action {
                    self.swap_pet_stats(pos_1, pos_2)?;
                    info!(target: "dev", "(\"{}\")\nSwapped stats for pets at positions {} and {}.", self.name, pos_1, pos_2);
                } else {
                    for pos in [pos_1, pos_2].into_iter() {
                        self._target_effect_idx(pos, effect, opponent)?;
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn _match_position_either_team(
        &mut self,
        effect_pet_idx: usize,
        trigger: &Outcome,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>> {
        match &effect.position {
            Position::Relative(rel_pos) => {
                let (team, adj_idx) = self._cvt_rel_idx_to_adj_idx(effect_pet_idx, *rel_pos)?;
                match team {
                    Target::Enemy => opponent._target_effect_idx(adj_idx, effect, self)?,
                    Target::Friend => self._target_effect_idx(adj_idx, effect, opponent)?,
                    _ => unreachable!("Cannot return other types."),
                }
            }
            Position::All(condition) => {
                let pos = self
                    .get_pets_by_cond(condition)
                    .into_iter()
                    .filter_map(|pet| pet.pos)
                    .collect_vec();
                let enemy_pos = opponent
                    .get_pets_by_cond(condition)
                    .into_iter()
                    .filter_map(|pet| pet.pos)
                    .collect_vec();
                for pet_idx in pos {
                    self._target_effect_idx(pet_idx, effect, opponent)?
                }
                for pet_idx in enemy_pos {
                    opponent._target_effect_idx(pet_idx, effect, self)?
                }
            }
            Position::Multiple(positions) => {
                for pos in positions {
                    // For each position:
                    // * Make a copy of the effect
                    // * Set the position to the desired position
                    // * Reduce the uses to 1 to not double the number of times an effect is activated.
                    let mut effect_copy = effect.clone();
                    effect_copy.position = pos.clone();
                    effect_copy.uses = Some(1);

                    // Add outcome to outcomes.
                    self.apply_effect(effect_pet_idx, trigger, &effect_copy, opponent)?
                }
            }
            Position::Range(effect_range) => {
                let adj_idxs = effect_range
                    .clone()
                    .into_iter()
                    .filter_map(|rel_idx| {
                        self._cvt_rel_idx_to_adj_idx(effect_pet_idx, rel_idx).ok()
                    })
                    .collect_vec();
                for (team, adj_idx) in adj_idxs {
                    if team == Target::Friend {
                        self._target_effect_idx(adj_idx, effect, opponent)?
                    } else {
                        opponent._target_effect_idx(adj_idx, effect, self)?
                    }
                }
            }
            Position::Trigger => {
                if let Target::Friend = trigger.from_target {
                    let trigger_pos = trigger
                        .to_idx
                        .ok_or("No idx position given to apply effect.")?;
                    self._target_effect_idx(trigger_pos, effect, opponent)?;
                } else if let Target::Enemy = trigger.from_target {
                    let trigger_pos = trigger
                        .from_idx
                        .ok_or("No idx position given to apply effect.")?;
                    opponent._target_effect_idx(trigger_pos, effect, self)?;
                } else {
                    unimplemented!("Trigger cannot come from both teams.")
                }
            }
            Position::None => {}
            _ => unimplemented!("Position not implemented."),
        };
        Ok(())
    }
}
