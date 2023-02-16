use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::SAPTestError;

#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
/// Names for [`Food`](crate::Food)s
pub enum FoodName {
    Apple,
    Bacon,
    Cookie,
    Peach,
    Strawberry,
    Cupcake,
    Croissant,
    Broccoli,
    FriedShrimp,
    SaladBowl,
    Pineapple,
    Cucumber,
    Lollipop,
    CannedFood,
    Pear,
    FortuneCookie,
    Cheese,
    Grapes,
    Chocolate,
    Sushi,
    Lemon,
    Carrot,
    Pepper,
    Stew,
    Taco,
    Pizza,
    ChickenLeg,
    SoftIce,
    HotDog,
    Orange,
    Popcorns,
    Chili,
    Coconut,
    Garlic,
    Honey,
    MeatBone,
    Melon,
    Mushroom,
    Milk,
    Peanut,
    Steak,
    Weak,
    SleepingPill,
    None,
    Custom(String),
}

impl Default for FoodName {
    fn default() -> Self {
        FoodName::Custom("CustomFood".to_string())
    }
}

impl FromStr for FoodName {
    type Err = SAPTestError;

    #[cfg(not(tarpaulin_include))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Chili" => Ok(FoodName::Chili),
            "Coconut" => Ok(FoodName::Coconut),
            "Garlic" => Ok(FoodName::Garlic),
            "Honey" => Ok(FoodName::Honey),
            "Meat Bone" => Ok(FoodName::MeatBone),
            "Melon" => Ok(FoodName::Melon),
            "Mushroom" => Ok(FoodName::Mushroom),
            "Peanut" => Ok(FoodName::Peanut),
            "Steak" => Ok(FoodName::Steak),
            "Weak" => Ok(FoodName::Weak),
            "Apple" => Ok(FoodName::Apple),
            "Bacon" => Ok(FoodName::Bacon),
            "Cookie" => Ok(FoodName::Cookie),
            "Peach" => Ok(FoodName::Peach),
            "Strawberry" => Ok(FoodName::Strawberry),
            "Cupcake" => Ok(FoodName::Cupcake),
            "Croissant" => Ok(FoodName::Croissant),
            "Broccoli" => Ok(FoodName::Broccoli),
            "Fried Shrimp" => Ok(FoodName::FriedShrimp),
            "Salad Bowl" => Ok(FoodName::SaladBowl),
            "Pineapple" => Ok(FoodName::Pineapple),
            "Cucumber" => Ok(FoodName::Cucumber),
            "Lollipop" => Ok(FoodName::Lollipop),
            "Canned Food" => Ok(FoodName::CannedFood),
            "Pear" => Ok(FoodName::Pear),
            "Fortune Cookie" => Ok(FoodName::FortuneCookie),
            "Cheese" => Ok(FoodName::Cheese),
            "Grapes" => Ok(FoodName::Grapes),
            "Chocolate" => Ok(FoodName::Chocolate),
            "Sushi" => Ok(FoodName::Sushi),
            "Lemon" => Ok(FoodName::Lemon),
            "Carrot" => Ok(FoodName::Carrot),
            "Pepper" => Ok(FoodName::Pepper),
            "Stew" => Ok(FoodName::Stew),
            "Taco" => Ok(FoodName::Taco),
            "Pizza" => Ok(FoodName::Pizza),
            "Chicken Leg" => Ok(FoodName::ChickenLeg),
            "Soft Ice" => Ok(FoodName::SoftIce),
            "Hot Dog" => Ok(FoodName::HotDog),
            "Orange" => Ok(FoodName::Orange),
            "Popcorns" => Ok(FoodName::Popcorns),
            "Milk" => Ok(FoodName::Milk),
            "Sleeping Pill" => Ok(FoodName::SleepingPill),
            "None" => Ok(FoodName::None),
            _ => Ok(FoodName::Custom(s.to_string())),
        }
    }
}

impl std::fmt::Display for FoodName {
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FoodName::Chili => write!(f, "Chili"),
            FoodName::Coconut => write!(f, "Coconut"),
            FoodName::Garlic => write!(f, "Garlic"),
            FoodName::Honey => write!(f, "Honey"),
            FoodName::MeatBone => write!(f, "Meat Bone"),
            FoodName::Melon => write!(f, "Melon"),
            FoodName::Mushroom => write!(f, "Mushroom"),
            FoodName::Peanut => write!(f, "Peanut"),
            FoodName::Steak => write!(f, "Steak"),
            FoodName::Weak => write!(f, "Weak"),
            FoodName::Apple => write!(f, "Apple"),
            FoodName::Bacon => write!(f, "Bacon"),
            FoodName::Cookie => write!(f, "Cookie"),
            FoodName::Peach => write!(f, "Peach"),
            FoodName::Strawberry => write!(f, "Strawberry"),
            FoodName::Cupcake => write!(f, "Cupcake"),
            FoodName::Croissant => write!(f, "Croissant"),
            FoodName::Broccoli => write!(f, "Broccoli"),
            FoodName::FriedShrimp => write!(f, "Fried Shrimp"),
            FoodName::SaladBowl => write!(f, "Salad Bowl"),
            FoodName::Pineapple => write!(f, "Pineapple"),
            FoodName::Cucumber => write!(f, "Cucumber"),
            FoodName::Lollipop => write!(f, "Lollipop"),
            FoodName::CannedFood => write!(f, "Canned Food"),
            FoodName::Pear => write!(f, "Pear"),
            FoodName::FortuneCookie => write!(f, "Fortune Cookie"),
            FoodName::Cheese => write!(f, "Cheese"),
            FoodName::Grapes => write!(f, "Grapes"),
            FoodName::Chocolate => write!(f, "Chocolate"),
            FoodName::Sushi => write!(f, "Sushi"),
            FoodName::Lemon => write!(f, "Lemon"),
            FoodName::Carrot => write!(f, "Carrot"),
            FoodName::Pepper => write!(f, "Pepper"),
            FoodName::Stew => write!(f, "Stew"),
            FoodName::Taco => write!(f, "Taco"),
            FoodName::Pizza => write!(f, "Pizza"),
            FoodName::ChickenLeg => write!(f, "Chicken Leg"),
            FoodName::SoftIce => write!(f, "Soft Ice"),
            FoodName::HotDog => write!(f, "Hot Dog"),
            FoodName::Orange => write!(f, "Orange"),
            FoodName::Popcorns => write!(f, "Popcorns"),
            FoodName::Milk => write!(f, "Milk"),
            FoodName::SleepingPill => write!(f, "Sleeping Pill"),
            FoodName::None => write!(f, "None"),
            FoodName::Custom(name) => write!(f, "{name}"),
        }
    }
}
