use super::{
    effect::{Effect, EffectAction, Outcome, Position, Status, Target},
    pet::{Combat, Pet},
};
use itertools::Itertools;
use log::info;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::VecDeque, fmt::Display, rc::Rc};

#[allow(dead_code)]
const TEAM_SIZE: usize = 5;
// If a pet is attacking
// Is a friend
// Its position relative to the curr pet is 0 (self).
const TRIGGER_SELF_ATTACK: Outcome = Outcome {
    status: Status::Attack,
    target: Target::Friend,
    position: Position::Specific(0),
};

// * If a pet is attacking.
// * Is a friend.
// * Its position relative to the curr pet is 1 (pet behind).
const TRIGGER_AHEAD_ATTACK: Outcome = Outcome {
    status: Status::Attack,
    target: Target::Friend,
    position: Position::Specific(1),
};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    // fn is_team_alive(&self) -> bool;
    fn clear_team(&self) -> &Self;
    fn get_next_pet(&self) -> Option<Rc<RefCell<Pet>>>;
    fn get_any_pet(&self) -> Option<Rc<RefCell<Pet>>>;
    fn get_all_pets(&self) -> Vec<Rc<RefCell<Pet>>>;
    fn summon_pet(&self, pet: &Option<Box<Pet>>, pos: usize) -> Result<[Outcome; 3], &'static str>;
    fn apply_triggers(&self, opponent: &Team) -> &Self;
    fn apply_effect(
        &self,
        pet_idx: usize,
        effect: Effect,
        opponent: &Team,
    ) -> Result<VecDeque<Outcome>, &'static str>;
    fn fight(&mut self, opponent: &mut Team);
}

impl Team {
    #[allow(dead_code)]
    pub fn new(name: &str, pets: &[Option<Pet>]) -> Result<Team, &'static str> {
        if pets.len() != TEAM_SIZE {
            return Err("Invalid team size.");
        };

