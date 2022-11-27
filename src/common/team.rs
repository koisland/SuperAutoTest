use super::pet::Pet;

pub struct Team<'a> {
    pub friends: &'a [Option<Pet>; 5],
}

trait Summary {
    fn mean(&self) -> f32;
    fn median(&self) -> f32;
}
trait Battle {
    fn fight(&mut self, opponent: &mut Team);
}

impl Team<'_> {
    fn new(pets: &[Option<Pet>; 5]) -> Team {
        Team {
            friends: pets
        }
    }
}

impl Battle for Team<'_> {
    fn fight(&mut self, opponent: &mut Team) {
        todo!()
    }
}

mod tests {
    use crate::common::{pets::names::PetName, effect::Statistics, pet::Pet, team::Team};

    #[test]
    fn test_build_team() {
        let pet_1 = Pet::new(PetName::Ant, Statistics { attack: 2, health: 1 }, 1, None).unwrap();
        let pet_2 = Pet::new(PetName::Ant, Statistics { attack: 2, health: 1 }, 1, None).unwrap();
        let pet_3 = Pet::new(PetName::Ant, Statistics { attack: 2, health: 1 }, 1, None).unwrap();

        let pets = [Some(pet_1), Some(pet_2), Some(pet_3), None, None];

        let team = Team::new(&pets);
    }

    fn test_battle_team() {
        let pet_1 = Pet::new(PetName::Ant, Statistics { attack: 2, health: 1 }, 1, None).unwrap();
        let pet_2 = Pet::new(PetName::Ant, Statistics { attack: 2, health: 1 }, 1, None).unwrap();
        let pet_3 = Pet::new(PetName::Ant, Statistics { attack: 2, health: 1 }, 1, None).unwrap();
        let enemy_pet_1 = Pet::new(PetName::Ant, Statistics { attack: 2, health: 1 }, 1, None).unwrap();
        let enemy_pet_2 = Pet::new(PetName::Ant, Statistics { attack: 2, health: 1 }, 1, None).unwrap();
        let enemy_pet_3 = Pet::new(PetName::Ant, Statistics { attack: 2, health: 1 }, 1, None).unwrap();

        let pets = [Some(pet_1), Some(pet_2), Some(pet_3), None, None];
        let enemy_pets = [Some(enemy_pet_1), Some(enemy_pet_2), Some(enemy_pet_3), None, None];

        let team = Team::new(&pets);
        let enemy_team = Team::new(&enemy_pets);

    }
}
