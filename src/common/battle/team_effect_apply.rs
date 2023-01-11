use crate::common::{
    battle::{
        effect::{Effect, EffectAction, Outcome, Position, Target},
        team::{Battle, Team, TEAM_SIZE},
    },
    pets::combat::Combat,
};

use itertools::Itertools;
use log::info;
use rand::seq::IteratorRandom;
use std::collections::VecDeque;

pub trait EffectApply {
    fn _target_effect_any(&self, effect_type: &EffectAction, outcomes: &mut VecDeque<Outcome>);
    fn _target_effect_all(&self, effect_type: &EffectAction, outcomes: &mut VecDeque<Outcome>);
    fn _target_effect_specific(
        &self,
        pos: usize,
        effect_type: &EffectAction,
        outcomes: &mut VecDeque<Outcome>,
    );
    // fn _target_effect_self(&self, trigger: Outcome, effect_type: &EffectAction, outcomes: &mut VecDeque<Outcome>);
    fn _target_effect_trigger(
        &self,
        trigger: &Outcome,
        effect_type: &EffectAction,
        outcomes: &mut VecDeque<Outcome>,
    );
    /// Apply effects based on a team's stored triggers.
    fn _apply_trigger_effects(&self, opponent: &Team) -> &Self;
    /// Apply a given effect to a team.
    fn _apply_effect(
        &self,
        effect_pet_idx: usize,
        trigger: Outcome,
        effect: Effect,
        opponent: &Team,
    ) -> Result<VecDeque<Outcome>, &'static str>;
}

impl EffectApply for Team {
    fn _target_effect_trigger(
        &self,
        trigger: &Outcome,
        effect_type: &EffectAction,
        outcomes: &mut VecDeque<Outcome>,
    ) {
        let trigger_pos = trigger.idx.expect("No idx position given to apply effect.");
        match effect_type {
            EffectAction::Add(stats) => {
                if let Some(target) = self.get_all_pets().get(trigger_pos) {
                    target.borrow_mut().stats.add(stats);
                    info!(target: "dev", "Added {} to {}.", stats, target.borrow());
                }
            }
            EffectAction::Remove(stats) => {
                if let Some(target) = self.get_all_pets().get(trigger_pos) {
                    outcomes.extend(target.borrow_mut().indirect_attack(stats));
                    info!(target: "dev", "Removed {} from {}.", stats, target.borrow());
                }
            }
            EffectAction::Gain(food) => {
                if let Some(target) = self.get_all_pets().get(trigger_pos) {
                    target.borrow_mut().item = Some(*food.clone());
                    info!(target: "dev", "Gave {:?} to {}.", food, target.borrow());
                }
            }
            // Must also emit EffectTrigger for summon.
            EffectAction::Summon(pet) => {
                let summon_triggers = self.add_pet(pet, trigger_pos);
                if let Ok(summon_triggers) = summon_triggers {
                    outcomes.extend(summon_triggers.into_iter())
                }
            }
            EffectAction::CopyStatsHealthiest => {
                // let healthiest_pet = self
                //     .friends
                //     .borrow()
                //     .iter()
                //     .enumerate()
                //     .max_by(|(_, pet_1), (_, pet_2)| {
                //         if let (Some(pet_1), Some(pet_2)) = (pet_1, pet_2) {
                //             pet_1
                //                 .borrow()
                //                 .stats
                //                 .health
                //                 .cmp(&pet_2.borrow().stats.health)
                //         } else {
                //             std::cmp::Ordering::Equal
                //         }
                //     })
                //     .expect("No healthiest pet found.");
            }
            _ => {}
        }
    }