        Ok(Team {
            name: name.to_string(),
            friends: RefCell::new(
                pets.iter()
                    .map(|pet| pet.as_ref().map(|pet| Rc::new(RefCell::new(pet.clone()))))
                    .collect_vec(),
            ),
            triggers: RefCell::new(VecDeque::new()),
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
    // /// Check that all friends in team are alive.
    // fn is_team_alive(&self) -> bool {
    //     self.friends
    //         .iter()
    //         .any(|pet| pet.as_ref().map_or(false, |pet| ))
    // }

    /// Remove gaps in `Team` and any fainted `Pet`s.
    /// Also adds a faint trigger if any dead.
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

    fn summon_pet(&self, pet: &Option<Box<Pet>>, pos: usize) -> Result<[Outcome; 3], &'static str> {
        if self.get_all_pets().len() == 5 {
            info!(target: "dev", "Team is full.");
            return Err("Team is full.");
        }
        if let Some(stored_pet) = pet.clone() {
            info!(target: "dev", "Summoned {:?}.", stored_pet);
            self.friends
                .borrow_mut()
                .insert(pos, Some(Rc::new(RefCell::new(*stored_pet))));
            Ok([
                Outcome {
                    status: Status::Summoned,
                    target: Target::Friend,
                    position: Position::Specific(0),
                },
                Outcome {
                    status: Status::Summoned,
                    target: Target::Friend,
                    position: Position::Any,
                },
                Outcome {
                    status: Status::Summoned,
                    target: Target::Enemy,
                    position: Position::Any,
                },
            ])
        } else {
            todo!()
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
                Position::Any => match &effect.effect {
                    EffectAction::Add(stats) => {
                        if let Some(target) = self.get_any_pet() {
                            target.borrow().stats.borrow_mut().add(stats);
                            info!(target: "dev", "Added {:?} to {:?}\n\t({:?}).", stats, &effect.target, target.borrow());
                        }
                    }
                    EffectAction::Remove(stats) => {
                        if let Some(target) = self.get_any_pet() {
                            target.borrow().stats.borrow_mut().sub(stats);
                            info!(target: "dev", "Removed {:?} to {:?}\n\t({:?}).", stats, &effect.target, target.borrow());
                        }
                    }
                    EffectAction::Gain(food) => {
                        if let Some(target) = self.get_any_pet() {
                            target.borrow_mut().item = Some(*food.clone());
                            info!(target: "dev", "Gave {:?} to {:?}\n\t({:?}).", food, &effect.target, target.borrow());
                        }
                    }
                    // Must also emit EffectTrigger for summon.
                    EffectAction::Summon(pet) => {
                        let mut rng = rand::thread_rng();
                        let random_pos = (0..5).choose(&mut rng).unwrap() as usize;

                        let summon_triggers = self.summon_pet(pet, random_pos);
                        if let Ok(summon_triggers) = summon_triggers {
                            outcomes.extend(summon_triggers.into_iter())
                        }
                    }
                    _ => {}
                },
                Position::All => match &effect.effect {
                    EffectAction::Add(stats) => {
                        for pet in self.get_all_pets() {
                            pet.borrow().stats.borrow_mut().add(stats);
                        }
                    }
                    EffectAction::Remove(stats) => {
                        for pet in self.get_all_pets() {
                            pet.borrow().stats.borrow_mut().sub(stats);
                        }
                    }
                    _ => {}
                },
                Position::Trigger => todo!(),
                Position::Specific(rel_pos) => {
                    let adj_idx: usize = pet_idx + *rel_pos;
                    match &effect.effect {
                        EffectAction::Add(stats) => {
                            if let Some(affected_pet) = self.get_all_pets().get(adj_idx) {
                                affected_pet.borrow().stats.borrow_mut().add(stats);
                            }
                        }
                        EffectAction::Remove(stats) => {
                            if let Some(affected_pet) = self.get_all_pets().get(adj_idx) {
                                affected_pet.borrow().stats.borrow_mut().sub(stats);
                            }
                        }
                        EffectAction::Gain(food) => {
                            if let Some(affected_pet) = self.get_all_pets().get(adj_idx) {
                                affected_pet.borrow_mut().item = Some(*food.clone())
                            }
                        }
                        EffectAction::Summon(pet) => {
                            let summon_triggers = self.summon_pet(pet, adj_idx);
                            if let Ok(summon_triggers) = summon_triggers {
                                outcomes.extend(summon_triggers.into_iter())
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            },

            Target::Enemy => match effect.position {
                Position::Any => match &effect.effect {
                    EffectAction::Add(stats) => {
                        if let Some(target) = opponent.get_any_pet() {
                            target.borrow().stats.borrow_mut().add(stats);
                            info!(target: "dev", "Added {:?} to {:?}.", stats, &effect.target)
                        }
                    }
                    EffectAction::Remove(stats) => {
                        if let Some(target) = opponent.get_any_pet() {
                            target.borrow().stats.borrow_mut().sub(stats);
                            info!(target: "dev", "Removed {:?} to {:?}.", stats, &effect.target)
                        }
                    }
                    EffectAction::Gain(food) => {
                        if let Some(target) = opponent.get_any_pet() {
                            target.borrow_mut().item = Some(*food.clone());
                            info!(target: "dev", "Gave {:?} to {:?}", food, target)
                        }
                    }
                    // Must also emit EffectTrigger for summon.
                    EffectAction::Summon(pet) => {
                        let mut rng = rand::thread_rng();
                        let random_pos = (0..5).choose(&mut rng).unwrap() as usize;
                        let summon_triggers = self.summon_pet(pet, random_pos);
                        if let Ok(summon_triggers) = summon_triggers {
                            outcomes.extend(summon_triggers.into_iter())
                        }
                    }
                    _ => {}
                },
                Position::All => todo!(),
                Position::Trigger => todo!(),
                Position::Specific(_) => todo!(),
                Position::None => todo!(),
            },
            Target::None | Target::OnSelf => {}
        };

        Ok(outcomes)
    }

    /// Apply provided effect triggers to both teams.
    fn apply_triggers(&self, opponent: &Team) -> &Self {
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

            // Cleanup fainted pets.
            self.clear_team();
        }
        self
    }

    fn fight(&mut self, opponent: &mut Team) {
        // Clear empty spaces and fainted pets.
        self.clear_team().apply_triggers(opponent);
        opponent.clear_team().apply_triggers(self);

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

            self.clear_team().apply_triggers(opponent);
            opponent.clear_team().apply_triggers(self);

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
            self.apply_triggers(opponent);
            opponent.apply_triggers(self);
        }
        if self.friends.borrow().is_empty() && opponent.friends.borrow().is_empty() {
            info!(target: "dev", "Draw!")
        } else if !self.friends.borrow().is_empty() {
            info!(target: "dev", "Your team won!")
        } else {
            info!(target: "dev", "Enemy team won...")
        }
    }
}
