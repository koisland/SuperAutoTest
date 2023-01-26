//! Battle logic for a [`Team`](crate::battle::team::Team).

/// Effect struct.
pub mod effect;
/// Logic for monitoring [`Pet`](crate::Pet) and [`Team`](crate::Team) state.
pub mod state;
/// Methods for constructing and modifying a Super Auto Pets [`Team`](crate::Team).
pub mod team;
/// Methods for applying [`Effect`](crate::Effect)s to a Super Auto Pets [`Team`](crate::Team).
pub mod team_effect_apply;
/// TODO: Methods for summarizing a Super Auto Pets [`Team`](crate::Team).
pub mod team_summary;
/// [`Effect`](crate::Effect) triggers.
pub mod trigger;
