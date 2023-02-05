use thiserror::Error;

#[derive(Error, Debug)]
pub enum SAPTestError {
    #[error("Failed interaction with database.")]
    DatabaseFailure(#[from] rusqlite::Error),
    #[error("Failed Query: {subject:?} due to {reason:?}")]
    QueryFailure { subject: String, reason: String },
    #[error("Cannot convert statistics to isize.")]
    ValueConversionFailure(#[from] std::num::TryFromIntError),
    #[error("Invalid team action: {subject:?} ({indices:?}) due to {reason:?}")]
    InvalidTeamAction {
        subject: String,
        indices: Vec<usize>,
        reason: String,
    },
    #[error("Invalid pet action: {subject:?} due to {reason:?}")]
    InvalidPetAction { subject: String, reason: String },
    #[error("Failed to parse: {subject:?} due to {reason:?}")]
    ParserFailure { subject: String, reason: String },
    #[error("Failed to convert name: {subject:?} due to {reason:?}")]
    NameFailure { subject: String, reason: String },
    #[error("Unknown error.")]
    Unknown,
}