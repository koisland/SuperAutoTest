use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct TeamError {
    pub reason: String,
}

impl Display for TeamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Team Error: {}", self.reason)
    }
}

impl Error for TeamError {}
