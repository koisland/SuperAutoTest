use std::str::FromStr;

use crate::{
    db::record::ToyRecord,
    effects::{
        actions::{
            Action, ConditionType, GainType, LogicType, RandomizeType, StatChangeType, SummonType,
        },
        state::{FrontToBackCondition, ShopCondition, Target, TeamCondition},
        trigger::Outcomes,
    },
    error::SAPTestError,
    Effect, FoodName, ItemCondition, PetName, Position, Statistics,
};

use super::names::ToyName;

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
                    trigger: outcome.clone(),
                    uses: Some(self.n_triggers),
                    temp: self.temp_effect,
                    ..Default::default()
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
                        base_effect.position = Position::Range(-(toy_lvl - 1)..=0);
                        base_effect.action = Action::Gain(GainType::DefaultItem(FoodName::Garlic));
                    }
                    ToyName::ToiletPaper => {
                        base_effect.target = Target::Enemy;
                        // Use toy level to get number of positions relative to first enemy pet.
                        base_effect.position = Position::Range(-(toy_lvl - 1)..=0);
                        base_effect.action = Action::Gain(GainType::DefaultItem(FoodName::Weak));
                    }
                    ToyName::OvenMitts => {
                        base_effect.target = Target::Shop;
                        base_effect.position = Position::None;
                        base_effect.action = Action::Multiple(vec![
                            Action::AddShopFood(
                                GainType::DefaultItem(FoodName::Lasagna)
                            );
                            self.lvl
                        ]);
                    }
                    ToyName::MelonHelmet => {
                        base_effect.target = Target::Friend;
                        // Use toy level to get number of positions relative to first enemy pet.
                        base_effect.position = Position::Range(-(toy_lvl - 1)..=0);
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
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::N {
                            condition: ItemCondition::Healthiest,
                            targets: 1,
                            random: false,
                        };
                        base_effect.action =
                            Action::Debuff(StatChangeType::Multiplier(effect_stats));
                    }
                    ToyName::Television => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::All(ItemCondition::None);
                        base_effect.action = Action::Add(StatChangeType::Static(effect_stats));
                    }
                    ToyName::PeanutJar => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::Range(-(toy_lvl - 1)..=0);
                        base_effect.action = Action::Gain(GainType::DefaultItem(FoodName::Peanut));
                    }
                    ToyName::AirPalmTree => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::Range(-(toy_lvl - 1)..=0);
                        base_effect.action = Action::Gain(GainType::DefaultItem(FoodName::Coconut));
                    }
                    // Hard mode.
                    ToyName::Boomerang => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::N {
                            condition: ItemCondition::Healthiest,
                            targets: 1,
                            random: false,
                        };
                        base_effect.action = Action::Remove(StatChangeType::Static(effect_stats))
                    }
                    ToyName::DiceCup => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::All(ItemCondition::None);
                        base_effect.action = Action::Shuffle(RandomizeType::Positions)
                    }
                    ToyName::Dodgeball => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::N {
                            condition: ItemCondition::Illest,
                            targets: 1,
                            random: false,
                        };
                        base_effect.action = Action::Multiple(vec![
                            Action::Remove(
                                StatChangeType::Static(effect_stats)
                            );
                            self.n_triggers
                        ])
                    }
                    ToyName::Handkerchief => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::FrontToBack(FrontToBackCondition::Shop(
                            ShopCondition::Tier(None),
                        ));
                        base_effect.action = Action::Gain(GainType::DefaultItem(FoodName::Weak))
                    }
                    ToyName::Pen => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::FrontToBack(FrontToBackCondition::Shop(
                            ShopCondition::Tier(None),
                        ));
                        base_effect.action = Action::Gain(GainType::DefaultItem(FoodName::Ink))
                    }
                    ToyName::PogoStick => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::N {
                            condition: ItemCondition::Illest,
                            targets: 1,
                            random: false,
                        };
                        base_effect.action = Action::Set(StatChangeType::Multiplier(Statistics {
                            attack: 400,
                            health: 400,
                        }))
                    }
                    ToyName::RockBag => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::Any(ItemCondition::None);
                        base_effect.action = Action::Conditional(
                            LogicType::ForEach(ConditionType::Team(
                                Target::Friend,
                                TeamCondition::NumberTurns(None),
                            )),
                            Box::new(Action::Remove(StatChangeType::Static(effect_stats))),
                            Box::new(Action::None),
                        );
                        base_effect.uses = None
                    }
                    ToyName::Scissors => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::N {
                            condition: ItemCondition::Healthiest,
                            targets: 2,
                            random: false,
                        };
                        base_effect.action = Action::Set(StatChangeType::StaticHealth(1))
                    }
                    ToyName::SpinningTop => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::Any(ItemCondition::None);
                        base_effect.action = Action::Remove(StatChangeType::Static(effect_stats))
                    }
                    ToyName::Unicycle => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::FrontToBack(FrontToBackCondition::Team(
                            TeamCondition::NumberTurns(None),
                        ));
                        base_effect.action = Action::Add(StatChangeType::Static(effect_stats))
                    }
                    ToyName::YoYo => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::All(ItemCondition::None);
                        base_effect.action = Action::Kill
                    }
                    ToyName::ActionFigure => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::FrontToBack(FrontToBackCondition::Shop(
                            ShopCondition::TierMultiple(2),
                        ));
                        base_effect.action = Action::Gain(GainType::DefaultItem(FoodName::Coconut))
                    }
                    ToyName::Dice | ToyName::OpenPiggyBank => {
                        base_effect.target = Target::Shop;
                        base_effect.position = Position::None;
                        base_effect.action = Action::AlterGold(-1)
                    }
                    ToyName::RubberDuck => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::First;
                        base_effect.action = Action::Summon(SummonType::CustomPet(
                            PetName::Duck,
                            StatChangeType::Static(effect_stats),
                            1,
                        ))
                    }
                    ToyName::BowlingBall => {
                        base_effect.target = Target::Friend;
                        // This might be incorrect behavior. If a pet at the center is knocked out.
                        base_effect.position = Position::Relative(-1);
                        base_effect.action = Action::Remove(StatChangeType::Static(effect_stats))
                    }
                    ToyName::Glasses => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::All(ItemCondition::None);
                        base_effect.action = Action::Set(StatChangeType::StaticHealth(5))
                    }
                    ToyName::Lunchbox => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::First;
                        base_effect.action =
                            Action::Multiple(vec![
                                Action::Summon(SummonType::CustomPet(
                                    PetName::Ant,
                                    StatChangeType::Static(effect_stats),
                                    1
                                ));
                                2
                            ])
                    }
                    ToyName::PaperShredder => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::TriggerAffected;
                        base_effect.action = Action::Kill;
                    }
                    ToyName::Spring => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::First;
                        base_effect.action = Action::Summon(SummonType::CustomPet(
                            PetName::Dog,
                            StatChangeType::Static(effect_stats),
                            1,
                        ))
                    }
                    ToyName::CardboardBox => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::First;
                        base_effect.action =
                            Action::Multiple(vec![
                                Action::Summon(SummonType::CustomPet(
                                    PetName::Scorpion,
                                    StatChangeType::Static(effect_stats),
                                    1
                                ));
                                2
                            ])
                    }
                    ToyName::Trampoline => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::First;
                        base_effect.action = Action::Summon(SummonType::CustomPet(
                            PetName::Kangaroo,
                            StatChangeType::Static(effect_stats),
                            1,
                        ))
                    }
                    ToyName::Boot => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::Last;
                        base_effect.action = Action::Remove(StatChangeType::Static(effect_stats));
                    }
                    ToyName::PillBottle => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::First;
                        base_effect.action = Action::Kill;
                    }
                    ToyName::RingPyramid => {
                        base_effect.target = Target::Friend;
                        base_effect.position = Position::All(ItemCondition::None);
                        base_effect.action = Action::Remove(StatChangeType::Static(effect_stats));
                    }
                    ToyName::RockingHorse => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::First;
                        base_effect.action =
                            Action::Multiple(vec![
                                Action::Summon(SummonType::CustomPet(
                                    PetName::Horse,
                                    StatChangeType::Static(effect_stats),
                                    1
                                ));
                                3
                            ])
                    }
                    ToyName::StuffedBear => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::First;
                        base_effect.action = Action::Summon(SummonType::CustomPet(
                            PetName::Bear,
                            StatChangeType::Static(effect_stats),
                            1,
                        ))
                    }
                    ToyName::ToyMouse => {
                        base_effect.target = Target::Enemy;
                        base_effect.position = Position::First;
                        // TODO: Felines should be a field in the db?
                        base_effect.action = Action::None;
                    }
                    ToyName::Custom(_) => todo!(),
                }
                base_effect
            })
            .collect())
    }
}
