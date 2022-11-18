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

impl Pack {
    pub fn new(name: &str) -> Pack {
        match name {
            "Turtle" => Pack::Turtle,
            "Puppy" => Pack::Puppy,
            "Star" => Pack::Star,
            "Weekly" => Pack::Weekly,
            _ => Pack::Unknown,
        }
    }
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
