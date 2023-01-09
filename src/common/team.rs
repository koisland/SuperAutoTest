use crate::common::{
    effect::{Effect, EffectAction, Outcome, Position, Target},
    error::TeamError,
    pet::{Combat, Pet},
    trigger::*,
};

use itertools::Itertools;
use log::info;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::VecDeque, fmt::Display, rc::Rc};

#[allow(dead_code)]
const TEAM_SIZE: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Team {
    pub name: String,
    pub friends: RefCell<Vec<Option<Rc<RefCell<Pet>>>>>,
    pub triggers: RefCell<VecDeque<Outcome>>,
}

pub trait Summary {
    fn mean(&self) -> f32;
    fn median(&self) -> f32;
}
pub trait Battle {
    /// Clear `Team` of empty slots and/or fainted `Pet`s.
    fn clear_team(&self) -> &Self;
    /// Get the next available `Pet`.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    fn get_next_pet(&self) -> Option<Rc<RefCell<Pet>>>;
    /// Get a random available `Pet`.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    fn get_any_pet(&self) -> Option<Rc<RefCell<Pet>>>;
    /// Get all available `Pet`s.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    fn get_all_pets(&self) -> Vec<Rc<RefCell<Pet>>>;
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
    /// Apply effects based on a team's stored triggers.
    fn apply_trigger_effects(&self, opponent: &Team) -> &Self;
    /// Apply a given effect to a team.
    fn apply_effect(
        &self,
        pet_idx: usize,
        effect: Effect,
        opponent: &Team,
    ) -> Result<VecDeque<Outcome>, &'static str>;
    fn fight<'a>(&'a mut self, opponent: &'a mut Team, turns: Option<usize>) -> Option<&Team>;
    fn _target_effect_any(&self, effect_type: &EffectAction, outcomes: &mut VecDeque<Outcome>);
    fn _target_effect_all(&self, effect_type: &EffectAction, outcomes: &mut VecDeque<Outcome>);
    fn _target_effect_specific(
        &self,
        pos: usize,
        effect_type: &EffectAction,
        outcomes: &mut VecDeque<Outcome>,
    );
}

impl Team {
    #[allow(dead_code)]
    pub fn new(name: &str, pets: &[Option<Pet>]) -> Result<Team, TeamError> {
        if pets.len() != TEAM_SIZE {
            return Err(TeamError {
                reason: format!("Invalid team size. ({})", pets.len()),
            });
        };

        Ok(Team {
            name: name.to_string(),
            friends: RefCell::new(
                pets.iter()
                    .map(|pet| pet.as_ref().map(|pet| Rc::new(RefCell::new(pet.clone()))))
                    .collect_vec(),
            ),
            triggers: RefCell::new(VecDeque::from_iter([TRIGGER_START_BATTLE])),
        })
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let friends_ref = self.friends.borrow();
        let friends = friends_ref
            .iter()
            .filter_map(|pet| pet.as_ref().map(|pet| pet.borrow()))
            .collect_vec();
        write!(f, "Team: {}\n{:?}", self.name, friends)
    }
}

