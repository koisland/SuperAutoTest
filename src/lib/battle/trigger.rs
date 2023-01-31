use crate::battle::state::{Condition, Outcome, Position, Statistics, Status, Target};

/// Get enemy faint triggers when a [`Pet`](crate::pets::pet::Pet) on the `self` team faints.
pub fn get_self_enemy_faint_triggers(
    pos: Option<usize>,
    health_diff_stats: &Option<Statistics>,
) -> [Outcome; 2] {
    // Add triggers for enemy.
    let mut enemy_faint = TRIGGER_SPEC_ENEMY_FAINT;
    let mut enemy_any_faint = TRIGGER_ANY_ENEMY_FAINT;
    enemy_faint.position = Position::Relative(pos.unwrap_or(0).try_into().unwrap());
    (enemy_faint.to_idx, enemy_any_faint.to_idx) = (pos, pos);
    (enemy_faint.stat_diff, enemy_any_faint.stat_diff) = (*health_diff_stats, *health_diff_stats);
    [enemy_faint, enemy_any_faint]
}

/// Get faint triggers when a [`Pet`](crate::pets::pet::Pet) on the `self` team faints.
pub fn get_self_faint_triggers(
    pos: Option<usize>,
    health_diff_stats: &Option<Statistics>,
) -> [Outcome; 3] {
    let (mut self_faint, mut any_faint, mut ahead_faint) =
        (TRIGGER_SELF_FAINT, TRIGGER_ANY_FAINT, TRIGGER_AHEAD_FAINT);

    (self_faint.to_idx, any_faint.to_idx, ahead_faint.to_idx) = (pos, pos, pos.map(|pos| pos + 1));
    (
        self_faint.stat_diff,
        any_faint.stat_diff,
        ahead_faint.stat_diff,
    ) = (*health_diff_stats, *health_diff_stats, *health_diff_stats);

    [self_faint, any_faint, ahead_faint]
}

/// All start of battle triggers.
/// * Currently start of turn triggers are included as `Shop`s have not been implemented.
pub const ALL_TRIGGERS_START_BATTLE: [Outcome; 2] = [TRIGGER_START_TURN, TRIGGER_START_BATTLE];

