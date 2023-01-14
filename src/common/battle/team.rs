use crate::common::{
    battle::{
        state::{Condition, Outcome, Position, Target},
        team_effect_apply::EffectApply,
        trigger::*,
    },
    error::TeamError,
    pets::{combat::Combat, pet::Pet},
};

use itertools::Itertools;
use log::info;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::VecDeque, fmt::Display, rc::Rc};

pub const TEAM_SIZE: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Team {
    pub name: String,
    pub friends: RefCell<Vec<Option<Rc<RefCell<Pet>>>>>,
    pub triggers: RefCell<VecDeque<Outcome>>,
}

pub trait Battle: EffectApply {
    /// Clear `Team` of empty slots and/or fainted `Pet`s.
    fn clear_team(&self) -> &Self;
    /// Get the next available `Pet`.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    fn get_next_pet(&self) -> Option<Rc<RefCell<Pet>>>;
    /// Get a random available `Pet`.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    fn get_any_pet(&self) -> Option<Rc<RefCell<Pet>>>;
    /// Get an available `Pet` at the specified index.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    fn get_idx_pet(&self, idx: usize) -> Option<Rc<RefCell<Pet>>>;
    /// Get all available `Pet`s.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    fn get_all_pets(&self) -> Vec<Rc<RefCell<Pet>>>;
    /// Get a single pet by a given `Condition`.
    fn get_pet_by_cond(&self, cond: &Condition) -> Option<Rc<RefCell<Pet>>>;
    /// Add a `Pet` to a `Team`.
    /// * `:param pet:`
    ///     * `Pet` to be summoned.
    /// * `:param pos:`
    ///     * Index on `self.friends` to add `Pet` to.
    ///
    /// Raises `Error`:
    /// * If `self.friends` at size limit of `TEAM_SIZE` (Default: 5)
    ///
    /// Returns:
    /// * Array of `Outcome` type.
    fn add_pet(&self, pet: &Option<Box<Pet>>, pos: usize) -> Result<[Outcome; 3], TeamError>;
    fn fight<'a>(&'a mut self, opponent: &'a mut Team, turns: Option<usize>) -> Option<&Team>;
}

impl Team {
    #[allow(dead_code)]
    pub fn new(name: &str, pets: &[Option<Pet>; TEAM_SIZE]) -> Team {
        Team {
            name: name.to_string(),
            friends: RefCell::new(
                pets.iter()
                    .map(|pet| pet.as_ref().map(|pet| Rc::new(RefCell::new(pet.clone()))))
                    .collect_vec(),
            ),
            triggers: RefCell::new(VecDeque::from_iter([TRIGGER_START_BATTLE])),
        }
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Team: {}", self.name)?;
        for friend in self
            .friends
            .borrow()
            .iter()
            .filter_map(|pet| pet.as_ref().map(|pet| pet.borrow()))
        {
            writeln!(f, "{}", friend)?;
        }
        Ok(())
    }
}