    fn _target_effect_any(&self, effect_type: &EffectAction, outcomes: &mut VecDeque<Outcome>) {
        match effect_type {
            EffectAction::Add(stats) => {
                if let Some(target) = self.get_any_pet() {
                    target.borrow_mut().stats.add(stats);
                    info!(target: "dev", "Added {} to {}.", stats, target.borrow());
                }
            }
            EffectAction::Remove(stats) => {
                if let Some(target) = self.get_any_pet() {
                    outcomes.extend(target.borrow_mut().indirect_attack(stats));
                    info!(target: "dev", "Removed {} from {}.", stats, target.borrow());
                }
            }
            EffectAction::Gain(food) => {
                if let Some(target) = self.get_any_pet() {
                    target.borrow_mut().item = Some(*food.clone());
                    info!(target: "dev", "Gave {:?} to {}.", food, target.borrow());
                }
            }
            // Must also emit EffectTrigger for summon.
            EffectAction::Summon(pet) => {
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

    fn _target_effect_all(&self, effect_type: &EffectAction, outcomes: &mut VecDeque<Outcome>) {
        match effect_type {
            EffectAction::Add(stats) => {
                for pet in self.get_all_pets() {
                    pet.borrow_mut().stats.add(stats);
                    info!(target: "dev", "Added {} to {}.", stats, pet.borrow());
                }
            }
            EffectAction::Remove(stats) => {
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
        effect_type: &EffectAction,
        outcomes: &mut VecDeque<Outcome>,
    ) {
        match effect_type {
            EffectAction::Add(stats) => {
                if let Some(affected_pet) = self.get_all_pets().get(pos) {
                    affected_pet.borrow_mut().stats.add(stats);
                    info!(target: "dev", "Added {} to {}.", stats, affected_pet.borrow())
                }
            }
            EffectAction::Remove(stats) => {
                if let Some(affected_pet) = self.get_all_pets().get(pos) {
                    info!(target: "dev", "Removed {} from {}.", stats, affected_pet.borrow());
                    outcomes.extend(affected_pet.borrow_mut().indirect_attack(stats));
                }
            }
            EffectAction::Gain(food) => {
                if let Some(affected_pet) = self.get_all_pets().get(pos) {
                    info!(target: "dev", "Gave {:?} to {}.", food, affected_pet.borrow());
                    affected_pet.borrow_mut().item = Some(*food.clone())
                }
            }
            EffectAction::Summon(pet) => {
                let summon_triggers = self.add_pet(pet, pos);
                if let Ok(summon_triggers) = summon_triggers {
                    outcomes.extend(summon_triggers.into_iter())
                }
            }
            _ => {}
        }
    }

    fn _apply_effect(
        &self,
        effect_pet_idx: usize,
        trigger: Outcome,
        effect: Effect,
        opponent: &Team,
    ) -> Result<VecDeque<Outcome>, &'static str> {
        // Store all outcomes from applying effects.
        // TODO: Look into changing so can use triggers from Team struct. Issues since iterating at same time.
        let mut outcomes: VecDeque<Outcome> = VecDeque::new();

        // Activate effect for each use.
        for _ in 0..effect.uses.unwrap_or(1) {
            match &effect.target {
                Target::Friend => match &effect.position {
                    Position::Any => self._target_effect_any(&effect.effect, &mut outcomes),
                    Position::All => self._target_effect_all(&effect.effect, &mut outcomes),
                    Position::OnSelf | Position::Trigger => {
                        self._target_effect_trigger(&trigger, &effect.effect, &mut outcomes)
                    }
                    // Position::Trigger => self._target_effect_trigger(trigger, &effect.effect, &mut outcomes),
                    Position::Specific(rel_pos) => {
                        let adj_idx: usize = (isize::try_from(effect_pet_idx)
                            .expect("Can't convert current pet idx to isize.")
                            + *rel_pos)
                            .clamp(0, TEAM_SIZE.try_into().expect("Invalid team size."))
                            .try_into()
                            .expect("Can't calculate adjusted pet index.");
                        self._target_effect_specific(adj_idx, &effect.effect, &mut outcomes)
                    }
                    Position::Range(_) => {}
                    _ => {}
                },
                Target::Enemy => match &effect.position {
                    Position::Any => opponent._target_effect_any(&effect.effect, &mut outcomes),
                    Position::All => opponent._target_effect_all(&effect.effect, &mut outcomes),
                    Position::OnSelf | Position::Trigger => {
                        opponent._target_effect_trigger(&trigger, &effect.effect, &mut outcomes)
                    }
                    // Position::Trigger => self._target_effect_trigger(trigger, &effect.effect, &mut outcomes),
                    Position::Specific(rel_pos) => {
                        let adj_idx: usize = (isize::try_from(effect_pet_idx)
                            .expect("Can't convert current pet idx to isize.")
                            + *rel_pos)
                            .clamp(0, TEAM_SIZE.try_into().expect("Invalid team size."))
                            .try_into()
                            .expect("Can't calculate adjusted pet index.");
                        opponent._target_effect_specific(adj_idx, &effect.effect, &mut outcomes)
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
