use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

use crate::common::{
    battle::{
        effect::Modify,
        state::{Action, Outcome, Position, Statistics},
        trigger::*,
    },
    foods::names::FoodName,
    pets::pet::Pet,
};
use std::{collections::VecDeque, fmt::Display, ops::Sub};

use super::pet::{MAX_PET_STATS, MIN_PET_STATS};

pub const MIN_DMG: isize = 1;
pub const MAX_DMG: isize = 150;
const FULL_DMG_NEG_ITEMS: [FoodName; 2] = [FoodName::Coconut, FoodName::Melon];

pub trait Combat {
    /// Perform damage calculation and get new health for self and opponent.
    fn calculate_damage(&self, enemy: &Pet) -> (isize, isize);
    /// Handle the logic of `Pet` interaction during the battle phase.
    /// ```
    /// use sapdb::common::pets::{pet::Pet, names::PetName, combat::Combat};
    ///
    /// let (mut ant_1, mut ant_2) = (Pet::from(PetName::Ant), Pet::from(PetName::Ant));
    ///
    /// assert_eq!(ant_1.stats.health, 1);
    /// assert_eq!(ant_2.stats.health, 1);
    ///
    /// ant_1.attack(&mut ant_2);
    ///
    /// assert_eq!(ant_1.stats.health, 0);
    /// assert_eq!(ant_2.stats.health, 0);
    ///
    /// ```
    /// * Alters a `Pet`'s `stats.attack` and `stats.health`
    /// * Decrements any held `Food`'s `uses` attribute.
    /// * Returns `BattleOutcome` showing status of pets.
    fn attack(&mut self, enemy: &mut Pet) -> BattleOutcome;
    /// Perform a projectile/indirect attack on a `Pet`.
    /// ```
    /// use sapdb::common::{
    ///     pets::{pet::Pet, names::PetName, combat::Combat},
    ///     battle::state::Statistics
    /// };
    ///
    /// let mut ant = Pet::from(PetName::Ant);
    ///
    /// ant.indirect_attack(&Statistics {attack: 2, health: 0});
    ///
    /// assert_eq!(ant.stats.health, 0);
    ///
    /// ```
    fn indirect_attack(&mut self, hit_stats: &Statistics) -> (Vec<Outcome>, Vec<Outcome>);
    /// Get `Outcome` when health is altered.
    fn get_outcome(&self, new_health: isize) -> (Vec<Outcome>, Vec<Outcome>);
    /// Gets the `Statistic` modifiers of held foods that alter a pet's stats during battle.
    ///
    /// If a pet has no held food, no `Statistics` are provided.
    fn get_food_stat_modifier(&self) -> Statistics;
}

impl Combat for Pet {
    fn indirect_attack(&mut self, hit_stats: &Statistics) -> (Vec<Outcome>, Vec<Outcome>) {
        // If pet already dead, return early.
        if self.stats.health == 0 {
            return (vec![], vec![]);
        }
        // Get food status modifier. ex. Melon/Garlic
        let stat_modifier = self.get_food_stat_modifier();
        // Subtract stat_modifer (150/2) from indirect attack.
        let enemy_atk = hit_stats
            .attack
            .sub(stat_modifier.health)
            // Must do a minimum of 1 damage.
            .clamp(MIN_DMG, MAX_DMG);
        let new_health = self
            .stats
            .health
            .sub(enemy_atk)
            .clamp(MIN_PET_STATS, MAX_PET_STATS);

        // Use health difference to determine outcome.
        let outcome = self.get_outcome(new_health);
        // Set new health.
        self.stats.health = new_health.clamp(MIN_PET_STATS, MAX_PET_STATS);
        outcome
    }

    fn get_outcome(&self, new_health: isize) -> (Vec<Outcome>, Vec<Outcome>) {
        let health_diff = self
            .stats
            .health
            .sub(new_health)
            .clamp(MIN_PET_STATS, MAX_PET_STATS);
        let health_diff_stats = Some(Statistics {
            health: health_diff,
            attack: 0,
        });
        let mut outcomes: Vec<Outcome> = vec![];
        let mut enemy_outcomes: Vec<Outcome> = vec![];

        // If difference between health before and after battle is equal the before battle health,
        // pet lost all health during fight and has fainted.
        if health_diff == self.stats.health {
            let [self_faint, any_faint, ahead_faint] =
                get_self_faint_triggers(self.pos, &health_diff_stats);
            let [enemy_faint, enemy_any_faint] =
                get_self_enemy_faint_triggers(self.pos, &health_diff_stats);

            outcomes.extend([self_faint, any_faint, ahead_faint]);
            enemy_outcomes.extend([enemy_faint, enemy_any_faint]);
        } else if health_diff == 0 {
            // If original health - new health is 0, pet wasn't hurt.
            let mut self_unhurt = TRIGGER_SELF_UNHURT;
            (self_unhurt.idx, self_unhurt.stat_diff) = (self.pos, health_diff_stats);

            outcomes.push(self_unhurt)
        } else {
            // Otherwise, pet was hurt.
            let mut self_hurt = TRIGGER_SELF_HURT;
            let mut any_hurt = TRIGGER_ANY_HURT;
            (self_hurt.idx, self_hurt.stat_diff) = (self.pos, health_diff_stats);
            (any_hurt.idx, any_hurt.stat_diff) = (self.pos, health_diff_stats);

            let mut enemy_any_hurt = TRIGGER_ANY_ENEMY_HURT;
            enemy_any_hurt.idx = self.pos;

            outcomes.extend([self_hurt, any_hurt]);
            enemy_outcomes.push(enemy_any_hurt)
        };
        (outcomes, enemy_outcomes)
    }

