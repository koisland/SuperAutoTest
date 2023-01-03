use super::{
    effect::{Action, Effect, EffectAction, EffectTrigger, Modify, Outcome, Position, Target},
    food::Food,
    pet::{self, Combat, Pet},
};
use itertools::Itertools;
use log::info;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const TEAM_SIZE: usize = 5;
const TRIGGER_SELF_ATTACK: EffectTrigger = EffectTrigger::Friend(Outcome {
    action: Action::Attack,
    position: Some(Position::Specific(0)),
});
const TRIGGER_AHEAD_ATTACK: EffectTrigger = EffectTrigger::Friend(Outcome {
    action: Action::Attack,
    position: Some(Position::Specific(-1)),
});
const TRIGGER_SELF_HURT: EffectTrigger = EffectTrigger::Friend(Outcome {
    action: Action::Hurt,
    position: Some(Position::Specific(0)),
});
const TRIGGER_SELF_FAINT: EffectTrigger = EffectTrigger::Friend(Outcome {
    action: Action::Faint,
    position: Some(Position::Specific(0)),
});
const TRIGGER_ANY_SUMMON: EffectTrigger = EffectTrigger::Friend(Outcome {
    action: Action::Summoned,
    position: Some(Position::Any),
});
const TRIGGER_AHEAD_FAINT: EffectTrigger = EffectTrigger::Friend(Outcome {
    action: Action::Faint,
    position: Some(Position::Specific(-1)),
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub friends: Vec<Option<Pet>>,
    pub triggers: VecDeque<EffectTrigger>,
}

pub trait Summary {
    fn mean(&self) -> f32;
    fn median(&self) -> f32;
}
pub trait Battle {
    // fn is_team_alive(&self) -> bool;
    fn clear_team(&mut self) -> &mut Self;
    fn get_next_pet(&mut self) -> Option<&mut Pet>;
    fn get_any_pet(&mut self) -> Option<&mut Pet>;
    fn get_all_pets(&mut self) -> Vec<&mut Pet>;
    fn apply_triggers(&mut self, opponent: &mut Team) -> &mut Self;
    fn fight(&mut self, opponent: &mut Team);
}

impl Team {
    fn new(pets: &[Option<Pet>]) -> Result<Team, &'static str> {
        if pets.len() != TEAM_SIZE {
            return Err("Invalid team size.");
        };

        Ok(Team {
            friends: pets.to_vec(),
            triggers: VecDeque::from_iter([EffectTrigger::StartBattle]),
        })
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
    fn clear_team(&mut self) -> &mut Self {
        let missing_pets = self
            .friends
            .iter()
            .enumerate()
            .filter_map(|(i, pet)| {
                if pet.is_none() {
                    Some(i)
                } else if pet.as_ref().map_or(false, |pet| pet.stats.health != 0) {
                    None
                } else {
                    // Pet is dead.
                    self.triggers.push_back(TRIGGER_SELF_FAINT);
                    Some(i)
                }
            })
            .rev()
            .collect_vec();
        for rev_idx in missing_pets.iter() {
            self.friends.remove(*rev_idx);
        }
        self
    }
    /// Get the next pet in team.
    fn get_next_pet(&mut self) -> Option<&mut Pet> {
        let next_pet = self.friends.iter_mut().next();
        if let Some(Some(pet)) = next_pet {
            Some(pet)
        } else {
            None
        }
    }

    fn get_any_pet(&mut self) -> Option<&mut Pet> {
        let mut rng = rand::thread_rng();
        self.friends
            .iter_mut()
            .choose(&mut rng)
            .map(|pet| pet.as_mut().unwrap())
    }

    fn get_all_pets(&mut self) -> Vec<&mut Pet> {
        self.friends
            .iter_mut()
            .filter_map(|pet| pet.as_mut())
            .collect_vec()
    }

    /// Apply provided effect triggers to both teams.
    fn apply_triggers(&mut self, opponent: &mut Team) -> &mut Self {
        // Continue iterating until all triggers consumed.
        while let Some(trigger) = self.triggers.front() {
            let mut valid_effects: VecDeque<Effect> = VecDeque::new();

            // Iterate through pets collecting valid effects.
            for (i, pet) in self
                .friends
                .iter_mut()
                .filter_map(|pet| pet.as_mut())
                .enumerate()
            {
                // TODO: check food effect
                if let Some(food) = pet.item.as_mut() {
                    if trigger == &food.ability.trigger {
                        // Adjust food effect's position to match pet.
                        food.ability.position = Position::Specific(i as isize);
                        valid_effects.push_back(food.ability.clone());
                        food.ability.remove_uses(1);
                    }
                }
                if let Some(effect) = pet.effect.as_mut() {
                    // Match on trigger in triggers.
                    // Also allow if effect triggers on any pet since cannot know until runtime.
                    if trigger == &effect.trigger || effect.trigger.affects_any() {
                        // Clone the effect
                        valid_effects.push_back(effect.clone());
                        // Decrement its use.
                        effect.remove_uses(1);
                    }
                }
            }
            for effect in valid_effects {
                match &effect.target {
                    Target::Friend => match &effect.position {
                        Position::Any => {
                            if let Some(target) = self.get_any_pet() {
                                match &effect.effect {
                                    EffectAction::Add(stats) => {
                                        target.stats.add(stats);
                                    }
                                    EffectAction::Remove(stats) => {
                                        target.stats.sub(stats);
                                    }
                                    EffectAction::Gain(food) => {
                                        target.item = Some(*food.clone());
                                    }
                                    // Must also emit EffectTrigger for summon.
                                    EffectAction::Summon(pet) => {
                                        if self.get_all_pets().len() == 5 {
                                            info!("Team is full.");
                                            continue;
                                        }
                                        if let Some(stored_pet) = pet.clone() {
                                            let mut rng = rand::thread_rng();
                                            let random_pos =
                                                (0..5).choose(&mut rng).unwrap() as usize;
                                            self.friends.insert(random_pos, Some(*stored_pet));
                                            self.triggers.push_back(EffectTrigger::Friend(
                                                Outcome {
                                                    action: Action::Summoned,
                                                    position: Some(Position::Specific(
                                                        random_pos as isize,
                                                    )),
                                                },
                                            ))
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Position::All => match &effect.effect {
                            EffectAction::Add(stats) => {
                                for pet in self.get_all_pets() {
                                    pet.stats.add(stats);
                                }
                            }
                            EffectAction::Remove(stats) => {
                                for pet in self.get_all_pets() {
                                    pet.stats.sub(stats);
                                }
                            }
                            _ => {}
                        },
                        Position::Trigger => todo!(),
                        Position::Specific(pos) => todo!(),
                        _ => {}
                    },

                    Target::Enemy => match effect.position {
                        Position::Any => {
                            if let Some(target) = opponent.get_any_pet() {
                                match &effect.effect {
                                    EffectAction::Add(stats) => {
                                        target.stats.add(stats);
                                        info!("Added {:?} to {:?}", stats, target)
                                    }
                                    EffectAction::Remove(stats) => {
                                        target.stats.sub(stats);
                                        info!("Removed {:?} to {:?}", stats, target)
                                    }
                                    EffectAction::Gain(food) => {
                                        target.item = Some(*food.clone());
                                        info!("Gave {:?} to {:?}", food, target)
                                    }
                                    // Must also emit EffectTrigger for summon.
                                    EffectAction::Summon(pet) => {
                                        if opponent.get_all_pets().len() == 5 {
                                            info!("Team is full. Cannot add {:?}", pet);
                                            continue;
                                        }
                                        if let Some(stored_pet) = pet.clone() {
                                            let mut rng = rand::thread_rng();
                                            let random_pos =
                                                (0..5).choose(&mut rng).unwrap() as usize;
                                            info!("Summoned {:?}.", &stored_pet);
                                            opponent.friends.insert(random_pos, Some(*stored_pet));
                                            opponent.triggers.push_back(EffectTrigger::Friend(
                                                Outcome {
                                                    action: Action::Summoned,
                                                    position: Some(Position::Specific(
                                                        random_pos as isize,
                                                    )),
                                                },
                                            ))
                                        }
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
            }
            // Cleanup fainted pets.
            self.clear_team();

            // Remove checked trigger.
            self.triggers.pop_front();
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
                .extend([TRIGGER_SELF_ATTACK, TRIGGER_AHEAD_ATTACK]);
            opponent
                .triggers
                .extend([TRIGGER_SELF_ATTACK, TRIGGER_AHEAD_ATTACK]);

            self.clear_team().apply_triggers(opponent);
            opponent.clear_team().apply_triggers(self);

            // Check that two pets exist and attack.
            // Attack will result in triggers being added.
            if let (Some(pet), Some(opponent_pet)) = (self.get_next_pet(), opponent.get_next_pet())
            {
                // Attack
                let outcome = pet.attack(opponent_pet);
                info!("Outcome of fight: {:?}", outcome);
            } else {
                // If either side has no available pets, exit loop.
                break;
            }

            // Occurs even if pet fainted as fighting and applying effects occurs simultaneously.
            // Apply any food effects that alter the opponents pets. ex. Chili
            if let Some(pet) = self.get_next_pet() {
                pet.apply_food_effect(opponent)
            }
            if let Some(opponent_pet) = opponent.get_next_pet() {
                opponent_pet.apply_food_effect(self)
            }

            // Apply effect triggers from combat phase.
            self.clear_team().apply_triggers(opponent);
            opponent.clear_team().apply_triggers(self);
        }
        if self.friends.is_empty() && opponent.friends.is_empty() {
            info!("Draw!")
        } else if self.friends.is_empty() {
            info!("Your team won!")
        } else {
            info!("Enemy team won...")
        }
    }
}

mod tests {
    use crate::common::{
        effect::Statistics, food::Food, foods::names::FoodName, pet::Pet, pets::names::PetName,
        team::Battle, team::Team,
    };

    #[test]
    fn test_build_team() {
        let pet_1 = Pet::new(
            PetName::Ant,
            Statistics {
                attack: 2,
                health: 1,
            },
            1,
            None,
        )
        .unwrap();
        let pet_2 = Pet::new(
            PetName::Ant,
            Statistics {
                attack: 2,
                health: 1,
            },
            1,
            None,
        )
        .unwrap();
        let pet_3 = Pet::new(
            PetName::Ant,
            Statistics {
                attack: 2,
                health: 1,
            },
            1,
            None,
        )
        .unwrap();

        let pets = [Some(pet_1), Some(pet_2), Some(pet_3), None, None];

        let team = Team::new(&pets);
    }

    #[test]
    fn test_battle_team() {
        let pet_1 = Pet::new(
            PetName::Ant,
            Statistics {
                attack: 2,
                health: 1,
            },
            1,
            None,
        )
        .unwrap();
        let pet_2 = Pet::new(
            PetName::Ant,
            Statistics {
                attack: 2,
                health: 1,
            },
            1,
            None,
        )
        .unwrap();
        let pet_3 = Pet::new(
            PetName::Ant,
            Statistics {
                attack: 2,
                health: 1,
            },
            1,
            None,
        )
        .unwrap();
        let enemy_pet_1 = Pet::new(
            PetName::Ant,
            Statistics {
                attack: 2,
                health: 1,
            },
            1,
            None,
        )
        .unwrap();
        let enemy_pet_2 = Pet::new(
            PetName::Ant,
            Statistics {
                attack: 2,
                health: 1,
            },
            1,
            None,
        )
        .unwrap();
        let enemy_pet_3 = Pet::new(
            PetName::Ant,
            Statistics {
                attack: 2,
                health: 1,
            },
            1,
            Some(Food::new(&FoodName::Honey)),
        )
        .unwrap();

        let pets = [Some(pet_1), Some(pet_2), Some(pet_3), None, None];
        let enemy_pets = [
            Some(enemy_pet_1),
            Some(enemy_pet_2),
            Some(enemy_pet_3),
            None,
            None,
        ];

        let mut team = Team::new(&pets).unwrap();
        let mut enemy_team = Team::new(&enemy_pets).unwrap();

        team.fight(&mut enemy_team)
    }
}