impl Battle for Team {
    fn clear_team(&self) -> &Self {
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
                    .map_or(false, |pet| pet.borrow().is_alive())
                {
                    None
                } else {
                    // Pet is dead.
                    info!(target: "dev", "Pet ({i}) {} is dead. Removing.", pet.as_ref().unwrap().borrow());
                    Some(i)
                }
            })
            .rev()
            .collect_vec();
        for rev_idx in missing_pets.iter() {
            self.friends.borrow_mut().remove(*rev_idx);
        }
        self
    }
    /// Get the next pet in team.
    fn get_next_pet(&self) -> Option<Rc<RefCell<Pet>>> {
        if let Some(Some(pet)) = self.friends.borrow().iter().next() {
            pet.borrow().is_alive().then(|| pet.clone())
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
                    pet.borrow().is_alive().then(|| pet.clone())
                } else {
                    None
                }
            })
            .collect_vec()
    }

    fn add_pet(&self, pet: &Option<Box<Pet>>, pos: usize) -> Result<[Outcome; 3], TeamError> {
        if self.get_all_pets().len() == 5 {
            return Err(TeamError {
                reason: "Team is full. Cannot add new pet.".to_string(),
            });
        }
        if let Some(stored_pet) = pet.clone() {
            info!(target: "dev", "Added pet to team: {:?}.", stored_pet);
            self.friends
                .borrow_mut()
                .insert(pos, Some(Rc::new(RefCell::new(*stored_pet))));
            Ok([
                // May run into issue with mushroomed scorpion.
                TRIGGER_SELF_SUMMON,
                TRIGGER_ANY_SUMMON,
                TRIGGER_ANY_ENEMY_SUMMON,
            ])
        } else {
            Err(TeamError {
                reason: "No pet to add.".to_string(),
            })
        }
    }

    fn _target_effect_any(&self, effect_type: &EffectAction, outcomes: &mut VecDeque<Outcome>) {
        match effect_type {
            EffectAction::Add(stats) => {
                if let Some(target) = self.get_any_pet() {
                    target.borrow().stats.borrow_mut().add(stats);
                    info!(target: "dev", "Added {:?}\n\t({:?}).", stats, target.borrow());
                }
            }
            EffectAction::Remove(stats) => {
                if let Some(target) = self.get_any_pet() {
                    outcomes.extend(target.borrow().indirect_attack(stats));
                    info!(target: "dev", "Removed {:?}\n\t({:?}).", stats, target.borrow());
                }
            }
            EffectAction::Gain(food) => {
                if let Some(target) = self.get_any_pet() {
                    target.borrow_mut().item = Some(*food.clone());
                    info!(target: "dev", "Gave {:?}\n\t({:?}).", food, target.borrow());
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
                    pet.borrow().stats.borrow_mut().add(stats);
                    info!(target: "dev", "Added {:?}\n\t({:?}).", stats, pet.borrow());
                }
            }
            EffectAction::Remove(stats) => {
                for pet in self.get_all_pets() {
                    outcomes.extend(pet.borrow().indirect_attack(stats));
                    info!(target: "dev", "Removed {:?}\n\t({:?}).", stats, pet.borrow());
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
                    affected_pet.borrow().stats.borrow_mut().add(stats);
                    info!(target: "dev", "Added {:?} to pos {} pet\n\t{:?}.", stats, pos, affected_pet.borrow())
                }
            }
            EffectAction::Remove(stats) => {
                if let Some(affected_pet) = self.get_all_pets().get(pos) {
                    info!(target: "dev", "Removed {:?} from pos {} pet\n\t{:?}.", stats, pos, affected_pet.borrow());
                    outcomes.extend(affected_pet.borrow().indirect_attack(stats));
                }
            }
            EffectAction::Gain(food) => {
                if let Some(affected_pet) = self.get_all_pets().get(pos) {
                    info!(target: "dev", "Gave {:?} to pos {} pet\n\t{:?}.", food, pos, affected_pet.borrow());
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

    fn apply_effect(
        &self,
        pet_idx: usize,
        effect: Effect,
        opponent: &Team,
    ) -> Result<VecDeque<Outcome>, &'static str> {
        // Store all outcomes from applying effects.
        // TODO: Look into changing so can use triggers from Team struct. Issues since iterating at same time.
        let mut outcomes: VecDeque<Outcome> = VecDeque::new();

        match &effect.target {
            Target::Friend => match &effect.position {
                Position::Any => self._target_effect_any(&effect.effect, &mut outcomes),
                Position::All => self._target_effect_all(&effect.effect, &mut outcomes),
                Position::Trigger => todo!(),
                Position::Specific(rel_pos) => {
                    self._target_effect_specific(pet_idx + *rel_pos, &effect.effect, &mut outcomes)
                }
                _ => {}
            },
            Target::Enemy => match effect.position {
                Position::Any => opponent._target_effect_any(&effect.effect, &mut outcomes),
                Position::All => todo!(),
                Position::Trigger => todo!(),
                Position::Specific(_) => todo!(),
                Position::None => todo!(),
            },
            Target::None | Target::OnSelf => {}
        };

        Ok(outcomes)
    }

    fn apply_trigger_effects(&self, opponent: &Team) -> &Self {
        // Get ownership of current triggers and clear team triggers.
        let mut curr_triggers = self.triggers.borrow_mut().to_owned();
        self.triggers.borrow_mut().clear();

        // Continue iterating until all triggers consumed.
        while let Some(mut trigger) = curr_triggers.pop_front() {
            let mut applied_effects: Vec<(usize, Effect)> = vec![];
            // Iterate through pets collecting valid effects.
            for (i, pet) in self.friends.borrow().iter().enumerate() {
                // Adjust specific trigger positions to the current pet.
                // Clamp to team size.
                // If trigger is Specific(0) for a friendly team.,
                // * 1st pet = Specific(0)
                // * 2nd pet = Specific(1)
                // * ...
                if let Position::Specific(rel_pos) = trigger.position {
                    trigger.position = Position::Specific((rel_pos + i).clamp(0, TEAM_SIZE - 1))
                }
                // Get food and pet effect based on if its trigger is equal to current trigger, if any.
                if let Some(Some(food_effect)) = pet.as_ref().map(|pet| {
                    pet.borrow()
                        .item
                        .as_ref()
                        .filter(|food| food.ability.trigger == trigger)
                        .map(|food| food.ability.clone())
                }) {
                    applied_effects.push((i, food_effect))
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
                    applied_effects.push((i, pet_effect))
                };
            }
            // Apply effects.
            // Extend in reverse so proper order followed.
            curr_triggers.extend(
                applied_effects
                    .into_iter()
                    .rev()
                    .filter_map(|(i, effect)| self.apply_effect(i, effect, opponent).ok())
                    .into_iter()
                    .flatten(),
            );
        }
        self
    }

    fn fight<'a>(&'a mut self, opponent: &'a mut Team, turns: Option<usize>) -> Option<&Team> {
        let mut n_turns: usize = 0;

        // Apply start of battle effects.
        self.clear_team().apply_trigger_effects(opponent);
        opponent.clear_team().apply_trigger_effects(self);

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

            self.clear_team().apply_trigger_effects(opponent);
            opponent.clear_team().apply_trigger_effects(self);

            // Check that two pets exist and attack.
            // Attack will result in triggers being added.
            let outcome = if let (Some(pet), Some(opponent_pet)) =
                (self.get_next_pet(), opponent.get_next_pet())
            {
                // Attack
                let outcome = pet.borrow_mut().attack(&mut opponent_pet.borrow_mut());
                info!(target: "dev", "Outcome of fight: {:?}", outcome);
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

            // Occurs even if pet fainted as fighting and applying effects occurs simultaneously.
            // Apply any food effects that alter the opponents pets. ex. Chili
            if let Some(pet) = self.get_next_pet() {
                pet.borrow_mut().apply_food_effect(opponent)
            }
            if let Some(opponent_pet) = opponent.get_next_pet() {
                opponent_pet.borrow_mut().apply_food_effect(self)
            }

            // Apply effect triggers from combat phase.
            self.apply_trigger_effects(opponent);
            opponent.apply_trigger_effects(self);

            // Stop fight after desired number of turns.
            if let Some(des_n_turns) = turns {
                if des_n_turns == n_turns {
                    break;
                }
            };

            n_turns += 1;
        }

        if self.friends.borrow().is_empty() && opponent.friends.borrow().is_empty() {
            info!(target: "dev", "Draw!");
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
