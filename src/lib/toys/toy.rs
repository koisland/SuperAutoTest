use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    db::record::ToyRecord,
    effects::{
        actions::{Action, GainType, StatChangeType},
        state::Target,
        trigger::Outcomes,
    },
    error::SAPTestError,
    Effect, FoodName, ItemCondition, Position, Statistics,
};

use super::names::ToyName;

/// A Super Auto Pets toy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Toy {
    /// Name of toy.
    name: ToyName,
    /// Tier of toy.
    tier: usize,
    /// Duration of toy effect.
    duration: Option<usize>,
    effect: Vec<Effect>,
}

impl TryInto<Vec<Effect>> for ToyRecord {
    type Error = SAPTestError;

    fn try_into(self) -> Result<Vec<Effect>, Self::Error> {
        let outcomes = Outcomes::from_str(&self.effect_trigger.unwrap_or_default())?;
        let effect_stats = Statistics::new(self.effect_atk, self.effect_health)?;
        let toy_lvl = TryInto::<isize>::try_into(self.lvl)?;

        Ok(outcomes
            .iter()
            .map(|outcome| {
                // Base effect get created with each outcome for Toy triggers.
                let mut base_effect = Effect {
                    owner: None,
                    trigger: outcome.clone(),
                    target: Target::Friend,
                    position: Position::First,
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(self.n_triggers),
                    temp: self.temp_effect,
                };
                match self.name {
                    ToyName::Balloon => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::First;
                        base_effect.action = Action::Add(StatChangeType::Static(effect_stats));
                    }
                    ToyName::TennisBall => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::N {
                            condition: ItemCondition::None,
                            targets: 2,
                            random: true,
                        };
                        base_effect.action = Action::Remove(StatChangeType::Static(effect_stats));
                    }
                    ToyName::Radio => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::All(ItemCondition::None);
                        base_effect.action = Action::Add(StatChangeType::Static(effect_stats));
                    }
                    ToyName::GarlicPress => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::First;
                        base_effect.action = Action::Gain(GainType::DefaultItem(FoodName::Garlic));
                    }
                    ToyName::ToiletPaper => {
                        base_effect.target = Target::Enemy;
                        // Use toy level to get number of positions relative to first enemy pet.
                        base_effect.position =
                            Position::Multiple((0..-toy_lvl).map(Position::Relative).collect());
                        base_effect.action = Action::Gain(GainType::DefaultItem(FoodName::Weak));
                    }
                    ToyName::OvenMitts => {
                        // TODO: Add lasagna
                        base_effect.target = Target::Shop;
                        base_effect.position = Position::None;
                        base_effect.action = Action::Multiple(vec![
                            Action::AddShopFood(
                                GainType::DefaultItem(FoodName::Custom("Lasagnas".to_owned()))
                            );
                            self.lvl
                        ]);
                    }
                    ToyName::MelonHelmet => {
                        base_effect.target = Target::Friend;
                        // Use toy level to get number of positions relative to first enemy pet.
                        base_effect.position =
                            Position::Multiple((0..-toy_lvl).map(Position::Relative).collect());
                        base_effect.action = Action::Gain(GainType::DefaultItem(FoodName::Melon));
                    }
                    ToyName::FoamSword => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::N {
                            condition: ItemCondition::Weakest,
                            targets: 1,
                            random: false,
                        };
                        base_effect.action = Action::Multiple(vec![
                            Action::Remove(
                                StatChangeType::Static(effect_stats)
                            );
                            self.n_triggers
                        ]);
                    }
                    ToyName::ToyGun => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::Last;
                        base_effect.action = Action::Multiple(vec![
                            Action::Remove(
                                StatChangeType::Static(effect_stats)
                            );
                            self.n_triggers
                        ]);
                    }
                    ToyName::Flashlight => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::First;
                        base_effect.action = Action::Add(StatChangeType::Static(effect_stats));
                    }
                    ToyName::StinkySock => {
                        // TODO: Syntax of effect differs from Skunk? Dunno if dev or user? Hard-code in meantime.
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::N {
                            condition: ItemCondition::Healthiest,
                            targets: 1,
                            random: false,
                        };
                        base_effect.action =
                            Action::Debuff(StatChangeType::Multiplier(Statistics {
                                attack: 40,
                                health: 40,
                            }));
                    }
                    ToyName::Television => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::All(ItemCondition::None);
                        base_effect.action = Action::Add(StatChangeType::Static(effect_stats));
                    }
                    ToyName::PeanutJar => {
                        base_effect.target = Target::Friend;
                        base_effect.position =
                            Position::Multiple((0..-toy_lvl).map(Position::Relative).collect());
                        base_effect.action = Action::Gain(GainType::DefaultItem(FoodName::Peanut));
                    }
                    ToyName::AirPalmTree => {
                        base_effect.target = Target::Friend;
                        base_effect.position =
                            Position::Multiple((0..-toy_lvl).map(Position::Relative).collect());
                        base_effect.action = Action::Gain(GainType::DefaultItem(FoodName::Coconut));
                    }
                    // Hard mode.
                    ToyName::Boomerang => todo!(),
                    ToyName::DiceCup => todo!(),
                    ToyName::Dodgeball => todo!(),
                    ToyName::Handerkerchief => todo!(),
                    ToyName::Pen => todo!(),
                    ToyName::PogoStick => todo!(),
                    ToyName::RockBag => todo!(),
                    ToyName::Scissors => todo!(),
                    ToyName::SpinningTop => todo!(),
                    ToyName::Unicycle => todo!(),
                    ToyName::YoYo => todo!(),
                    ToyName::ActionFigure => todo!(),
                    ToyName::Dice => todo!(),
                    ToyName::OpenPiggyBank => todo!(),
                    ToyName::RubberDuck => todo!(),
                    ToyName::BowlingBall => todo!(),
                    ToyName::Glasses => todo!(),
                    ToyName::Lunchbox => todo!(),
                    ToyName::PaperShredder => todo!(),
                    ToyName::Spring => todo!(),
                    ToyName::CardboardBox => todo!(),
                    ToyName::Trampoline => todo!(),
                    ToyName::Boot => todo!(),
                    ToyName::PillBottle => todo!(),
                    ToyName::RingPyramid => todo!(),
                    ToyName::RockingHorse => todo!(),
                    ToyName::StuffedBear => todo!(),
                    ToyName::ToyMouse => todo!(),
                    ToyName::Custom(_) => todo!(),
                }
                base_effect
            })
            .collect())
    }
}
