//! ### Into `JSON`
//! Convert a team into JSON.
//! ```
//! use saptest::{Pet, PetName, Team, error::SAPTestError};
//! let team = Team::new(&[Some(Pet::try_from(PetName::Ant).unwrap())], 5).unwrap();
//! let json_team: Result<String, SAPTestError> = (&team).try_into();
//! assert!(json_team.is_ok());
//! ```
//!
//! ### From `JSON`
//! Create a team from JSON.
//! ```
//! use std::str::FromStr;
//! use saptest::{Pet, PetName, Team};
//! let team = Team::new(&[Some(Pet::try_from(PetName::Ant).unwrap())], 5).unwrap();
//! let json_team: String = (&team).try_into().unwrap();
//! assert!(Team::from_str(&json_team).is_ok());
//! ```

use crate::{error::SAPTestError, Team};
use std::str::FromStr;

impl TryFrom<&Team> for String {
    type Error = SAPTestError;

    fn try_from(team: &Team) -> Result<Self, Self::Error> {
        serde_json::to_string(team).map_err(Into::into)
    }
}

impl FromStr for Team {
    type Err = SAPTestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut new_team: Team = serde_json::from_str(s)?;
        new_team.reset_pet_references(None);
        new_team.shop.restock()?;
        Ok(new_team)
    }
}