impl Battle for Team {
    fn clear_team(&self) -> &Self {
        let mut new_idx_cnt = 0;
        let missing_pets = self
            .friends
            .borrow()
            .iter()
            .enumerate()
            .filter_map(|(i, pet)| {
                // Check if empty slot
                if pet.is_none() {
                    Some(i)
                } else if pet
                    .as_ref()
                    .map_or(false, |pet| pet.borrow().stats.health != 0)
                {
                    // Pet is Some so safe to unwrap.
                    // Set new pet index and increment
                    pet.as_ref().unwrap().borrow_mut().set_pos(new_idx_cnt).unwrap();
                    new_idx_cnt += 1;
                    None
                } else {
                    // Pet is dead.
                    info!(target: "dev", "Pet ({i}) {} is dead. Removing.", pet.as_ref().unwrap().borrow());
                    Some(i)
                }
            })
            .collect_vec();
        // Iterate in reverse to maintain correct removal order.
        for rev_idx in missing_pets.iter().rev() {
            self.friends.borrow_mut().remove(*rev_idx);
        }
        self
    }
    fn get_pet_by_cond(&self, cond: &Condition) -> Option<Rc<RefCell<Pet>>> {
        match cond {
            Condition::Healthiest => {
                if let Some(healthiest_pet) = self.friends.borrow().iter().max_by(|pet_1, pet_2| {
                    if let (Some(pet_1), Some(pet_2)) = (pet_1, pet_2) {
                        pet_1
                            .borrow()
                            .stats
                            .health
                            .cmp(&pet_2.borrow().stats.health)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }) {
                    healthiest_pet.clone()
                } else {
                    None
                }
            }
            Condition::Illest => {
                if let Some(illest_pet) = self.friends.borrow().iter().min_by(|pet_1, pet_2| {
                    if let (Some(pet_1), Some(pet_2)) = (pet_1, pet_2) {
                        pet_1
                            .borrow()
                            .stats
                            .health
                            .cmp(&pet_2.borrow().stats.health)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }) {
                    illest_pet.clone()
                } else {
                    None
                }
            }
            Condition::Strongest => {
                if let Some(strongest_pet) = self.friends.borrow().iter().max_by(|pet_1, pet_2| {
                    if let (Some(pet_1), Some(pet_2)) = (pet_1, pet_2) {
                        pet_1
                            .borrow()
                            .stats
                            .attack
                            .cmp(&pet_2.borrow().stats.attack)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }) {
                    strongest_pet.clone()
                } else {
                    None
                }
            }
            Condition::Weakest => {
                if let Some(weakest_pet) = self.friends.borrow().iter().min_by(|pet_1, pet_2| {
                    if let (Some(pet_1), Some(pet_2)) = (pet_1, pet_2) {
                        pet_1
                            .borrow()
                            .stats
                            .attack
                            .cmp(&pet_2.borrow().stats.attack)
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }) {
                    weakest_pet.clone()
                } else {
                    None
                }
            }
        }
    }

    fn get_idx_pet(&self, idx: usize) -> Option<Rc<RefCell<Pet>>> {
        if let Some(Some(pet)) = self.friends.borrow().get(idx) {
            (pet.borrow().stats.health != 0).then(|| pet.clone())
        } else {
            None
        }
    }
    /// Get the next pet in team.
    fn get_next_pet(&self) -> Option<Rc<RefCell<Pet>>> {
        if let Some(Some(pet)) = self.friends.borrow().iter().next() {
            (pet.borrow().stats.health != 0).then(|| pet.clone())
        } else {
            None
        }
    }

    fn get_any_pet(&self) -> Option<Rc<RefCell<Pet>>> {
        let mut rng = rand::thread_rng();
        self.get_all_pets().into_iter().choose(&mut rng)
    }

    fn get_all_pets(&self) -> Vec<Rc<RefCell<Pet>>> {
        self.friends
            .borrow()
            .iter()
            .filter_map(|pet| {
                if let Some(pet) = pet {
                    (pet.borrow().stats.health != 0).then(|| pet.clone())
                } else {
                    None
                }
            })
            .collect_vec()
    }

    fn add_pet(&self, pet: &Option<Box<Pet>>, pos: usize) -> Result<[Outcome; 3], TeamError> {
        if self.get_all_pets().len() == TEAM_SIZE {
            return Err(TeamError {
                reason: "Team is full. Cannot add new pet.".to_string(),
            });
        }
        if let Some(stored_pet) = pet.clone() {
            // Handle case where pet in front faints and vector is empty.
            // Would panic attempting to insert at any position not at 0.
            // Also update position to be correct.
            let pos = if pos > self.friends.borrow().len() {
                self.friends
                    .borrow_mut()
                    .push(Some(Rc::new(RefCell::new(*stored_pet))));
                0
            } else {
                self.friends
                    .borrow_mut()
                    .insert(pos, Some(Rc::new(RefCell::new(*stored_pet))));
                pos
            };

            info!(target: "dev", "Added pet to pos {pos} on team {}: {}.", self.name, self.get_idx_pet(pos).unwrap().borrow());

            // Set summon triggers.
            let mut self_trigger = TRIGGER_SELF_SUMMON;
            let mut any_trigger = TRIGGER_ANY_SUMMON;
            let mut any_enemy_trigger = TRIGGER_ANY_ENEMY_SUMMON;

            (self_trigger.idx, any_trigger.idx, any_enemy_trigger.idx) =
                (Some(pos), Some(pos), Some(pos));

            // Update old triggers and their positions that store a pet's idx after inserting new pet.
            // TODO: Look into more edge cases that may cause issue when triggers activate simultaneously.
            for trigger in self.triggers.borrow_mut().iter_mut() {
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

            Ok([
                // May run into issue with mushroomed scorpion.
                self_trigger,
                any_trigger,
                any_enemy_trigger,
            ])
        } else {
            Err(TeamError {
                reason: "No pet to add.".to_string(),
            })
        }
    }
    fn fight<'a>(&'a mut self, opponent: &'a mut Team, turns: Option<usize>) -> Option<&Team> {
        let mut n_turns: usize = 0;

        // Apply start of battle effects.
        self.clear_team()._apply_trigger_effects(opponent);
        opponent.clear_team()._apply_trigger_effects(self);

        // Check that both teams have a pet that is alive.
        loop {
            // Trigger Before Attack && Friend Ahead attack.
            self.triggers
                .borrow_mut()
                .extend([TRIGGER_SELF_ATTACK, TRIGGER_AHEAD_ATTACK]);
            opponent
                .triggers
                .borrow_mut()
                .extend([TRIGGER_SELF_ATTACK, TRIGGER_AHEAD_ATTACK]);

            self.clear_team()._apply_trigger_effects(opponent);
            opponent.clear_team()._apply_trigger_effects(self);

            // Check that two pets exist and attack.
            // Attack will result in triggers being added.
            let outcome = if let (Some(pet), Some(opponent_pet)) =
                (self.get_next_pet(), opponent.get_next_pet())
            {
                info!(target: "dev", "Fight!\nPet: {}\nOpponent: {}", pet.borrow(), opponent_pet.borrow());
                // Attack
                let outcome = pet.borrow_mut().attack(&mut opponent_pet.borrow_mut());
                info!(target: "dev", "Outcome:\n{}", outcome);
                info!(target: "dev", "Self:\n{}", self);
                info!(target: "dev", "Opponent:\n{}", opponent);
                outcome
            } else {
                // If either side has no available pets, exit loop.
                break;
            };

            // Add triggers to team from outcome of battle.
            self.triggers
                .borrow_mut()
                .extend(outcome.friends.into_iter());
            opponent
                .triggers
                .borrow_mut()
                .extend(outcome.opponents.into_iter());

            // Apply effect triggers from combat phase.
            self._apply_trigger_effects(opponent).clear_team();
            opponent._apply_trigger_effects(self).clear_team();

            // Stop fight after desired number of turns.
            if let Some(des_n_turns) = turns.map(|n| n.saturating_sub(1)) {
                if des_n_turns == n_turns {
                    break;
                }
            };

            n_turns += 1;
        }

        if self.friends.borrow().is_empty() && opponent.friends.borrow().is_empty() {
            info!(target: "dev", "Draw!");
            None
        } else if !self.friends.borrow().is_empty() && !opponent.friends.borrow().is_empty() {
            info!(target: "dev", "Incomplete.");
            None
        } else if !self.friends.borrow().is_empty() {
            info!(target: "dev", "Your team won!");
            Some(self)
        } else {
            info!(target: "dev", "Enemy team won...");
            Some(opponent)
        }
    }
}
