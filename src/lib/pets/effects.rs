use itertools::Itertools;

use crate::{
    db::record::PetRecord,
    effects::{
        actions::{
            Action, ConditionType, CopyType, GainType, LogicType, RandomizeType, StatChangeType,
            SummonType,
        },
        effect::{Effect, Entity, EntityName},
        state::{
            EqualityCondition, ItemCondition, Position, ShopCondition, Status, Target,
            TeamCondition,
        },
        trigger::*,
    },
    error::SAPTestError,
    foods::{food::Food, names::FoodName},
    shop::{store::ShopState, trigger::*},
    Pet, PetName, Statistics,
};
use std::convert::TryInto;

use super::pet::{MAX_PET_LEVEL, MIN_PET_LEVEL};

impl TryFrom<PetRecord> for Vec<Effect> {
    type Error = SAPTestError;

    fn try_from(record: PetRecord) -> Result<Self, Self::Error> {
        let effect_stats = Statistics::new(record.effect_atk, record.effect_health)?;

        Ok(match &record.name {
            PetName::Beaver => vec![Effect {
                entity: Entity::Pet,
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Friend,
                position: Position::N(ItemCondition::None, record.lvl, true),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Duck => vec![Effect {
                entity: Entity::Pet,
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Shop,
                position: Position::All(ItemCondition::None),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Fish => match record.lvl {
                1 | 2 => vec![Effect {
                    entity: Entity::Pet,
                    owner: None,
                    trigger: TRIGGER_SELF_LEVELUP,
                    target: Target::Friend,
                    position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }],
                _ => vec![],
            },
            PetName::Otter => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_BOUGHT,
                target: Target::Friend,
                position: Position::Multiple(vec![
                    Position::Any(ItemCondition::NotEqual(
                        EqualityCondition::IsSelf
                    ));
                    record.lvl
                ]),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Pig => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Shop,
                position: Position::None,
                action: Action::Multiple(vec![Action::Profit; record.lvl]),
                uses: Some(record.n_triggers),
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Chinchilla => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Multiple(vec![
                    Action::Summon(SummonType::DefaultPet(
                        PetName::LoyalChinchilla
                    ));
                    record.lvl
                ]),
                uses: Some(record.n_triggers),
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Marmoset => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Shop,
                position: Position::None,
                action: Action::Multiple(vec![Action::FreeRoll; record.lvl]),
                uses: Some(record.n_triggers),
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Beetle => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FOOD_EATEN,
                target: Target::Shop,
                position: Position::First,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Bluebird => vec![Effect {
                owner: None,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::Any(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Ladybug => vec![Effect {
                owner: None,
                trigger: TRIGGER_ANY_FOOD_BOUGHT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Cockroach => vec![Effect {
                owner: None,
                trigger: TRIGGER_START_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Cockroach,
                uses: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Duckling => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Shop,
                position: Position::First,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Kiwi => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Friend,
                position: Position::Any(ItemCondition::Equal(EqualityCondition::Name(
                    EntityName::Food(FoodName::Strawberry),
                ))),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Mouse => {
                let mut free_apple = Food::try_from(FoodName::Apple)?;
                free_apple.cost = 0;
                let mut actions =
                    vec![
                        Action::AddShopFood(GainType::StoredItem(Box::new(free_apple)));
                        record.lvl
                    ];
                actions.insert(0, Action::ClearShop(Entity::Food));
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_PET_SOLD,
                    target: Target::Shop,
                    position: Position::None,
                    action: Action::Multiple(actions),
                    uses: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Pillbug => vec![Effect {
                owner: None,
                trigger: TRIGGER_SHOP_TIER_UPGRADED,
                target: Target::Friend,
                position: Position::Nearest(-2),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Ant => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Any(ItemCondition::None),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Mosquito => vec![Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Any(ItemCondition::None),
                action: Action::Remove(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Cricket => {
                let zombie_cricket = Box::new(Pet::new(
                    PetName::ZombieCricket,
                    None,
                    Some(effect_stats),
                    record.lvl,
                )?);
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Summon(SummonType::StoredPet(zombie_cricket)),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Horse => vec![Effect {
                owner: None,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::TriggerAffected,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Crab => vec![Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Copy(
                    CopyType::PercentStats(effect_stats),
                    Target::Friend,
                    Position::N(ItemCondition::Healthiest, 1, false),
                ),
                uses: Some(record.n_triggers),
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Dodo => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::Nearest(1),
                    action: Action::Add(StatChangeType::SelfMultValue(effect_stats)),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Elephant => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_ATTACK,
                    target: Target::Friend,
                    position: Position::Nearest(-1),
                    action: Action::Multiple(vec![
                        Action::Remove(StatChangeType::StaticValue(
                            effect_stats
                        ));
                        record.n_triggers
                    ]),
                    uses: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Flamingo => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Nearest(-2),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Hedgehog => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Either,
                position: Position::All(ItemCondition::None),
                action: Action::Remove(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Peacock => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Rat => {
                let rats_summoned = vec![
                    Action::Summon(SummonType::StoredPet(Box::new(Pet::new(
                        PetName::DirtyRat,
                        None,
                        Some(effect_stats),
                        record.lvl,
                    )?)));
                    record.lvl
                ];
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Enemy,
                    position: Position::OnSelf,
                    action: Action::Multiple(rats_summoned),
                    // Activates multiple times per trigger.
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Shrimp => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_ANY_PET_SOLD,
                    target: Target::Friend,
                    position: Position::Any(ItemCondition::None),
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    // Activates multiple times per trigger.
                    uses: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Spider => {
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Summon(SummonType::QueryPet(
                        "SELECT * FROM pets WHERE lvl = ? AND tier = 3 AND pack = 'Turtle'"
                            .to_string(),
                        vec![record.lvl.to_string()],
                        None,
                    )),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Swan => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_START_TURN,
                    target: Target::Shop,
                    position: Position::None,
                    action: Action::Multiple(vec![Action::Profit; record.lvl]),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Frigatebird => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_PET_BOUGHT,
                    target: Target::Friend,
                    position: Position::Any(ItemCondition::Equal(EqualityCondition::Trigger(
                        Status::Hurt,
                    ))),
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::GoldFish => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_START_TURN,
                    target: Target::Shop,
                    position: Position::Multiple(vec![Position::First, Position::Relative(-1)]),
                    action: Action::Discount(Entity::Pet, record.lvl),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Dromedary => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_START_TURN,
                    target: Target::Shop,
                    position: Position::Multiple(vec![Position::First, Position::Relative(-1)]),
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::TabbyCat => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_FOOD_EATEN,
                    target: Target::Friend,
                    position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::GuineaPig => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_PET_BOUGHT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Summon(SummonType::SelfPet(effect_stats)),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Jellyfish => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_ANY_LEVELUP,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Salamander => {
                vec![Effect {
                    owner: None,
                    trigger: trigger_any_pet_bought_status(Status::StartOfBattle),
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Yak => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Multiple(vec![
                        // Damage is hardcoded as not captured by regex.
                        Action::Remove(StatChangeType::StaticValue(Statistics {
                            attack: 1,
                            health: 0,
                        })),
                        Action::Add(StatChangeType::StaticValue(effect_stats)),
                    ]),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Badger => {
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Either,
                    position: Position::Multiple(vec![
                        Position::Relative(1),
                        Position::Relative(-1),
                    ]),
                    action: Action::Remove(StatChangeType::SelfMultValue(effect_stats)),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Blowfish => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Enemy,
                position: Position::Any(ItemCondition::None),
                action: Action::Remove(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Camel => {
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_HURT,
                    target: Target::Friend,
                    position: Position::Nearest(-1),
                    action: Action::Multiple(vec![
                        Action::Add(StatChangeType::StaticValue(
                            effect_stats
                        ));
                        record.n_triggers
                    ]),
                    uses: None,
                }]
            }
            PetName::Dog => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Dolphin => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::N(ItemCondition::Illest, 1, false),
                action: Action::Remove(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Kangaroo => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_AHEAD_ATTACK,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Ox => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_AHEAD_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Multiple(vec![
                    Action::Add(StatChangeType::StaticValue(effect_stats)),
                    Action::Gain(GainType::DefaultItem(FoodName::Melon)),
                ]),
                uses: None,
            }],
            PetName::Giraffe => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::Relative(1),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Rabbit => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_FOOD_EATEN,
                target: Target::Friend,
                position: Position::TriggerAffected,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Snail => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_PET_BOUGHT,
                target: Target::Friend,
                position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Conditional(
                    LogicType::If(ConditionType::Team(TeamCondition::PreviousLoss)),
                    Box::new(Action::Add(StatChangeType::StaticValue(effect_stats))),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::EmperorTamarin => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Shop,
                position: Position::First,
                action: Action::Add(StatChangeType::SelfMultValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Wasp => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SHOP_TIER_UPGRADED,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::SelfMultValue(effect_stats)),
                uses: None,
            }],
            PetName::HatchingChick => {
                let mut base_effect = Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: Position::Nearest(1),
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                };
                match record.lvl {
                    1 | 2 => {}
                    3 => {
                        base_effect.trigger = TRIGGER_START_TURN;
                        base_effect.action = Action::Experience;
                    }
                    _ => {
                        return Err(SAPTestError::QueryFailure {
                            subject: "Invalid Pet Level".to_string(),
                            reason: format!("PetRecord for {} has an invalid level.", record.name),
                        })
                    }
                }
                vec![base_effect]
            }
            PetName::Owl => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Friend,
                position: Position::Any(ItemCondition::None),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Puppy => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::If(ConditionType::Shop(ShopCondition::GoldGreaterEqual(2))),
                    Box::new(Action::Add(StatChangeType::StaticValue(effect_stats))),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::TropicalFish => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::Adjacent,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Capybara => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ROLL,
                target: Target::Shop,
                position: Position::All(ItemCondition::NotEqual(EqualityCondition::Frozen)),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Cassowary => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::If(ConditionType::Pet(
                        Target::Friend,
                        ItemCondition::Equal(EqualityCondition::Name(EntityName::Food(
                            FoodName::Strawberry,
                        ))),
                    )),
                    Box::new(Action::Add(StatChangeType::StaticValue(effect_stats))),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Leech => vec![
                // Dmg is hardcoded as regex on captures health buff.
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: Position::Nearest(1),
                    action: Action::Remove(StatChangeType::StaticValue(Statistics {
                        attack: 1,
                        health: 0,
                    })),
                    uses: Some(record.n_triggers),
                },
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                },
            ],
            PetName::Okapi => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ROLL,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Starfish => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: trigger_any_pet_sold_status(Status::Sell),
                target: Target::Friend,
                position: Position::Any(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Sheep => {
                let ram = Box::new(Pet::new(
                    PetName::Ram,
                    None,
                    Some(effect_stats),
                    record.lvl,
                )?);
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Multiple(vec![
                        Action::Summon(SummonType::StoredPet(ram.clone())),
                        Action::Summon(SummonType::StoredPet(ram)),
                    ]),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Bison => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::IfAny(ConditionType::Pet(
                        Target::Friend,
                        ItemCondition::Equal(EqualityCondition::Level(3)),
                    )),
                    Box::new(Action::Add(StatChangeType::StaticValue(effect_stats))),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Penguin => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::N(
                    ItemCondition::Multiple(vec![
                        ItemCondition::Equal(EqualityCondition::Level(2)),
                        ItemCondition::Equal(EqualityCondition::Level(3)),
                    ]),
                    2,
                    true,
                ),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Squirrel => vec![
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_TURN,
                    target: Target::Shop,
                    position: Position::None,
                    action: Action::AddShopFood(GainType::RandomShopItem),
                    uses: Some(record.n_triggers),
                },
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_TURN,
                    target: Target::Shop,
                    position: Position::All(ItemCondition::None),
                    action: Action::Discount(Entity::Food, record.lvl),
                    uses: Some(record.n_triggers),
                },
            ],
            PetName::Worm => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FOOD_EATEN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Dragonfly => {
                let positions = Position::Multiple(
                    (MIN_PET_LEVEL..=MAX_PET_LEVEL)
                        .map(|lvl| {
                            Position::Any(ItemCondition::MultipleAll(vec![
                                ItemCondition::NotEqual(EqualityCondition::IsSelf),
                                ItemCondition::Equal(EqualityCondition::Level(lvl)),
                            ]))
                        })
                        .collect_vec(),
                );
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: positions,
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Jerboa => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: trigger_self_food_ate_name(FoodName::Apple),
                target: Target::Friend,
                position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Mole => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_PET_BOUGHT,
                target: Target::Friend,
                position: Position::Adjacent,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Buffalo => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_PET_BOUGHT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Llama => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::IfNot(ConditionType::Team(TeamCondition::OpenSpaceEqual(0))),
                    Box::new(Action::Add(StatChangeType::StaticValue(effect_stats))),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Lobster => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::TriggerAffected,
                action: Action::Conditional(
                    LogicType::If(ConditionType::Shop(ShopCondition::InState(ShopState::Open))),
                    Box::new(Action::Add(StatChangeType::StaticValue(effect_stats))),
                ),
                uses: None,
            }],
            PetName::Crow => {
                let mut free_chocolate = Food::try_from(FoodName::Chocolate)?;
                // Amount of experience tied to level of crow.
                free_chocolate.ability.action =
                    Action::Multiple(vec![Action::Experience; record.lvl]);
                // Clear shop, add chocolate, and then discount it.
                let actions = vec![
                    Action::ClearShop(Entity::Food),
                    Action::AddShopFood(GainType::StoredItem(Box::new(free_chocolate))),
                ];
                vec![
                    Effect {
                        owner: None,
                        trigger: TRIGGER_SELF_PET_SOLD,
                        target: Target::Shop,
                        position: Position::None,
                        action: Action::Multiple(actions),
                        uses: None,
                        entity: Entity::Pet,
                        temp: record.temp_effect,
                    },
                    Effect {
                        owner: None,
                        trigger: TRIGGER_SELF_PET_SOLD,
                        target: Target::Shop,
                        position: Position::First,
                        action: Action::Discount(Entity::Food, 3),
                        uses: None,
                        entity: Entity::Pet,
                        temp: record.temp_effect,
                    },
                ]
            }
            PetName::Orangutan => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::Any(ItemCondition::Illest),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Platypus => {
                let duck = Pet::new(PetName::Duck, None, None, record.lvl)?;
                let beaver = Pet::new(PetName::Beaver, None, None, record.lvl)?;

                let summon_actions = vec![
                    Action::Summon(SummonType::StoredPet(Box::new(beaver))),
                    Action::Summon(SummonType::StoredPet(Box::new(duck))),
                ];
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_PET_SOLD,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Multiple(summon_actions),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::PrayingMantis => vec![
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_TURN,
                    target: Target::Friend,
                    position: Position::Adjacent,
                    action: Action::Kill,
                    uses: Some(record.n_triggers),
                },
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_TURN,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                },
            ],
            PetName::Deer => {
                let mut bus = Pet::new(PetName::Bus, None, Some(effect_stats), record.lvl)?;
                bus.item = Some(Food::try_from(FoodName::Chili)?);
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Summon(SummonType::StoredPet(Box::new(bus))),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Hippo => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_KNOCKOUT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Parrot => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Copy(
                    CopyType::Effect(vec![], Some(record.lvl)),
                    Target::Friend,
                    Position::Nearest(1),
                ),
                uses: None,
            }],
            PetName::Rooster => {
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Multiple(vec![
                        Action::Summon(SummonType::CustomPet(
                            PetName::Chick,
                            StatChangeType::SelfMultValue(effect_stats),
                            record.lvl
                        ));
                        record.lvl
                    ]),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Skunk => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::N(ItemCondition::Healthiest, 1, false),
                action: Action::Debuff(effect_stats),
                uses: Some(record.n_triggers),
            }],
            PetName::Turtle => {
                let max_pets_behind: isize = record.lvl.try_into()?;
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::Nearest(-max_pets_behind),
                    action: Action::Gain(GainType::DefaultItem(FoodName::Melon)),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Whale => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Whale(record.lvl, Position::Nearest(1)),
                uses: Some(record.n_triggers),
            }],
            PetName::Crocodile => {
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Enemy,
                    position: Position::Last,
                    action: Action::Multiple(vec![
                        Action::Remove(StatChangeType::StaticValue(
                            effect_stats
                        ));
                        record.n_triggers
                    ]),
                    uses: Some(1),
                }]
            }
            PetName::Rhino => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_KNOCKOUT,
                target: Target::Enemy,
                position: Position::First,
                action: Action::Rhino(effect_stats),
                uses: None,
            }],
            // No shops so start of turn.
            PetName::Scorpion => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_SUMMON,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Gain(GainType::DefaultItem(FoodName::Peanut)),
                uses: None,
            }],
            PetName::Shark => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Turkey => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::TriggerAffected,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Boar => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_ATTACK,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Fly => {
                // Add exception for other zombie flies.
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_ANY_FAINT,
                    target: Target::Friend,
                    position: Position::TriggerAffected,
                    action: Action::Summon(SummonType::StoredPet(Box::new(Pet::new(
                        PetName::ZombieFly,
                        None,
                        Some(effect_stats),
                        record.lvl,
                    )?))),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Gorilla => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Gain(GainType::DefaultItem(FoodName::Coconut)),
                uses: Some(record.n_triggers),
            }],
            PetName::Leopard => {
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Enemy,
                    position: Position::Any(ItemCondition::None),
                    action: Action::Multiple(vec![
                        Action::Remove(StatChangeType::SelfMultValue(
                            effect_stats
                        ));
                        record.n_triggers
                    ]),
                    uses: Some(1),
                }]
            }
            PetName::Mammoth => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::All(ItemCondition::None),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Snake => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_AHEAD_ATTACK,
                target: Target::Enemy,
                position: Position::Any(ItemCondition::None),
                action: Action::Remove(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],

            PetName::FrilledDragon => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::ForEach(ConditionType::Pet(
                        Target::Friend,
                        ItemCondition::Equal(EqualityCondition::Trigger(Status::Faint)),
                    )),
                    Box::new(Action::Add(StatChangeType::StaticValue(effect_stats))),
                ),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            // Only level one for now.
            PetName::Frog => {
                let mut effects = vec![];
                let mut base_effect = Effect {
                    owner: None,
                    entity: Entity::Pet,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::Adjacent,
                    action: Action::Swap(RandomizeType::Stats),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                };
                match record.lvl {
                    1 => {}
                    2 => {
                        base_effect.trigger = TRIGGER_SELF_PET_SOLD;
                    }
                    3 => {
                        base_effect.trigger = TRIGGER_END_TURN;
                    }
                    _ => {
                        return Err(SAPTestError::QueryFailure {
                            subject: "Invalid Pet Level".to_string(),
                            reason: format!("PetRecord for {} has an invalid level.", record.name),
                        })
                    }
                }
                effects.push(base_effect);
                effects
            }
            PetName::Hummingbird => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Any(ItemCondition::Equal(EqualityCondition::Name(
                    EntityName::Food(FoodName::Strawberry),
                ))),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            // Iguana has two effects that are the same except for their triggers.
            PetName::Iguana => vec![
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    trigger: TRIGGER_ANY_ENEMY_SUMMON,
                    target: Target::Enemy,
                    position: Position::TriggerAffected,
                    action: Action::Remove(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                },
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    trigger: TRIGGER_ANY_ENEMY_PUSHED,
                    target: Target::Enemy,
                    position: Position::TriggerAffected,
                    action: Action::Remove(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                },
            ],
            PetName::Moth => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::First,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Seahorse => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Last,
                action: Action::Push(Position::Relative(record.lvl.try_into()?)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Bat => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Any(ItemCondition::None),
                action: Action::Gain(GainType::DefaultItem(FoodName::Weak)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::AtlanticPuffin => {
                // For each level, do an action that removes some amount of stats based on the number of enemies with strawberries.
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Enemy,
                    position: Position::Any(ItemCondition::None),
                    action: Action::Multiple(vec![
                        Action::Conditional(
                            LogicType::ForEach(ConditionType::Pet(
                                // Note: The reason it is enemy is because all actions act on the team being affected.
                                Target::Enemy,
                                ItemCondition::Equal(EqualityCondition::Name(EntityName::Food(
                                    FoodName::Strawberry
                                )))
                            )),
                            Box::new(Action::Remove(StatChangeType::StaticValue(effect_stats))),
                        );
                        record.lvl
                    ]),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            PetName::Dove => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::N(
                    ItemCondition::Equal(EqualityCondition::Name(EntityName::Food(
                        FoodName::Strawberry,
                    ))),
                    2,
                    true,
                ),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Koala => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_ANY_HURT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Panda => {
                vec![
                    Effect {
                        owner: None,
                        entity: Entity::Pet,
                        trigger: TRIGGER_START_BATTLE,
                        target: Target::Friend,
                        position: Position::Nearest(1),
                        action: Action::Add(StatChangeType::SelfMultValue(effect_stats)),
                        uses: Some(record.n_triggers),
                        temp: record.temp_effect,
                    },
                    Effect {
                        owner: None,
                        entity: Entity::Pet,
                        trigger: TRIGGER_START_BATTLE,
                        target: Target::Friend,
                        position: Position::OnSelf,
                        action: Action::Kill,
                        uses: Some(record.n_triggers),
                        temp: record.temp_effect,
                    },
                ]
            }
            PetName::Pug => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Relative(1),
                action: Action::Multiple(vec![Action::Experience; record.lvl]),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Stork => {
                // TODO: Not fully functional as needs team tier num.
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Summon(SummonType::QueryPet(
                        "SELECT * FROM pets WHERE tier = ? AND pack = 'Star' AND lvl = ?"
                            .to_string(),
                        vec![(record.tier - 1).to_string(), record.lvl.to_string()],
                        None,
                    )),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Racoon => vec![
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_BEFORE_ATTACK,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Copy(CopyType::Item(None), Target::Enemy, Position::First),
                    uses: Some(record.n_triggers),
                },
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_BEFORE_ATTACK,
                    target: Target::Enemy,
                    position: Position::First,
                    action: Action::Gain(GainType::NoItem),
                    uses: Some(record.n_triggers),
                },
            ],
            PetName::Toucan => {
                let n_pets_behind: isize = record.lvl.try_into()?;
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::Nearest(-n_pets_behind),
                    // If None, update during team init with current item.
                    action: Action::Gain(GainType::SelfItem),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Wombat => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Copy(
                    CopyType::Effect(vec![], Some(record.lvl)),
                    Target::Enemy,
                    Position::N(
                        ItemCondition::MultipleAll(vec![
                            ItemCondition::HighestTier,
                            ItemCondition::Equal(EqualityCondition::Trigger(Status::Faint)),
                        ]),
                        1,
                        false,
                    ),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Aardvark => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_ENEMY_SUMMON,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Bear => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Either,
                position: Position::Multiple(vec![Position::Relative(-1), Position::Relative(1)]),
                action: Action::Gain(GainType::DefaultItem(FoodName::Honey)),
                uses: Some(record.n_triggers),
            }],
            PetName::Seagull => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::TriggerAffected,
                // Give currently held food.
                action: Action::Gain(GainType::SelfItem),
                uses: Some(record.n_triggers),
            }],
            PetName::Blobfish => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Nearest(-1),
                action: Action::Experience,
                uses: Some(record.n_triggers),
            }],
            PetName::Clownfish => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_LEVELUP,
                target: Target::Friend,
                position: Position::TriggerAffected,
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Toad => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_ENEMY_HURT,
                target: Target::Enemy,
                position: Position::TriggerAffected,
                action: Action::Gain(GainType::DefaultItem(FoodName::Weak)),
                uses: Some(record.n_triggers),
            }],
            PetName::Woodpecker => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Either,
                position: Position::Nearest(2),
                action: Action::Multiple(vec![
                    Action::Remove(StatChangeType::StaticValue(
                        effect_stats
                    ));
                    record.lvl
                ]),
                uses: Some(record.n_triggers),
            }],
            PetName::Armadillo => vec![
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                },
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_HURT,
                    target: Target::Friend,
                    position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                },
            ],
            PetName::Doberman => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::If(ConditionType::Pet(
                        Target::Friend,
                        ItemCondition::LowestTier,
                    )),
                    Box::new(Action::Multiple(vec![
                        Action::Gain(GainType::DefaultItem(FoodName::Coconut)),
                        Action::Add(StatChangeType::StaticValue(effect_stats)),
                    ])),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Lynx => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Any(ItemCondition::None),
                action: Action::Multiple(vec![Action::Lynx; record.lvl]),
                uses: Some(record.n_triggers),
            }],
            PetName::Porcupine => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Either,
                position: Position::TriggerAfflicting,
                action: Action::Remove(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Caterpillar => match record.lvl {
                1 | 2 => vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_TURN,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Experience,
                    uses: None,
                }],
                3 => {
                    vec![Effect {
                        owner: None,
                        entity: Entity::Pet,
                        temp: record.temp_effect,
                        trigger: TRIGGER_START_BATTLE,
                        target: Target::Friend,
                        position: Position::OnSelf,
                        action: Action::Transform(
                            PetName::Butterfly,
                            Some(Statistics {
                                attack: 1,
                                health: 1,
                            }),
                            record.lvl,
                        ),
                        uses: Some(record.n_triggers),
                    }]
                }
                _ => vec![],
            },
            PetName::Butterfly => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_BEFORE_FIRST_BATTLE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Copy(
                    CopyType::Stats(None),
                    Target::Friend,
                    Position::N(ItemCondition::Strongest, 1, false),
                ),
                uses: Some(1),
            }],
            PetName::Anteater => {
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Multiple(vec![
                        Action::Summon(SummonType::StoredPet(Box::new(
                            Pet::new(PetName::Ant, None, None, record.lvl)?
                        )));
                        2
                    ]),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Donkey => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_ENEMY_FAINT,
                target: Target::Enemy,
                position: Position::Last,
                action: Action::Push(Position::First),
                uses: Some(record.n_triggers),
            }],
            PetName::Eel => {
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Add(StatChangeType::SelfMultValue(effect_stats)),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Hawk => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Opposite,
                action: Action::Remove(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Pelican => {
                let start_battle_effect = Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::Any(ItemCondition::Equal(EqualityCondition::Name(
                        EntityName::Food(FoodName::Strawberry),
                    ))),
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: Some(record.n_triggers),
                };
                let mut start_turn_effect = start_battle_effect.clone();
                start_turn_effect.trigger = TRIGGER_START_TURN;
                vec![start_turn_effect, start_battle_effect]
            }
            PetName::Hyena => {
                let mut first_effect = Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Either,
                    position: Position::All(ItemCondition::None),
                    action: Action::None,
                    uses: Some(record.n_triggers),
                };
                match record.lvl {
                    1 => {
                        first_effect.action = Action::Shuffle(RandomizeType::Stats);
                        vec![first_effect]
                    }
                    2 => {
                        first_effect.action = Action::Shuffle(RandomizeType::Positions);
                        vec![first_effect]
                    }
                    3 => {
                        let mut second_effect = first_effect.clone();
                        first_effect.action = Action::Shuffle(RandomizeType::Stats);
                        second_effect.action = Action::Shuffle(RandomizeType::Positions);
                        vec![first_effect, second_effect]
                    }
                    _ => {
                        return Err(SAPTestError::QueryFailure {
                            subject: "Invalid Pet Level".to_string(),
                            reason: format!("PetRecord for {} has an invalid level.", record.name),
                        })
                    }
                }
            }
            PetName::Lionfish => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_BEFORE_ATTACK,
                target: Target::Enemy,
                position: Position::First,
                action: Action::Gain(GainType::DefaultItem(FoodName::Weak)),
                uses: None,
            }],
            PetName::Eagle => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Summon(SummonType::QueryPet(
                    "SELECT * FROM pets WHERE tier = ? AND pack = 'Puppy' AND lvl = ?".to_string(),
                    vec![6.to_string(), record.lvl.to_string()],
                    None,
                )),
                uses: Some(record.n_triggers),
            }],
            PetName::Microbe => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Either,
                position: Position::All(ItemCondition::None),
                action: Action::Gain(GainType::DefaultItem(FoodName::Weak)),
                uses: Some(record.n_triggers),
            }],
            PetName::Lion => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::If(ConditionType::Pet(
                        Target::Friend,
                        ItemCondition::HighestTier,
                    )),
                    Box::new(Action::Add(StatChangeType::SelfMultValue(effect_stats))),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Swordfish => {
                let self_dmg_effect = Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Remove(StatChangeType::SelfMultValue(effect_stats)),
                    uses: Some(record.n_triggers),
                };
                let mut enemy_dmg_effect = self_dmg_effect.clone();
                enemy_dmg_effect.target = Target::Enemy;
                enemy_dmg_effect.position = Position::N(ItemCondition::Healthiest, 1, false);
                vec![self_dmg_effect, enemy_dmg_effect]
            }
            PetName::Triceratops => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Friend,
                position: Position::Any(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Vulture => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_FAINT,
                target: Target::Enemy,
                position: Position::Any(ItemCondition::None),
                action: Action::Vulture(effect_stats),
                uses: None,
            }],
            PetName::Alpaca => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::Any(ItemCondition::NotEqual(EqualityCondition::Name(
                    EntityName::Pet(PetName::Alpaca),
                ))),
                action: Action::Multiple(vec![Action::Experience; record.lvl]),
                uses: None,
            }],
            PetName::Tapir => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Tapir,
                uses: Some(record.n_triggers),
            }],
            PetName::Walrus => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::N(ItemCondition::None, record.lvl, true),
                action: Action::Gain(GainType::DefaultItem(FoodName::Peanut)),
                uses: Some(record.n_triggers),
            }],
            PetName::WhiteTiger => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Nearest(-record.lvl.try_into()?),
                action: Action::Multiple(vec![Action::Experience; 3]),
                uses: Some(record.n_triggers),
            }],
            PetName::Octopus => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_BEFORE_ATTACK,
                target: Target::Enemy,
                position: Position::Multiple(vec![Position::Any(ItemCondition::None); record.lvl]),
                action: Action::Remove(StatChangeType::StaticValue(effect_stats)),
                uses: None,
            }],
            PetName::Orca => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Multiple(vec![
                    Action::Summon(SummonType::QueryPet(
                        "SELECT * FROM pets WHERE effect_trigger = ? AND lvl = ?".to_string(),
                        vec!["Faint".to_string(), 1.to_string()],
                        None
                    ));
                    record.lvl
                ]),
                uses: Some(record.n_triggers),
            }],
            PetName::Piranha => vec![
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::All(ItemCondition::None),
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: None,
                },
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_HURT,
                    target: Target::Friend,
                    position: Position::All(ItemCondition::None),
                    action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                    uses: None,
                },
            ],
            PetName::Reindeer => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_BEFORE_ATTACK,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Gain(GainType::DefaultItem(FoodName::Melon)),
                uses: Some(record.n_triggers),
            }],
            PetName::SabertoothTiger => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Summon(SummonType::QueryPet(
                    "SELECT * FROM pets WHERE lvl = ? AND tier = ? AND name != ?".to_string(),
                    vec![1.to_string(), 1.to_string(), "Sloth".to_string()],
                    Some(effect_stats),
                )),
                uses: None,
            }],
            PetName::Spinosaurus => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_FAINT,
                target: Target::Friend,
                position: Position::Any(ItemCondition::None),
                action: Action::Add(StatChangeType::StaticValue(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Stegosaurus => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Any(ItemCondition::MultipleAll(vec![
                    ItemCondition::Equal(EqualityCondition::Name(EntityName::Food(FoodName::None))),
                    ItemCondition::NotEqual(EqualityCondition::IsSelf),
                ])),
                action: Action::Stegosaurus(effect_stats),
                uses: Some(record.n_triggers),
            }],
            PetName::Velociraptor => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::N(
                    ItemCondition::Equal(EqualityCondition::Name(EntityName::Food(
                        FoodName::Strawberry,
                    ))),
                    record.lvl,
                    true,
                ),
                action: Action::Gain(GainType::DefaultItem(FoodName::Coconut)),
                uses: Some(record.n_triggers),
            }],
            _ => vec![],
        })
    }
}
