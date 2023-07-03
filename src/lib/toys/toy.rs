use serde::{Deserialize, Serialize};

use crate::Effect;

use super::names::ToyName;

/// A Super Auto Pets toy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Toy {
    name: ToyName,
    tier: usize,
    duration: Option<usize>,
    effect: Vec<Effect>,
}