/// Start of battle trigger.
pub const TRIGGER_START_BATTLE: Outcome = Outcome {
    from_target: Target::None,
    status: Status::StartOfBattle,
    to_target: Target::None,
    position: Position::None,
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for phase after start of battle but before first battle.
/// * Used for Butterfly.
pub const TRIGGER_BEFORE_FIRST_BATTLE: Outcome = Outcome {
    from_target: Target::None,
    status: Status::BeforeFirstBattle,
    to_target: Target::None,
    position: Position::None,
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Start of turn trigger.
pub const TRIGGER_START_TURN: Outcome = Outcome {
    from_target: Target::None,
    status: Status::StartTurn,
    to_target: Target::None,
    position: Position::None,
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// End of turn trigger.
pub const TRIGGER_END_TURN: Outcome = Outcome {
    from_target: Target::None,
    status: Status::EndTurn,
    to_target: Target::None,
    position: Position::None,
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// End of battle trigger.
pub const TRIGGER_END_BATTLE: Outcome = Outcome {
    from_target: Target::None,
    status: Status::EndOfBattle,
    to_target: Target::None,
    position: Position::None,
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for nothing?
/// * Dunno why I made this.
pub const TRIGGER_NONE: Outcome = Outcome {
    from_target: Target::None,
    status: Status::None,
    to_target: Target::None,
    position: Position::None,
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when friendly [`Pet`](crate::pets::pet::Pet) is unhurt after an attack.
pub const TRIGGER_SELF_UNHURT: Outcome = Outcome {
    from_target: Target::None,
    status: Status::None,
    to_target: Target::Friend,
    position: Position::OnSelf,
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when friendly [`Pet`](crate::pets::pet::Pet) faints.
pub const TRIGGER_SELF_FAINT: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Faint,
    to_target: Target::Friend,
    position: Position::OnSelf,
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when any friendly [`Pet`](crate::pets::pet::Pet) faints.
pub const TRIGGER_ANY_FAINT: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Faint,
    to_target: Target::Friend,
    position: Position::Any(Condition::None),
    // Gets replaced at runtime.
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when any enemy [`Pet`](crate::pets::pet::Pet) faints.
pub const TRIGGER_ANY_ENEMY_FAINT: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Faint,
    to_target: Target::Enemy,
    position: Position::Any(Condition::None),
    // Gets replaced at runtime.
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when an enemy [`Pet`](crate::pets::pet::Pet) is knocked out.
pub const TRIGGER_KNOCKOUT: Outcome = Outcome {
    from_target: Target::None,
    status: Status::KnockOut,
    to_target: Target::Enemy,
    position: Position::Relative(0),
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when a specific enemy [`Pet`](crate::pets::pet::Pet) faints.
pub const TRIGGER_SPEC_ENEMY_FAINT: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Faint,
    to_target: Target::Enemy,
    position: Position::Relative(0),
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when the friend ahead [`Pet`](crate::pets::pet::Pet) faints.
pub const TRIGGER_AHEAD_FAINT: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Faint,
    to_target: Target::Friend,
    position: Position::Relative(-1),
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when the the current friendly [`Pet`](crate::pets::pet::Pet) is hurt.
pub const TRIGGER_SELF_HURT: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Hurt,
    to_target: Target::Friend,
    position: Position::OnSelf,
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when the any friendly [`Pet`](crate::pets::pet::Pet) is hurt.
pub const TRIGGER_ANY_HURT: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Hurt,
    to_target: Target::Friend,
    position: Position::Any(Condition::None),
    // Gets replaced at runtime.
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when the any enemy [`Pet`](crate::pets::pet::Pet) is hurt.
pub const TRIGGER_ANY_ENEMY_HURT: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Hurt,
    to_target: Target::Enemy,
    position: Position::Any(Condition::None),
    // Gets replaced at runtime.
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when the current [`Pet`](crate::pets::pet::Pet) attacks.
pub const TRIGGER_SELF_ATTACK: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Attack,
    to_target: Target::Friend,
    position: Position::OnSelf,
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when the [`Pet`](crate::pets::pet::Pet) ahead attacks.
pub const TRIGGER_AHEAD_ATTACK: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Attack,
    to_target: Target::Friend,
    position: Position::Relative(1),
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when the current [`Pet`](crate::pets::pet::Pet) is summoned.
pub const TRIGGER_SELF_SUMMON: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Summoned,
    to_target: Target::Friend,
    position: Position::OnSelf,
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when any friendly [`Pet`](crate::pets::pet::Pet) is summoned.
pub const TRIGGER_ANY_SUMMON: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Summoned,
    to_target: Target::Friend,
    position: Position::Any(Condition::None),
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when any enemy [`Pet`](crate::pets::pet::Pet) is summoned.
pub const TRIGGER_ANY_ENEMY_SUMMON: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Summoned,
    to_target: Target::Enemy,
    position: Position::Any(Condition::None),
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when any friendly [`Pet`](crate::pets::pet::Pet) is pushed.
pub const TRIGGER_ANY_PUSHED: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Pushed,
    to_target: Target::Friend,
    position: Position::Any(Condition::None),
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when any enemy [`Pet`](crate::pets::pet::Pet) is pushed.
pub const TRIGGER_ANY_ENEMY_PUSHED: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Pushed,
    to_target: Target::Enemy,
    position: Position::Any(Condition::None),
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};

/// Trigger for when any friend [`Pet`](crate::pets::pet::Pet) levels up.
pub const TRIGGER_ANY_LEVELUP: Outcome = Outcome {
    from_target: Target::None,
    status: Status::Levelup,
    to_target: Target::Friend,
    position: Position::Any(Condition::None),
    to_idx: None,
    from_idx: None,
    stat_diff: None,
};
