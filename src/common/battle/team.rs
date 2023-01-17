use crate::common::{
    battle::{
        state::{Condition, Outcome, Position, Status, Target, TeamFightOutcome},
        team_effect_apply::EffectApply,
        trigger::*,
    },
    error::TeamError,
    graph::effect_graph::History,
    pets::{combat::Combat, pet::Pet},
};

use itertools::Itertools;
use log::info;
use petgraph::Graph;
use rand::seq::IteratorRandom;
use std::{collections::VecDeque, fmt::Display};

/// A Super Auto Pets team.
#[derive(Debug)]
pub struct Team {
    pub name: String,
    pub friends: Vec<Option<Pet>>,
    pub fainted: Vec<Option<Pet>>,
    pub max_size: usize,
    pub triggers: VecDeque<Outcome>,
    pub history: History,
    pub pet_count: usize,
}

impl Team {
    /// Create a new team of a given size.
    pub fn new(name: &str, pets: &[Option<Pet>], max_size: usize) -> Result<Team, TeamError> {
        if pets.len() > max_size {
            Err(TeamError {
                reason: format!(
                    "Pets provided exceed specified max size. {} > {}",
                    pets.len(),
                    max_size
                ),
            })
        } else {
            // Index pets.
            let mut pet_count = 0;
            let idx_pets = pets
                .iter()
                .cloned()
                .map(|mut slot| {
                    if let Some(pet) = slot.as_mut() {
                        pet.set_pos(pet_count);
                        // Create id if one not assigned.
                        pet.id = Some(
                            pet.id
                                .clone()
                                .unwrap_or(format!("{}_{}", pet.name, pet_count)),
                        );

                        pet_count += 1;
                    }
                    slot
                })
                .collect_vec();
            let mut team = Team {
                name: name.to_string(),
                friends: idx_pets,
                fainted: vec![],
                max_size,
                triggers: VecDeque::from_iter([TRIGGER_START_BATTLE]),
                history: History {
                    curr_turn: 0,
                    curr_node: None,
                    prev_node: None,
                    effect_graph: Graph::new(),
                },
                pet_count,
            };
            // Set starting node.
            let starting_node = team.history.effect_graph.add_node(TRIGGER_START_BATTLE);
            team.history.prev_node = Some(starting_node);
            team.history.curr_node = Some(starting_node);
            Ok(team)
        }
    }

    /// Clear `Team` of empty slots and/or fainted `Pet`s.
    pub fn clear_team(&mut self) -> &mut Self {
        let mut new_idx_cnt = 0;
        let missing_pets = self
            .friends
            .iter_mut()
            .enumerate()
            .filter_map(|(i, pet)| {
                // Check if empty slot
                if pet.is_none() {
                    Some(i)
                } else if pet.as_ref().map_or(false, |pet| pet.stats.health != 0) {
                    // Pet is Some so safe to unwrap.
                    // Set new pet index and increment
                    pet.as_mut().unwrap().set_pos(new_idx_cnt);
                    new_idx_cnt += 1;
                    None
                } else {
                    // Pet is dead.
                    info!(target: "dev", "(\"{}\")\n{} fainted.", self.name, pet.as_ref().unwrap());
                    Some(i)
                }
            })
            .collect_vec();
        // Iterate in reverse to maintain correct removal order.
        for rev_idx in missing_pets.iter().rev() {
            // Remove the pet and store its id.
            let dead_pet = self.friends.remove(*rev_idx);
            self.fainted.push(dead_pet);
        }
        self
    }

    /// Get a single pet by a given `Condition`.
    pub fn get_pet_by_cond(&mut self, cond: &Condition) -> Option<(usize, &mut Pet)> {
        let pets = self.get_all_pets().into_iter().enumerate();

        match cond {
            Condition::Healthiest => {
                pets.max_by(|(_, pet_1), (_, pet_2)| pet_1.stats.health.cmp(&pet_2.stats.health))
            }
            Condition::Illest => {
                pets.min_by(|(_, pet_1), (_, pet_2)| pet_1.stats.health.cmp(&pet_2.stats.health))
            }
            Condition::Strongest => {
                pets.max_by(|(_, pet_1), (_, pet_2)| pet_1.stats.attack.cmp(&pet_2.stats.attack))
            }
            Condition::Weakest => {
                pets.min_by(|(_, pet_1), (_, pet_2)| pet_1.stats.attack.cmp(&pet_2.stats.attack))
            }
        }
    }

