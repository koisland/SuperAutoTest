use std::{
    str::FromStr,
    sync::{RwLock, Weak},
};

use crate::{
    effects::{
        state::{ItemCondition, Outcome, Position, Status, Target},
        stats::Statistics,
    },
    error::SAPTestError,
    shop::trigger::{
        TRIGGER_ANY_FOOD_BOUGHT, TRIGGER_ANY_FOOD_EATEN, TRIGGER_ANY_GAIN_PERK,
        TRIGGER_ANY_PET_BOUGHT, TRIGGER_ANY_PET_SOLD, TRIGGER_ROLL, TRIGGER_SELF_FOOD_EATEN,
        TRIGGER_SELF_PET_BOUGHT, TRIGGER_SELF_PET_SOLD, TRIGGER_SHOP_TIER_UPGRADED,
        TRIGGER_TOY_BREAK,
    },
    Pet,
};

use super::state::{EqualityCondition, TeamCondition};

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

/// Get faint triggers when a [`Pet`](crate::pets::pet::Pet) on the `self` team is summoned.
pub fn get_summon_triggers(pet: Weak<RwLock<Pet>>) -> [Outcome; 3] {
    let mut self_trigger = TRIGGER_SELF_SUMMON;
    let mut any_trigger = TRIGGER_ANY_SUMMON;
    let mut any_enemy_trigger = TRIGGER_ANY_ENEMY_SUMMON;

    (
        self_trigger.affected_pet,
        any_trigger.affected_pet,
        any_enemy_trigger.affected_pet,
    ) = (Some(pet.clone()), Some(pet.clone()), Some(pet));
    [self_trigger, any_trigger, any_enemy_trigger]
}

/// Wrapper for [`Vec<Outcomes>`].
pub struct Outcomes(Vec<Outcome>);

