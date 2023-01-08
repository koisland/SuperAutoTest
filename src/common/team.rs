use super::{
    effect::{Action, Effect, EffectAction, EffectTrigger, Outcome, Position, Target},
    pet::{Combat, Pet},
};
use itertools::Itertools;
use log::info;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::VecDeque, fmt::Display, rc::Rc};

#[allow(dead_code)]
const TEAM_SIZE: usize = 5;
const TRIGGER_SELF_ATTACK: EffectTrigger = EffectTrigger::Friend(Outcome {
    action: Action::Attack,
    position: Some(Position::Specific(0)),
});
const TRIGGER_AHEAD_ATTACK: EffectTrigger = EffectTrigger::Friend(Outcome {
    action: Action::Attack,
    position: Some(Position::Specific(-1)),
});
// const TRIGGER_SELF_HURT: EffectTrigger = EffectTrigger::Friend(Outcome {
//     action: Action::Hurt,
//     position: Some(Position::Specific(0)),
// });
const TRIGGER_SELF_FAINT: EffectTrigger = EffectTrigger::Friend(Outcome {
    action: Action::Faint,
    position: Some(Position::Specific(0)),
});
// const TRIGGER_ANY_SUMMON: EffectTrigger = EffectTrigger::Friend(Outcome {
//     action: Action::Summoned,
//     position: Some(Position::Any),
// });
// const TRIGGER_AHEAD_FAINT: EffectTrigger = EffectTrigger::Friend(Outcome {
//     action: Action::Faint,
//     position: Some(Position::Specific(-1)),
// });

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub friends: RefCell<Vec<Option<Rc<RefCell<Pet>>>>>,
    pub triggers: RefCell<VecDeque<EffectTrigger>>,
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
    fn summon_pet(pet: &Option<Box<Pet>>, pos: usize, team: &Team) -> Result<(), &'static str>;
    fn apply_triggers(&self, opponent: &Team) -> &Self;
    fn apply_effect(
        pet_idx: usize,
        effect: Effect,
        team: &Team,
        opponent: &Team,
    ) -> Result<(), &'static str>;
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
            triggers: RefCell::new(VecDeque::from_iter([EffectTrigger::StartBattle])),
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
                if pet.is_none() {
                    Some(i)
                } else if pet
                    .as_ref()
                    .map_or(false, |pet| pet.borrow().stats.borrow().health != 0)
                {
                    None
                } else {
                    // Pet is dead.
                    info!(target: "dev", "Pet ({i}) {} is dead. Removing.", pet.as_ref().unwrap().borrow());
                    self.triggers.borrow_mut().push_back(TRIGGER_SELF_FAINT);
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
            Some(pet.clone())
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
            .filter_map(|pet| pet.as_ref().cloned())
            .collect_vec()
    }

    fn summon_pet(pet: &Option<Box<Pet>>, pos: usize, team: &Team) -> Result<(), &'static str> {
        if team.get_all_pets().len() == 5 {
            info!(target: "dev", "Team is full.");
            return Err("Team is full.");
        }
        if let Some(stored_pet) = pet.clone() {
            info!(target: "dev", "Summoned {:?}.", stored_pet);
            team.friends
                .borrow_mut()
                .insert(pos, Some(Rc::new(RefCell::new(*stored_pet))));
            team.triggers
                .borrow_mut()
                .push_back(EffectTrigger::Friend(Outcome {
                    action: Action::Summoned,
                    position: Some(Position::Specific(pos as isize)),
                }));
        }
        Ok(())
    }
    fn apply_effect(
        pet_idx: usize,
        effect: Effect,
        team: &Team,
        opponent: &Team,
    ) -> Result<(), &'static str> {
        match &effect.target {
            Target::Friend => match &effect.position {
                Position::Any => {
                    if let Some(target) = team.get_any_pet() {
                        match &effect.effect {
                            EffectAction::Add(stats) => {
                                target.borrow().stats.borrow_mut().add(stats);
                                info!(target: "dev", "Added {:?} to {:?}\n\t({:?}).", stats, &effect.target, target.borrow());
                            }
                            EffectAction::Remove(stats) => {
                                target.borrow().stats.borrow_mut().sub(stats);
                                info!(target: "dev", "Removed {:?} to {:?}\n\t({:?}).", stats, &effect.target, target.borrow());
                            }
                            EffectAction::Gain(food) => {
                                target.borrow_mut().item = Some(*food.clone());
                                info!(target: "dev", "Gave {:?} to {:?}\n\t({:?}).", food, &effect.target, target.borrow());
                            }
                            // Must also emit EffectTrigger for summon.
                            EffectAction::Summon(pet) => {
                                let mut rng = rand::thread_rng();
                                let random_pos = (0..5).choose(&mut rng).unwrap() as usize;
                                Team::summon_pet(pet, random_pos, team).unwrap();
                            }
                            _ => {}
                        }
                    }
                }
                Position::All => match &effect.effect {
                    EffectAction::Add(stats) => {
                        for pet in team.get_all_pets() {
                            pet.borrow().stats.borrow_mut().add(stats);
                        }
                    }
                    EffectAction::Remove(stats) => {
                        for pet in team.get_all_pets() {
                            pet.borrow().stats.borrow_mut().sub(stats);
                        }
                    }
                    _ => {}
                },
                Position::Trigger => todo!(),
                Position::Specific(rel_pos) => {
                    let adj_idx: usize = ((pet_idx as isize) + *rel_pos) as usize;
                    if let Some(affected_pet) = team.get_all_pets().get(adj_idx) {
                        match &effect.effect {
                            EffectAction::Add(stats) => {
                                affected_pet.borrow().stats.borrow_mut().add(stats);
                            }
                            EffectAction::Remove(stats) => {
                                affected_pet.borrow().stats.borrow_mut().sub(stats);
                            }
                            EffectAction::Gain(food) => {
                                affected_pet.borrow_mut().item = Some(*food.clone())
                            }
                            EffectAction::Summon(pet) => {
                                Team::summon_pet(pet, adj_idx, team).unwrap();
                            }
                            _ => {}
                        }
                    } else {
                        info!("Cannot access friend at {:?}.", &adj_idx);
                    }
                }
                _ => {}
            },

            Target::Enemy => match effect.position {
                Position::Any => {
                    if let Some(target) = opponent.get_any_pet() {
                        match &effect.effect {
                            EffectAction::Add(stats) => {
                                target.borrow().stats.borrow_mut().add(stats);
                                info!(target: "dev", "Added {:?} to {:?}.", stats, &effect.target)
                            }
                            EffectAction::Remove(stats) => {
                                target.borrow().stats.borrow_mut().sub(stats);
                                info!(target: "dev", "Removed {:?} to {:?}.", stats, &effect.target)
                            }
                            EffectAction::Gain(food) => {
                                target.borrow_mut().item = Some(*food.clone());
                                info!(target: "dev", "Gave {:?} to {:?}", food, target)
                            }
                            // Must also emit EffectTrigger for summon.
                            EffectAction::Summon(pet) => {
                                let mut rng = rand::thread_rng();
                                let random_pos = (0..5).choose(&mut rng).unwrap() as usize;
                                Team::summon_pet(pet, random_pos, opponent).unwrap();
                            }
                            _ => {}
                        }
                    }
                }
                Position::All => todo!(),
                Position::Trigger => todo!(),
                Position::Specific(_) => todo!(),
                Position::None => todo!(),
            },
            Target::None | Target::OnSelf => {}
        };

        Ok(())
    }

    /// Apply provided effect triggers to both teams.
    fn apply_triggers(&self, opponent: &Team) -> &Self {
        // Continue iterating until all triggers consumed.
        while let Some(trigger) = self.triggers.borrow_mut().pop_front() {
            // Iterate through pets collecting valid effects.
            for (i, pet) in self.get_all_pets().iter().enumerate() {
                let food_effect = pet
                    .borrow()
                    .item
                    .as_ref()
                    .filter(|food| food.ability.trigger == trigger)
                    .map(|food| food.ability.clone());
                let pet_effect = pet
                    .borrow()
                    .effect
                    .as_ref()
                    .filter(|effect| effect.trigger == trigger)
                    .cloned();

                if let Some(food_effect) = food_effect {
                    Team::apply_effect(i, food_effect, self, opponent).unwrap();
                }
                if let Some(pet_effect) = pet_effect {
                    Team::apply_effect(i, pet_effect, self, opponent).unwrap();
                }
            }
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
            if let (Some(pet), Some(opponent_pet)) = (self.get_next_pet(), opponent.get_next_pet())
            {
                // Attack
                let outcome = pet.borrow_mut().attack(&mut opponent_pet.borrow_mut());
                info!(target: "dev", "Outcome of fight: {:?}", outcome);
                info!(target: "dev", "Self:\n{}", self);
                info!(target: "dev", "Opponent:\n{}", opponent);
            } else {
                // If either side has no available pets, exit loop.
                break;
            }

            // Occurs even if pet fainted as fighting and applying effects occurs simultaneously.
            // Apply any food effects that alter the opponents pets. ex. Chili
            if let Some(pet) = self.get_next_pet() {
                pet.borrow_mut().apply_food_effect(opponent)
            }
            if let Some(opponent_pet) = opponent.get_next_pet() {
                opponent_pet.borrow_mut().apply_food_effect(self)
            }

            // Apply effect triggers from combat phase.
            self.clear_team().apply_triggers(opponent);
            opponent.clear_team().apply_triggers(self);
        }
        if self.friends.borrow().is_empty() && opponent.friends.borrow().is_empty() {
            info!(target: "dev", "Draw!")
        } else if self.friends.borrow().is_empty() {
            info!(target: "dev", "Your team won!")
        } else {
            info!(target: "dev", "Enemy team won...")
        }
    }
}
