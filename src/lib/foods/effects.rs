use crate::{
    db::record::FoodRecord,
    effects::{
        actions::{Action, RandomizeType, StatChangeType, SummonType},
        effect::Effect,
        state::{ItemCondition, Position, Target},
        stats::Statistics,
        trigger::*,
    },
    error::SAPTestError,
    foods::names::FoodName,
    pets::{
        names::PetName,
        pet::{MAX_PET_STATS, MIN_PET_STATS},
    },
    shop::trigger::TRIGGER_SELF_FOOD_EATEN,
};

/// May need to be similar to Pet effects as `Vec<Effect>` at some point.
impl TryFrom<&FoodRecord> for Effect {
    type Error = SAPTestError;

    fn try_from(record: &FoodRecord) -> Result<Self, Self::Error> {
        let effect_stats = Statistics::new(
            record.effect_atk.clamp(MIN_PET_STATS, MAX_PET_STATS),
            record.effect_health.clamp(MIN_PET_STATS, MAX_PET_STATS),
        )?;
        let uses = record.single_use.then_some(1);

        Ok(match record.name {
            FoodName::Chili => Effect {
                owner: None,
                target: Target::Enemy,
                // Next enemy relative to current pet position.
                position: Position::Relative(-1),
                action: Action::Remove(StatChangeType::Static(effect_stats)),
                uses,
                trigger: TRIGGER_BATTLE_FOOD,
                temp: record.end_of_battle,
            },
            FoodName::Coconut => Effect {
                owner: None,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Invincible,
                uses,
                trigger: TRIGGER_DMG_CALC,
                temp: record.end_of_battle,
            },
            FoodName::Garlic | FoodName::Lemon => Effect {
                owner: None,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Negate(effect_stats),
                uses,
                trigger: TRIGGER_DMG_CALC,
                temp: record.end_of_battle,
            },
            FoodName::Honey => Effect {
                owner: None,
                target: Target::Friend,
                position: Position::TriggerAffected,
                action: Action::Summon(SummonType::DefaultPet(PetName::Bee)),
                uses,
                trigger: TRIGGER_SELF_FAINT,
                temp: record.end_of_battle,
            },
            FoodName::MeatBone => Effect {
                owner: None,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses,
                trigger: TRIGGER_ATK_DMG_CALC,
                temp: record.end_of_battle,
            },
            FoodName::Melon => Effect {
                owner: None,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Negate(effect_stats),
                uses,
                trigger: TRIGGER_DMG_CALC,
                temp: record.end_of_battle,
            },
            FoodName::Mushroom => Effect {
                owner: None,
                target: Target::Friend,
                position: Position::TriggerAffected,
                action: Action::Summon(SummonType::SelfPet(
                    Some(Statistics {
                        attack: 1,
                        health: 1,
                    }),
                    None,
                    false,
                )),
                uses,
                trigger: TRIGGER_SELF_FAINT,
                temp: record.end_of_battle,
            },
            FoodName::Peanut => Effect {
                owner: None,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Kill,
                uses,
                trigger: TRIGGER_ATK_DMG_CALC,
                temp: record.end_of_battle,
            },
            FoodName::Steak => Effect {
                owner: None,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses,
                trigger: TRIGGER_ATK_DMG_CALC,
                temp: record.end_of_battle,
            },
            FoodName::Weak => {
                // Invert attack to health and reverse sign so additional damage taken.
                let mut vulnerable_stats = effect_stats;
                vulnerable_stats.invert();
                vulnerable_stats.health = -vulnerable_stats.health;

                Effect {
                    owner: None,
                    trigger: TRIGGER_DMG_CALC,
                    target: Target::Friend,
                    position: Position::OnSelf,
                    action: Action::Remove(StatChangeType::Static(vulnerable_stats)),
                    uses,
                    temp: record.end_of_battle,
                }
            }
            FoodName::SleepingPill => Effect {
                owner: None,
                trigger: TRIGGER_SELF_FOOD_EATEN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Kill,
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Croissant | FoodName::Cucumber | FoodName::Carrot => Effect {
                owner: None,
                trigger: TRIGGER_END_TURN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Grapes => Effect {
                owner: None,
                trigger: TRIGGER_START_TURN,
                target: Target::Shop,
                position: Position::None,
                action: Action::AlterGold(1),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Chocolate => Effect {
                owner: None,
                trigger: TRIGGER_SELF_FOOD_EATEN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Experience(1),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Pepper => Effect {
                owner: None,
                trigger: TRIGGER_DMG_CALC,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Endure,
                uses,
                temp: record.end_of_battle,
            },
            FoodName::CannedFood => Effect {
                owner: None,
                trigger: TRIGGER_NONE,
                target: Target::Shop,
                position: Position::None,
                action: Action::AddShopStats(effect_stats),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::FortuneCookie => Effect {
                owner: None,
                trigger: TRIGGER_ATK_DMG_CALC,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Critical(50),
                uses: None,
                temp: record.end_of_battle,
            },
            FoodName::Cheese => Effect {
                owner: None,
                trigger: TRIGGER_ATK_DMG_CALC,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Critical(100),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Pineapple => Effect {
                owner: None,
                trigger: TRIGGER_INDIR_DMG_CALC,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses: None,
                temp: record.end_of_battle,
            },
            FoodName::SaladBowl
            | FoodName::Sushi
            | FoodName::Stew
            | FoodName::Taco
            | FoodName::Pizza
            | FoodName::SoftIce
            | FoodName::HotDog
            | FoodName::Orange => Effect {
                owner: None,
                trigger: TRIGGER_NONE,
                target: Target::Friend,
                position: Position::N {
                    condition: ItemCondition::None,
                    targets: record.n_targets,
                    random: true,
                },
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Apple
            | FoodName::Bacon
            | FoodName::Cookie
            | FoodName::Broccoli
            | FoodName::FriedShrimp
            | FoodName::Cupcake
            | FoodName::Peach
            | FoodName::ChickenLeg => Effect {
                owner: None,
                trigger: TRIGGER_SELF_FOOD_EATEN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Add(StatChangeType::Static(effect_stats)),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Strawberry => Effect {
                owner: None,
                trigger: TRIGGER_NONE,
                target: Target::Friend,
                position: Position::None,
                action: Action::None,
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Lollipop => Effect {
                owner: None,
                trigger: TRIGGER_SELF_FOOD_EATEN,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Swap(RandomizeType::Stats),
                uses,
                temp: record.end_of_battle,
            },
            FoodName::Popcorn => Effect {
                owner: None,
                trigger: TRIGGER_SELF_FAINT,
                target: Target::Friend,
                position: Position::OnSelf,
                action: Action::Summon(SummonType::SelfTierPet(None, None)),
                uses,
                temp: record.end_of_battle,
            },
            _ => Effect {
                owner: None,
                trigger: TRIGGER_NONE,
                target: Target::None,
                position: Position::None,
                action: Action::None,
                uses,
                temp: record.end_of_battle,
            },
        })
    }
}