impl std::ops::Deref for Outcomes {
    type Target = Vec<Outcome>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Outcomes {
    type Err = SAPTestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut outcomes: Vec<Outcome> = Vec::with_capacity(3);

        match s {
            "Sell" => outcomes.push(TRIGGER_SELF_PET_SOLD),
            "Level up" => outcomes.push(TRIGGER_SELF_LEVELUP),
            "Friend summoned" => outcomes.push(TRIGGER_ANY_SUMMON),
            "Start of battle" => outcomes.push(TRIGGER_START_BATTLE),
            "Buy" => outcomes.push(TRIGGER_SELF_PET_BOUGHT),
            "Buy from shop" => outcomes.extend([TRIGGER_ANY_PET_BOUGHT, TRIGGER_ANY_FOOD_BOUGHT]),
            "Eat shop food" => outcomes.push(TRIGGER_SELF_FOOD_EATEN),
            "End turn" => outcomes.push(TRIGGER_END_TURN),
            "Shop food bought" => outcomes.push(TRIGGER_ANY_FOOD_BOUGHT),
            "Start of turn" => outcomes.push(TRIGGER_START_TURN),
            "Enemy summoned or pushed" => {
                outcomes.extend([TRIGGER_ANY_ENEMY_SUMMON, TRIGGER_ANY_ENEMY_PUSHED])
            }
            "On break" => outcomes.push(TRIGGER_TOY_BREAK),
            "Shop tier upgraded" => outcomes.push(TRIGGER_SHOP_TIER_UPGRADED),
            "Hurt" => outcomes.push(TRIGGER_SELF_HURT),
            "Friend ahead hurt" => outcomes.push(TRIGGER_AHEAD_HURT),
            "None" => outcomes.push(TRIGGER_NONE),
            "Before attack" => outcomes.push(TRIGGER_SELF_BEFORE_ATTACK),
            "Friend gained perk" => outcomes.push(TRIGGER_ANY_GAIN_PERK),
            // TODO:
            "Before faint" => outcomes.push(TRIGGER_AHEAD_ATTACK),

            "Friend sold" => outcomes.push(TRIGGER_ANY_PET_SOLD),
            "Pet level-up" => outcomes.push(TRIGGER_ANY_LEVELUP),
            "Friend hurt" => outcomes.push(TRIGGER_ANY_HURT),
            "Friend bought" => outcomes.push(TRIGGER_ANY_PET_BOUGHT),

            // TODO:
            "Empty Front Space" => outcomes.push(TRIGGER_AHEAD_ATTACK),

            "Friend ahead attacks" => outcomes.push(TRIGGER_AHEAD_ATTACK),
            "Friend ahead faints" => outcomes.push(TRIGGER_AHEAD_FAINT),
            "Shop food eaten" => outcomes.push(TRIGGER_ANY_FOOD_EATEN),
            "Enemy summoned" => outcomes.push(TRIGGER_ANY_ENEMY_SUMMON),
            "Roll" => outcomes.push(TRIGGER_ROLL),
            "Friendly pet level-up" => outcomes.push(TRIGGER_ANY_LEVELUP),
            "Enemy hurt" => outcomes.push(TRIGGER_ANY_ENEMY_HURT),
            "Knock out" => outcomes.push(TRIGGER_KNOCKOUT),
            "Hurt & Faint" => outcomes.extend([TRIGGER_SELF_HURT, TRIGGER_SELF_FAINT]),
            // TODO:
            "Eats Apple" => outcomes.push(TRIGGER_SELF_FOOD_EATEN),
            "Friend faints" => outcomes.push(TRIGGER_ANY_FAINT),
            "End turn & Start of battle" => {
                outcomes.extend([TRIGGER_END_TURN, TRIGGER_START_BATTLE])
            }
            "Summoned" => outcomes.push(TRIGGER_SELF_SUMMON),

            // TODO:
            "Before friend attacks" => outcomes.push(TRIGGER_SELF_BEFORE_ATTACK),

            // Two friends faint is handled by conditional Effect.
            "Two friends faint" => outcomes.push(TRIGGER_ANY_FAINT),
            "Buy & Sell" => outcomes.extend([TRIGGER_SELF_PET_BOUGHT, TRIGGER_SELF_PET_SOLD]),
            // Conditional action determines outcome.
            "Tier 1 friend bought" => outcomes.push(TRIGGER_ANY_PET_BOUGHT),
            "All enemies fainted" => outcomes.push(TRIGGER_NO_ENEMIES_LEFT),
            _ => {}
        }

        Ok(Outcomes(outcomes))
    }
}

/// Trigger for when one pet left on team.
pub const TRIGGER_NO_ENEMIES_LEFT: Outcome = Outcome {
    status: Status::IsTeam(TeamCondition::NumberPets(Some(0))),
    affected_pet: None,
    affected_team: Target::Enemy,
    afflicting_pet: None,
    afflicting_team: Target::Friend,
    position: Position::None,
    stat_diff: None,
    afflicting_food: None,
};

/// Trigger for when one pet left on team.
pub const TRIGGER_ONE_OR_ZERO_PET_LEFT: Outcome = Outcome {
    status: Status::IsTeam(TeamCondition::NumberPetsLessEqual(1)),
    affected_pet: None,
    affected_team: Target::Friend,
    afflicting_pet: None,
    afflicting_team: Target::Enemy,
    position: Position::None,
    stat_diff: None,
    afflicting_food: None,
};

/// Start of battle trigger.
pub const TRIGGER_START_BATTLE: Outcome = Outcome {
    status: Status::StartOfBattle,
    position: Position::None,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
    affected_team: Target::None,
    afflicting_team: Target::None,
};

/// Triggers for either attack or indirect attack damage calculation.
/// * Will not activate anything during other phases.
pub const TRIGGER_DMG_CALC: Outcome = Outcome {
    status: Status::AnyDmgCalc,
    affected_pet: None,
    affected_team: Target::None,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::OnSelf,
    stat_diff: None,
    afflicting_food: None,
};

