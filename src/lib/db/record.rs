use crate::{db::pack::Pack, FoodName, PetName};
use serde::{Deserialize, Serialize};

/// A record with information about a food from Super Auto Pets.
///
/// This information is queried and parsed from the Super Auto Pets Fandom wiki.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FoodRecord {
    /// Name of food.
    pub name: FoodName,
    /// Food tier.
    pub tier: usize,
    /// Food effect description.
    pub effect: String,
    /// Pack of food.
    pub pack: Pack,
    /// If food is holdable or not.
    pub holdable: bool,
    /// If food has a single-use.
    pub single_use: bool,
    /// If food lasts only until the end of battle.
    pub end_of_battle: bool,
    /// If the food targets a random friend or has some randomness associated with it.
    pub random: bool,
    /// The number of targets this food affects.
    pub n_targets: usize,
    /// Effect attack/damage. Can be:
    /// * A summoned pet's attack.
    /// * The amount of attack to give/remove.
    /// * A percentage of attack to buff/debuff by.
    pub effect_atk: isize,
    /// Effect health. Can be:
    /// * A summoned pet's health.
    /// * The amount of health to give/remove.
    /// * A percentage of health to buff/debuff by.
    pub effect_health: isize,
    /// If the food has a turn-based effect.
    /// * ex. [`Grapes`](crate::foods::names::FoodName::Grapes) give +1 gold at the start of a turn.
    pub turn_effect: bool,
    /// The cost of the food.
    pub cost: usize,
}

/// A record with information about a pet from Super Auto Pets.
///
/// This information is queried and parsed from the Super Auto Pets Fandom wiki.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PetRecord {
    /// Name of pet.
    pub name: PetName,
    /// Tier of pet.
    pub tier: usize,
    /// Attack of pet.
    pub attack: usize,
    /// Health of pet.
    pub health: usize,
    /// Pack of pet.
    pub pack: Pack,
    /// The effect trigger of the pet.
    pub effect_trigger: Option<String>,
    /// The effect description.
    pub effect: Option<String>,
    /// The effect attack. This can be:
    /// * The attack of a summoned pet from the effect.
    /// * The damage dealt by the effect.
    /// * The percentage of attack buffed/debuffed
    pub effect_atk: usize,
    /// The effect health. This can be:
    /// * The health of a summoned pet from the effect.
    /// * The health given by the effect.
    /// * The percentage of health buffed/debuffed
    pub effect_health: usize,
    /// The number of triggers the pet's effect has.
    pub n_triggers: usize,
    /// If the effect the pet has is temporary.
    pub temp_effect: bool,
    /// The pet's level.
    pub lvl: usize,
    /// The cost of the pet.
    pub cost: usize,
}