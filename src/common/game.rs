use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Pack {
    Turtle,
    Puppy,
    Star,
    Weekly,
    Unknown,
}
