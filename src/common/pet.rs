use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::db::{setup::get_connection, utils::map_row_to_pet};

use super::{
    effect::PetEffect,
    effect::{
        Action, Effect, EffectTrigger, Outcome, Position, Statistics, Target, RGX_ATK, RGX_HEALTH,
        RGX_N_TRIGGERS, RGX_SUMMON_ATK, RGX_SUMMON_HEALTH, Modify,
    },
    food::Food,
    game::Pack,
    pets::names::PetName,
};

/// A record with information about a pet from *Super Auto Pets*.
///
/// This information is queried and parsed from the *Super Auto Pets* *Fandom* wiki.
#[derive(Debug, Serialize, Deserialize)]
pub struct PetRecord {
    pub name: String,
    pub tier: usize,
    pub attack: usize,
    pub health: usize,
    pub pack: Pack,
    pub effect_trigger: Option<String>,
    pub effect: Option<String>,
    pub lvl: usize,
}

/// A Super Auto Pet.
#[derive(Debug, Serialize, Deserialize)]
pub struct Pet {
    pub name: PetName,
    pub tier: usize,
    pub stats: Statistics,
    pub lvl: usize,
    pub effect: Option<PetEffect>,
    pub item: Option<Food>,
}

pub fn num_regex(
    pattern: &'static lazy_regex::Lazy<lazy_regex::Regex>,
    string: &str,
) -> Result<usize, Box<dyn Error>> {
    Ok(pattern.captures(string).map_or(Ok(0), |cap| {
        cap.get(1)
            .map_or(Ok(0), |mtch| mtch.as_str().parse::<usize>())
    })?)
}

/// Maps a pet to its effects.
pub fn get_pet_effect(
    pet: &PetName,
    effect_stats: Statistics,
    n_triggers: usize,
) -> Option<PetEffect> {
    match pet {
        PetName::Ant => Some(PetEffect {
            trigger: EffectTrigger::OnSelf(Outcome {
                action: Action::Faint,
                position: None,
            }),
            target: Target::Friend,
            position: Position::Any,
            effect: Effect::Add(effect_stats),
            uses: Some(n_triggers),
        }),
        PetName::Mosquito => Some(PetEffect {
            trigger: EffectTrigger::StartBattle,
            target: Target::Enemy,
            position: Position::Any,
            effect: Effect::Remove(effect_stats),
            uses: Some(n_triggers),
        }),
        PetName::Cricket => Some(PetEffect {
            trigger: EffectTrigger::OnSelf(Outcome {
                action: Action::Faint,
                position: None,
            }),
            target: Target::None,
            position: Position::Specific(0),
            effect: Effect::Summon(Some(PetName::ZombieCricket), effect_stats),
            uses: Some(n_triggers),
        }),
        PetName::Horse => Some(PetEffect {
            trigger: EffectTrigger::Friend(Outcome {
                action: Action::Summoned,
                position: Some(Position::Trigger),
            }),
            target: Target::Friend,
            position: Position::Trigger,
            effect: Effect::Add(effect_stats),
            uses: None,
        }),
        _ => None,
    }
}


impl Pet {
    /// Create a new `Pet` with given stats and level
    pub fn new(
        name: PetName,
        stats: Statistics,
        lvl: usize,
        item: Option<Food>,
    ) -> Result<Pet, Box<dyn Error>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT * FROM pets where name = ? and lvl = ?")?;
        let pet_record = stmt.query_row([name.to_string(), lvl.to_string()], map_row_to_pet)?;
        let pet_effect = pet_record.effect.unwrap_or("None".to_string());

        let mut effect_stats = Statistics {
            attack: num_regex(RGX_ATK, &pet_effect).ok().unwrap_or(0),
            health: num_regex(RGX_HEALTH, &pet_effect).ok().unwrap_or(0),
        };
        // If a pet has a summon effect, replace attack and health stats from effect_stats.
        if pet_effect.contains("Summon") {
            effect_stats.attack = num_regex(RGX_SUMMON_ATK, &pet_effect).ok().unwrap_or(1);
            effect_stats.health = num_regex(RGX_SUMMON_HEALTH, &pet_effect).ok().unwrap_or(1);
        }
        let n_triggers = num_regex(RGX_N_TRIGGERS, &pet_effect).ok().unwrap_or(1) as usize;
        // TODO: Parse from pet description.
        let effect = get_pet_effect(&name, effect_stats, n_triggers);

        Ok(Pet {
            name,
            tier: pet_record.tier,
            stats,
            lvl: pet_record.lvl,
            effect,
            item,
        })
    }
}

trait Shop {
    fn feed(&mut self, food: &Food);
    fn upgrade(&mut self, pet: &Pet);
}

