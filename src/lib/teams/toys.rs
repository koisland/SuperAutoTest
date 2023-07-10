use crate::Team;

/// Toy mechanics.
pub trait ToyEffects {
    /// Choose a toy based on Team's tier/turns.
    fn choose_toy(self);
}

impl ToyEffects for Team {
    fn choose_toy(self) {}
}
