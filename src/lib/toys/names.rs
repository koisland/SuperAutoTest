use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::error::SAPTestError;

#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToyName {
    Balloon,
    TennisBall,
    Radio,
    GarlicPress,
    ToiletPaper,
    OvenMitts,
    MelonHelmet,
    FoamSword,
    ToyGun,
    Flashlight,
    StinkySock,
    Television,
    PeanutJar,
    AirPalmTree,
    Boomerang,
    DiceCup,
    Dodgeball,
    Handkerchief,
    Pen,
    PogoStick,
    RockBag,
    Scissors,
    SpinningTop,
    Unicycle,
    YoYo,
    ActionFigure,
    Dice,
    OpenPiggyBank,
    RubberDuck,
    BowlingBall,
    Glasses,
    Lunchbox,
    PaperShredder,
    Spring,
    CardboardBox,
    Trampoline,
    Boot,
    PillBottle,
    RingPyramid,
    RockingHorse,
    StuffedBear,
    ToyMouse,
    Custom(String),
}

impl Display for ToyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToyName::Balloon => write!(f, "Balloon"),
            ToyName::TennisBall => write!(f, "Tennis Ball"),
            ToyName::Radio => write!(f, "Radio"),
            ToyName::GarlicPress => write!(f, "Garlic Press"),
            ToyName::ToiletPaper => write!(f, "Toilet Paper"),
            ToyName::OvenMitts => write!(f, "Oven Mitts"),
            ToyName::MelonHelmet => write!(f, "Melon Helmet"),
            ToyName::FoamSword => write!(f, "Foam Sword"),
            ToyName::ToyGun => write!(f, "Toy Gun"),
            ToyName::Flashlight => write!(f, "Flashlight"),
            ToyName::StinkySock => write!(f, "Stinky Sock"),
            ToyName::Television => write!(f, "Television"),
            ToyName::PeanutJar => write!(f, "Peanut Jar"),
            ToyName::AirPalmTree => write!(f, "Air Palm Tree"),
            ToyName::Boomerang => write!(f, "Boomerang"),
            ToyName::DiceCup => write!(f, "Dice Cup"),
            ToyName::Dodgeball => write!(f, "Dodgeball"),
            ToyName::Handkerchief => write!(f, "Handkerchief"),
            ToyName::Pen => write!(f, "Pen"),
            ToyName::PogoStick => write!(f, "Pogo Stick"),
            ToyName::RockBag => write!(f, "Rock Bag"),
            ToyName::Scissors => write!(f, "Scissors"),
            ToyName::SpinningTop => write!(f, "Spinning Top"),
            ToyName::Unicycle => write!(f, "Unicycle"),
            ToyName::YoYo => write!(f, "Yo Yo"),
            ToyName::ActionFigure => write!(f, "Action Figure"),
            ToyName::Dice => write!(f, "Dice"),
            ToyName::OpenPiggyBank => write!(f, "Open Piggy Bank"),
            ToyName::RubberDuck => write!(f, "Rubber Duck"),
            ToyName::BowlingBall => write!(f, "Bowling Ball"),
            ToyName::Glasses => write!(f, "Glasses"),
            ToyName::Lunchbox => write!(f, "Lunchbox"),
            ToyName::PaperShredder => write!(f, "Paper Shredder"),
            ToyName::Spring => write!(f, "Spring"),
            ToyName::CardboardBox => write!(f, "Cardboard Box"),
            ToyName::Trampoline => write!(f, "Trampoline"),
            ToyName::Boot => write!(f, "Boot"),
            ToyName::PillBottle => write!(f, "Pill Bottle"),
            ToyName::RingPyramid => write!(f, "Ring Pyramid"),
            ToyName::RockingHorse => write!(f, "Rocking Horse"),
            ToyName::StuffedBear => write!(f, "Stuffed Bear"),
            ToyName::ToyMouse => write!(f, "Toy Mouse"),
            ToyName::Custom(toy) => write!(f, "{toy}"),
        }
    }
}

impl FromStr for ToyName {
    type Err = SAPTestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Balloon" => ToyName::Balloon,
            "Tennis Ball" => ToyName::TennisBall,
            "Radio" => ToyName::Radio,
            "Garlic Press" => ToyName::GarlicPress,
            "Toilet Paper" => ToyName::ToiletPaper,
            "Oven Mitts" => ToyName::OvenMitts,
            "Melon Helmet" => ToyName::MelonHelmet,
            "Foam Sword" => ToyName::FoamSword,
            "Toy Gun" => ToyName::ToyGun,
            "Flashlight" => ToyName::Flashlight,
            "Stinky Sock" => ToyName::StinkySock,
            "Television" => ToyName::Television,
            "Peanut Jar" => ToyName::PeanutJar,
            "Air Palm Tree" => ToyName::AirPalmTree,
            "Boomerang" => ToyName::Boomerang,
            "Dice Cup" => ToyName::DiceCup,
            "Dodgeball" => ToyName::Dodgeball,
            "Handkerchief" => ToyName::Handkerchief,
            "Pen" => ToyName::Pen,
            "Pogo Stick" => ToyName::PogoStick,
            "Rock Bag" => ToyName::RockBag,
            "Scissors" => ToyName::Scissors,
            "Spinning Top" => ToyName::SpinningTop,
            "Unicycle" => ToyName::Unicycle,
            "Yo Yo" => ToyName::YoYo,
            "Action Figure" => ToyName::ActionFigure,
            "Dice" => ToyName::Dice,
            "Open Piggy Bank" => ToyName::OpenPiggyBank,
            "Rubber Duck" => ToyName::RubberDuck,
            "Bowling Ball" => ToyName::BowlingBall,
            "Glasses" => ToyName::Glasses,
            "Lunchbox" => ToyName::Lunchbox,
            "Paper Shredder" => ToyName::PaperShredder,
            "Spring" => ToyName::Spring,
            "Cardboard Box" => ToyName::CardboardBox,
            "Trampoline" => ToyName::Trampoline,
            "Boot" => ToyName::Boot,
            "Pill Bottle" => ToyName::PillBottle,
            "Ring Pyramid" => ToyName::RingPyramid,
            "Rocking Horse" => ToyName::RockingHorse,
            "Stuffed Bear" => ToyName::StuffedBear,
            "Toy Mouse" => ToyName::ToyMouse,
            _ => ToyName::Custom(s.to_owned()),
        })
    }
}
