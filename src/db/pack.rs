use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use crate::{api::utils::capitalize_names, wiki_scraper::error::WikiParserError};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum Pack {
    Turtle,
    Puppy,
    Star,
    Weekly,
    Unknown,
}

impl FromStr for Pack {
    type Err = WikiParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let capitalized_s = capitalize_names(s);
        match &capitalized_s[..] {
            "Turtle" => Ok(Pack::Turtle),
            "Puppy" => Ok(Pack::Puppy),
            "Star" => Ok(Pack::Star),
            "Weekly" => Ok(Pack::Weekly),
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
        assert_eq!(Pack::Turtle, Pack::from_str("turtle").unwrap());
        assert_ne!(Pack::Turtle, Pack::from_str("TURTLE").unwrap());
        assert_eq!(Pack::Unknown, Pack::from_str("Golden").unwrap());
    }
}
