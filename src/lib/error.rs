//! Error types.

use thiserror::Error;

/// Error types.
#[derive(Error, Debug)]
pub enum SAPTestError {
    /// Failure to serialize team.
    #[error("Failed to serialize/deserialize team.")]
    SerializeFailure(#[from] serde_json::Error),

    /// Failure to request data from Fandom wiki.
    #[error("Failed to get data from Fandom wiki.")]
    RequestFailure(#[from] ureq::Error),

    /// Invalid Fandom wiki page. This error only comes up from ureq's [`Response::into_string`](https://docs.rs/ureq/latest/ureq/struct.Response.html#method.into_string) method.
    ///
    /// This should never be an issue unless a file is misplaced on the Fandom wiki.
    #[error("Fandom wiki page invalid.")]
    InvalidRequestFailure(#[from] std::io::Error),

    /// Failure to execute query from SQLite database.
    #[error("Failed database query execution.")]
    DatabaseFailure(#[from] rusqlite::Error),

    /// Failure to initalize pooled connection with SQLite database.
    #[error("Failed to initialize database.")]
    DatabasePoolFailure(#[from] r2d2::Error),

    /// Empty/invalid query for SQLite database.
    #[error("Failed Query: {subject:?} due to {reason:?}")]
    QueryFailure {
        /// Subject of query failure.
        subject: String,
        /// Reason for query failure.
        reason: String,
    },

    /// Failure to convert [`Statisitics`](crate::Statistics) value to `isize`.
    #[error("Cannot convert Statistics value to isize.")]
    ValueConversionFailure(#[from] std::num::TryFromIntError),

    /// Failed to parse `usize` from string.
    /// * Used in wiki parsing.
    #[error("Failed to parse `usize` from string.")]
    ValueParseFailure(#[from] std::num::ParseIntError),

    /// Invalid [`Team`](crate::Team) action.
    #[error("Invalid team action: {subject:?} due to {reason:?}")]
    InvalidTeamAction {
        /// Subject of action.
        subject: String,
        /// Reason for action failure.
        reason: String,
    },

    /// Invalid [`Pet`](crate::Pet) action.
    #[error("Invalid pet action: {subject:?} due to {reason:?}")]
    InvalidPetAction {
        /// Subject of action.
        subject: String,
        /// Reason for action failure.
        reason: String,
    },

    /// Shop error.
    #[error("Invalid shop action: {subject:?} due to {reason:?}")]
    InvalidShopAction {
        /// Subject of invalid shop action.
        subject: String,
        /// Failure reason.
        reason: String,
    },

    /// Failed to parse wiki page.
    #[error("Failed to parse: {subject:?} due to {reason:?}")]
    ParserFailure {
        /// Subject of wiki failure.
        subject: String,
        /// Reason for failure.
        reason: String,
    },

    /// Fallible action used in trigger_effects. Never invoked otherwise.
    #[error("Fallible action.")]
    FallibleAction,
}
