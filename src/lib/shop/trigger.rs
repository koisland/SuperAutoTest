use crate::{
    effects::state::{Condition, EqualityCondition, Outcome, Status, Target},
    Position,
};

/// Create a trigger for when a pet is bought with an effect that triggers with this status.
pub(crate) fn trigger_any_pet_bought_status(status: Status) -> Outcome {
    Outcome {
        status: Status::BuyPet,
        affected_pet: None,
        affected_team: Target::Friend,
        afflicting_pet: None,
        afflicting_team: Target::None,
        position: Position::Any(Condition::Equal(EqualityCondition::Trigger(status))),
        stat_diff: None,
    }
}
/// Trigger when food bought.
pub const TRIGGER_ANY_FOOD_BOUGHT: Outcome = Outcome {
    status: Status::BuyFood,
    affected_pet: None,
    affected_team: Target::Friend,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::Any(Condition::None),
    stat_diff: None,
};

/// Trigger when food bought.
pub const TRIGGER_SELF_FOOD_EATEN: Outcome = Outcome {
    status: Status::AteFood,
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

/// Trigger when pet bought.
pub const TRIGGER_ANY_PET_BOUGHT: Outcome = Outcome {
    status: Status::BuyPet,
    affected_pet: None,
    affected_team: Target::Friend,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::Any(Condition::None),
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

/// Trigger when food bought.
pub const TRIGGER_SHOP_TIER_UPGRADED: Outcome = Outcome {
    status: Status::ShopTierUpgrade,
    affected_pet: None,
    affected_team: Target::Friend,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::None,
    stat_diff: None,
};