/// Triggers for only attack dmg calculation.
/// * Will not activate anything during other phases.
pub const TRIGGER_ATK_DMG_CALC: Outcome = Outcome {
    status: Status::AttackDmgCalc,
    affected_pet: None,
    affected_team: Target::None,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::OnSelf,
    stat_diff: None,
    afflicting_food: None,
};

/// Triggers for only indirect attack calculation.
/// * Will not activate anything during other phases.
pub const TRIGGER_INDIR_DMG_CALC: Outcome = Outcome {
    status: Status::IndirectAttackDmgCalc,
    affected_pet: None,
    affected_team: Target::None,
    afflicting_pet: None,
    afflicting_team: Target::None,
    position: Position::OnSelf,
    stat_diff: None,
    afflicting_food: None,
};

/// Trigger for nothing?
/// * Dunno why I made this.
pub const TRIGGER_NONE: Outcome = Outcome {
    status: Status::None,
    position: Position::None,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
    affected_team: Target::Enemy,
    afflicting_team: Target::None,
};

/// TODO: Needs to work if pet faints.
/// Trigger for when a [`Pet`](crate::pets::pet::Pet) knocks out another pet.
pub const TRIGGER_KNOCKOUT: Outcome = Outcome {
    status: Status::KnockOut,
    position: Position::OnSelf,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    afflicting_food: None,
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
    afflicting_food: None,
    affected_team: Target::Enemy,
    afflicting_team: Target::None,
};

/// Trigger for when the friend directly one position ahead [`Pet`](crate::pets::pet::Pet) faints.
pub const TRIGGER_AHEAD_FAINT: Outcome = Outcome {
    status: Status::Faint,
    position: Position::Nearest(1),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
    affected_team: Target::Enemy,
    afflicting_team: Target::None,
};

/// Trigger for a [`Food`](crate::Food) battle effect.
/// * This causes the effect to activate **during the attack** and can **alter other pets aside from the owner**.
///     * Ex. [`Garlic`](crate::FoodName::Garlic) only alters the owner.
///     * Ex. [`Chili`](crate::FoodName::Chili) can alter the pet behind the attacking pet.
/// * Its behavior also differs from other triggers/effects as this is unaffected by pet attack order/position.
pub const TRIGGER_BATTLE_FOOD: Outcome = Outcome {
    status: Status::BattleFoodEffect,
    position: Position::OnSelf,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    afflicting_food: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when the current [`Pet`](crate::pets::pet::Pet) attacks.
pub const TRIGGER_SELF_ATTACK: Outcome = Outcome {
    status: Status::Attack,
    position: Position::OnSelf,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    afflicting_food: None,
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
    afflicting_food: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// TODO: Needs to work if pet faints.
/// Trigger for after the current [`Pet`](crate::pets::pet::Pet) attacks.
pub const TRIGGER_SELF_AFTER_ATTACK: Outcome = Outcome {
    status: Status::AfterAttack,
    position: Position::OnSelf,
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    afflicting_food: None,
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
    afflicting_food: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when the [`Pet`](crate::pets::pet::Pet) ahead attacks.
pub const TRIGGER_AHEAD_ATTACK: Outcome = Outcome {
    status: Status::Attack,
    position: Position::Nearest(1),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    afflicting_food: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// TODO: Needs to work if pet faints.
/// Trigger for when the [`Pet`](crate::pets::pet::Pet) ahead hurt.
pub const TRIGGER_AHEAD_HURT: Outcome = Outcome {
    status: Status::Hurt,
    position: Position::Nearest(1),
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
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
    afflicting_food: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};

/// Trigger for when any friendly [`Pet`](crate::pets::pet::Pet) gains an ailment.
pub const TRIGGER_ANY_GAIN_AILMENT: Outcome = Outcome {
    status: Status::GainAilment,
    position: Position::Any(ItemCondition::None),
    // Gets replaced at runtime.
    affected_pet: None,
    afflicting_pet: None,
    stat_diff: None,
    afflicting_food: None,
    affected_team: Target::Friend,
    afflicting_team: Target::None,
};
