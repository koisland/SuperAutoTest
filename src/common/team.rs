use super::{
    effect::{Action, Effect, EffectTrigger, Modify, Outcome, PetEffect, Position, Target},
    pet::{self, Combat, Pet},
};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const TEAM_SIZE: usize = 5;
const TRIGGER_BEFORE_ATTACK: EffectTrigger = EffectTrigger::Friend(Outcome {
    action: Action::Attack,
    position: Some(Position::Specific(0)),
});
const TRIGGER_AHEAD_ATTACK: EffectTrigger = EffectTrigger::Friend(Outcome {
    action: Action::Attack,
    position: Some(Position::Specific(-1)),
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
}

pub trait Summary {
    fn mean(&self) -> f32;
    fn median(&self) -> f32;
}
pub trait Battle {
    // fn is_team_alive(&self) -> bool;
    fn next_pet(&mut self) -> Option<&mut Pet>;
    fn apply_triggers(&mut self, opponent: &mut Team, triggers: &mut VecDeque<EffectTrigger>);
    fn fight(&mut self, opponent: &mut Team);
}

impl Team {
    fn new(pets: &[Option<Pet>]) -> Result<Team, &'static str> {
        if pets.len() != TEAM_SIZE {
            return Err("Invalid team size.");
        };

        Ok(Team {
            friends: pets.to_vec(),
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

    /// Get the next pet in team.
    fn next_pet(&mut self) -> Option<&mut Pet> {
        self.friends
            .iter_mut()
            // Allows for spaces in team.
            .find(|pet| pet.is_some() && pet.as_ref().map_or(false, |pet| pet.stats.health != 0))
            .map(|pet| pet.as_mut().unwrap())
    }

    /// Apply provided effect triggers to both teams.
    fn apply_triggers(&mut self, opponent: &mut Team, triggers: &mut VecDeque<EffectTrigger>) {
        // Continue iterating until all triggers consumed.
        while let Some(trigger) = triggers.front() {
            // Iterate through pets.
            for pet in self.friends.iter() {
                if let Some(pet) = pet {
                    pet.effect
                        .as_ref()
                        // Check if pet's effect_trigger is in provided triggers.
                        .map(|effect| if trigger == &effect.trigger {});
                }
            }
            // Cleanup fainted pets.
        }
    }

    fn fight(&mut self, opponent: &mut Team) {
        let mut triggers: VecDeque<EffectTrigger> =
            VecDeque::from_iter([EffectTrigger::StartBattle]);

        self.apply_triggers(opponent, &mut triggers);

        // Check that both teams have a pet that is alive.
        while let (Some(first_pet), Some(first_enemy_pet)) = (self.next_pet(), opponent.next_pet())
        {
            // TODO: Trigger Before Attack && Friend Ahead attack.
            // TRIGGER_AHEAD_ATTACK
            // TRIGGER_BEFORE_ATTACK
            // self.friends.iter().map(|pet| );
            // Attack
            let outcome = first_pet.attack(first_enemy_pet);
            info!("Outcome of fight: {:?}", outcome);

            // Apply food effect that targets after combat. ex. Splash
            if let Some(food) = first_pet.item.as_mut() {
                let food_effect = &food.ability;
                let target = &food_effect.target;
                match food_effect.target {
                    Target::Enemy => match &food_effect.effect {
                        Effect::Remove(_) => {}
                        _ => {}
                    },
                    _ => {}
                }
                food.remove_uses(1);
            }
            // If any hurt.

            // If any faint.
        }
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
