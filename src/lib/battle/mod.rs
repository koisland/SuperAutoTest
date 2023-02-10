//! Battle logic for a [`Team`](crate::battle::team::Team).

/// [`Effect`](crate::Effect) actions and their possible types.
pub mod actions;
/// [`Effect`](crate::Effect) struct.
pub mod effect;
/// [`Pet`](crate::Pet) and [`Team`](crate::Team) state.
pub mod state;
/// [`Pet`](crate::Pet) and [`Food`](crate::Food) stats.
pub mod stats;
/// [`Team`](crate::Team) of [`Pet`](crate::Pet)s.
pub mod team;
/// [`Effect`](crate::Effect) application to one or more [`Team`](crate::Team)s.
pub mod team_effect_apply;
/// Serialize a [`Team`](crate::Team) using [`serde_json`].
pub mod team_serialize;
/// [`Effect`](crate::Effect) triggers.
pub mod trigger;
