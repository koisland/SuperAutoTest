use crate::{
    effects::{
        actions::{Action, GainType, StatChangeType},
        state::Target,
        trigger::{TRIGGER_ANY_FAINT, TRIGGER_START_BATTLE},
    },
    Effect, FoodName, ItemCondition, Position, Statistics,
};

#[allow(missing_docs)]
/// Possible Hard Mode Options
pub enum HardModeOption {
    Boomerang,
    DodgeBall,
    BowlingBall,
    Glasses,
    ActionFigure,
    Handkerchief,
    PogoStick,
    Pen,
    PillBottle,
    RingPyramid,
    RubberDuck,
    Unicycle,
}

impl From<HardModeOption> for Effect {
    fn from(value: HardModeOption) -> Self {
        match value {
            HardModeOption::Boomerang => Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Any(ItemCondition::Healthiest),
                action: Action::Multiple(vec![
                    Action::Remove(StatChangeType::SetStatistics(
                        Statistics {
                            attack: 30,
                            health: 0
                        }
                    ));
                    2
                ]),
                uses: Some(1),
                temp: true,
            },
            HardModeOption::DodgeBall => Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Any(ItemCondition::Illest),
                action: Action::Multiple(vec![
                    Action::Remove(StatChangeType::SetStatistics(
                        Statistics {
                            attack: 30,
                            health: 0
                        }
                    ));
                    2
                ]),
                uses: Some(1),
                temp: true,
            },
            HardModeOption::BowlingBall => Effect {
                owner: None,
                trigger: TRIGGER_ANY_FAINT,
                target: Target::Friend,
                position: Position::Nearest(-1),
                action: Action::Remove(StatChangeType::SetStatistics(Statistics {
                    attack: 3,
                    health: 0,
                })),
                uses: None,
                temp: true,
            },
            HardModeOption::Glasses => Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::All(ItemCondition::None),
                action: Action::Set(StatChangeType::SetHealth(5)),
                uses: Some(1),
                temp: true,
            },
            HardModeOption::ActionFigure => Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Multiple(vec![Position::First, Position::Last]),
                action: Action::Gain(GainType::DefaultItem(FoodName::Coconut)),
                uses: Some(1),
                temp: true,
            },
            // HardModeOption::Handkerchief => Effect {
            //     owner: None,
            //     trigger: TRIGGER_START_BATTLE,
            //     target: Target::Friend,
            //     position: Position::Range(0..=-5), // TODO: Add Position::FrontToBack
            //     action: Action::Gain(GainType::DefaultItem(FoodName::Weak)),
            //     uses: Some(1),
            //     temp: true,
            // },
            HardModeOption::PogoStick => Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::N(ItemCondition::Illest, 1, false),
                action: Action::Set(StatChangeType::SelfMultStatistics(Statistics {
                    attack: 400,
                    health: 400,
                })),
                uses: Some(1),
                temp: true,
            },
            HardModeOption::Pen => todo!(),
            HardModeOption::PillBottle => todo!(),
            HardModeOption::RingPyramid => todo!(),
            HardModeOption::RubberDuck => todo!(),
            HardModeOption::Unicycle => todo!(),
            _ => todo!(),
        }
    }
}
