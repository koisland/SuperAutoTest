use serde::{Deserialize, Serialize};

use crate::{db::record::ToyRecord, error::SAPTestError, Effect};

use super::names::ToyName;

/// A Super Auto Pets toy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Toy {
    /// Name of toy.
    name: ToyName,
    /// Tier of toy.
    tier: usize,
    /// Level of toy.
    lvl: usize,
    /// Duration of toy effect.
    duration: Option<usize>,
    /// Effect of toy.
    effect: Vec<Effect>,
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
