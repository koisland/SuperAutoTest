use itertools::Itertools;

use crate::{
    db::record::PetRecord,
    effects::{
        actions::{
            Action, ConditionType, CopyType, GainType, LogicType, RandomizeType, StatChangeType,
            SummonType, ToyType,
        },
        effect::{Effect, Entity, EntityName},
        state::{
            CondOrdering, EqualityCondition, ItemCondition, Position, ShopCondition, Status,
            Target, TeamCondition,
        },
        trigger::*,
    },
    error::SAPTestError,
    foods::{food::Food, names::FoodName},
    shop::{
        store::{ShopState, MAX_SHOP_TIER, MIN_SHOP_TIER},
        trigger::*,
    },
    teams::team::TeamFightOutcome,
    Pet, PetName, SAPQuery, Statistics,
};
use std::convert::TryInto;

use super::pet::{MAX_PET_LEVEL, MIN_PET_LEVEL};

impl TryFrom<PetRecord> for Vec<Effect> {
    type Error = SAPTestError;

    fn try_from(record: PetRecord) -> Result<Self, Self::Error> {
        let effect_stats = Statistics::new(record.effect_atk, record.effect_health)?;

        Ok(match &record.name {
            PetName::Beaver => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Friend,
                position: Position::N {
                    condition: ItemCondition::None,
                    targets: record.lvl,
                    random: true,
                    exact_n_targets: false,
                },
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Duck => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Shop,
                position: Position::All(ItemCondition::None),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Fish => match record.lvl {
                1 | 2 => vec![Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_LEVELUP,
                    target: Target::Friend,
                    position: Position::N {
                        condition: ItemCondition::NotEqual(EqualityCondition::IsSelf),
                        targets: 2,
                        random: true,
                        exact_n_targets: false,
                    },
                    action: Action::Add(StatChangeType::Static(effect_stats)),
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
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::Pig => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Shop,
                position: Position::None,
                action: Action::AlterGold(record.lvl.try_into()?),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Chinchilla => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Summon(SummonType::CustomPet(
                    PetName::LoyalChinchilla,
                    StatChangeType::Static(effect_stats),
                    record.lvl,
                )),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Marmoset => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Shop,
                position: Position::None,
                action: Action::FreeRoll(record.lvl),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Beetle => vec![{
                let food = match record.lvl {
                    1 => Food::try_from(FoodName::Honey)?,
                    2 => Food::try_from(FoodName::MeatBone)?,
                    3 => Food::try_from(FoodName::Garlic)?,
                    _ => unreachable!(),
                };
                Effect {
                    owner: None,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Gain(GainType::StoredItem(Box::new(food))),
                    uses: None,
                    temp: record.temp_effect,
                }
            }],
            PetName::Bluebird => vec![Effect {
                owner: None,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::Any(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::Ladybug => vec![Effect {
                owner: None,
                trigger: TRIGGER_ANY_GAIN_PERK,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::Cockroach => vec![Effect {
                owner: None,
                trigger: TRIGGER_START_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Cockroach,
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::Duckling => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Shop,
                position: Position::First,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::Kiwi => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Friend,
                position: Position::Any(ItemCondition::Equal(EqualityCondition::Name(
                    EntityName::Food(FoodName::Strawberry),
                ))),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::Mouse => {
                let mut free_apple = Food::try_from(FoodName::Apple)?;
                free_apple.cost = 0;
                if let Action::Add(stat_change_type) = &free_apple.ability.action {
                    let add_stats = stat_change_type.to_stats(None, None, false)?;
                    // Multiple by current level to create better apples.
                    let new_apple_stats = add_stats * Statistics::new(record.lvl, record.lvl)?;
                    free_apple.ability.action =
                        Action::Add(StatChangeType::Static(new_apple_stats));
                };
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_PET_SOLD,
                    target: Target::Shop,
                    position: Position::None,
                    action: Action::Multiple(vec![
                        Action::ClearShop(Entity::Food),
                        Action::AddShopFood(GainType::StoredItem(Box::new(free_apple))),
                    ]),
                    uses: None,
                    temp: record.temp_effect,
                }]
            }
            PetName::Pillbug => vec![Effect {
                owner: None,
                trigger: TRIGGER_SHOP_TIER_UPGRADED,
                target: Target::Friend,
                position: Position::Nearest(-2),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::Ant => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Any(ItemCondition::None),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Mosquito => vec![Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::N {
                    condition: ItemCondition::None,
                    targets: record.lvl,
                    random: true,
                    exact_n_targets: false,
                },
                action: Action::Remove(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Cricket => {
                let zombie_cricket = Box::new(Pet::new(
                    PetName::ZombieCricket,
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
                    temp: record.temp_effect,
                }]
            }
            PetName::Horse => vec![Effect {
                owner: None,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::TriggerAffected(None),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::Bulldog => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_AFTER_ATTACK,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Multiple(vec![
                    Action::Set(StatChangeType::CurrentHealth),
                    Action::Add(StatChangeType::StaticAttack(record.lvl.try_into()?)),
                ]),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::Chipmunk => vec![
                Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_PET_SOLD,
                    target: Target::Shop,
                    position: Position::None,
                    action: Action::ClearShop(Entity::Food),
                    uses: None,
                    temp: record.temp_effect,
                },
                Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_PET_SOLD,
                    target: Target::Shop,
                    position: Position::None,
                    action: Action::Multiple(vec![
                        Action::AddShopFood(GainType::SelfItem);
                        record.lvl
                    ]),
                    uses: None,
                    temp: record.temp_effect,
                },
                Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_PET_SOLD,
                    target: Target::Shop,
                    position: Position::None,
                    action: Action::Discount(Entity::Food, 2),
                    uses: None,
                    temp: record.temp_effect,
                },
            ],
            PetName::Groundhog => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                // Must have a position to activate effect.
                position: Position::TriggerAffected(None),
                action: Action::AddToCounter(String::from("Trumpets"), record.lvl.try_into()?),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::ConeSnail => vec![Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Nearest(-1),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::Goose => vec![Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::First,
                action: Action::Debuff(StatChangeType::Static(effect_stats)),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::PiedTamarin => vec![
                // Ranged attack.
                Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Enemy,
                    position: Position::N {
                        condition: ItemCondition::None,
                        targets: record.lvl,
                        random: true,
                        exact_n_targets: false,
                    },
                    action: Action::Conditional(
                        LogicType::If(ConditionType::Shop(ShopCondition::InState(
                            ShopState::Closed,
                        ))),
                        Box::new(Action::Conditional(
                            LogicType::IfNot(ConditionType::Team(
                                Target::Enemy,
                                TeamCondition::Counter(
                                    String::from("Trumpets"),
                                    Some(CondOrdering::Equal(0)),
                                ),
                            )),
                            Box::new(Action::Remove(StatChangeType::Static(effect_stats))),
                            Box::new(Action::None),
                        )),
                        Box::new(Action::None),
                    ),
                    uses: None,
                    temp: record.temp_effect,
                },
                // Decrement trumpets.
                Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::TriggerAffected(None),
                    action: Action::Conditional(
                        LogicType::If(ConditionType::Shop(ShopCondition::InState(
                            ShopState::Closed,
                        ))),
                        Box::new(Action::Conditional(
                            LogicType::IfNot(ConditionType::Team(
                                Target::Friend,
                                TeamCondition::Counter(
                                    String::from("Trumpets"),
                                    Some(CondOrdering::Equal(0)),
                                ),
                            )),
                            Box::new(Action::AddToCounter(String::from("Trumpets"), -1)),
                            Box::new(Action::None),
                        )),
                        Box::new(Action::None),
                    ),
                    uses: None,
                    temp: record.temp_effect,
                },
            ],
            PetName::Opossum => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Shop,
                position: Position::Any(ItemCondition::Equal(EqualityCondition::Trigger(
                    Status::Faint,
                ))),
                action: Action::AddShopStats(effect_stats),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::Silkmoth => vec![Effect {
                owner: None,
                trigger: TRIGGER_AHEAD_HURT,
                target: Target::Friend,
                position: Position::Nearest(1),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Magpie => vec![Effect {
                owner: None,
                trigger: TRIGGER_END_TURN,
                target: Target::Shop,
                position: Position::None,
                action: Action::SaveGold { limit: record.lvl },
                uses: Some(record.n_triggers),
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
                    Position::N {
                        condition: ItemCondition::Healthiest,
                        targets: 1,
                        random: false,
                        exact_n_targets: true,
                    },
                ),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Dodo => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::Nearest(1),
                    action: Action::Add(StatChangeType::Multiplier(effect_stats)),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            PetName::Elephant => {
                vec![
                    Effect {
                        owner: None,
                        trigger: TRIGGER_SELF_AFTER_ATTACK,
                        target: Target::Friend,
                        position: Position::Nearest(-1),
                        action: Action::Remove(StatChangeType::Static(effect_stats)),
                        uses: None,
                        temp: record.temp_effect,
                    };
                    record.n_triggers
                ]
            }
            PetName::Flamingo => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Nearest(-2),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Hedgehog => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Either,
                position: Position::All(ItemCondition::None),
                action: Action::Remove(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Peacock => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
                temp: record.temp_effect,
            }],
            PetName::Rat => {
                vec![
                    Effect {
                        owner: None,
                        trigger: TRIGGER_SELF_FAINT,
                        target: Target::Enemy,
                        // TODO: This isn't correct behavior.
                        position: Position::OnSelf,
                        action: Action::Summon(SummonType::StoredPet(Box::new(Pet::new(
                            PetName::DirtyRat,
                            Some(effect_stats),
                            record.lvl,
                        )?))),
                        // Activates multiple times per trigger.
                        uses: Some(record.n_triggers),
                        temp: record.temp_effect,
                    };
                    record.lvl
                ]
            }
            PetName::Shrimp => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_ANY_PET_SOLD,
                    target: Target::Friend,
                    position: Position::Any(ItemCondition::None),
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    // Activates multiple times per trigger.
                    uses: None,
                    temp: record.temp_effect,
                }]
            }
            PetName::Spider => {
                vec![Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Summon(SummonType::QueryPet(
                        SAPQuery::builder()
                            .set_table(Entity::Pet)
                            .set_param("lvl", vec![record.lvl])
                            .set_param("tier", vec![3]),
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
                    action: Action::AlterGold(record.lvl.try_into()?),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            PetName::Frigatebird => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_ANY_GAIN_AILMENT,
                    target: Target::Friend,
                    position: Position::TriggerAffected(None),
                    // Remove ailment.
                    action: Action::Gain(GainType::NoItem),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            PetName::GoldFish => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::AlterCost(record.lvl.try_into()?),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            PetName::Dromedary => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_START_TURN,
                    target: Target::Shop,
                    position: Position::Multiple(vec![Position::First, Position::Relative(-1)]),
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            PetName::TabbyCat => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_GAIN_PERK,
                    target: Target::Friend,
                    position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            PetName::GuineaPig => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_PET_BOUGHT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Summon(SummonType::SelfPet(Some(effect_stats), None, false)),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            PetName::Jellyfish => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_ANY_LEVELUP,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            PetName::Salamander => {
                vec![Effect {
                    owner: None,
                    trigger: trigger_any_pet_bought_status(Status::StartOfBattle),
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
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
                        Action::Remove(StatChangeType::Static(Statistics {
                            attack: 1,
                            health: 0,
                        })),
                        Action::Add(StatChangeType::Static(effect_stats)),
                    ]),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            PetName::Badger => {
                vec![Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Either,
                    position: Position::Multiple(vec![
                        Position::Relative(1),
                        Position::Relative(-1),
                    ]),
                    action: Action::Remove(StatChangeType::Multiplier(effect_stats)),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Blowfish => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Enemy,
                position: Position::Any(ItemCondition::None),
                action: Action::Remove(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Camel => {
                vec![Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_HURT,
                    target: Target::Friend,
                    position: Position::Nearest(-1),
                    action: Action::Multiple(vec![
                        Action::Add(StatChangeType::Static(effect_stats));
                        record.n_triggers
                    ]),
                    uses: None,
                }]
            }
            PetName::Dog => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Dolphin => vec![
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Enemy,
                    position: Position::N {
                        condition: ItemCondition::Illest,
                        targets: 1,
                        random: false,
                        exact_n_targets: true
                    },
                    action: Action::Remove(StatChangeType::Static(effect_stats)),
                    uses: Some(1),
                };
                record.n_triggers
            ],
            PetName::Kangaroo => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_AHEAD_ATTACK,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Ox => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_AHEAD_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Multiple(vec![
                    Action::Add(StatChangeType::Static(effect_stats)),
                    Action::Gain(GainType::DefaultItem(FoodName::Melon)),
                ]),
                uses: None,
            }],
            PetName::Giraffe => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::Relative(1),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Rabbit => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_FOOD_EATEN,
                target: Target::Friend,
                position: Position::TriggerAffected(None),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Snail => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Conditional(
                    LogicType::If(ConditionType::Team(
                        Target::Friend,
                        TeamCondition::PreviousBattle(TeamFightOutcome::Loss),
                    )),
                    Box::new(Action::Add(StatChangeType::Static(effect_stats))),
                    Box::new(Action::None),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::EmperorTamarin => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_PET_SOLD,
                target: Target::Shop,
                position: Position::First,
                action: Action::Add(StatChangeType::Multiplier(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Wasp => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SHOP_TIER_UPGRADED,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Multiplier(effect_stats)),
                uses: None,
            }],
            PetName::HatchingChick => {
                let mut base_effect = Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: Position::Nearest(1),
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                };
                match record.lvl {
                    1 | 2 => {}
                    3 => {
                        base_effect.trigger = TRIGGER_START_TURN;
                        base_effect.action = Action::Experience(1);
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
                temp: record.temp_effect,
                trigger: TRIGGER_START_TURN,
                target: Target::Friend,
                position: Position::Any(ItemCondition::None),
                action: Action::Summon(SummonType::StoredPet(Box::new({
                    Pet::new(PetName::Mouse, None, record.lvl)?
                }))),
                uses: Some(record.n_triggers),
            }],
            PetName::Ferret | PetName::Puppy => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_PET_BOUGHT,
                target: Target::Shop,
                position: Position::None,
                action: Action::GetToy(ToyType::QueryOneToy(
                    SAPQuery::builder()
                        .set_table(Entity::Toy)
                        .set_param("source", vec![record.name])
                        .set_param("lvl", vec![record.lvl])
                        .to_owned(),
                )),
                uses: Some(record.n_triggers),
            }],
            PetName::TropicalFish => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::Adjacent,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Capybara => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ROLL,
                target: Target::Shop,
                position: Position::All(ItemCondition::NotEqual(EqualityCondition::Frozen)),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Cassowary => vec![Effect {
                owner: None,
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
                    Box::new(Action::Add(StatChangeType::Static(effect_stats))),
                    Box::new(Action::None),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Leech => vec![
                // Dmg is hardcoded as regex on captures health buff.
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: Position::Nearest(1),
                    action: Action::Remove(StatChangeType::Static(Statistics {
                        attack: 1,
                        health: 0,
                    })),
                    uses: Some(record.n_triggers),
                },
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                },
            ],
            PetName::Okapi => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ROLL,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Starfish => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: trigger_any_pet_sold_status(Status::Sell),
                target: Target::Friend,
                position: Position::Any(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Sheep => {
                vec![
                    Effect {
                        owner: None,
                        temp: record.temp_effect,
                        trigger: TRIGGER_SELF_FAINT,
                        target: Target::Friend,
                        position: Position::OnSelf,
                        action: Action::Summon(SummonType::StoredPet(Box::new(Pet::new(
                            PetName::Ram,
                            Some(effect_stats),
                            record.lvl,
                        )?))),
                        uses: Some(record.n_triggers),
                    };
                    2
                ]
            }
            PetName::Bison => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::IfAny(ConditionType::Pet(
                        Target::Friend,
                        ItemCondition::Equal(EqualityCondition::Level(3)),
                    )),
                    Box::new(Action::Add(StatChangeType::Static(effect_stats))),
                    Box::new(Action::None),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Penguin => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::N {
                    condition: ItemCondition::Multiple(vec![
                        ItemCondition::Equal(EqualityCondition::Level(2)),
                        ItemCondition::Equal(EqualityCondition::Level(3)),
                    ]),
                    targets: 2,
                    random: true,
                    exact_n_targets: false,
                },
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Squirrel => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_START_TURN,
                target: Target::Shop,
                position: Position::All(ItemCondition::None),
                action: Action::Discount(Entity::Food, record.lvl),
                uses: Some(record.n_triggers),
            }],
            PetName::Worm => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_START_TURN,
                target: Target::Shop,
                position: Position::None,
                action: Action::AddShopFood(GainType::StoredItem(Box::new({
                    let mut apple = Food::try_from(FoodName::Apple)?;
                    // Replace apple action with buffed effect based on record level.
                    if let Action::Add(stat_change_type) = &apple.ability.action {
                        let add_stats = stat_change_type.to_stats(None, None, false)?;
                        // Multiple by current level to create better apples.
                        let new_apple_stats = add_stats * Statistics::new(record.lvl, record.lvl)?;
                        apple.ability.action = Action::Add(StatChangeType::Static(new_apple_stats));
                        // Apple is discounted.
                        apple.cost = 2;
                    };
                    apple
                }))),
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
                    temp: record.temp_effect,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: positions,
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Jerboa => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FOOD_EATEN,
                target: Target::Friend,
                position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Conditional(
                    LogicType::If(ConditionType::Trigger(
                        Entity::Food,
                        EqualityCondition::Name(EntityName::Food(FoodName::Apple)),
                    )),
                    Box::new(Action::Add(StatChangeType::Static(effect_stats))),
                    Box::new(Action::None),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Mole => {
                const REQ_PERK_PETS: usize = 3;
                vec![
                    Effect {
                        owner: None,
                        temp: record.temp_effect,
                        trigger: TRIGGER_SELF_FAINT,
                        target: Target::Friend,
                        position: Position::OnSelf,
                        action: Action::Conditional(
                            LogicType::If(ConditionType::Team(
                                Target::Friend,
                                TeamCondition::NumberPerkPets(Some(CondOrdering::Equal(
                                    REQ_PERK_PETS,
                                ))),
                            )),
                            Box::new(Action::Summon(SummonType::CustomPet(
                                record.name.clone(),
                                StatChangeType::Static(effect_stats),
                                1,
                            ))),
                            Box::new(Action::None),
                        ),
                        uses: Some(record.n_triggers),
                    },
                    Effect {
                        owner: None,
                        temp: record.temp_effect,
                        trigger: TRIGGER_SELF_FAINT,
                        target: Target::Friend,
                        position: Position::N {
                            condition: ItemCondition::Equal(EqualityCondition::HasPerk),
                            targets: REQ_PERK_PETS,
                            random: false,
                            exact_n_targets: true,
                        },
                        action: Action::Gain(GainType::NoItem),
                        uses: Some(record.n_triggers),
                    },
                ]
            }
            PetName::Buffalo => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_PET_SOLD,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::If(ConditionType::Shop(ShopCondition::NumberSoldMultiple(3))),
                    Box::new(Action::Add(StatChangeType::Static(effect_stats))),
                    Box::new(Action::None),
                ),
                uses: None,
            }],
            PetName::Llama => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::IfNot(ConditionType::Team(
                        Target::Friend,
                        TeamCondition::OpenSpace(Some(CondOrdering::Equal(0))),
                    )),
                    Box::new(Action::Add(StatChangeType::Static(effect_stats))),
                    Box::new(Action::None),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Lobster => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::TriggerAffected(None),
                action: Action::Conditional(
                    LogicType::If(ConditionType::Shop(ShopCondition::InState(ShopState::Open))),
                    Box::new(Action::Add(StatChangeType::Static(effect_stats))),
                    Box::new(Action::None),
                ),
                uses: None,
            }],
            PetName::Crow => {
                let mut free_chocolate = Food::try_from(FoodName::Chocolate)?;
                // Amount of experience tied to level of crow.
                free_chocolate.ability.action = Action::Experience(record.lvl);
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
                        temp: record.temp_effect,
                    },
                    Effect {
                        owner: None,
                        trigger: TRIGGER_SELF_PET_SOLD,
                        target: Target::Shop,
                        position: Position::First,
                        action: Action::Discount(Entity::Food, 3),
                        uses: None,
                        temp: record.temp_effect,
                    },
                ]
            }
            PetName::Orangutan => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::Any(ItemCondition::Illest),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Platypus => {
                let duck = Pet::new(PetName::Duck, None, record.lvl)?;
                let beaver = Pet::new(PetName::Beaver, None, record.lvl)?;

                let summon_actions = vec![
                    Action::Summon(SummonType::StoredPet(Box::new(beaver))),
                    Action::Summon(SummonType::StoredPet(Box::new(duck))),
                ];
                vec![Effect {
                    owner: None,

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
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_TURN,
                    target: Target::Friend,
                    position: Position::Adjacent,
                    action: Action::Kill,
                    uses: Some(record.n_triggers),
                },
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_TURN,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                },
            ],
            PetName::Deer => {
                let mut bus = Pet::new(PetName::Bus, Some(effect_stats), record.lvl)?;
                bus.item = Some(Food::try_from(FoodName::Chili)?);
                vec![Effect {
                    owner: None,
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
                temp: record.temp_effect,
                trigger: TRIGGER_KNOCKOUT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Parrot => vec![Effect {
                owner: None,
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
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Multiple(vec![
                        Action::Summon(SummonType::CustomPet(
                            PetName::Chick,
                            StatChangeType::Multiplier(effect_stats),
                            record.lvl
                        ));
                        record.lvl
                    ]),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Skunk => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::N {
                    condition: ItemCondition::Healthiest,
                    targets: 1,
                    random: false,
                    exact_n_targets: true,
                },
                action: Action::Debuff(StatChangeType::Multiplier(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Turtle => {
                let max_pets_behind: isize = record.lvl.try_into()?;
                vec![Effect {
                    owner: None,
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
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Enemy,
                    position: Position::Last,
                    action: Action::Multiple(vec![
                        Action::Remove(StatChangeType::Static(
                            effect_stats
                        ));
                        record.n_triggers
                    ]),
                    uses: Some(1),
                }]
            }
            PetName::Rhino => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_KNOCKOUT,
                target: Target::Enemy,
                position: Position::First,
                // action: Action::Rhino(effect_stats, 1),
                action: Action::Conditional(
                    LogicType::If(ConditionType::Pet(
                        Target::Friend,
                        ItemCondition::Equal(EqualityCondition::Tier(1)),
                    )),
                    Box::new(Action::Remove(StatChangeType::Static(
                        effect_stats
                            * Statistics {
                                attack: 2,
                                health: 0,
                            },
                    ))),
                    Box::new(Action::Remove(StatChangeType::Static(effect_stats))),
                ),
                uses: None,
            }],
            // No shops so start of turn.
            PetName::Scorpion => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_SUMMON,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Gain(GainType::DefaultItem(FoodName::Peanut)),
                uses: None,
            }],
            PetName::Shark => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Turkey => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::TriggerAffected(None),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Cow => {
                const NUM_MILK: usize = 2;
                const MILK_DISCOUNT: usize = 3;
                // Milk stats from food record are incorrect as foods do not store levels.
                // Stats are set here.
                let mut milk = Food::try_from(FoodName::Milk)?;
                milk.ability.action = Action::Add(StatChangeType::Static(effect_stats));

                let mut add_milk_actions =
                    vec![Action::AddShopFood(GainType::StoredItem(Box::new(milk))); NUM_MILK];
                add_milk_actions.insert(0, Action::ClearShop(Entity::Food));
                vec![
                    Effect {
                        owner: None,
                        temp: record.temp_effect,
                        trigger: TRIGGER_SELF_PET_BOUGHT,
                        target: Target::Shop,
                        position: Position::None,
                        action: Action::Multiple(add_milk_actions),
                        uses: None,
                    },
                    Effect {
                        owner: None,
                        temp: record.temp_effect,
                        trigger: TRIGGER_SELF_PET_BOUGHT,
                        target: Target::Shop,
                        position: Position::All(ItemCondition::None),
                        action: Action::Discount(Entity::Food, MILK_DISCOUNT),
                        uses: None,
                    },
                ]
            }
            PetName::Monkey => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::First,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Seal => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FOOD_EATEN,
                target: Target::Friend,
                position: Position::N {
                    condition: ItemCondition::NotEqual(EqualityCondition::IsSelf),
                    targets: 3,
                    random: true,
                    exact_n_targets: false,
                },
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            // TODO: Moose action must change. Consider adding unfreeze shop action and using Action::Multiple().
            PetName::Moose => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::Any(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Moose {
                    stats: effect_stats,
                    tier: 1,
                },
                uses: None,
            }],
            PetName::Goat => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_PET_BOUGHT,
                target: Target::Shop,
                position: Position::None,
                action: Action::AlterGold(record.lvl.try_into()?),
                uses: Some(record.n_triggers),
            }],
            PetName::Poodle => {
                let target_positions = (MIN_SHOP_TIER..=MAX_SHOP_TIER)
                    .map(|tier| {
                        Position::Any(ItemCondition::MultipleAll(vec![
                            ItemCondition::NotEqual(EqualityCondition::IsSelf),
                            ItemCondition::Equal(EqualityCondition::Tier(tier)),
                        ]))
                    })
                    .collect_vec();
                vec![Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: Position::Multiple(target_positions),
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Fox => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Fox(Entity::Food, record.lvl),
                uses: Some(record.n_triggers),
            }],
            PetName::Hamster => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ROLL,
                target: Target::Shop,
                position: Position::None,
                action: Action::AlterGold(1),
                uses: Some(record.n_triggers),
            }],
            PetName::PolarBear => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_START_TURN,
                target: Target::Shop,
                position: Position::Any(ItemCondition::Equal(EqualityCondition::Frozen)),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Shoebill => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::All(ItemCondition::Equal(EqualityCondition::Name(
                    EntityName::Food(FoodName::Strawberry),
                ))),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::SiberianHusky => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::All(ItemCondition::MultipleAll(vec![
                    ItemCondition::Equal(EqualityCondition::Name(EntityName::Food(FoodName::None))),
                    ItemCondition::NotEqual(EqualityCondition::IsSelf),
                ])),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Zebra => vec![
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_PET_BOUGHT,
                    target: Target::Friend,
                    position: Position::Any(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                },
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_PET_SOLD,
                    target: Target::Friend,
                    position: Position::Any(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                },
            ],
            PetName::Boar => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_ATTACK,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Fly => {
                // Add exception for other zombie flies.
                vec![Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_ANY_FAINT,
                    target: Target::Friend,
                    position: Position::TriggerAffected(None),
                    action: Action::Summon(SummonType::StoredPet(Box::new(Pet::new(
                        PetName::ZombieFly,
                        Some(effect_stats),
                        record.lvl,
                    )?))),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Gorilla => vec![Effect {
                owner: None,
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
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Enemy,
                    position: Position::Any(ItemCondition::None),
                    action: Action::Multiple(vec![
                        Action::Remove(StatChangeType::Multiplier(
                            effect_stats
                        ));
                        record.n_triggers
                    ]),
                    uses: Some(1),
                }]
            }
            PetName::Mammoth => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::All(ItemCondition::None),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Snake => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_AHEAD_ATTACK,
                target: Target::Enemy,
                position: Position::Any(ItemCondition::None),
                action: Action::Remove(StatChangeType::Static(effect_stats)),
                uses: None,
            }],

            PetName::FrilledDragon => vec![Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::ForEach(ConditionType::Pet(
                        Target::Friend,
                        ItemCondition::Equal(EqualityCondition::Trigger(Status::Faint)),
                    )),
                    Box::new(Action::Add(StatChangeType::Static(effect_stats))),
                    Box::new(Action::None),
                ),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            // Only level one for now.
            PetName::Frog => {
                let mut effects = vec![];
                let mut base_effect = Effect {
                    owner: None,
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
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Any(ItemCondition::Equal(EqualityCondition::Name(
                    EntityName::Food(FoodName::Strawberry),
                ))),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            // Iguana has two effects that are the same except for their triggers.
            PetName::Iguana => vec![
                Effect {
                    owner: None,
                    trigger: TRIGGER_ANY_ENEMY_SUMMON,
                    target: Target::Enemy,
                    position: Position::TriggerAffected(None),
                    action: Action::Remove(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                },
                Effect {
                    owner: None,
                    trigger: TRIGGER_ANY_ENEMY_PUSHED,
                    target: Target::Enemy,
                    position: Position::TriggerAffected(None),
                    action: Action::Remove(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                },
            ],
            PetName::Moth => vec![Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::First,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Seahorse => vec![Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Last,
                action: Action::Push(Position::Relative(record.lvl.try_into()?)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Bat => vec![Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::N {
                    condition: ItemCondition::None,
                    targets: record.lvl,
                    random: true,
                    exact_n_targets: false,
                },
                action: Action::Gain(GainType::DefaultItem(FoodName::Weak)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::AtlanticPuffin => {
                // For each level, do an action that removes some amount of stats based on the number of enemies with strawberries.
                vec![Effect {
                    owner: None,
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
                            Box::new(Action::Remove(StatChangeType::Static(effect_stats))),
                            Box::new(Action::None),
                        );
                        record.lvl
                    ]),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            PetName::Dove => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::N {
                    condition: ItemCondition::Equal(EqualityCondition::Name(EntityName::Food(
                        FoodName::Strawberry,
                    ))),
                    targets: 2,
                    random: true,
                    exact_n_targets: false,
                },
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Koala => vec![Effect {
                owner: None,
                trigger: TRIGGER_ANY_HURT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Panda => {
                vec![
                    Effect {
                        owner: None,
                        trigger: TRIGGER_START_BATTLE,
                        target: Target::Friend,
                        position: Position::Nearest(1),
                        action: Action::Add(StatChangeType::Multiplier(effect_stats)),
                        uses: Some(record.n_triggers),
                        temp: record.temp_effect,
                    },
                    Effect {
                        owner: None,
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
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Relative(1),
                action: Action::Experience(record.lvl),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Stork => {
                vec![Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Summon(SummonType::ShopTierPet {
                        stats: None,
                        lvl: Some(record.lvl),
                        tier_diff: Some(-1),
                    }),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Raccoon => vec![
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_BEFORE_ATTACK,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Copy(CopyType::Item(None), Target::Enemy, Position::First),
                    uses: Some(record.n_triggers),
                },
                Effect {
                    owner: None,
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
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Copy(
                    CopyType::Effect(vec![], Some(record.lvl)),
                    Target::Enemy,
                    Position::N {
                        condition: ItemCondition::MultipleAll(vec![
                            ItemCondition::HighestTier,
                            ItemCondition::Equal(EqualityCondition::Trigger(Status::Faint)),
                        ]),
                        targets: 1,
                        random: false,
                        exact_n_targets: false,
                    },
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Aardvark => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_ENEMY_SUMMON,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Bear => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Either,
                position: Position::Multiple(vec![Position::Relative(-1), Position::Relative(1)]),
                action: Action::Gain(GainType::DefaultItem(FoodName::Honey)),
                uses: Some(record.n_triggers),
            }],
            PetName::Seagull => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::TriggerAffected(None),
                // Give currently held food.
                action: Action::Gain(GainType::SelfItem),
                uses: Some(record.n_triggers),
            }],
            PetName::Blobfish => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Nearest(-1),
                action: Action::Experience(1),
                uses: Some(record.n_triggers),
            }],
            PetName::Clownfish => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_LEVELUP,
                target: Target::Friend,
                position: Position::TriggerAffected(None),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Toad => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_ENEMY_HURT,
                target: Target::Enemy,
                position: Position::TriggerAffected(None),
                action: Action::Gain(GainType::DefaultItem(FoodName::Weak)),
                uses: Some(record.n_triggers),
            }],
            PetName::Woodpecker => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Either,
                position: Position::Nearest(2),
                action: Action::Multiple(vec![
                    Action::Remove(StatChangeType::Static(effect_stats));
                    record.lvl
                ]),
                uses: Some(record.n_triggers),
            }],
            PetName::Armadillo => vec![
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                },
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_HURT,
                    target: Target::Friend,
                    position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                },
            ],
            PetName::Doberman => vec![Effect {
                owner: None,
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
                        Action::Add(StatChangeType::Static(effect_stats)),
                    ])),
                    Box::new(Action::None),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Lynx => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Any(ItemCondition::None),
                action: Action::Multiple(vec![Action::Lynx; record.lvl]),
                uses: Some(record.n_triggers),
            }],
            PetName::Porcupine => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Either,
                position: Position::TriggerAfflicting(None),
                action: Action::Remove(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Caterpillar => match record.lvl {
                1 | 2 => vec![Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_TURN,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Experience(1),
                    uses: None,
                }],
                3 => {
                    vec![Effect {
                        owner: None,
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
                temp: record.temp_effect,
                trigger: TRIGGER_BEFORE_FIRST_BATTLE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Copy(
                    CopyType::Stats(None),
                    Target::Enemy,
                    Position::N {
                        condition: ItemCondition::Strongest,
                        targets: 1,
                        random: false,
                        exact_n_targets: true,
                    },
                ),
                uses: Some(1),
            }],
            PetName::Anteater => {
                vec![Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Multiple(vec![
                        Action::Summon(SummonType::StoredPet(Box::new(
                            Pet::new(PetName::Ant, None, record.lvl)?
                        )));
                        2
                    ]),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Donkey => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_FAINT,
                target: Target::Enemy,
                position: Position::Last,
                action: Action::Push(Position::First),
                uses: Some(record.n_triggers),
            }],
            PetName::Eel => {
                vec![Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Add(StatChangeType::Multiplier(effect_stats)),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Hawk => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Opposite,
                action: Action::Remove(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Pelican => {
                let start_battle_effect = Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::Any(ItemCondition::Equal(EqualityCondition::Name(
                        EntityName::Food(FoodName::Strawberry),
                    ))),
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                };
                let mut start_turn_effect = start_battle_effect.clone();
                start_turn_effect.trigger = TRIGGER_START_TURN;
                vec![start_turn_effect, start_battle_effect]
            }
            PetName::Hyena => {
                let mut first_effect = Effect {
                    owner: None,
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
            // TODO: Add new Lionfish action. No way currently to implement with existing code. :/
            PetName::Lionfish => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_BEFORE_ATTACK,
                target: Target::Enemy,
                position: Position::First,
                action: Action::Gain(GainType::DefaultItem(FoodName::Weak)),
                uses: None,
            }],
            PetName::Eagle => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Summon(SummonType::ShopTierPet {
                    stats: Some(effect_stats),
                    lvl: Some(record.lvl),
                    tier_diff: Some(1),
                }),
                uses: Some(record.n_triggers),
            }],
            PetName::Microbe => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Either,
                position: Position::All(ItemCondition::None),
                action: Action::Gain(GainType::DefaultItem(FoodName::Weak)),
                uses: Some(record.n_triggers),
            }],
            PetName::Lion => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::If(ConditionType::Pet(
                        Target::Friend,
                        ItemCondition::HighestTier,
                    )),
                    Box::new(Action::Add(StatChangeType::Multiplier(effect_stats))),
                    Box::new(Action::None),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Swordfish => {
                let self_dmg_effect = Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Remove(StatChangeType::Multiplier(effect_stats)),
                    uses: Some(record.n_triggers),
                };
                let mut enemy_dmg_effect = self_dmg_effect.clone();
                enemy_dmg_effect.target = Target::Enemy;
                enemy_dmg_effect.position = Position::N {
                    condition: ItemCondition::Healthiest,
                    targets: 1,
                    random: false,
                    exact_n_targets: true,
                };
                vec![self_dmg_effect, enemy_dmg_effect]
            }
            PetName::Triceratops => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Friend,
                position: Position::Any(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Vulture => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_FAINT,
                target: Target::Enemy,
                position: Position::Any(ItemCondition::None),
                action: Action::Conditional(
                    LogicType::If(ConditionType::Team(
                        Target::Enemy,
                        TeamCondition::NumberFaintedMultiple(2),
                    )),
                    Box::new(Action::Remove(StatChangeType::Static(effect_stats))),
                    Box::new(Action::None),
                ),
                uses: None,
            }],
            PetName::Alpaca => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::TriggerAffected(None),
                action: Action::Experience(1),
                uses: Some(record.n_triggers),
            }],
            // TODO: Cat needs to have limit. Try to reimplement so not hard-coded effect.
            PetName::Tapir => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Summon(SummonType::SelfTeamPet(
                    None,
                    Some(record.lvl),
                    record.name,
                )),
                uses: Some(record.n_triggers),
            }],
            PetName::Walrus => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::N {
                    condition: ItemCondition::None,
                    targets: record.lvl,
                    random: true,
                    exact_n_targets: false,
                },
                action: Action::Gain(GainType::DefaultItem(FoodName::Peanut)),
                uses: Some(record.n_triggers),
            }],
            PetName::WhiteTiger => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Nearest(-record.lvl.try_into()?),
                action: Action::Experience(3),
                uses: Some(record.n_triggers),
            }],
            PetName::Octopus => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_BEFORE_ATTACK,
                target: Target::Enemy,
                position: Position::N {
                    condition: ItemCondition::None,
                    targets: record.lvl,
                    random: true,
                    exact_n_targets: false,
                },
                action: Action::Remove(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Orca => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Multiple(vec![
                    Action::Summon(SummonType::QueryPet(
                        SAPQuery::builder()
                            .set_table(Entity::Pet)
                            .set_param("effect_trigger", vec!["Faint"])
                            .set_param("lvl", vec![1])
                            .set_param("is_token", vec![false]),
                        None
                    ));
                    record.lvl
                ]),
                uses: Some(record.n_triggers),
            }],
            PetName::Piranha => vec![
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::All(ItemCondition::None),
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: None,
                },
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_HURT,
                    target: Target::Friend,
                    position: Position::All(ItemCondition::None),
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: None,
                },
            ],
            PetName::Reindeer => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_BEFORE_ATTACK,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Gain(GainType::DefaultItem(FoodName::Melon)),
                uses: Some(record.n_triggers),
            }],
            PetName::SabertoothTiger => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Summon(SummonType::QueryPet(
                    SAPQuery::builder()
                        .set_table(Entity::Pet)
                        .set_param("lvl", vec![1])
                        .set_param("tier", vec![1])
                        .set_param("-name", vec![PetName::Sloth])
                        .set_param("is_token", vec![false]),
                    Some(effect_stats),
                )),
                uses: None,
            }],
            PetName::Spinosaurus => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_FAINT,
                target: Target::Friend,
                position: Position::Any(ItemCondition::None),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Stegosaurus => vec![Effect {
                owner: None,
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
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::N {
                    condition: ItemCondition::Equal(EqualityCondition::Name(EntityName::Food(
                        FoodName::Strawberry,
                    ))),
                    targets: record.lvl,
                    random: true,
                    exact_n_targets: false,
                },
                action: Action::Gain(GainType::DefaultItem(FoodName::Coconut)),
                uses: Some(record.n_triggers),
            }],
            PetName::Dragon => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: trigger_any_pet_bought_tier(1),
                target: Target::Friend,
                position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
            }],
            PetName::Lioness => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Shop,
                position: Position::None,
                action: Action::AddShopStats(effect_stats),
                uses: None,
            }],
            PetName::Chicken => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Shop,
                position: Position::None,
                action: Action::AddShopStats(effect_stats),
                uses: None,
            }],
            PetName::Sauropod => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_FOOD_BOUGHT,
                target: Target::Shop,
                position: Position::None,
                action: Action::AlterGold(record.lvl.try_into()?),
                uses: Some(record.n_triggers),
            }],
            PetName::Tyrannosaurus => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::All(ItemCondition::NotEqual(EqualityCondition::IsSelf)),
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: Some(record.n_triggers),
            }],
            PetName::Hammershark => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_START_TURN,
                target: Target::Shop,
                position: Position::None,
                action: Action::Conditional(
                    LogicType::IfAny(ConditionType::Pet(
                        Target::Friend,
                        ItemCondition::Equal(EqualityCondition::Level(3)),
                    )),
                    Box::new(Action::AlterGold((record.lvl * 3).try_into()?)),
                    Box::new(Action::None),
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Komodo => vec![
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: Position::Ahead,
                    action: Action::Add(StatChangeType::Static(effect_stats)),
                    uses: Some(record.n_triggers),
                },
                Effect {
                    owner: None,
                    temp: record.temp_effect,
                    trigger: TRIGGER_END_TURN,
                    target: Target::Friend,
                    position: Position::Ahead,
                    action: Action::Shuffle(RandomizeType::Positions),
                    uses: Some(record.n_triggers),
                },
            ],
            PetName::Ostrich => vec![Effect {
                owner: None,
                temp: record.temp_effect,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Conditional(
                    LogicType::ForEach(ConditionType::Pet(
                        Target::Shop,
                        ItemCondition::Multiple(vec![
                            ItemCondition::Equal(EqualityCondition::Tier(5)),
                            ItemCondition::Equal(EqualityCondition::Tier(6)),
                        ]),
                    )),
                    Box::new(Action::Add(StatChangeType::Static(effect_stats))),
                    Box::new(Action::None),
                ),
                uses: Some(record.n_triggers),
            }],
            // PetName::Cat => todo!(),
            // PetName::Tiger => todo!(),
            PetName::Gecko => {
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Conditional(
                        LogicType::If(ConditionType::Team(
                            Target::Friend,
                            TeamCondition::NumberToys(Some(CondOrdering::Greater(0))),
                        )),
                        Box::new(Action::Add(StatChangeType::Static(effect_stats))),
                        Box::default(),
                    ),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            // PetName::AfricanPenguin => todo!(),
            // PetName::BlackNeckedStilt => todo!(),
            // PetName::DoorHeadAnt => todo!(),
            // PetName::Gazelle => todo!(),
            // PetName::HerculesBeetle => todo!(),
            // PetName::Lizard => todo!(),
            // PetName::SeaTurtle => todo!(),
            // PetName::SeaUrchin => todo!(),
            // PetName::Squid => todo!(),
            // PetName::Stoat => todo!(),
            // PetName::BelugaSturgeon => todo!(),
            // PetName::Lemur => todo!(),
            // PetName::Mandrill => todo!(),
            // PetName::Robin => todo!(),
            // PetName::Baboon => todo!(),
            // PetName::BettaFish => todo!(),
            // PetName::Flea => todo!(),
            // PetName::FlyingFish => todo!(),
            // PetName::Guineafowl => todo!(),
            // PetName::Meekrat => todo!(),
            // PetName::MuskOx => todo!(),
            // PetName::Osprey => todo!(),
            // PetName::RoyalFlycatcher => todo!(),
            // PetName::SurgeonFish => todo!(),
            // PetName::Weasel => todo!(),
            // PetName::FlyingSquirrel => todo!(),
            // PetName::HoopoeBird => todo!(),
            // PetName::Pangolin => todo!(),
            // PetName::Cuttlefish => todo!(),
            // PetName::EgyptianVulture => todo!(),
            // PetName::Falcon => todo!(),
            // PetName::Manatee => todo!(),
            // PetName::MantaRay => todo!(),
            // PetName::PoisonDartFrog => todo!(),
            // PetName::SaigaAntelope => todo!(),
            // PetName::Sealion => todo!(),
            // PetName::SecretaryBird => todo!(),
            // PetName::Slug => todo!(),
            // PetName::Vaquita => todo!(),
            // PetName::Chameleon => todo!(),
            // PetName::Gharial => todo!(),
            // PetName::Tahr => todo!(),
            // PetName::WhaleShark => todo!(),
            // PetName::BelugaWhale => todo!(),
            // PetName::BlueRingedOctopus => todo!(),
            // PetName::Cockatoo => todo!(),
            // PetName::Crane => todo!(),
            // PetName::Emu => todo!(),
            // PetName::FireAnt => todo!(),
            // PetName::Macaque => todo!(),
            // PetName::NurseShark => todo!(),
            // PetName::Nyala => todo!(),
            // PetName::SilverFox => todo!(),
            // PetName::Wolf => todo!(),
            // PetName::Axolotl => todo!(),
            // PetName::Mosasaurus => todo!(),
            // PetName::Panther => todo!(),
            // PetName::SnappingTurtle => todo!(),
            // PetName::StingRay => todo!(),
            // PetName::Stonefish => todo!(),
            // PetName::BirdofParadise => todo!(),
            // PetName::Catfish => todo!(),
            // PetName::Cobra => todo!(),
            // PetName::GermanShepherd => todo!(),
            // PetName::GrizzlyBear => todo!(),
            // PetName::HighlandCow => todo!(),
            // PetName::Oyster => todo!(),
            // PetName::Pteranodon => todo!(),
            // PetName::Warthog => todo!(),
            // PetName::Wildebeest => todo!(),
            // PetName::AnglerFish => todo!(),
            // PetName::ElephantSeal => todo!(),
            // PetName::MantisShrimp => todo!(),
            // PetName::Mongoose => todo!(),
            // PetName::Puma => todo!(),
            _ => Vec::default(),
        })
    }
}