trait Combat {
    fn attack(&mut self, enemy: &mut Pet);
    fn get_effect_stat_modifier(&self) -> Statistics;
}

impl Modify for Pet {
    fn add_uses(&mut self, n: usize) -> &Self {
        if let Some(ability) = self.effect.as_mut() {
            ability.uses.as_mut().map(|uses| *uses += n );
        }
        self
    }

    fn remove_uses(&mut self, n: usize) -> &Self {
        if let Some(ability) = self.effect.as_mut() {
            ability.uses.as_mut().map(|uses| if *uses >= n { *uses -= n } );
        }
        self
    }
}

impl Combat for Pet {
    /// Gets the `Statistic` modifiers of held foods that alter a pet's stats during battle.
    ///
    /// If a pet has no held food, no `Statistics` are provided.
    fn get_effect_stat_modifier(&self) -> Statistics {
        // If a pet has an item that alters stats...
        // Otherwise, no stat modifier added.
        self.item.as_ref().map_or(
            Statistics {
                attack: 0,
                health: 0,
            },
            |food| {
                let food_effect = food.ability.uses.as_ref().map_or(&Effect::None, |uses| {
                    if *uses > 0 {
                        // Return the food effect.
                        &food.ability.effect
                    } else {
                        &Effect::None
                    }
                });

                match food_effect {
                    // Get stat modifiers from effects.
                    Effect::Add(stats) | Effect::Remove(stats) | Effect::Negate(stats) => {
                        stats.clone()
                    }
                    // Otherwise, no change.
                    _ => Statistics {
                        attack: 0,
                        health: 0,
                    },
                }
            },
        )
    }
    /// Handle the logic of a single pet interaction during the battle phase.
    /// * Alters a `Pet`'s `stats.attack` and `stats.health`
    /// * Decrements any held `Food`'s `uses` attribute.
    ///
    /// ```
    /// use crate::common::{
    ///     effect::Statistics, pet::{Combat, Pet},
    ///     pets::names::PetName,
    /// };
    ///
    /// fn main() {
    ///     let ant_1 = Pet::new(PetName::Ant,
    ///         Statistics {attack: 2, health: 1}, 1, None).unwrap();
    ///     let ant_2 = Pet::new(PetName::Ant,
    ///         Statistics {attack: 2, health: 3}, 1, None).unwrap();
    ///
    ///     ant_t1.attack(&mut ant_t2);
    ///
    ///     assert!(ant_t1.stats.health == 0 && ant_t2.stats.health == 1);
    /// }
    /// ```
    fn attack(&mut self, enemy: &mut Pet) {
        // TODO: Convert to Result<(), Box<dyn Error>>
        let stat_modifier = self.get_effect_stat_modifier();
        let enemy_stat_modifer = enemy.get_effect_stat_modifier();

        // Any modifiers must apply to ATTACK as we want to only temporarily modify the health attribute of a pet.
        let enemy_atk = (enemy.stats.attack + enemy_stat_modifer.attack)
            .checked_sub(stat_modifier.health)
            .unwrap_or(0);
        let new_health = self.stats.health.checked_sub(enemy_atk).unwrap_or(0);

        let atk = (self.stats.attack + stat_modifier.attack)
            .checked_sub(enemy_stat_modifer.health)
            .unwrap_or(0);
        let new_enemy_health = enemy.stats.health.checked_sub(atk).unwrap_or(0);

        // Decrement number of uses on items, if any.
        self.item.as_mut().map(|item| item.remove_uses(1));
        enemy.item.as_mut().map(|item| item.remove_uses(1));

        // Set the new health of a pet.
        self.stats.health = new_health;
        enemy.stats.health = new_enemy_health;
    }
}

mod tests {
    use crate::common::{
        effect::Statistics,
        food::Food,
        foods::names::FoodName,
        pet::{Combat, Pet},
        pets::names::PetName,
    };

    #[test]
    fn test_attack() {
        let mut ant_t1 = Pet::new(
            PetName::Ant,
            Statistics {
                attack: 2,
                health: 1,
            },
            1,
            None,
        )
        .unwrap();
        let mut ant_t2 = Pet::new(
            PetName::Ant,
            Statistics {
                attack: 2,
                health: 3,
            },
            1,
            None,
        )
        .unwrap();

        ant_t1.attack(&mut ant_t2);

        assert!(ant_t1.stats.health == 0 && ant_t2.stats.health == 1);
    }

    #[test]
    fn test_attack_meat() {}

    #[test]
    fn test_attack_melon() {}

    // #[test]
    // fn test_attack_melon() {

    // }
}
