use serde::{Deserialize, Serialize};

use crate::common::game::Pack;

#[derive(Debug, Serialize, Deserialize)]
pub struct Food {
    pub name: String,
    pub tier: usize,
    pub effect: String,
    pub pack: Pack,
}
