use crate::{
    battle::{
        effect::{Effect, Entity},
        state::{Action, Condition, Position, Statistics, Target},
        trigger::*,
    },
    db::record::FoodRecord,
    foods::names::FoodName,
    pets::{
        names::PetName,
        pet::{Pet, MAX_PET_STATS, MIN_PET_STATS},
    },
};

impl From<&FoodRecord> for Effect {
    fn from(record: &FoodRecord) -> Self {
        let effect_stats = Statistics::new(
            record.effect_atk.clamp(MIN_PET_STATS, MAX_PET_STATS),
            record.effect_health.clamp(MIN_PET_STATS, MAX_PET_STATS),
        );
        let uses = record.single_use.then_some(1);

        match record.name {
            FoodName::Chili => Effect {
                target: Target::Enemy,
                // Next enemy relative to current pet position.
                position: Position::Relative(-1),
                action: Action::Remove(effect_stats),
                uses,
                entity: Entity::Food,
                trigger: TRIGGER_SELF_ATTACK,
                temp: record.end_of_battle,
            },
            FoodName::Coconut => Effect {
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Invincible,
                uses,
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                temp: record.end_of_battle,
            },
            FoodName::Garlic | FoodName::Lemon => Effect {
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Negate(effect_stats),
                uses,
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                temp: record.end_of_battle,
            },
            FoodName::Honey => {
                let bee = Box::new(Pet::new(PetName::Bee, None, Some(effect_stats), 1).unwrap());
                Effect {
                    target: Target::Friend,
                    position: Position::Trigger,
                    action: Action::Summon(Some(bee), None),
                    uses,
                    entity: Entity::Food,
                    trigger: TRIGGER_SELF_FAINT,
                    temp: record.end_of_battle,
                }
            }
            FoodName::MeatBone => Effect {
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(effect_stats),
                uses,
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                temp: record.end_of_battle,
            },
            FoodName::Melon => Effect {
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Negate(effect_stats),
                uses,
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                temp: record.end_of_battle,
            },
            FoodName::Mushroom => Effect {
                target: Target::Friend,
                position: Position::Trigger,
                // Replace during runtime.
                action: Action::Summon(
                    None,
                    Some(Statistics {
                        attack: 1,
                        health: 1,
                    }),
                ),
                uses,
                entity: Entity::Food,
                trigger: TRIGGER_SELF_FAINT,
                temp: record.end_of_battle,
            },
            FoodName::Peanuts => Effect {
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Kill,
                uses,
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                temp: record.end_of_battle,
            },
            FoodName::Steak => Effect {
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(effect_stats),
                uses,
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                temp: record.end_of_battle,
            },
            FoodName::Weakness => Effect {
                entity: Entity::Food,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Remove(effect_stats),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::SleepingPill => Effect {
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Kill,
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Croissant | FoodName::Cucumber | FoodName::Carrot => Effect {
                entity: Entity::Food,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(effect_stats),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Grapes => Effect {
                entity: Entity::Food,
                trigger: TRIGGER_START_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Profit,
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Chocolate => Effect {
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Experience,
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Pepper => Effect {
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Endure,
                uses,
                temp: record.end_of_battle,
            },
            FoodName::CannedFood => Effect {
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                target: Target::Shop,
                position: Position::All(Condition::None),
                action: Action::Add(effect_stats),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::FortuneCookie => Effect {
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Critical(50),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Cheese => Effect {
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Critical(100),
                uses,
                temp: record.end_of_battle,
            },
            // FoodName::Pineapple => Effect {
            //     entity: Entity::Food,
            //     trigger: TRIGGER_NONE,
            //     target: Target::Enemy,
            //     position: Position::OnSelf,
            //     action: Action::Add(effect_stats),
            //     uses: None,
            //     temp: record.end_of_battle,
            // },
            FoodName::SaladBowl
            | FoodName::Sushi
            | FoodName::Stew
            | FoodName::Taco
            | FoodName::Pizza
            | FoodName::SoftIce
            | FoodName::HotDog
            | FoodName::Orange => {
                let actions = vec![Action::Add(effect_stats); record.n_targets];
                Effect {
                    entity: Entity::Food,
                    trigger: TRIGGER_NONE,
                    target: Target::Friend,
                    position: Position::Any(Condition::None),
                    action: Action::Multiple(actions),
                    uses,
                    temp: record.end_of_battle,
                }
            }
            FoodName::Apple
            | FoodName::Bacon
            | FoodName::Cookie
            | FoodName::Broccoli
            | FoodName::FriedShrimp
            | FoodName::Cupcake
            | FoodName::Peach
            | FoodName::ChickenLeg => Effect {
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(effect_stats),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Strawberry => Effect {
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                target: Target::Friend,
                position: Position::None,
                action: Action::None,
                uses,
                temp: record.end_of_battle,
            },
            // TODO: Milk, popcorns, lollipop, strawberry
            _ => Effect {
                entity: Entity::Food,
                trigger: TRIGGER_NONE,
                target: Target::None,
                position: Position::None,
                action: Action::None,
                uses,
                temp: record.end_of_battle,
            },
        }
    }
}
