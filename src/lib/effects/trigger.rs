use crate::effects::{
    state::{ItemCondition, Outcome, Position, Status, Target},
    stats::Statistics,
};

use super::state::EqualityCondition;

/// Get enemy faint triggers when a [`Pet`](crate::pets::pet::Pet) on the `self` team faints.
pub fn get_self_enemy_faint_triggers(health_diff_stats: &Option<Statistics>) -> [Outcome; 2] {
    // Add triggers for enemy.
    let mut enemy_faint = TRIGGER_SPEC_ENEMY_FAINT;
    let mut enemy_any_faint = TRIGGER_ANY_ENEMY_FAINT;
    (enemy_faint.stat_diff, enemy_any_faint.stat_diff) = (*health_diff_stats, *health_diff_stats);
    [enemy_faint, enemy_any_faint]
}

/// Get faint triggers when a [`Pet`](crate::pets::pet::Pet) on the `self` team faints.
pub fn get_self_faint_triggers(health_diff_stats: &Option<Statistics>) -> [Outcome; 3] {
    let (mut self_faint, mut any_faint, mut ahead_faint) =
        (TRIGGER_SELF_FAINT, TRIGGER_ANY_FAINT, TRIGGER_AHEAD_FAINT);
    self_faint.stat_diff = *health_diff_stats;
    any_faint.stat_diff = *health_diff_stats;
    ahead_faint.stat_diff = *health_diff_stats;

    [self_faint, any_faint, ahead_faint]
}

/// Start of battle trigger.
pub const TRIGGER_START_BATTLE: Outcome = Outcome {
    status: Status::StartOfBattle,
    position: Position::None,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::None,
    afflicting_team: Target::None,
};

/// Trigger for phase after start of battle but before first battle.
/// * Used for Butterfly.
pub const TRIGGER_BEFORE_FIRST_BATTLE: Outcome = Outcome {
    status: Status::BeforeFirstBattle,
    position: Position::None,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::None,
    afflicting_team: Target::None,
};

/// Start of turn trigger.
pub const TRIGGER_START_TURN: Outcome = Outcome {
    status: Status::StartTurn,
    position: Position::None,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::None,
    afflicting_team: Target::None,
};

/// End of turn trigger.
pub const TRIGGER_END_TURN: Outcome = Outcome {
    status: Status::EndTurn,
    position: Position::None,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::None,
    afflicting_team: Target::None,
};

/// Won battle trigger.
pub const TRIGGER_WIN_BATTLE: Outcome = Outcome {
    status: Status::WinBattle,
    position: Position::None,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::None,
    afflicting_team: Target::None,
};

/// Won battle trigger.
pub const TRIGGER_LOSE_BATTLE: Outcome = Outcome {
    status: Status::LoseBattle,
    position: Position::None,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::None,
    afflicting_team: Target::None,
};

/// Draw battle trigger.
pub const TRIGGER_DRAW_BATTLE: Outcome = Outcome {
    status: Status::DrawBattle,
    position: Position::None,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::None,
    afflicting_team: Target::None,
};

/// Triggers for either attack or indirect attack damage calculation.
pub const TRIGGER_DMG_CALC: Outcome = Outcome {
    status: Status::AnyDmgCalc,
    affected_pet: None,
    affected_team: Target::None,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::OnSelf,
    stat_diff: None,
};

/// Triggers for only attack dmg calculation.
pub const TRIGGER_ATK_DMG_CALC: Outcome = Outcome {
    status: Status::AttackDmgCalc,
    affected_pet: None,
    affected_team: Target::None,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::OnSelf,
    stat_diff: None,
};

/// Triggers for only indirect attack calculation.
pub const TRIGGER_INDIR_DMG_CALC: Outcome = Outcome {
    status: Status::IndirectAttackDmgCalc,
    affected_pet: None,
    affected_team: Target::None,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::OnSelf,
    stat_diff: None,
};

/// Trigger for nothing?
/// * Dunno why I made this.
pub const TRIGGER_NONE: Outcome = Outcome {
    status: Status::None,
    position: Position::None,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::None,
    afflicting_team: Target::None,
};

