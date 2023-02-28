//! [`Team`](crate::teams::team::Team) battle and effect logic..

/// Implements [`Team`](crate::Team) battle mechanics.
pub mod combat;
/// [`Effect`](crate::Effect) application to one or more [`Team`](crate::Team)s.
pub mod effects;
/// Serialize a [`Team`](crate::Team) using [`serde_json`].
pub mod serialize;
/// [`Team`](crate::Team) of [`Pet`](crate::Pet)s.
pub mod team;
/// View a [`Team`](crate::Team)'s [`Pet`](crate::Pet)s.
pub mod viewer;
