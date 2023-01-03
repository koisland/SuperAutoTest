use log::error;
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct WikiParserError {
    pub reason: String,
}

impl Display for WikiParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error!(target: "wiki_parser", "{}", self.reason);
        write!(f, "Failed to parse SAP wiki: {}", self.reason)
    }
}

impl Error for WikiParserError {}
