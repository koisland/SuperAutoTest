use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum FoodName {
    Chili,
    Coconut,
    Garlic,
    Honey,
    MeatBone,
    Melon,
    Mushroom,
    Peanuts,
    Steak,
}

impl std::fmt::Display for FoodName {
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FoodName::Chili => write!(f, "Chili"),
            FoodName::Coconut => write!(f, "Coconut"),
            FoodName::Garlic => write!(f, "Garlic"),
            FoodName::Honey => write!(f, "Honey"),
            FoodName::MeatBone => write!(f, "MeatBone"),
            FoodName::Melon => write!(f, "Melon"),
            FoodName::Mushroom => write!(f, "Mushroom"),
            FoodName::Peanuts => write!(f, "Peanuts"),
            FoodName::Steak => write!(f, "Steak"),
        }
    }
}