    fn get_food_stat_modifier(&self) -> Statistics {
        // If a pet has an item that alters stats...
        // Otherwise, no stat modifier added.
        self.item.as_ref().map_or(Statistics::default(), |food| {
            let food_effect = if let Some(n_uses) = food.ability.uses {
                if n_uses > 0 && food.ability.position == Position::OnSelf {
                    // Return the food effect.
                    &food.ability.action
                } else {
                    &Action::None
                }
            } else {
                // None means unlimited uses.
                &food.ability.action
            };

            match food_effect {
                // Get stat modifiers from effects.
                Action::Add(stats) | Action::Remove(stats) => *stats,
                Action::Negate(stats) => {
                    let mut mod_stats = *stats;
                    // Reverse values so that (2 atk, 0 health) -> (0 atk, 2 health).
                    mod_stats.invert();
                    mod_stats
                }
                Action::Critical(prob) => {
                    let mut rng = ChaCha12Rng::seed_from_u64(self.seed);
                    let prob = *prob.clamp(&0, &100) as f64;
                    // Deal double damage if probabilty yields true.
                    let dmg = if rng.gen_bool(prob) {
                        self.stats.attack * 2
                    } else {
                        self.stats.attack
                    };

                    Statistics {
                        attack: dmg,
                        health: 0,
                    }
                }
                // Otherwise, no change.
                _ => Statistics::default(),
            }
        })
    }

    fn calculate_damage(&self, enemy: &Pet) -> (isize, isize) {
        // Get stat modifier from food.
        let stat_modifier = self.get_food_stat_modifier();
        let enemy_stat_modifier = enemy.get_food_stat_modifier();

        // If has melon or coconut, minimum dmg can be 0, Otherwise, should be 1.
        let min_enemy_dmg = if self
            .item
            .as_ref()
            .map_or(false, |food| FULL_DMG_NEG_ITEMS.contains(&food.name))
        {
            0
        } else {
            MIN_DMG
        };
        let min_dmg = if enemy
            .item
            .as_ref()
            .map_or(false, |food| FULL_DMG_NEG_ITEMS.contains(&food.name))
        {
            0
        } else {
            MIN_DMG
        };

        // If has coconut, maximum dmg is 0. Otherwise, the normal 150.
        let max_enemy_dmg = if self
            .item
            .as_ref()
            .map_or(false, |food| food.ability.action == Action::Invincible)
        {
            0
        } else {
            MAX_DMG
        };
        let max_dmg = if enemy
            .item
            .as_ref()
            .map_or(false, |food| food.ability.action == Action::Invincible)
        {
            0
        } else {
            MAX_DMG
        };

        // Any modifiers must apply to ATTACK as we want to only temporarily modify the health attribute of a pet.
        let enemy_atk = (enemy.stats.attack + enemy_stat_modifier.attack)
            .sub(stat_modifier.health)
            .clamp(min_enemy_dmg, max_enemy_dmg);

        let atk = (self.stats.attack + stat_modifier.attack)
            .sub(enemy_stat_modifier.health)
            .clamp(min_dmg, max_dmg);

        // Insta-kill if any amount of damage is dealt and enemy has death's touch.
        let new_health = if enemy_atk != 0
            && enemy
                .item
                .as_ref()
                .map_or(false, |item| item.ability.action == Action::Kill)
        {
            0
        } else {
            self.stats.health.sub(enemy_atk)
        };

        let new_enemy_health = if atk != 0
            && self
                .item
                .as_ref()
                .map_or(false, |item| item.ability.action == Action::Kill)
        {
            0
        } else {
            enemy.stats.health.sub(atk)
        };

        (
            new_health.clamp(MIN_PET_STATS, MAX_PET_STATS),
            new_enemy_health.clamp(MIN_PET_STATS, MAX_PET_STATS),
        )
    }

    fn attack(&mut self, enemy: &mut Pet) -> BattleOutcome {
        let (new_health, new_enemy_health) = self.calculate_damage(enemy);

        // Decrement number of uses on items, if any.
        self.item.as_mut().map(|item| item.ability.remove_uses(1));
        enemy.item.as_mut().map(|item| item.ability.remove_uses(1));

        // Get outcomes for both pets.
        // This doesn't factor in splash effects as pets outside of battle are affected.
        let mut outcome = self.get_outcome(new_health);
        let mut enemy_outcome = enemy.get_outcome(new_enemy_health);

        // Add specific trigger if directly knockout.
        if new_health == 0 {
            enemy_outcome.0.push(TRIGGER_KNOCKOUT)
        }
        if new_enemy_health == 0 {
            outcome.0.push(TRIGGER_KNOCKOUT)
        }

        // Set the new health of a pet.
        self.stats.health = new_health;
        enemy.stats.health = new_enemy_health;

        // Extend outcomes from both sides.
        outcome.0.extend(enemy_outcome.1);
        enemy_outcome.0.extend(outcome.1);

        BattleOutcome {
            friends: VecDeque::from_iter(outcome.0),
            opponents: VecDeque::from_iter(enemy_outcome.0),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct BattleOutcome {
    pub friends: VecDeque<Outcome>,
    pub opponents: VecDeque<Outcome>,
}

impl Display for BattleOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Friends: {:?}\nOpponent: {:?}",
            self.friends, self.opponents,
        )
    }
}
