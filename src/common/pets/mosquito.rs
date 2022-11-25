mod tests {
    use crate::common::{effect::Statistics, pet::Pet, pets::names::PetName};

    #[test]
    fn test_ant() {
        let stats = Statistics {
            attack: 2,
            health: 1,
        };
        let ant = Pet::new(PetName::Mosquito, stats, 3, None);

        if let Err(err) = ant {
            panic!("{}", err.to_string())
        } else {
            println!("{:?}", ant.unwrap());
        }
    }
}
