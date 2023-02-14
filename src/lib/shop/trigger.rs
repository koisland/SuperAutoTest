use crate::{
    battle::state::{Condition, Status, Target},
    Outcome, Position,
};

/// Trigger when food bought.
pub const TRIGGER_FOOD_BOUGHT: Outcome = Outcome {
    status: Status::BuyFood,
    affected_pet: None,
    affected_team: Target::Friend,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::OnSelf,
    stat_diff: None,
};

/// Trigger when pet bought.
pub const TRIGGER_SELF_PET_BOUGHT: Outcome = Outcome {
    status: Status::BuyPet,
    affected_pet: None,
    affected_team: Target::Friend,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::OnSelf,
    stat_diff: None,
};

/// Trigger when any pet sold.
pub const TRIGGER_ANY_PET_SOLD: Outcome = Outcome {
    status: Status::Sell,
    affected_pet: None,
    affected_team: Target::Friend,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::Any(Condition::None),
    stat_diff: None,
};

/// Trigger when pet sold.
pub const TRIGGER_SELF_PET_SOLD: Outcome = Outcome {
    status: Status::Sell,
    affected_pet: None,
    affected_team: Target::Friend,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::OnSelf,
    stat_diff: None,
};

/// Trigger when shop rolled.
pub const TRIGGER_ROLL: Outcome = Outcome {
    status: Status::Roll,
    affected_pet: None,
    affected_team: Target::Friend,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::TriggerAffected,
    stat_diff: None,
};