/// Trigger for when friendly [`Pet`](crate::pets::pet::Pet) is unhurt after an attack.
pub const TRIGGER_SELF_UNHURT: Outcome = Outcome {
    status: Status::None,
    position: Position::OnSelf,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when friendly [`Pet`](crate::pets::pet::Pet) faints.
pub const TRIGGER_SELF_FAINT: Outcome = Outcome {
    status: Status::Faint,
    position: Position::OnSelf,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when friendly [`Pet`](crate::pets::pet::Pet) levelsup.
pub const TRIGGER_SELF_LEVELUP: Outcome = Outcome {
    status: Status::Levelup,
    position: Position::OnSelf,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when any friendly [`Pet`](crate::pets::pet::Pet) faints.
pub const TRIGGER_ANY_FAINT: Outcome = Outcome {
    status: Status::Faint,
    position: Position::Any(ItemCondition::None),
    // Gets replaced at runtime.
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when any enemy [`Pet`](crate::pets::pet::Pet) faints.
pub const TRIGGER_ANY_ENEMY_FAINT: Outcome = Outcome {
    status: Status::Faint,
    position: Position::Any(ItemCondition::None),
    // Gets replaced at runtime.
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Enemy,
    afflicting_team: Target::None,
};

/// Trigger for when an enemy [`Pet`](crate::pets::pet::Pet) is knocked out.
pub const TRIGGER_KNOCKOUT: Outcome = Outcome {
    status: Status::KnockOut,
    position: Position::Relative(0),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when a specific enemy [`Pet`](crate::pets::pet::Pet) faints.
pub const TRIGGER_SPEC_ENEMY_FAINT: Outcome = Outcome {
    status: Status::Faint,
    position: Position::Relative(0),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Enemy,
    afflicting_team: Target::None,
};

/// Trigger for when the friend ahead [`Pet`](crate::pets::pet::Pet) faints.
pub const TRIGGER_AHEAD_FAINT: Outcome = Outcome {
    status: Status::Faint,
    position: Position::Relative(1),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when the the current friendly [`Pet`](crate::pets::pet::Pet) is hurt.
pub const TRIGGER_SELF_HURT: Outcome = Outcome {
    status: Status::Hurt,
    position: Position::OnSelf,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when the any friendly [`Pet`](crate::pets::pet::Pet) is hurt.
pub const TRIGGER_ANY_HURT: Outcome = Outcome {
    status: Status::Hurt,
    position: Position::Any(ItemCondition::None),
    // Gets replaced at runtime.
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when the any enemy [`Pet`](crate::pets::pet::Pet) is hurt.
pub const TRIGGER_ANY_ENEMY_HURT: Outcome = Outcome {
    status: Status::Hurt,
    position: Position::Any(ItemCondition::None),
    // Gets replaced at runtime.
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Enemy,
    afflicting_team: Target::None,
};

/// Trigger for when the current [`Pet`](crate::pets::pet::Pet) attacks.
pub const TRIGGER_SELF_ATTACK: Outcome = Outcome {
    status: Status::Attack,
    position: Position::OnSelf,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for before the current [`Pet`](crate::pets::pet::Pet) attacks.
pub const TRIGGER_SELF_BEFORE_ATTACK: Outcome = Outcome {
    status: Status::BeforeAttack,
    position: Position::OnSelf,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for before any [`Pet`](crate::pets::pet::Pet) attacks.
/// * Ignore self.
pub const TRIGGER_ANY_BEFORE_ATTACK: Outcome = Outcome {
    status: Status::BeforeAttack,
    position: Position::Any(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when the [`Pet`](crate::pets::pet::Pet) ahead attacks.
pub const TRIGGER_AHEAD_ATTACK: Outcome = Outcome {
    status: Status::Attack,
    position: Position::Relative(1),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when the current [`Pet`](crate::pets::pet::Pet) is summoned.
pub const TRIGGER_SELF_SUMMON: Outcome = Outcome {
    status: Status::Summoned,
    position: Position::OnSelf,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when any friendly [`Pet`](crate::pets::pet::Pet) is summoned.
pub const TRIGGER_ANY_SUMMON: Outcome = Outcome {
    status: Status::Summoned,
    position: Position::Any(ItemCondition::None),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when any enemy [`Pet`](crate::pets::pet::Pet) is summoned.
pub const TRIGGER_ANY_ENEMY_SUMMON: Outcome = Outcome {
    status: Status::Summoned,
    position: Position::Any(ItemCondition::None),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Enemy,
    afflicting_team: Target::None,
};

/// Trigger for when any friendly [`Pet`](crate::pets::pet::Pet) is pushed.
pub const TRIGGER_ANY_PUSHED: Outcome = Outcome {
    status: Status::Pushed,
    position: Position::Any(ItemCondition::None),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when any enemy [`Pet`](crate::pets::pet::Pet) is pushed.
pub const TRIGGER_ANY_ENEMY_PUSHED: Outcome = Outcome {
    status: Status::Pushed,
    position: Position::Any(ItemCondition::None),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Enemy,
    afflicting_team: Target::None,
};

/// Trigger for when any friend [`Pet`](crate::pets::pet::Pet) levels up.
pub const TRIGGER_ANY_LEVELUP: Outcome = Outcome {
    status: Status::Levelup,
    position: Position::Any(ItemCondition::None),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};
