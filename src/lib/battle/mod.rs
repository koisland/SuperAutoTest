//! Battle logic for a [`Team`](crate::battle::team::Team).

/// Effect actions.
pub mod actions;
/// Effect struct.
pub mod effect;
/// Logic for monitoring [`Pet`](crate::Pet) and [`Team`](crate::Team) state.
pub mod state;
/// Logic for [`Pet`](crate::Pet) and [`Food`](crate::Food) stats.
pub mod stats;
/// Methods for constructing and modifying a Super Auto Pets [`Team`](crate::Team).
pub mod team;
/// Methods for applying [`Effect`](crate::Effect)s to a Super Auto Pets [`Team`](crate::Team).
pub mod team_effect_apply;
/// TODO: Methods for summarizing a Super Auto Pets [`Team`](crate::Team).
pub mod team_summary;
/// [`Effect`](crate::Effect) triggers.
pub mod trigger;
