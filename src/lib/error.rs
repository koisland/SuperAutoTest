//! Error types for library.

use thiserror::Error;

/// Error types.
#[derive(Error, Debug)]
pub enum SAPTestError {
    #[error("Shop error")]
    ShopError {
        // Failure reason.
        reason: String,
    },

    /// Failure to serialize team.
    #[error("Failed to serialize/deserialize team.")]
    SerializeFailure(#[from] serde_json::Error),

    /// Failure to request data from Fandom wiki.
    #[error("Failed to scrape data from Fandom wiki.")]
    RequestFailure(#[from] reqwest::Error),

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

    /// Failed to parse wiki page.
    #[error("Failed to parse: {subject:?} due to {reason:?}")]
    ParserFailure {
        /// Subject of wiki failure.
        subject: String,
        /// Reason for failure.
        reason: String,
    },
}
