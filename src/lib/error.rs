use thiserror::Error;

#[derive(Error, Debug)]
pub enum SAPTestError {
    #[error("Invalid team action: {subject:?} ({indices:?}) due to {reason:?}")]
    InvalidTeamAction {
        subject: String,
        indices: Vec<usize>,
        reason: String,
    },
    #[error("Failed to parse: {subject:?} due to {reason:?}")]
    ParserFailure { subject: String, reason: String },
    #[error("Failed to convert name: {subject:?} due to {reason:?}")]
    NameFailure { subject: String, reason: String },
    #[error("Unknown error.")]
    Unknown,
}
