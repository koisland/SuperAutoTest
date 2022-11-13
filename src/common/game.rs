use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Pack {
    Turtle,
    Puppy,
    Star,
    Weekly,
    Unknown,
}

impl fmt::Display for Pack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pack::Turtle => write!(f, "Turtle"),
            Pack::Puppy => write!(f, "Puppy"),
            Pack::Star => write!(f, "Star"),
            Pack::Weekly => write!(f, "Weekly"),
            Pack::Unknown => write!(f, "Unknown"),
        }
    }
}
