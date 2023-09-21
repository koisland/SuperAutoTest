use serde::{Deserialize, Serialize};

use crate::{
    db::record::{SAPRecord, ToyRecord},
    error::SAPTestError,
    Effect, Entity, SAPQuery, SAPDB,
};

use super::names::ToyName;

/// A Super Auto Pets toy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Toy {
    /// Name of toy.
    pub name: ToyName,
    /// Tier of toy.
    pub(crate) tier: usize,
    /// Level of toy.
    pub(crate) lvl: usize,
    /// Duration of toy effect.
    pub duration: Option<usize>,
    /// Effect of toy.
    pub effect: Vec<Effect>,
}

impl Toy {
    /// Create a toy.
    ///
    /// ```
    /// use saptest::{Toy, ToyName};
    ///
    /// let toy = Toy::new(ToyName::Balloon, 1).unwrap();
    /// assert_eq!(toy.name, ToyName::Balloon);
    /// ```
    pub fn new(name: ToyName, lvl: usize) -> Result<Toy, SAPTestError> {
        let query = SAPQuery::builder()
            .set_table(Entity::Toy)
            .set_param("name", vec![&name])
            .set_param("lvl", vec![lvl]);

        if let Some(SAPRecord::Toy(record)) = SAPDB
            .execute_query(query)
            .map(|records| records.into_iter().next())?
        {
            record.try_into()
        } else {
            Err(SAPTestError::QueryFailure {
                subject: String::from("No Toy Record"),
                reason: format!("Toy {name} at level {lvl} did not yield a ToyRecord."),
            })
        }
    }
}

impl TryFrom<ToyRecord> for Toy {
    type Error = SAPTestError;

    fn try_from(record: ToyRecord) -> Result<Toy, SAPTestError> {
        let (name, tier, lvl) = (record.name.clone(), record.tier, record.lvl);
        Ok(Toy {
            name,
            tier,
            lvl,
            duration: Some(2),
            effect: record.try_into()?,
        })
    }
}

impl TryFrom<ToyName> for Toy {
    type Error = SAPTestError;

    fn try_from(name: ToyName) -> Result<Self, Self::Error> {
        let query = SAPQuery::builder()
            .set_table(Entity::Toy)
            .set_param("name", vec![&name]);

        if let Some(SAPRecord::Toy(record)) = SAPDB
            .execute_query(query)
            .map(|records| records.into_iter().next())?
        {
            record.try_into()
        } else {
            Err(SAPTestError::QueryFailure {
                subject: String::from("No Toy Record"),
                reason: format!("Toy {name} did not yield a ToyRecord."),
            })
        }
    }
}
