use crate::common::{
    battle::{
        effect::Effect,
        state::{Action, CopyAttr, Outcome, Position, Target},
        team::{Battle, Team, TEAM_SIZE},
    },
    pets::combat::Combat,
};

use itertools::Itertools;
use log::info;
use rand::seq::IteratorRandom;
use std::{collections::VecDeque, error::Error};

pub trait EffectApply {
    fn _target_effect_any(&self, effect_type: &Action, outcomes: &mut VecDeque<Outcome>);
    fn _target_effect_all(&self, effect_type: &Action, outcomes: &mut VecDeque<Outcome>);
    fn _target_effect_specific(
        &self,
        pos: usize,
        effect_type: &Action,
        outcomes: &mut VecDeque<Outcome>,
    );
    // fn _target_effect_self(&self, trigger: Outcome, effect_type: &Action, outcomes: &mut VecDeque<Outcome>);
    fn _target_effect_trigger(
        &self,
        trigger: &Outcome,
        effect_type: &Action,
        outcomes: &mut VecDeque<Outcome>,
    ) -> Result<(), Box<dyn Error>>;
    fn _target_effect_onself(
        &self,
        effect_pet_idx: usize,
        effect_type: &Action,
        outcomes: &mut VecDeque<Outcome>,
    ) -> Result<(), Box<dyn Error>>;
    /// Apply effects based on a team's stored triggers.
    fn _apply_trigger_effects(&self, opponent: &Team) -> &Self;
    /// Apply a given effect to a team.
    fn _apply_effect(
        &self,
        effect_pet_idx: usize,
        trigger: Outcome,
        effect: Effect,
        opponent: &Team,
    ) -> Result<VecDeque<Outcome>, Box<dyn Error>>;
    fn _cvt_rel_pos_to_adj_idx(curr_idx: usize, rel_idx: isize) -> Result<usize, Box<dyn Error>>;
}

