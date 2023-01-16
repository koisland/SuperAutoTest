use crate::common::{
    battle::{
        effect::Effect,
        state::{Action, CopyAttr, Outcome, Position, Status, Target},
        team::Team,
    },
    pets::{
        combat::Combat,
        pet::{Pet, MAX_PET_STATS, MIN_PET_STATS},
    },
};

use itertools::Itertools;
use log::info;
use rand::seq::IteratorRandom;
use std::{cell::RefCell, error::Error, ops::RangeInclusive, rc::Rc};

pub trait EffectApply {
    fn _target_effect_any(&mut self, effect_type: &Action) -> Result<(), Box<dyn Error>>;
    fn _target_effect_all(&mut self, effect_type: &Action) -> Result<(), Box<dyn Error>>;
    fn _target_effect_specific(
        &mut self,
        pos: usize,
        effect_type: &Action,
    ) -> Result<(), Box<dyn Error>>;
    // fn _target_effect_self(&mut self, trigger: Outcome, effect_type: &Action, outcomes: &mut VecDeque<Outcome>);
    fn _target_effect_trigger(
        &mut self,
        trigger: &Outcome,
        effect_type: &Action,
    ) -> Result<(), Box<dyn Error>>;
    fn _target_effect_onself(
        &mut self,
        effect_pet_idx: usize,
        effect_type: &Action,
    ) -> Result<(), Box<dyn Error>>;
    fn _target_effect_range(
        &mut self,
        effect_pet_idx: usize,
        range_idxs: &RangeInclusive<isize>,
        effect_type: &Action,
    ) -> Result<(), Box<dyn Error>>;
    fn _target_effect_condition(
        &mut self,
        pet: Rc<RefCell<Pet>>,
        effect_type: &Action,
    ) -> Result<(), Box<dyn Error>>;
    /// Apply effects based on a team's stored triggers.
    fn apply_trigger_effects(&mut self, opponent: &mut Team) -> &mut Self;
    /// Apply a given effect to a team.
    fn _apply_effect(
        &mut self,
        effect_pet_idx: usize,
        trigger: Outcome,
        effect: Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>>;
    fn _match_position_one_team(
        &mut self,
        effect_pet_idx: usize,
        trigger: &Outcome,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>>;
    fn _match_position_either_team(
        &mut self,
        effect_pet_idx: usize,
        trigger: &Outcome,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>>;
    fn _cvt_rel_pos_to_adj_idx(
        &mut self,
        curr_idx: usize,
        rel_idx: isize,
    ) -> Result<(Target, usize), Box<dyn Error>>;
}

impl EffectApply for Team {
    fn _target_effect_trigger(
        &mut self,
        trigger: &Outcome,
        effect_type: &Action,
    ) -> Result<(), Box<dyn Error>> {
        let trigger_pos = trigger
            .idx
            .ok_or("No idx position given to apply effect.")?;
        match effect_type {
            Action::Add(stats) => {
                if let Some(target) = self.get_all_pets().get(trigger_pos) {
                    target.borrow_mut().stats += stats.clone();
                    info!(target: "dev", "(\"{}\")\nAdded {} to {}.", self.name, stats, target.borrow());
                }
            }
            Action::Remove(stats) => {
                if let Some(target) = self.get_all_pets().get(trigger_pos) {
                    self.triggers
                        .extend(target.borrow_mut().indirect_attack(stats));
                    info!(target: "dev", "(\"{}\")\nRemoved {} from {}.", self.name, stats.clone().invert(), target.borrow());
                }
            }
            Action::Gain(food) => {
                if let Some(target) = self.get_all_pets().get(trigger_pos) {
                    target.borrow_mut().set_item(Some(*food.clone()));
                    info!(target: "dev", "(\"{}\")\nGave {:?} to {}.", self.name, food, target.borrow());
                }
            }
            // Must also emit EffectTrigger for summon.
            Action::Summon(pet) => {
                if self.add_pet(pet, trigger_pos).is_err() {
                    info!(target: "dev", "(\"{}\")\nCouldn't summon {:?} to {}.", self.name, pet, trigger_pos);
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
                            let (team, adj_idx) =
                                self._cvt_rel_pos_to_adj_idx(trigger_pos, *rel_pos)?;
                            if team == Target::Friend {
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
                                let mut new_stats = chosen_pet.borrow().stats.clone();
                                new_stats *= perc_stats_mult.clone();
                                new_stats.clamp(MIN_PET_STATS, MAX_PET_STATS);

                                let old_stats = target.borrow().stats.clone();
                                info!(
                                    target: "dev", "(\"{}\")\nCopied {}% atk and {}% health from {} to {}.",
                                    self.name,
                                    perc_stats_mult.attack,
                                    perc_stats_mult.health,
                                    chosen_pet.borrow(),
                                    target.borrow()
                                );
                                target.borrow_mut().stats =
                                    new_stats.comp_set_value(&old_stats, 0).clone()
                            }
                            CopyAttr::Effect => {
                                info!(
                                    target: "dev", "(\"{}\")\nCopied effect from {} to {}.",
                                    self.name,
                                    chosen_pet.borrow(),
                                    target.borrow()
                                );
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
    fn _target_effect_onself(
        &mut self,
        effect_pet_idx: usize,
        effect_type: &Action,
    ) -> Result<(), Box<dyn Error>> {
        match effect_type {
            Action::Add(stats) => {
                if let Some(target) = self.get_idx_pet(effect_pet_idx) {
                    target.borrow_mut().stats += stats.clone();
                    info!(target: "dev", "(\"{}\")\nAdded {} to {}.", self.name, stats, target.borrow());
                }
            }
            Action::Remove(stats) => {
                if let Some(target) = self.get_idx_pet(effect_pet_idx) {
                    self.triggers
                        .extend(target.borrow_mut().indirect_attack(stats));
                    info!(target: "dev", "(\"{}\")\nRemoved {} from {}.", self.name, stats.clone().invert(), target.borrow());
                }
            }
            Action::Gain(food) => {
                if let Some(target) = self.get_idx_pet(effect_pet_idx) {
                    target.borrow_mut().set_item(Some(*food.clone()));
                    info!(target: "dev", "(\"{}\")\nGave {:?} to {}.", self.name, food, target.borrow());
                }
            }
            Action::Summon(pet) => {
                if self.add_pet(pet, effect_pet_idx).is_err() {
                    info!(target: "dev", "(\"{}\")\nCouldn't summon {:?} to {}.", self.name, pet, effect_pet_idx);
                }
            }
            Action::Multiple(actions) => {
                for action in actions {
                    self._target_effect_onself(effect_pet_idx, action)?
                }
            }
            // TODO: May need to also choose to copy from an enemy pet at some point.
            Action::Copy(attr, pos) => {
                // Chose the target of recipient of copied pet stats/effect.
                if let Some(target) = self.get_idx_pet(effect_pet_idx) {
                    // Based on position, select the pet to copy.
                    let chosen_pet = match pos {
                        Position::Any => self.get_any_pet(),
                        Position::Specific(rel_pos) => {
                            let (team, adj_idx) =
                                self._cvt_rel_pos_to_adj_idx(effect_pet_idx, *rel_pos)?;
                            if team == Target::Friend {
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
                                let mut new_stats = chosen_pet.borrow().stats.clone();
                                new_stats *= perc_stats_mult.clone();
                                new_stats.clamp(MIN_PET_STATS, MAX_PET_STATS);

                                let old_stats = target.borrow().stats.clone();

                                info!(
                                    target: "dev", "(\"{}\")\nCopied {}% atk and {}% health from {} to {}.",
                                    self.name,
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

    fn _target_effect_any(&mut self, effect_type: &Action) -> Result<(), Box<dyn Error>> {
        match effect_type {
            Action::Add(stats) => {
                if let Some(target) = self.get_any_pet() {
                    target.borrow_mut().stats += stats.clone();
                    info!(target: "dev", "(\"{}\")\nAdded {} to {}.", self.name, stats, target.borrow());
                }
            }
            Action::Remove(stats) => {
                if let Some(target) = self.get_any_pet() {
                    self.triggers
                        .extend(target.borrow_mut().indirect_attack(stats));
                    info!(target: "dev", "(\"{}\")\nRemoved {} from {}.", self.name, stats.clone().invert(), target.borrow());
                }
            }
            Action::Gain(food) => {
                if let Some(target) = self.get_any_pet() {
                    target.borrow_mut().set_item(Some(*food.clone()));
                    info!(target: "dev", "(\"{}\")\nGave {:?} to {}.", self.name, food, target.borrow());
                }
            }
            // Must also emit EffectTrigger for summon.
            Action::Summon(pet) => {
                let mut rng = rand::thread_rng();
                let random_pos = (0..5).choose(&mut rng).unwrap() as usize;
                if self.add_pet(pet, random_pos).is_err() {
                    info!(target: "dev", "(\"{}\")\nCouldn't summon {:?} to {}.", self.name, pet, random_pos);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn _target_effect_all(&mut self, effect_type: &Action) -> Result<(), Box<dyn Error>> {
        match effect_type {
            Action::Add(stats) => {
                for pet in self.get_all_pets() {
                    pet.borrow_mut().stats += stats.clone();
                    info!(target: "dev", "(\"{}\")\nAdded {} to {}.", self.name, stats, pet.borrow());
                }
            }
            Action::Remove(stats) => {
                for pet in self.get_all_pets() {
                    self.triggers
                        .extend(pet.borrow_mut().indirect_attack(stats));
                    info!(target: "dev", "(\"{}\")\nRemoved {} from {}.", self.name, stats.clone().invert(), pet.borrow());
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn _target_effect_range(
        &mut self,
        effect_pet_idx: usize,
        range_idxs: &RangeInclusive<isize>,
        effect_type: &Action,
    ) -> Result<(), Box<dyn Error>> {
        let pet_ranges = range_idxs
            .clone()
            .into_iter()
            .filter_map(|rel_idx| self._cvt_rel_pos_to_adj_idx(effect_pet_idx, rel_idx).ok())
            .collect_vec();

        for (target, pos) in pet_ranges {
            // self._cvt_rel_pos_to_adj_idx may return a pet idx from either team.
            // Because each _target_* method operates only on a single team, it must match on own team to be valid.
            match (target, effect_type) {
                (Target::Friend, Action::Add(stats)) => {
                    if let Some(pet) = self.get_idx_pet(pos) {
                        pet.borrow_mut().stats += stats.clone();
                        info!(target: "dev", "(\"{}\")\nAdded {} to {}.", self.name, stats, pet.borrow());
                    }
                }
                (Target::Friend, Action::Remove(stats)) => {
                    if let Some(pet) = self.get_idx_pet(pos) {
                        self.triggers
                            .extend(pet.borrow_mut().indirect_attack(stats));
                        info!(target: "dev", "(\"{}\")\nRemoved {} from {}.", self.name, stats.clone().invert(), pet.borrow());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn _target_effect_specific(
        &mut self,
        pos: usize,
        effect_type: &Action,
    ) -> Result<(), Box<dyn Error>> {
        match effect_type {
            Action::Add(stats) => {
                if let Some(affected_pet) = self.get_all_pets().get(pos) {
                    affected_pet.borrow_mut().stats += stats.clone();
                    info!(target: "dev", "(\"{}\")\nAdded {} to {}.", self.name, stats, affected_pet.borrow())
                }
            }
            Action::Remove(stats) => {
                if let Some(affected_pet) = self.get_all_pets().get(pos) {
                    info!(target: "dev", "(\"{}\")\nRemoved {} from {}.", self.name, stats.clone().invert(), affected_pet.borrow());
                    self.triggers
                        .extend(affected_pet.borrow_mut().indirect_attack(stats));
                }
            }
            Action::Gain(food) => {
                if let Some(affected_pet) = self.get_all_pets().get(pos) {
                    info!(target: "dev", "(\"{}\")\nGave {:?} to {}.", self.name, food, affected_pet.borrow());
                    affected_pet.borrow_mut().set_item(Some(*food.clone()));
                }
            }
            Action::Summon(pet) => {
                self.add_pet(pet, pos)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn _target_effect_condition(
        &mut self,
        pet: Rc<RefCell<Pet>>,
        effect_type: &Action,
    ) -> Result<(), Box<dyn Error>> {
        match effect_type {
            Action::Remove(stats) => {
                info!(target: "dev", "(\"{}\")\nRemoved {} from {}.", self.name, stats.clone().invert(), pet.borrow());
                self.triggers
                    .extend(pet.borrow_mut().indirect_attack(stats));
            }
            Action::Add(_) => {}
            _ => {}
        }

        Ok(())
    }

    /// Calculates an adjusted index based on the current index and a relative index.
    /// * `:param curr_idx:` The current index.
    /// * `:param rel_idx:` Number of positions relative to the current index.
    ///     * If *negative*, the index is **behind** the current index.
    ///     * If *positive*, the index is **ahead** of the current index.
    ///
    /// Output:
    /// * Value of the new index on a team represented by a variant in the enum `Target`.
    fn _cvt_rel_pos_to_adj_idx(
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
            Position::Any => self._target_effect_any(&effect.action)?,
            Position::All => self._target_effect_all(&effect.action)?,
            Position::OnSelf => self._target_effect_onself(effect_pet_idx, &effect.action)?,
            Position::Trigger => self._target_effect_trigger(trigger, &effect.action)?,
            // Position::Trigger => self._target_effect_trigger(trigger, &effect.effect),
            Position::Specific(rel_pos) => {
                let (team, adj_idx) = self._cvt_rel_pos_to_adj_idx(effect_pet_idx, *rel_pos)?;
                if team == Target::Friend {
                    self._target_effect_specific(adj_idx, &effect.action)?
                }
            }
            Position::Range(effect_range) => {
                self._target_effect_range(effect_pet_idx, effect_range, &effect.action)?
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

                    self._apply_effect(effect_pet_idx, trigger.clone(), effect_copy, opponent)?
                }
            }
            Position::Condition(condition) => {
                if let Some(pet) = self.get_pet_by_cond(condition) {
                    self._target_effect_condition(pet, &effect.action)?
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
            Position::Specific(rel_pos) => {
                let (team, mut adj_idx) = self._cvt_rel_pos_to_adj_idx(effect_pet_idx, *rel_pos)?;
                match team {
                    Target::Enemy => opponent._target_effect_specific(adj_idx, &effect.action)?,
                    Target::Friend => {
                        // Adjust index if targets specific friendly position and also faints.
                        if let Status::Faint = trigger.status {
                            adj_idx -= 1
                        }
                        self._target_effect_specific(adj_idx, &effect.action)?
                    }
                    _ => unreachable!("Cannot return other types."),
                }
            }
            Position::All => {
                self._target_effect_all(&effect.action)?;
                opponent._target_effect_all(&effect.action)?;
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
                    self._apply_effect(effect_pet_idx, trigger.clone(), effect_copy, opponent)?
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn _apply_effect(
        &mut self,
        effect_pet_idx: usize,
        trigger: Outcome,
        effect: Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>> {
        // Activate effect for each use.
        for _ in 0..effect.uses.unwrap_or(1) {
            match effect.target {
                Target::Friend => {
                    self._match_position_one_team(effect_pet_idx, &trigger, &effect, opponent)?
                }
                Target::Enemy => {
                    opponent._match_position_one_team(effect_pet_idx, &trigger, &effect, self)?
                }
                Target::Either => {
                    self._match_position_either_team(effect_pet_idx, &trigger, &effect, opponent)?
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn apply_trigger_effects(&mut self, opponent: &mut Team) -> &mut Self {
        // Get ownership of current triggers and clear team triggers.
        let mut curr_triggers = self.triggers.to_owned();
        self.triggers.clear();

        info!(target: "dev", "(\"{}\")\nTriggers:\n{}", self.name, curr_triggers.iter().join("\n"));

        // Continue iterating until all triggers consumed.
        while let Some(trigger) = curr_triggers.pop_front() {
            let mut applied_effects: Vec<(usize, Outcome, Effect)> = vec![];

            // Iterate through pets in descending order by attack strength collecting valid effects.
            for (effect_pet_idx, pet) in self
                .friends
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
            for (effect_pet_idx, trigger, effect) in applied_effects.into_iter().rev() {
                if let Err(err) = self._apply_effect(effect_pet_idx, trigger, effect, opponent) {
                    println!("(\"{}\")\nSomething went wrong. {:?}", self.name, err)
                };
            }
            curr_triggers.extend(self.triggers.iter().cloned());
            self.triggers.clear();
        }
        self
    }
}