    /// Get an available `Pet` at the specified index.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    pub fn get_idx_pet(&mut self, idx: usize) -> Option<&mut Pet> {
        if let Some(Some(pet)) = self.friends.get_mut(idx) {
            (pet.stats.health != 0).then_some(pet)
        } else {
            None
        }
    }
    /// Get the next available `Pet`.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    pub fn get_next_pet(&mut self) -> Option<&mut Pet> {
        if let Some(Some(pet)) = self.friends.first_mut() {
            (pet.stats.health != 0).then_some(pet)
        } else {
            None
        }
    }

    /// Get a random available `Pet`.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    pub fn get_any_pet(&mut self) -> Option<&mut Pet> {
        let mut rng = rand::thread_rng();
        self.get_all_pets().into_iter().choose(&mut rng)
    }

    /// Get all available `Pet`s.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    pub fn get_all_pets(&mut self) -> Vec<&mut Pet> {
        self.friends
            .iter_mut()
            .filter_map(|pet| {
                if let Some(pet) = pet.as_mut() {
                    (pet.stats.health != 0).then_some(pet)
                } else {
                    None
                }
            })
            .collect_vec()
    }

    /// Add a `Pet` to a `Team`.
    /// * `:param pet:`
    ///     * `Pet` to be summoned.
    /// * `:param pos:`
    ///     * Index on `self.friends` to add `Pet` to.
    ///
    /// Raises `TeamError`:
    /// * If `self.friends` at speciedd size limit of `self.max_size`
    pub fn add_pet(
        &mut self,
        pet: &Option<Box<Pet>>,
        pos: usize,
    ) -> Result<&Option<Pet>, TeamError> {
        if self.get_all_pets().len() == self.max_size {
            // Add overflow to dead pets.
            if let Some(stored_pet) = pet.clone() {
                self.fainted.push(Some(*stored_pet));
            }
            return Err(TeamError {
                reason: format!(
                    "(\"{}\")\nMaximum number of pets reached. Cannot add {:?}.",
                    self.name, pet
                ),
            });
        }
        if let Some(mut stored_pet) = pet.clone() {
            // Assign id to pet if not any.
            stored_pet.id = Some(stored_pet.id.clone().unwrap_or(format!(
                "{}_{}",
                stored_pet.name,
                self.pet_count + 1
            )));

            // Handle case where pet in front faints and vector is empty.
            // Would panic attempting to insert at any position not at 0.
            // Also update position to be correct.
            let pos = if pos > self.friends.len() { 0 } else { pos };

            self.friends.insert(pos, Some(*stored_pet));
            info!(target: "dev", "(\"{}\")\nAdded pet to pos {pos}: {}.", self.name.to_string(), self.get_idx_pet(pos).unwrap());

            // Set summon triggers.
            let mut self_trigger = TRIGGER_SELF_SUMMON;
            let mut any_trigger = TRIGGER_ANY_SUMMON;
            let mut any_enemy_trigger = TRIGGER_ANY_ENEMY_SUMMON;

            (self_trigger.idx, any_trigger.idx, any_enemy_trigger.idx) =
                (Some(pos), Some(pos), Some(pos));

            // Update old triggers and their positions that store a pet's idx after inserting new pet.
            // TODO: Look into more edge cases that may cause issue when triggers activate simultaneously.
            for trigger in self.triggers.iter_mut() {
                match (&mut trigger.position, &mut trigger.target) {
                    (Position::Specific(orig_pos), Target::Friend)
                    | (Position::Specific(orig_pos), Target::Enemy) => *orig_pos += 1,
                    (Position::Trigger, Target::Friend)
                    | (Position::Trigger, Target::Enemy)
                    | (Position::OnSelf, Target::Friend)
                    | (Position::OnSelf, Target::Enemy) => {
                        if let Some(idx) = trigger.idx.as_mut() {
                            *idx += 1
                        }
                    }
                    _ => {}
                }
            }

            self.triggers.extend([
                // May run into issue with mushroomed scorpion.
                self_trigger,
                any_trigger,
                any_enemy_trigger,
            ]);
            Ok(self.friends.get(pos).unwrap())
        } else {
            Err(TeamError {
                reason: "No pet to add.".to_string(),
            })
        }
    }

    pub fn fight(&mut self, opponent: &mut Team) -> TeamFightOutcome {
        info!(target: "dev", "(\"{}\")\n{}", self.name, self);
        info!(target: "dev", "(\"{}\")\n{}", opponent.name, opponent);

        // Apply start of battle effects.
        self.clear_team();
        opponent.clear_team();
        while !self.triggers.is_empty() || !opponent.triggers.is_empty() {
            self.trigger_effects(opponent);
            opponent.trigger_effects(self);
        }

        // Check that both teams have a pet that is alive.
        // Increment turn counter.
        self.history.curr_turn += 1;

        // Trigger Before Attack && Friend Ahead attack.
        self.triggers
            .extend([TRIGGER_SELF_ATTACK, TRIGGER_AHEAD_ATTACK]);
        opponent
            .triggers
            .extend([TRIGGER_SELF_ATTACK, TRIGGER_AHEAD_ATTACK]);

        while !self.triggers.is_empty() || !opponent.triggers.is_empty() {
            self.trigger_effects(opponent);
            opponent.trigger_effects(self);
        }

        self.clear_team();
        opponent.clear_team();

        // Check that two pets exist and attack.
        // Attack will result in triggers being added.
        if let (Some(pet), Some(opponent_pet)) = (self.get_next_pet(), opponent.get_next_pet()) {
            // Attack and get outcome of fight.
            info!(target: "dev", "Fight!\nPet: {}\nOpponent: {}", pet, opponent_pet);
            let outcome = pet.attack(opponent_pet);
            info!(target: "dev", "(\"{}\")\n{}", self.name, self);
            info!(target: "dev", "(\"{}\")\n{}", opponent.name, opponent);

            // Create attack node.
            self.create_node(&TRIGGER_SELF_ATTACK);
            opponent.create_node(&TRIGGER_SELF_ATTACK);

            if let Some(hurt_trigger) = outcome
                .friends
                .iter()
                .find(|trigger| trigger.status == Status::Hurt)
            {
                self.create_node(hurt_trigger);
            }

            if let Some(opponent_hurt_trigger) = outcome
                .opponents
                .iter()
                .find(|trigger| trigger.status == Status::Hurt)
            {
                opponent.create_node(opponent_hurt_trigger);
            }

            // Add triggers to team from outcome of battle.
            self.triggers.extend(outcome.friends.into_iter());
            opponent.triggers.extend(outcome.opponents.into_iter());

            // Apply effect triggers from combat phase.
            while !self.triggers.is_empty() || !opponent.triggers.is_empty() {
                self.trigger_effects(opponent);
                opponent.trigger_effects(self);
            }
            self.clear_team();
            opponent.clear_team();
        }
        if !self.friends.is_empty() && !opponent.friends.is_empty() {
            TeamFightOutcome::None
        } else {
            // Add end of battle node.
            self.history.prev_node = self.history.curr_node;
            self.history.curr_node = Some(self.history.effect_graph.add_node(TRIGGER_END_BATTLE));

            if self.friends.is_empty() && opponent.friends.is_empty() {
                info!(target: "dev", "Draw!");
                TeamFightOutcome::Draw
            } else if !opponent.friends.is_empty() {
                info!(target: "dev", "Enemy team won...");
                TeamFightOutcome::Loss
            } else {
                info!(target: "dev", "Your team won!");
                TeamFightOutcome::Win
            }
        }
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for friend in self.friends.iter().filter_map(|pet| pet.as_ref()) {
            writeln!(f, "{}", friend)?;
        }
        Ok(())
    }
}
