use crate::{
    battle::{
        effect::{Effect, Entity},
        state::{Action, Condition, CopyAttr, Position, Statistics, Status, Target},
        trigger::*,
    },
    db::{query::query_pet, record::PetRecord, setup::get_connection},
    error::SAPTestError,
    foods::{food::Food, names::FoodName},
    pets::{
        names::PetName,
        pet::{Pet, MAX_PET_STATS},
    },
};
use std::convert::TryInto;

impl TryFrom<PetRecord> for Vec<Effect> {
    type Error = SAPTestError;

    fn try_from(record: PetRecord) -> Result<Self, Self::Error> {
        let pet_stats = Statistics::new(record.attack, record.health)?;
        let effect_stats = Statistics::new(record.effect_atk, record.effect_health)?;

        Ok(match &record.name {
            PetName::Ant => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Any(Condition::None),
                action: Action::Add(effect_stats),
                uses: Some(record.n_triggers),
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Mosquito => vec![Effect {
                owner: None,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Any(Condition::None),
                action: Action::Remove(effect_stats),
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
                    action: Action::Summon(Some(zombie_cricket), None),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Horse => vec![Effect {
                owner: None,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::Trigger,
                action: Action::Add(effect_stats),
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
                    CopyAttr::PercentStats(effect_stats),
                    Target::Friend,
                    Position::N(Condition::Healthiest, 1),
                ),
                uses: Some(record.n_triggers),
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Dodo => {
                let add_stats = pet_stats * effect_stats;

                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::Relative(1),
                    action: Action::Add(add_stats),
                    uses: Some(record.n_triggers),
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Elephant => {
                let n_removes = vec![Action::Remove(effect_stats); record.n_triggers];
                vec![Effect {
                    owner: None,
                    trigger: TRIGGER_SELF_ATTACK,
                    target: Target::Friend,
                    position: Position::Relative(-1),
                    action: Action::Multiple(n_removes),
                    uses: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                }]
            }
            PetName::Flamingo => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Range(-2..=-1),
                action: Action::Add(effect_stats),
                uses: Some(record.n_triggers),
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Hedgehog => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Either,
                position: Position::All(Condition::None),
                action: Action::Remove(effect_stats),
                uses: Some(record.n_triggers),
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Peacock => vec![Effect {
                owner: None,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(effect_stats),
                uses: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
            }],
            PetName::Rat => {
                let dirty_rat = Box::new(Pet::new(
                    PetName::DirtyRat,
                    None,
                    Some(effect_stats),
                    record.lvl,
                )?);
                let rats_summoned = vec![Action::Summon(Some(dirty_rat), None); record.lvl];
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
            PetName::Spider => {
                let conn = get_connection().expect("Can't get connection.");
                let pet_record = query_pet(
                    &conn,
                    "SELECT * FROM pets where lvl = ? and tier = 3 and pack = 'Turtle' ORDER BY RANDOM() LIMIT 1",
                    &[record.lvl.to_string()]
                );
                if let Ok(Some(record)) = pet_record.as_ref().map(|query| query.first()) {
                    let summoned_pet = Box::new(Pet::new(
                        record.name.clone(),
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
                        action: Action::Summon(Some(summoned_pet), None),
                        uses: Some(record.n_triggers),
                    }]
                } else {
                    vec![]
                }
            }
            PetName::Badger => {
                let effect_dmg_stats = pet_stats * effect_stats;

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
                    action: Action::Remove(effect_dmg_stats),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Blowfish => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Enemy,
                position: Position::Any(Condition::None),
                action: Action::Remove(effect_stats),
                uses: None,
            }],
            PetName::Camel => {
                let n_adds = vec![Action::Add(effect_stats); record.n_triggers];
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_HURT,
                    target: Target::Friend,
                    position: Position::Relative(-1),
                    action: Action::Multiple(n_adds),
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
                action: Action::Add(effect_stats),
                uses: None,
            }],
            PetName::Dolphin => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::N(Condition::Illest, 1),
                action: Action::Remove(effect_stats),
                uses: Some(record.n_triggers),
            }],
            PetName::Kangaroo => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_AHEAD_ATTACK,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(effect_stats),
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
                    Action::Add(effect_stats),
                    Action::Gain(Some(Box::new(Food::try_from(FoodName::Melon)?))),
                ]),
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
                        Action::Summon(Some(ram.clone()), None),
                        Action::Summon(Some(ram), None),
                    ]),
                    uses: Some(record.n_triggers),
                }]
            }
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
                    action: Action::Summon(Some(Box::new(bus)), None),
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
                action: Action::Add(effect_stats),
                uses: None,
            }],
            PetName::Parrot => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Copy(
                    CopyAttr::Effect(vec![], Some(record.lvl)),
                    Target::Friend,
                    Position::Relative(1),
                ),
                uses: None,
            }],
            PetName::Rooster => {
                let mut chick_stats = pet_stats * effect_stats;
                chick_stats.clamp(1, MAX_PET_STATS);

                let chick = Box::new(Pet::new(
                    PetName::Chick,
                    None,
                    Some(chick_stats),
                    record.lvl,
                )?);
                let n_chicks = vec![Action::Summon(Some(chick), None); record.lvl];
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Multiple(n_chicks),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Skunk => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::N(Condition::Healthiest, 1),
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
                    position: Position::Range(-max_pets_behind..=-1),
                    action: Action::Gain(Some(Box::new(Food::try_from(FoodName::Melon)?))),
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
                action: Action::Whale(record.lvl, Position::Relative(1)),
                uses: Some(record.n_triggers),
            }],
            PetName::Crocodile => {
                let n_removes = vec![Action::Remove(effect_stats); record.n_triggers];
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Enemy,
                    position: Position::Last,
                    action: Action::Multiple(n_removes),
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
                trigger: TRIGGER_START_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Gain(Some(Box::new(Food::try_from(FoodName::Peanut)?))),
                uses: None,
            }],
            PetName::Shark => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(effect_stats),
                uses: None,
            }],
            PetName::Turkey => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::Trigger,
                action: Action::Add(effect_stats),
                uses: None,
            }],
            PetName::Boar => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_ATTACK,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(effect_stats),
                uses: None,
            }],
            PetName::Fly => {
                let zombie_fly = Box::new(Pet::new(
                    PetName::ZombieFly,
                    None,
                    Some(effect_stats),
                    record.lvl,
                )?);
                // Add exception for other zombie flies.
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_ANY_FAINT,
                    target: Target::Friend,
                    position: Position::Trigger,
                    action: Action::Summon(Some(zombie_fly), None),
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
                action: Action::Gain(Some(Box::new(Food::try_from(FoodName::Coconut)?))),
                uses: Some(record.n_triggers),
            }],
            PetName::Leopard => {
                let effect_dmg = pet_stats * effect_stats;
                let n_removes = vec![Action::Remove(effect_dmg); record.n_triggers];
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Enemy,
                    position: Position::Any(Condition::None),
                    action: Action::Multiple(n_removes),
                    uses: Some(1),
                }]
            }
            PetName::Mammoth => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::All(Condition::None),
                action: Action::Add(effect_stats),
                uses: Some(record.n_triggers),
            }],
            PetName::Snake => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_AHEAD_ATTACK,
                target: Target::Enemy,
                position: Position::Any(Condition::None),
                action: Action::Remove(effect_stats),
                uses: None,
            }],

            PetName::FrilledDragon => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::ForEachCondition(
                    Box::new(Action::Add(effect_stats)),
                    Target::Friend,
                    Condition::TriggeredBy(Status::Faint),
                ),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            // Only level one for now.
            PetName::Frog => {
                let mut effects = vec![];
                if record.lvl == 1 {
                    effects.push(Effect {
                        owner: None,
                        entity: Entity::Pet,
                        trigger: TRIGGER_START_BATTLE,
                        target: Target::Friend,
                        position: Position::Adjacent,
                        action: Action::SwapStats,
                        uses: Some(record.n_triggers),
                        temp: record.temp_effect,
                    })
                };
                effects
            }
            PetName::Hummingbird => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Any(Condition::HasFood(FoodName::Strawberry)),
                action: Action::Add(effect_stats),
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
                    position: Position::Trigger,
                    action: Action::Remove(effect_stats),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                },
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    trigger: TRIGGER_ANY_ENEMY_PUSHED,
                    target: Target::Enemy,
                    position: Position::Trigger,
                    action: Action::Remove(effect_stats),
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
                action: Action::Add(effect_stats),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Seahorse => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Last,
                action: Action::Push(Position::Relative(
                    record.lvl.try_into().expect("Invalid level for seahorse."),
                )),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Bat => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Any(Condition::None),
                action: Action::Gain(Some(Box::new(Food::try_from(FoodName::Weak)?))),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::AtlanticPuffin => {
                // For each level, do an action that removes some amount of stats based on the number of enemies with strawberries.
                let num_hits = vec![
                    Action::ForEachCondition(
                        Box::new(Action::Remove(effect_stats)),
                        Target::Enemy,
                        Condition::HasFood(FoodName::Strawberry)
                    );
                    record.lvl
                ];
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Enemy,
                    position: Position::Any(Condition::None),
                    action: Action::Multiple(num_hits),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                }]
            }
            PetName::Dove => vec![
                // TODO: May select same pet.
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::N(Condition::HasFood(FoodName::Strawberry), 2),
                    action: Action::Add(effect_stats),
                    uses: Some(record.n_triggers),
                    temp: record.temp_effect,
                },
            ],
            PetName::Koala => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                trigger: TRIGGER_ANY_HURT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(effect_stats),
                uses: Some(record.n_triggers),
                temp: record.temp_effect,
            }],
            PetName::Panda => {
                let add_stats = Statistics::new(record.attack, record.health)
                    .map(|pet_stats| pet_stats * effect_stats)
                    .expect("Can't convert stats from record.");
                vec![
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
                    Effect {
                        owner: None,
                        entity: Entity::Pet,
                        trigger: TRIGGER_START_BATTLE,
                        target: Target::Friend,
                        position: Position::Relative(1),
                        action: Action::Add(add_stats),
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
                // TODO: Not fully functional as needs tier num.
                let conn = get_connection().expect("Can't get connection.");
                let pet_record = query_pet(
                    &conn,
                    "SELECT * FROM pets where tier = ? and pack = 'Star' and lvl = ? ORDER BY RANDOM() LIMIT 1",
                    &[(record.tier - 1).to_string(), record.lvl.to_string()]
                );
                if let Ok(Some(record)) = pet_record.as_ref().map(|query| query.first()) {
                    // Use default stats.
                    let summoned_pet =
                        Box::new(Pet::new(record.name.clone(), None, None, record.lvl)?);
                    vec![Effect {
                        owner: None,
                        entity: Entity::Pet,
                        temp: record.temp_effect,
                        trigger: TRIGGER_SELF_FAINT,
                        target: Target::Friend,
                        position: Position::OnSelf,
                        action: Action::Summon(Some(summoned_pet), None),
                        uses: Some(record.n_triggers),
                    }]
                } else {
                    vec![]
                }
            }
            PetName::Racoon => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_ATTACK,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Copy(CopyAttr::Item(None), Target::Enemy, Position::First),
                uses: Some(record.n_triggers),
            }],
            PetName::Toucan => {
                let n_pets_behind: isize = record
                    .lvl
                    .try_into()
                    .expect("Can't convert level to isize.");
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::Range(-n_pets_behind..=-1),
                    // If None, update during team init with current item.
                    action: Action::Gain(None),
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
                    CopyAttr::Effect(vec![], Some(record.lvl)),
                    Target::Enemy,
                    Position::N(
                        Condition::MultipleAll(vec![
                            Condition::HighestTier,
                            Condition::TriggeredBy(Status::Faint),
                        ]),
                        1,
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
                action: Action::Add(effect_stats),
                uses: None,
            }],
            PetName::Bear => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Either,
                position: Position::Multiple(vec![Position::Relative(-1), Position::Relative(1)]),
                action: Action::Gain(Some(Box::new(Food::try_from(FoodName::Honey)?))),
                uses: Some(record.n_triggers),
            }],
            PetName::Seagull => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_SUMMON,
                target: Target::Friend,
                position: Position::Trigger,
                // Give currently held food.
                action: Action::Gain(None),
                uses: Some(record.n_triggers),
            }],
            PetName::Blobfish => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::Relative(-1),
                action: Action::Experience,
                uses: Some(record.n_triggers),
            }],
            PetName::Clownfish => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_LEVELUP,
                target: Target::Friend,
                position: Position::Trigger,
                action: Action::Add(effect_stats),
                uses: None,
            }],
            PetName::Toad => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_ANY_ENEMY_HURT,
                target: Target::Enemy,
                position: Position::Trigger,
                action: Action::Gain(Some(Box::new(Food::try_from(FoodName::Weak)?))),
                uses: Some(record.n_triggers),
            }],
            PetName::Woodpecker => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Either,
                position: Position::Range(1..=2),
                action: Action::Multiple(vec![Action::Remove(effect_stats); record.lvl]),
                uses: Some(record.n_triggers),
            }],
            PetName::Armadillo => vec![
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::All(Condition::IgnoreSelf),
                    action: Action::Add(effect_stats),
                    uses: Some(record.n_triggers),
                },
                Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_HURT,
                    target: Target::Friend,
                    position: Position::All(Condition::IgnoreSelf),
                    action: Action::Add(effect_stats),
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
                action: Action::IfTargetCondition(
                    Box::new(Action::Multiple(vec![
                        Action::Gain(Some(Box::new(Food::try_from(FoodName::Coconut)?))),
                        Action::Add(effect_stats),
                    ])),
                    Condition::LowestTier,
                ),
                uses: Some(record.n_triggers),
            }],
            PetName::Lynx => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::Any(Condition::None),
                action: Action::Multiple(vec![Action::Lynx; record.lvl]),
                uses: Some(record.n_triggers),
            }],
            PetName::Porcupine => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_SELF_HURT,
                target: Target::Either,
                position: Position::Trigger,
                action: Action::Thorns(effect_stats),
                uses: None,
            }],
            PetName::Caterpillar => match record.lvl {
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
                    CopyAttr::Stats(None),
                    Target::Friend,
                    Position::N(Condition::Strongest, 1),
                ),
                uses: Some(1),
            }],
            PetName::Anteater => {
                let lvl_ant = Pet::new(PetName::Ant, None, None, record.lvl)
                    .expect("Can't create ants for anteater.");

                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_SELF_FAINT,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Multiple(vec![
                        Action::Summon(Some(Box::new(lvl_ant)), None);
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
                let added_stats = effect_stats * pet_stats;
                vec![Effect {
                    owner: None,
                    entity: Entity::Pet,
                    temp: record.temp_effect,
                    trigger: TRIGGER_START_BATTLE,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Add(added_stats),
                    uses: Some(record.n_triggers),
                }]
            }
            PetName::Hawk => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Enemy,
                position: Position::OnSelf,
                action: Action::Remove(effect_stats),
                uses: Some(record.n_triggers),
            }],
            PetName::Pelican => vec![Effect {
                owner: None,
                entity: Entity::Pet,
                temp: record.temp_effect,
                trigger: TRIGGER_START_BATTLE,
                target: Target::Friend,
                position: Position::Any(Condition::HasFood(FoodName::Strawberry)),
                action: Action::Add(effect_stats),
                uses: Some(record.n_triggers),
            }],
            _ => vec![],
        })
    }
}
