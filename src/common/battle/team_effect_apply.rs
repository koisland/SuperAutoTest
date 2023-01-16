use crate::common::{
    battle::{
        effect::Effect,
        state::{Action, CopyAttr, Outcome, Position, Target},
        team::Team,
    },
    pets::{
        combat::Combat,
        pet::{MAX_PET_STATS, MIN_PET_STATS},
    },
};

use itertools::Itertools;
use log::info;
use rand::seq::IteratorRandom;
use std::error::Error;

pub trait EffectApply {
    fn _target_effect_specific(
        &mut self,
        target_pos: usize,
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
    fn _target_effect_specific(
        &mut self,
        target_pos: usize,
        effect_type: &Action,
    ) -> Result<(), Box<dyn Error>> {
        let name = self.name.clone();
        match effect_type {
            Action::Add(stats) => {
                if let Some(target) = self.get_idx_pet(target_pos) {
                    target.stats += stats.clone();
                    info!(target: "dev", "(\"{}\")\nAdded {} to {}.", name, stats, target);
                }
            }
            Action::Remove(stats) => {
                let mut outcomes: Vec<Outcome> = vec![];
                if let Some(target) = self.get_idx_pet(target_pos) {
                    outcomes.extend(target.indirect_attack(stats));
                    info!(target: "dev", "(\"{}\")\nRemoved {} from {}.", name, stats.clone().invert(), target);
                }
                self.triggers.extend(outcomes)
            }
            Action::Gain(food) => {
                if let Some(target) = self.get_idx_pet(target_pos) {
                    target.set_item(Some(*food.clone()));
                    info!(target: "dev", "(\"{}\")\nGave {:?} to {}.", name, food, target);
                }
            }
            Action::Summon(pet) => {
                if self.add_pet(pet, target_pos).is_err() {
                    info!(target: "dev", "(\"{}\")\nCouldn't summon {:?} to {}.", name, pet, target_pos);
                }
            }
            Action::Multiple(actions) => {
                for action in actions {
                    self._target_effect_specific(target_pos, action)?
                }
            }
            // TODO: May need to also choose to copy from an enemy pet at some point.
            Action::Copy(attr, pos) => {
                // Based on position, select the pet to copy.
                let chosen_pet = match pos {
                    Position::Any => self.get_any_pet(),
                    Position::Specific(rel_pos) => {
                        let (team, adj_idx) = self._cvt_rel_pos_to_adj_idx(target_pos, *rel_pos)?;
                        if team == Target::Friend {
                            self.get_idx_pet(adj_idx)
                        } else {
                            None
                        }
                    }
                    Position::Condition(condition) => {
                        self.get_pet_by_cond(condition).map(|(_, pet)| pet)
                    }
                    _ => None,
                };
                let copied_attr = if let Some(pet) = chosen_pet {
                    match attr.clone() {
                        CopyAttr::PercentStats(perc_stats_mult) => {
                            // Multiply the stats of a chosen pet by some multiplier
                            let mut new_stats = pet.stats.clone();
                            new_stats *= perc_stats_mult.clone();
                            new_stats.clamp(MIN_PET_STATS, MAX_PET_STATS);
                            info!(
                                target: "dev", "(\"{}\")\nCopied {}% atk and {}% health from {}.",
                                name,
                                perc_stats_mult.attack,
                                perc_stats_mult.health,
                                pet
                            );
                            Some(CopyAttr::Stats(new_stats))
                        }
                        CopyAttr::Effect(_) => Some(CopyAttr::Effect(Box::new(pet.effect.clone()))),
                        _ => None,
                    }
                } else {
                    None
                };

                // Chose the target of recipient of copied pet stats/effect.
                if let Some(target) = self.get_idx_pet(target_pos) {
                    // Calculate stats or set ability.
                    match copied_attr.unwrap_or(CopyAttr::None) {
                        CopyAttr::Stats(mut new_stats) => {
                            // If the stats are 0, use the target's original stats, otherwise, use the news stats.
                            let old_stats = target.stats.clone();

                            target.stats = new_stats.comp_set_value(&old_stats, 0).clone();

                            info!(
                                target: "dev", "(\"{}\")\nSet stats for {} to {}.",
                                name,
                                target,
                                target.stats
                            );
                        }
                        CopyAttr::Effect(effect) => {
                            target.effect = *effect;
                            info!(
                                target: "dev", "(\"{}\")\nSet effect for {} to {:?}.",
                                name,
                                target,
                                target.effect
                            );
                        }
                        CopyAttr::None => {}
                        CopyAttr::PercentStats(_) => {}
                    }
                }
            }
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
    ) -> Result<(), Box<dyn Error>> {
        match &effect.position {
            Position::Any => {
                let mut rng = rand::thread_rng();
                if let Some(Some(random_pet_idx)) = self
                    .get_all_pets()
                    .iter()
                    .choose(&mut rng)
                    .map(|pet| pet.pos)
                {
                    self._target_effect_specific(random_pet_idx, &effect.action)?
                }
            }
            Position::All => {
                for pet_idx in 0..self.get_all_pets().len() {
                    self._target_effect_specific(pet_idx, &effect.action)?
                }
            }
            Position::OnSelf => self._target_effect_specific(effect_pet_idx, &effect.action)?,
            Position::Trigger => {
                let trigger_pos = trigger
                    .idx
                    .ok_or("No idx position given to apply effect.")?;
                self._target_effect_specific(trigger_pos, &effect.action)?
            }
            Position::Specific(rel_pos) => {
                let (team, adj_idx) = self._cvt_rel_pos_to_adj_idx(effect_pet_idx, *rel_pos)?;
                // One team so should only target self team.
                if team == Target::Friend {
                    self._target_effect_specific(adj_idx, &effect.action)?
                }
            }
            Position::Range(effect_range) => {
                let adj_idxs = effect_range
                    .clone()
                    .into_iter()
                    .filter_map(|rel_idx| {
                        self._cvt_rel_pos_to_adj_idx(effect_pet_idx, rel_idx).ok()
                    })
                    .collect_vec();
                for (team, adj_idx) in adj_idxs {
                    if team == Target::Friend {
                        self._target_effect_specific(adj_idx, &effect.action)?
                    }
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

                    self._match_position_one_team(effect_pet_idx, trigger, &effect_copy)?
                }
            }
            Position::Condition(condition) => {
                if let Some((idx, _)) = self.get_pet_by_cond(condition) {
                    self._target_effect_specific(idx, &effect.action)?
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
                let (team, adj_idx) = self._cvt_rel_pos_to_adj_idx(effect_pet_idx, *rel_pos)?;
                match team {
                    Target::Enemy => opponent._target_effect_specific(adj_idx, &effect.action)?,
                    Target::Friend => self._target_effect_specific(adj_idx, &effect.action)?,
                    _ => unreachable!("Cannot return other types."),
                }
            }
            Position::All => {
                for pet_idx in 0..self.get_all_pets().len() {
                    self._target_effect_specific(pet_idx, &effect.action)?
                }
                for pet_idx in 0..opponent.get_all_pets().len() {
                    opponent._target_effect_specific(pet_idx, &effect.action)?
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
                    self._match_position_one_team(effect_pet_idx, &trigger, &effect)?
                }
                Target::Enemy => {
                    opponent._match_position_one_team(effect_pet_idx, &trigger, &effect)?
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
                        .map_or(0, |pet| pet.stats.attack)
                        .cmp(&pet_2.as_ref().map_or(0, |pet| pet.stats.attack))
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
                    pet.item
                        .as_ref()
                        .filter(|food| food.ability.trigger == trigger)
                        .map(|food| food.ability.clone())
                }) {
                    applied_effects.push((effect_pet_idx, trigger.clone(), food_effect))
                };
                if let Some(Some(pet_effect)) = pet
                    .as_ref()
                    .filter(|pet| {
                        if let Some(effect) = &pet.effect {
                            effect.trigger == trigger
                        } else {
                            false
                        }
                    })
                    .map(|pet| pet.effect.clone())
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
