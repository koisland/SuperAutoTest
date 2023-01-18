use crate::common::{
    battle::{
        effect::Effect,
        state::{Action, CopyAttr, Outcome, Position, Target},
        team::Team,
        trigger::{get_self_enemy_faint_triggers, get_self_faint_triggers, TRIGGER_SELF_FAINT},
    },
    error::TeamError,
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
    /// Apply an `Action` to a target idx on a `Team`.
    fn _target_effect_idx(
        &mut self,
        target_idx: usize,
        effect: &mut Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>>;
    /// Apply effects based on a team's stored triggers.
    fn trigger_effects(&mut self, opponent: &mut Team) -> &mut Self;
    /// Apply a given effect to a `Team`.
    fn _apply_effect(
        &mut self,
        effect_pet_idx: usize,
        trigger: Outcome,
        effect: Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>>;
    /// Match statement applying effect to exclusively one `Team`.
    fn _match_position_one_team(
        &mut self,
        effect_pet_idx: usize,
        trigger: &Outcome,
        effect: &mut Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>>;
    /// Match statement applying effect to either self or opponent `Team`.
    fn _match_position_either_team(
        &mut self,
        effect_pet_idx: usize,
        trigger: &Outcome,
        effect: &mut Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>>;
    /// Create a node logging an effect's result for a `Team`'s history.
    fn create_node(&mut self, trigger: &Outcome) -> &mut Self;
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

impl EffectApply for Team {
    fn _target_effect_idx(
        &mut self,
        target_idx: usize,
        effect: &mut Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>> {
        let name = self.name.clone();
        let mut target_ids: Vec<Option<String>> = vec![];

        match &effect.action {
            Action::Add(stats) => {
                if let Some(target) = self.get_idx_pet(target_idx) {
                    target.stats += stats.clone();
                    info!(target: "dev", "(\"{}\")\nAdded {} to {}.", name, stats, target);
                    target_ids.push(target.id.clone())
                }
            }
            Action::Remove(stats) => {
                let mut outcomes: Vec<Outcome> = vec![];
                let target_id = if let Some(target) = self.get_idx_pet(target_idx) {
                    let (triggers, enemy_triggers) = target.indirect_attack(stats);
                    // Collectr triggers for both teams.
                    outcomes.extend(triggers);
                    opponent.triggers.extend(enemy_triggers);

                    info!(target: "dev", "(\"{}\")\nRemoved {} from {}.", name, stats.clone().invert(), target);
                    target.id.clone()
                } else {
                    None
                };
                self.triggers.extend(outcomes);
                target_ids.push(target_id)
            }
            Action::Gain(food) => {
                if let Some(target) = self.get_idx_pet(target_idx) {
                    target.set_item(Some(*food.clone()));
                    info!(target: "dev", "(\"{}\")\nGave {} to {}.", name, food, target);
                    target_ids.push(target.id.clone())
                }
            }
            Action::Summon(pet) => {
                if let Ok(Some(summoned_pet)) = self.add_pet(pet, target_idx) {
                    target_ids.push(summoned_pet.id.clone())
                } else {
                    info!(target: "dev", "(\"{}\")\nCouldn't summon {:?} to {}.", name, pet, target_idx);
                }
            }
            Action::Multiple(actions) => {
                for action in actions {
                    // Create new effect with single action.
                    let mut effect_copy = effect.clone();
                    effect_copy.action = action.clone();
                    self._target_effect_idx(target_idx, &mut effect_copy, opponent)?
                }
            }
            Action::Kill => {
                let dead_pet_pos = self.get_idx_pet(target_idx).map(|pet| {
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
                if let Some(target_pet) = self.get_idx_pet(target_idx) {
                    // Set pet health to 0 and allow clear_team() to do the rest.
                    target_pet.stats.health = 0;
                }
            }
            Action::Debuff(perc_stats) => {
                if let Some(pet) = self.get_idx_pet(target_idx) {
                    let mut debuff_stats = pet.stats.clone();
                    debuff_stats *= perc_stats.clone();
                    pet.stats -= debuff_stats;
                    info!(target: "dev", "(\"{}\")\nMultiplied stats of {} by {}.", name, pet, perc_stats)
                }
            }
            Action::Evolve(lvl, rel_pos) => {
                // Based on a specific relative position, select the pet to 'swallow' and remove.
                let chosen_pet = if let Position::Specific(rel_pos) = rel_pos {
                    // Do in new scope so mut ref to pet is dropped.
                    let (_, adj_idx) = self._cvt_rel_idx_to_adj_idx(target_idx, *rel_pos)?;
                    let evolved_pet = self.get_idx_pet(adj_idx).map(|pet| {
                        let mut pet_copy = pet.clone();
                        if let Ok(leveled_pet) = pet_copy.set_level(*lvl) {
                            pet.stats.health = 0;
                            info!(target: "dev", "(\"{}\")\nKilled pet {}.", name, pet);
                            Some(leveled_pet.clone())
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
                    if let Some(old_effect) = target_pet.effect.as_mut() {
                        old_effect.position = Position::OnSelf;
                        old_effect.action = Action::Summon(Some(Box::new(leveled_pet.clone())));
                        old_effect.trigger = TRIGGER_SELF_FAINT;
                        info!(target: "dev", "(\"{}\")\nEvolving {}.", name, leveled_pet);
                        info!(target: "dev", "(\"{}\")\nSet pet {} to summon evolved pet on faint.", name, target_pet);
                    }
                } else {
                    return Err(Box::new(TeamError {
                        reason: format!(
                            "Cannot access leveled pet at {:?} or targeted pet at {}.",
                            rel_pos, target_idx
                        ),
                    }));
                }
            }
            // TODO: May need to also choose to copy from an enemy pet at some point.
            Action::Copy(attr, pos) => {
                // Based on position, select the pet to copy.
                let chosen_pet = match pos {
                    Position::Any => self.get_any_pet(),
                    Position::Specific(rel_pos) => {
                        let (team, adj_idx) = self._cvt_rel_idx_to_adj_idx(target_idx, *rel_pos)?;
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
                        CopyAttr::Effect(_, lvl) => Some(CopyAttr::Effect(
                            Box::new(pet.get_effect(lvl.unwrap_or(1))?),
                            lvl,
                        )),
                        _ => None,
                    }
                } else {
                    None
                };

                // Chose the target of recipient of copied pet stats/effect.
                if let Some(target) = self.get_idx_pet(target_idx) {
                    // Calculate stats or set ability.
                    match copied_attr.unwrap_or(CopyAttr::None) {
                        CopyAttr::Stats(mut new_stats) => {
                            // If the stats are 0, use the target's original stats, otherwise, use the news stats.
                            let old_stats = target.stats.clone();
                            target.stats = new_stats.comp_set_value(&old_stats, 0).clone();

                            // Set Action to show stats added.
                            let stats_diff = target.stats.clone() - old_stats;
                            effect.action = Action::Add(stats_diff);

                            info!(
                                target: "dev", "(\"{}\")\nSet stats for {} to {}.",
                                name,
                                target,
                                target.stats
                            );
                        }
                        CopyAttr::Effect(effect, _) => {
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
                    target_ids.push(target.id.clone())
                }
            }
            _ => {}
        }
        // Create edge by iterating over all targets affected.
        if let (Some(prev_node), Some(curr_node)) = (self.history.prev_node, self.history.curr_node)
        {
            for target_id in target_ids {
                self.history.effect_graph.add_edge(
                    prev_node,
                    curr_node,
                    (
                        effect.target.clone(),
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
        effect: &mut Effect,
        opponent: &mut Team,
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
                    self._target_effect_idx(random_pet_idx, effect, opponent)?
                }
            }
            Position::All => {
                for pet_idx in 0..self.get_all_pets().len() {
                    self._target_effect_idx(pet_idx, effect, opponent)?
                }
            }
            Position::OnSelf => self._target_effect_idx(effect_pet_idx, effect, opponent)?,
            Position::Trigger => {
                let trigger_pos = trigger
                    .idx
                    .ok_or("No idx position given to apply effect.")?;
                self._target_effect_idx(trigger_pos, effect, opponent)?
            }
            Position::Specific(rel_pos) => {
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
            Position::Multiple(positions) => {
                for pos in positions {
                    // For each position:
                    // * Make a copy of the effect
                    // * Set the position to the desired position
                    // * Reduce the uses to 1 to not double the number of times an effect is activated.
                    let mut effect_copy = effect.clone();
                    effect_copy.position = pos.clone();
                    effect_copy.uses = Some(1);

                    self._match_position_one_team(
                        effect_pet_idx,
                        trigger,
                        &mut effect_copy,
                        opponent,
                    )?
                }
            }
            Position::Condition(condition) => {
                if let Some((idx, _)) = self.get_pet_by_cond(condition) {
                    self._target_effect_idx(idx, effect, opponent)?
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
        effect: &mut Effect,
        opponent: &mut Team,
    ) -> Result<(), Box<dyn Error>> {
        match &effect.position {
            Position::Specific(rel_pos) => {
                let (team, adj_idx) = self._cvt_rel_idx_to_adj_idx(effect_pet_idx, *rel_pos)?;
                match team {
                    Target::Enemy => opponent._target_effect_idx(adj_idx, effect, self)?,
                    Target::Friend => self._target_effect_idx(adj_idx, effect, opponent)?,
                    _ => unreachable!("Cannot return other types."),
                }
            }
            Position::All => {
                for pet_idx in 0..self.get_all_pets().len() {
                    self._target_effect_idx(pet_idx, effect, opponent)?
                }
                for pet_idx in 0..opponent.get_all_pets().len() {
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
        let mut effect_copy = effect.clone();

        match effect.target {
            Target::Friend => {
                self._match_position_one_team(effect_pet_idx, &trigger, &mut effect_copy, opponent)?
            }
            Target::Enemy => opponent._match_position_one_team(
                effect_pet_idx,
                &trigger,
                &mut effect_copy,
                self,
            )?,
            Target::Either => self._match_position_either_team(
                effect_pet_idx,
                &trigger,
                &mut effect_copy,
                opponent,
            )?,
            _ => {}
        }

        Ok(())
    }

    fn create_node(&mut self, trigger: &Outcome) -> &mut Self {
        let node_idx = self.history.effect_graph.add_node(trigger.clone());
        self.history.prev_node = self.history.curr_node;
        self.history.curr_node = Some(node_idx);
        self
    }

    fn trigger_effects(&mut self, opponent: &mut Team) -> &mut Self {
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
                // Add node here for activated effect.
                let node_idx = self.history.effect_graph.add_node(trigger.clone());
                self.history.curr_node = Some(node_idx);

                if let Err(err) = self._apply_effect(effect_pet_idx, trigger, effect, opponent) {
                    println!("(\"{}\")\nSomething went wrong. {:?}", self.name, err)
                };
            }

            // Set curr node to previous.
            self.history.prev_node = self.history.curr_node;

            curr_triggers.extend(self.triggers.iter().cloned());
            self.triggers.clear();
        }

        self
    }
}
