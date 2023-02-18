//! [`Team`](crate::battle::team::Team) battle and effect logic..

/// [`Team`](crate::Team) of [`Pet`](crate::Pet)s.
pub mod team;
/// [`Effect`](crate::Effect) application to one or more [`Team`](crate::Team)s.
pub mod team_effect_apply;
/// Serialize a [`Team`](crate::Team) using [`serde_json`].
pub mod team_serialize;
/// View a [`Team`](crate::Team)'s [`Pet`](crate::Pet)s.
pub mod team_viewer;
