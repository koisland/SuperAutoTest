use crate::common::pack::Pack;
use serde::{Deserialize, Serialize};

/// A record with information about a food from *Super Auto Pets*.
///
/// This information is queried and parsed from the *Super Auto Pets* *Fandom* wiki.
#[derive(Debug, Serialize, Deserialize)]
pub struct FoodRecord {
    pub name: String,
    pub tier: usize,
    pub effect: String,
    pub pack: Pack,
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
    pub lvl: usize,
}
