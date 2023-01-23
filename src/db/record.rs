use crate::db::pack::Pack;
use serde::{Deserialize, Serialize};

/// A record with information about a food from *Super Auto Pets*.
///
/// This information is queried and parsed from the *Super Auto Pets* *Fandom* wiki.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FoodRecord {
    pub name: String,
    pub tier: usize,
    pub effect: String,
    pub pack: Pack,
    pub holdable: bool,
    pub single_use: bool,
    pub end_of_battle: bool,
    pub random: bool,
    pub n_targets: usize,
    pub effect_atk: isize,
    pub effect_health: isize,
    pub turn_effect: bool,
    pub cost: usize,
}

/// A record with information about a pet from *Super Auto Pets*.
///
/// This information is queried and parsed from the *Super Auto Pets* *Fandom* wiki.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PetRecord {
    pub name: String,
    pub tier: usize,
    pub attack: usize,
    pub health: usize,
    pub pack: Pack,
    pub effect_trigger: Option<String>,
    pub effect: Option<String>,
    pub effect_atk: usize,
    pub effect_health: usize,
    pub n_triggers: usize,
    pub temp_effect: bool,
    pub lvl: usize,
    pub cost: usize,
}
