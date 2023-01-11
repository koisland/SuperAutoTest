use crate::common::battle::state::{Outcome, Position, Status, Target};

pub const TRIGGER_START_BATTLE: Outcome = Outcome {
    status: Status::StartBattle,
    target: Target::None,
    position: Position::None,
    idx: None,
};

pub const TRIGGER_NONE: Outcome = Outcome {
    status: Status::None,
    target: Target::None,
    position: Position::None,
    idx: None,
};

pub const TRIGGER_SELF_UNHURT: Outcome = Outcome {
    status: Status::None,
    target: Target::Friend,
    position: Position::OnSelf,
    idx: Some(0),
};

// * If a pet faints.
// * Is a friend.
// * Its position relative to the curr pet is 0 (self).
pub const TRIGGER_SELF_FAINT: Outcome = Outcome {
    status: Status::Faint,
    target: Target::Friend,
    position: Position::OnSelf,
    idx: Some(0),
};

pub const TRIGGER_ANY_FAINT: Outcome = Outcome {
    status: Status::Faint,
    target: Target::Friend,
    position: Position::Any,
    // Gets replaced at runtime.
    idx: None,
};

pub const TRIGGER_SELF_HURT: Outcome = Outcome {
    status: Status::Hurt,
    target: Target::Friend,
    position: Position::OnSelf,
    idx: Some(0),
};

// If a pet is attacking
// Is a friend
// Its position relative to the curr pet is 0 (self).
pub const TRIGGER_SELF_ATTACK: Outcome = Outcome {
    status: Status::Attack,
    target: Target::Friend,
    position: Position::OnSelf,
    idx: Some(0),
};

pub const TRIGGER_AHEAD_ATTACK: Outcome = Outcome {
    status: Status::Attack,
    target: Target::Friend,
    position: Position::Specific(1),
    idx: Some(1),
};

pub const TRIGGER_SELF_SUMMON: Outcome = Outcome {
    status: Status::Summoned,
    target: Target::Friend,
    position: Position::OnSelf,
    idx: Some(0),
};

pub const TRIGGER_ANY_SUMMON: Outcome = Outcome {
    status: Status::Summoned,
    target: Target::Friend,
    position: Position::Any,
    idx: None,
};

pub const TRIGGER_ANY_ENEMY_SUMMON: Outcome = Outcome {
    status: Status::Summoned,
    target: Target::Enemy,
    position: Position::Any,
    idx: None,
};
