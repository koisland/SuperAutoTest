use crate::{Outcome, Position, battle::state::{Status, Target}};


pub const TRIGGER_FOOD_BOUGHT: Outcome = Outcome {
    status: Status::BuyFood,
    affected_pet: None,
    affected_team: Target::Friend,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::TriggerAffected,
    stat_diff: None,
};

pub const TRIGGER_PET_BOUGHT: Outcome = Outcome {
    status: Status::BuyPet,
    affected_pet: None,
    affected_team: Target::Friend,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::TriggerAffected,
    stat_diff: None,
};

pub const TRIGGER_ROLL: Outcome = Outcome {
    status: Status::Roll,
    affected_pet: None,
    affected_team: Target::Friend,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::TriggerAffected,
    stat_diff: None,
};