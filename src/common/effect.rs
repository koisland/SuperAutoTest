use lazy_regex::regex;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{food::Food, pets::names::PetName};

pub static RGX_PERC: &lazy_regex::Lazy<lazy_regex::Regex> = regex!(r#"(\d+)%"#);
pub static RGX_ATK: &lazy_regex::Lazy<lazy_regex::Regex> = regex!(r#"(\d)\sattack"#);
pub static RGX_HEALTH: &lazy_regex::Lazy<lazy_regex::Regex> = regex!(r#"(\d)\shealth"#);
pub static RGX_DMG: &lazy_regex::Lazy<lazy_regex::Regex> = regex!(r#"(\d+)\sdamage"#);
pub static RGX_N_TRIGGERS: &lazy_regex::Lazy<lazy_regex::Regex> =
    regex!(r#"Triggers\s(\d+)\stimes"#);
pub static RGX_SUMMON_ATK: &lazy_regex::Lazy<lazy_regex::Regex> = regex!(r#"(\d+)/"#);
pub static RGX_SUMMON_HEALTH: &lazy_regex::Lazy<lazy_regex::Regex> = regex!(r#"/(\d+)"#);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Statistics {
    pub attack: usize,
    pub health: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Action {
    Attack,
    Hurt,
    KnockOut,
    Faint,
    Summoned,
    Pushed,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Position {
    Any,
    All,
    Trigger,
    Specific(isize),
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PetEffect {
    pub trigger: EffectTrigger,
    pub target: Target,
    pub position: Position,
    pub effect: Effect,
    pub uses: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FoodEffect {
    pub target: Target,
    pub position: Position,
    pub effect: Effect,
    pub uses: Option<usize>,
}

pub trait Modify {
    /// Add `n` uses to a `FoodEffect` or `PetEffect` if possible.
    fn add_uses(&mut self, n: usize) -> &Self;

    /// Remove `n` uses to a `FoodEffect` or `PetEffect` if possible.
    fn remove_uses(&mut self, n: usize) -> &Self;
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Target {
    OnSelf,
    Friend,
    Enemy,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Outcome {
    pub action: Action,
    pub position: Option<Position>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum EffectTrigger {
    StartBattle,
    Friend(Outcome),
    Enemy(Outcome),
    None,
    NotImplemented,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Effect {
    Add(Statistics),
    Remove(Statistics),
    Negate(Statistics),
    Gain(Box<Food>),
    Summon(Option<PetName>, Statistics),
    None,
    NotImplemented,
}