impl EffectApply for Team {
    fn _target_effect_trigger(
        &self,
        trigger: &Outcome,
        effect_type: &Action,
        outcomes: &mut VecDeque<Outcome>,
    ) -> Result<(), Box<dyn Error>> {
        let trigger_pos = trigger
            .idx
            .ok_or("No idx position given to apply effect.")?;
        match effect_type {
            Action::Add(stats) => {
                if let Some(target) = self.get_all_pets().get(trigger_pos) {
                    target.borrow_mut().stats.add(stats);
                    info!(target: "dev", "Added {} to {}.", stats, target.borrow());
                }
            }
            Action::Remove(stats) => {
                if let Some(target) = self.get_all_pets().get(trigger_pos) {
                    outcomes.extend(target.borrow_mut().indirect_attack(stats));
                    info!(target: "dev", "Removed {} from {}.", stats, target.borrow());
                }
            }
            Action::Gain(food) => {
                if let Some(target) = self.get_all_pets().get(trigger_pos) {
                    target.borrow_mut().item = Some(*food.clone());
                    info!(target: "dev", "Gave {:?} to {}.", food, target.borrow());
                }
            }
            // Must also emit EffectTrigger for summon.
            Action::Summon(pet) => {
                let summon_triggers = self.add_pet(pet, trigger_pos);
                if let Ok(summon_triggers) = summon_triggers {
                    outcomes.extend(summon_triggers.into_iter())
                }
            }
            // TODO: May need to also choose to copy from an enemy pet at some point.
            Action::Copy(attr, pos) => {
                // Chose the target of recipient of copied pet stats/effect.
                if let Some(target) = self.get_all_pets().get(trigger_pos) {
                    // Based on position, select the pet to copy.
                    let chosen_pet = match pos {
                        Position::Any => self.get_any_pet(),
                        Position::Specific(rel_pos) => {
                            if let Ok(adj_idx) =
                                Team::_cvt_rel_pos_to_adj_idx(trigger_pos, *rel_pos)
                            {
                                self.get_idx_pet(adj_idx)
                            } else {
                                None
                            }
                        }
                        Position::Condition(condition) => self.get_pet_by_cond(condition),
                        _ => None,
                    };
                    // Calculate stats or set ability.
                    if let Some(chosen_pet) = chosen_pet {
                        match attr.clone() {
                            CopyAttr::PercentStats(perc_stats_mult) => {
                                // Multiply the stats of a chosen pet by some multiplier
                                // If the stats are 0, use the target's original stats, otherwise, use the news stats.
                                let mut new_stats =
                                    chosen_pet.borrow().stats.mult(&perc_stats_mult);
                                let old_stats = target.borrow().stats.clone();
                                info!(
                                    target: "dev", "Copied {}% atk and {}% health from {} to {}.",
                                    perc_stats_mult.attack,
                                    perc_stats_mult.health,
                                    chosen_pet.borrow(),
                                    target.borrow()
                                );
                                target.borrow_mut().stats =
                                    new_stats.comp_set_value(&old_stats, 0).clone()
                            }
                            CopyAttr::Effect => {
                                target.borrow_mut().effect = chosen_pet.borrow().effect.clone()
                            }
                        }
                        info!(target: "dev", "Copied to {}.", target.borrow());
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn _target_effect_onself(
        &self,
        effect_pet_idx: usize,
        effect_type: &Action,
        outcomes: &mut VecDeque<Outcome>,
    ) -> Result<(), Box<dyn Error>> {
        match effect_type {
            Action::Add(stats) => {
                if let Some(target) = self.get_all_pets().get(effect_pet_idx) {
                    target.borrow_mut().stats.add(stats);
                    info!(target: "dev", "Added {} to {}.", stats, target.borrow());
                }
            }
            Action::Remove(stats) => {
                if let Some(target) = self.get_all_pets().get(effect_pet_idx) {
                    outcomes.extend(target.borrow_mut().indirect_attack(stats));
                    info!(target: "dev", "Removed {} from {}.", stats, target.borrow());
                }
            }
            Action::Gain(food) => {
                if let Some(target) = self.get_all_pets().get(effect_pet_idx) {
                    target.borrow_mut().item = Some(*food.clone());
                    info!(target: "dev", "Gave {:?} to {}.", food, target.borrow());
                }
            }
            // Must also emit EffectTrigger for summon.
            Action::Summon(pet) => {
                let summon_triggers = self.add_pet(pet, effect_pet_idx);
                if let Ok(summon_triggers) = summon_triggers {
                    outcomes.extend(summon_triggers.into_iter())
                }
            }
            // TODO: May need to also choose to copy from an enemy pet at some point.
            Action::Copy(attr, pos) => {
                // Chose the target of recipient of copied pet stats/effect.
                if let Some(target) = self.get_all_pets().get(effect_pet_idx) {
                    // Based on position, select the pet to copy.
                    let chosen_pet = match pos {
                        Position::Any => self.get_any_pet(),
                        Position::Specific(rel_pos) => {
                            if let Ok(adj_idx) =
                                Team::_cvt_rel_pos_to_adj_idx(effect_pet_idx, *rel_pos)
                            {
                                self.get_idx_pet(adj_idx)
                            } else {
                                None
                            }
                        }
                        Position::Condition(condition) => self.get_pet_by_cond(condition),
                        _ => None,
                    };
                    // Calculate stats or set ability.
                    if let Some(chosen_pet) = chosen_pet {
                        match attr.clone() {
                            CopyAttr::PercentStats(perc_stats_mult) => {
                                // Multiply the stats of a chosen pet by some multiplier
                                // If the stats are 0, use the target's original stats, otherwise, use the news stats.
                                let mut new_stats =
                                    chosen_pet.borrow().stats.mult(&perc_stats_mult);
                                let old_stats = target.borrow().stats.clone();
                                info!(
                                    target: "dev", "Copied {}% atk and {}% health from {} to {}.",
                                    perc_stats_mult.attack,
                                    perc_stats_mult.health,
                                    chosen_pet.borrow(),
                                    target.borrow()
                                );
                                target.borrow_mut().stats =
                                    new_stats.comp_set_value(&old_stats, 0).clone()
                            }
                            CopyAttr::Effect => {
                                target.borrow_mut().effect = chosen_pet.borrow().effect.clone()
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn _target_effect_any(&self, effect_type: &Action, outcomes: &mut VecDeque<Outcome>) {
        match effect_type {
            Action::Add(stats) => {
                if let Some(target) = self.get_any_pet() {
                    target.borrow_mut().stats.add(stats);
                    info!(target: "dev", "Added {} to {}.", stats, target.borrow());
                }
            }
            Action::Remove(stats) => {
                if let Some(target) = self.get_any_pet() {
                    outcomes.extend(target.borrow_mut().indirect_attack(stats));
                    info!(target: "dev", "Removed {} from {}.", stats, target.borrow());
                }
            }
            Action::Gain(food) => {
                if let Some(target) = self.get_any_pet() {
                    target.borrow_mut().item = Some(*food.clone());
                    info!(target: "dev", "Gave {:?} to {}.", food, target.borrow());
                }
            }
            // Must also emit EffectTrigger for summon.
            Action::Summon(pet) => {
                let mut rng = rand::thread_rng();
                let random_pos = (0..5).choose(&mut rng).unwrap() as usize;

                let summon_triggers = self.add_pet(pet, random_pos);
                if let Ok(summon_triggers) = summon_triggers {
                    outcomes.extend(summon_triggers.into_iter())
                }
            }
            _ => {}
        }
    }

    fn _target_effect_all(&self, effect_type: &Action, outcomes: &mut VecDeque<Outcome>) {
        match effect_type {
            Action::Add(stats) => {
                for pet in self.get_all_pets() {
                    pet.borrow_mut().stats.add(stats);
                    info!(target: "dev", "Added {} to {}.", stats, pet.borrow());
                }
            }
            Action::Remove(stats) => {
                for pet in self.get_all_pets() {
                    outcomes.extend(pet.borrow_mut().indirect_attack(stats));
                    info!(target: "dev", "Removed {} from {}.", stats, pet.borrow());
                }
            }
            _ => {}
        }
    }

    fn _target_effect_specific(
        &self,
        pos: usize,
        effect_type: &Action,
        outcomes: &mut VecDeque<Outcome>,
    ) {
        match effect_type {
            Action::Add(stats) => {
                if let Some(affected_pet) = self.get_all_pets().get(pos) {
                    affected_pet.borrow_mut().stats.add(stats);
                    info!(target: "dev", "Added {} to {}.", stats, affected_pet.borrow())
                }
            }
            Action::Remove(stats) => {
                if let Some(affected_pet) = self.get_all_pets().get(pos) {
                    info!(target: "dev", "Removed {} from {}.", stats, affected_pet.borrow());
                    outcomes.extend(affected_pet.borrow_mut().indirect_attack(stats));
                }
            }
            Action::Gain(food) => {
                if let Some(affected_pet) = self.get_all_pets().get(pos) {
                    info!(target: "dev", "Gave {:?} to {}.", food, affected_pet.borrow());
                    affected_pet.borrow_mut().item = Some(*food.clone())
                }
            }
            Action::Summon(pet) => {
                let summon_triggers = self.add_pet(pet, pos);
                if let Ok(summon_triggers) = summon_triggers {
                    outcomes.extend(summon_triggers.into_iter())
                }
            }
            _ => {}
        }
    }

    fn _cvt_rel_pos_to_adj_idx(curr_idx: usize, rel_idx: isize) -> Result<usize, Box<dyn Error>> {
        let effect_pet_idx = isize::try_from(curr_idx)?;
        // Negative idx means behind.
        // Positive idx mean ahead.
        // We adjust so within bounds of team.
        let adj_idx = if rel_idx.is_negative() {
            -rel_idx + effect_pet_idx
        } else {
            let new_idx = effect_pet_idx - rel_idx;
            if new_idx.is_negative() {
                return Err("Invalid index.".into());
            } else {
                new_idx
            }
        };
        Ok(adj_idx.clamp(0, TEAM_SIZE.try_into()?).try_into()?)
    }
    fn _apply_effect(
        &self,
        effect_pet_idx: usize,
        trigger: Outcome,
        effect: Effect,
        opponent: &Team,
    ) -> Result<VecDeque<Outcome>, Box<dyn Error>> {
        // Store all outcomes from applying effects.
        // TODO: Look into changing so can use triggers from Team struct. Issues since iterating at same time.
        let mut outcomes: VecDeque<Outcome> = VecDeque::new();

        // Activate effect for each use.
        for _ in 0..effect.uses.unwrap_or(1) {
            match &effect.target {
                Target::Friend => match &effect.position {
                    Position::Any => self._target_effect_any(&effect.action, &mut outcomes),
                    Position::All => self._target_effect_all(&effect.action, &mut outcomes),
                    Position::OnSelf => {
                        self._target_effect_onself(effect_pet_idx, &effect.action, &mut outcomes)?
                    }
                    Position::Trigger => {
                        self._target_effect_trigger(&trigger, &effect.action, &mut outcomes)?
                    }
                    // Position::Trigger => self._target_effect_trigger(trigger, &effect.effect, &mut outcomes),
                    Position::Specific(rel_pos) => {
                        if let Ok(adj_idx) = Team::_cvt_rel_pos_to_adj_idx(effect_pet_idx, *rel_pos)
                        {
                            self._target_effect_specific(adj_idx, &effect.action, &mut outcomes)
                        }
                    }
                    Position::Range(_) => {}
                    _ => {}
                },
                Target::Enemy => match &effect.position {
                    Position::Any => opponent._target_effect_any(&effect.action, &mut outcomes),
                    Position::All => opponent._target_effect_all(&effect.action, &mut outcomes),
                    Position::OnSelf => opponent._target_effect_onself(
                        effect_pet_idx,
                        &effect.action,
                        &mut outcomes,
                    )?,
                    Position::Trigger => {
                        opponent._target_effect_trigger(&trigger, &effect.action, &mut outcomes)?
                    }
                    // Position::Trigger => self._target_effect_trigger(trigger, &effect.effect, &mut outcomes),
                    Position::Specific(rel_pos) => {
                        if let Ok(adj_idx) = Team::_cvt_rel_pos_to_adj_idx(effect_pet_idx, *rel_pos)
                        {
                            opponent._target_effect_specific(adj_idx, &effect.action, &mut outcomes)
                        }
                    }
                    Position::Range(_) => {}
                    _ => {}
                },
                Target::None => {}
            };
        }

        info!(target: "dev", "Triggers:\n{:?}", outcomes);
        Ok(outcomes)
    }

    fn _apply_trigger_effects(&self, opponent: &Team) -> &Self {
        // Get ownership of current triggers and clear team triggers.
        let mut curr_triggers = self.triggers.borrow_mut().to_owned();
        self.triggers.borrow_mut().clear();

        // Continue iterating until all triggers consumed.
        while let Some(trigger) = curr_triggers.pop_front() {
            let mut applied_effects: Vec<(usize, Outcome, Effect)> = vec![];

            // Iterate through pets in descending order by attack strength collecting valid effects.
            for (effect_pet_idx, pet) in self
                .friends
                .borrow()
                .iter()
                .enumerate()
                .sorted_by(|(_, pet_1), (_, pet_2)| {
                    pet_1
                        .as_ref()
                        .map_or(0, |pet| pet.borrow().stats.attack)
                        .cmp(&pet_2.as_ref().map_or(0, |pet| pet.borrow().stats.attack))
                })
                .rev()
            {
                // This checks whether or not a trigger should cause a pet's effect to activate.
                // * Effects that trigger on Any position are automatically allowed.
                // * Tests trigger idx so that multiple triggers aren't all activated.
                //     * For pets with Position::OnSelf and idx 0 like Crickets.
                if trigger.position != Position::Any
                    && trigger.idx.is_some()
                    && trigger.idx != Some(effect_pet_idx)
                {
                    continue;
                }

                // Get food and pet effect based on if its trigger is equal to current trigger, if any.
                if let Some(Some(food_effect)) = pet.as_ref().map(|pet| {
                    pet.borrow()
                        .item
                        .as_ref()
                        .filter(|food| food.ability.trigger == trigger)
                        .map(|food| food.ability.clone())
                }) {
                    applied_effects.push((effect_pet_idx, trigger.clone(), food_effect))
                };
                if let Some(Some(pet_effect)) = pet
                    .as_ref()
                    .filter(|pet| {
                        if let Some(effect) = &pet.borrow().effect {
                            effect.trigger == trigger
                        } else {
                            false
                        }
                    })
                    .map(|pet| pet.borrow().effect.clone())
                {
                    applied_effects.push((effect_pet_idx, trigger.clone(), pet_effect))
                };
            }
            // Apply effects.
            // Extend in reverse so proper order followed.
            curr_triggers.extend(
                applied_effects
                    .into_iter()
                    .rev()
                    .filter_map(|(effect_pet_idx, trigger, effect)| {
                        self._apply_effect(effect_pet_idx, trigger, effect, opponent)
                            .ok()
                    })
                    .into_iter()
                    .flatten(),
            );
        }
        self
    }
}
