use crate::{
    db::{
        pack::Pack,
        record::{FoodRecord, PetRecord},
    },
    error::SAPTestError,
    toys::names::ToyName,
    FoodName, PetName,
};
use rusqlite::Row;
use std::str::FromStr;

use super::record::ToyRecord;

impl TryFrom<&Row<'_>> for PetRecord {
    type Error = SAPTestError;

    fn try_from(pet_row: &Row<'_>) -> Result<Self, Self::Error> {
        let pet_name: String = pet_row.get(1)?;
        let pack: String = pet_row.get(5)?;
        let is_temp_effect_str: String = pet_row.get(11)?;
        let is_token: String = pet_row.get(15)?;
        Ok(PetRecord {
            name: PetName::from_str(&pet_name)?,
            tier: pet_row.get(2)?,
            attack: pet_row.get(3)?,
            health: pet_row.get(4)?,
            pack: Pack::from_str(&pack)?,
            effect_trigger: pet_row.get(6)?,
            effect: pet_row.get(7)?,
            effect_atk: pet_row.get(8)?,
            effect_health: pet_row.get(9)?,
            n_triggers: pet_row.get(10)?,
            temp_effect: is_temp_effect_str == *"true",
            lvl: pet_row.get(12)?,
            cost: pet_row.get(13)?,
            img_url: pet_row.get(14)?,
            is_token: is_token == *"true",
        })
    }
}

impl TryFrom<&Row<'_>> for FoodRecord {
    type Error = SAPTestError;

    fn try_from(food_row: &Row<'_>) -> Result<Self, Self::Error> {
        let food_name: String = food_row.get(1)?;
        let pack: String = food_row.get(4)?;
        let holdable_str: String = food_row.get(5)?;
        let single_use_str: String = food_row.get(6)?;
        let end_of_battle_str: String = food_row.get(7)?;
        let random_str: String = food_row.get(8)?;
        let turn_effect_str: String = food_row.get(12)?;
        Ok(FoodRecord {
            name: FoodName::from_str(&food_name)?,
            tier: food_row.get(2)?,
            effect: food_row.get(3)?,
            pack: Pack::from_str(&pack)?,
            holdable: holdable_str == *"true",
            single_use: single_use_str == *"true",
            end_of_battle: end_of_battle_str == *"true",
            random: random_str == *"true",
            n_targets: food_row.get(9)?,
            effect_atk: food_row.get(10)?,
            effect_health: food_row.get(11)?,
            turn_effect: turn_effect_str == *"true",
            cost: food_row.get(13)?,
            img_url: food_row.get(14)?,
        })
    }
}

impl TryFrom<&Row<'_>> for ToyRecord {
    type Error = SAPTestError;

    fn try_from(toy_row: &Row<'_>) -> Result<Self, Self::Error> {
        let toy_name: String = toy_row.get(1)?;
        let toy_effect_trigger: String = toy_row.get(3)?;
        let toy_effect: String = toy_row.get(4)?;
        let is_temp_effect: String = toy_row.get(8)?;
        let source: String = toy_row.get(10)?;
        let is_hard_mode: String = toy_row.get(12)?;
        Ok(ToyRecord {
            name: ToyName::from_str(&toy_name)?,
            tier: toy_row.get(2)?,
            effect_trigger: Some(toy_effect_trigger),
            effect: Some(toy_effect),
            effect_atk: toy_row.get(5)?,
            effect_health: toy_row.get(6)?,
            n_triggers: toy_row.get(7)?,
            temp_effect: is_temp_effect == *"true",
            lvl: toy_row.get(9)?,
            source: Some(source),
            img_url: toy_row.get(11)?,
            hard_mode: is_hard_mode == *"true",
        })
    }
}
