use serde::{Deserialize, Serialize};

use super::{food::Food, pet::Pet};

#[derive(Debug, Deserialize, Serialize)]
pub struct Statistics {
    pub attack: usize,
    pub health: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Action {
    Attack,
    Faint,
    Summoned,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Position {
    Any,
    All,
    Specific(usize),
    None,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PetEffect {
    pub trigger: EffectTrigger,
    pub target: Target,
    pub position: Position,
    pub effect: Effect,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FoodEffect {
    pub target: Target,
    pub position: Position,
    pub effect: Effect,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Target {
    ToSelf,
    Friend,
    Enemy,
    None,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum EffectTrigger {
    Hurt,
    Faint,
    StartBattle,
    KnockOut,
    Friend(Action, Position),
    Enemy(Action, Position),
    None,
    NotImplemented,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Effect {
    Add(Statistics),
    Remove(Statistics),
    Negate(Statistics),
    Splash,
    Gain(Box<Food>),
    Summon(Box<Pet>),
    None,
    NotImplemented,
}
