use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use crate::error::SAPTestError;

/// Packs in Super Auto Pets.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum Pack {
    /// The [Turtle pack](https://superautopets.fandom.com/wiki/Turtle_Pack).
    Turtle,
    /// The [Puppy pack](https://superautopets.fandom.com/wiki/Puppy_Pack).
    Puppy,
    /// The [Star pack](https://superautopets.fandom.com/wiki/Star_Pack).
    Star,
    /// The [Weekly pack](https://superautopets.fandom.com/wiki/Weekly_Pack)
    Weekly,
    /// The [Golden pack]().
    Golden,
    /// An unknown pack. Indicates a typo or a new update.
    Unknown,
}

impl FromStr for Pack {
    type Err = SAPTestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // let capitalized_s = capitalize_names(s);
        match s {
            "Turtle" => Ok(Pack::Turtle),
            "Puppy" => Ok(Pack::Puppy),
            "Star" => Ok(Pack::Star),
            "Weekly" => Ok(Pack::Weekly),
            "Golden" => Ok(Pack::Golden),
            _ => Ok(Pack::Unknown),
        }
    }
}

impl fmt::Display for Pack {
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pack::Turtle => write!(f, "Turtle"),
            Pack::Puppy => write!(f, "Puppy"),
            Pack::Star => write!(f, "Star"),
            Pack::Weekly => write!(f, "Weekly"),
            Pack::Golden => write!(f, "Golden"),
            Pack::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Pack;
    use std::str::FromStr;

    #[test]
    fn test_str_to_pack() {
        assert_eq!(Pack::Turtle, Pack::from_str("Turtle").unwrap());
        assert_ne!(Pack::Turtle, Pack::from_str("TURTLE").unwrap());
        assert_eq!(Pack::Unknown, Pack::from_str("camel").unwrap());
    }
}