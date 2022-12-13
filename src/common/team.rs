use super::{
    effect::{Action, Effect, EffectAction, EffectTrigger, Modify, Outcome, Position, Target},
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
const TRIGGER_ANY_FAINT: EffectTrigger = EffectTrigger::Friend(Outcome {
    action: Action::Faint,
    position: Some(Position::Any),
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
    fn clear_team(&mut self) -> &mut Self {
        self.friends = self
            .friends
            .iter()
            .filter(|pet| pet.is_some() && pet.as_ref().map_or(false, |pet| pet.stats.health != 0))
            .cloned()
            .collect_vec();
        self
    }
    /// Get the next pet in team.
    fn get_next_pet(&mut self) -> Option<&mut Pet> {
        self.friends
            .iter_mut()
            .next()
            .map(|pet| pet.as_mut().unwrap())
    }

    fn get_any_pet(&mut self) -> Option<&mut Pet> {
        let mut rng = rand::thread_rng();
        self.friends
            .iter_mut()
            .choose(&mut rng)
            .map(|pet| pet.as_mut().unwrap())
    }

    /// Apply provided effect triggers to both teams.
    fn apply_triggers(&mut self, opponent: &mut Team) -> &mut Self {
        // Continue iterating until all triggers consumed.
        while let Some(trigger) = self.triggers.front() {
            // Iterate through pets.
            for (pos, pet) in self
                .friends
                .iter_mut()
                .filter_map(|pet| pet.as_mut())
                .enumerate()
            {
                if let Some(effect) = pet.effect.as_mut() {
                    for _ in 0..effect.uses.unwrap_or(0) {
                        if trigger == &effect.trigger {
                            match effect.target {
                                Target::OnSelf => {}
                                Target::Friend => {}
                                Target::Enemy => {}
                                Target::None => todo!(),
                            };
                            effect.remove_uses(1);
                        }
                    }
                }
            }
            // Cleanup fainted pets.

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
        while self.friends.len() != 0 && opponent.friends.len() != 0 {
            // Trigger Before Attack && Friend Ahead attack.
            self.triggers
                .extend([TRIGGER_SELF_ATTACK, TRIGGER_AHEAD_ATTACK]);
            opponent
                .triggers
                .extend([TRIGGER_SELF_ATTACK, TRIGGER_AHEAD_ATTACK]);

            self.clear_team().apply_triggers(opponent);
            opponent.clear_team().apply_triggers(self);

            // Check that two pets exist
            if let (Some(pet), Some(opponent_pet)) = (self.get_next_pet(), opponent.get_next_pet())
            {
                // Attack
                let outcome = pet.attack(opponent_pet);
                info!("Outcome of fight: {:?}", outcome);

                // Apply food effect that:
                //  * Targets another pet during combat
                //  * Does damage.
                //  * And affects a specific position
                //  * ex. Chili
                if let Some(food) = pet.item.as_mut() {
                    let food_effect = &food.ability;
                    if let (Target::Enemy, EffectAction::Remove(stats), Position::Specific(idx)) = (
                        &food_effect.target,
                        &food_effect.effect,
                        &food_effect.position,
                    ) {
                        if let Some(target) = opponent
                            .friends
                            .get_mut(*idx as usize)
                            .map(|pet| pet.as_mut().unwrap())
                        {
                            target.stats.sub(stats);
                            food.ability.remove_uses(1);
                        }
                    }
                }

                // Apply effect triggers from combat
                self.clear_team().apply_triggers(opponent);
                opponent.clear_team().apply_triggers(self);
            }
        }
        self.clear_team();
        opponent.clear_team();
    }
}

mod tests {
    use crate::common::{
        effect::Statistics, pet::Pet, pets::names::PetName, team::Battle, team::Team,
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
            None,
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
