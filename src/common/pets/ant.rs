use crate::{
    common::{
        effect::{Effect, EffectTrigger, FoodEffect, PetEffect, Position, Statistics, Target},
        food::Food,
        pet::Pet,
    },
    db::{setup::get_connection, utils::map_row_to_pet},
};
use std::error::Error;

impl Pet {
    pub fn ant(stats: Statistics, lvl: usize, item: Option<Food>) -> Result<Pet, Box<dyn Error>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM pets where name = ? and lvl = ?")?;
        let pet_record = stmt.query_row(["Ant", &lvl.to_string()], map_row_to_pet)?;

        // TODO: Parse from pet description.
        let pet_effect = PetEffect {
            trigger: EffectTrigger::Faint,
            target: Target::Friend,
            position: Position::Any,
            effect: Effect::Add(Statistics {
                attack: 0,
                health: 0,
            }),
            limit: Some(1),
        };

        Ok(Pet {
            name: "Ant".to_string(),
            tier: pet_record.tier,
            attack: stats.attack,
            health: stats.health,
            lvl: pet_record.lvl,
            effect: pet_effect,
            item,
        })
    }
}

mod tests {
    use crate::common::{effect::Statistics, food::Food, pet::Pet};
    use std::error::Error;

    #[test]
    fn test_ant() {
        let stats = Statistics {
            attack: 2,
            health: 1,
        };
        let ant = Pet::ant(stats, 1, None);

        if let Err(err) = ant {
            panic!("{}", err.to_string())
        } else {
            println!("{:?}", ant.unwrap());
        }
    }
}
