use rand::{random, Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

use crate::{
    effects::{
        actions::{Action, StatChangeType},
        effect::Modify,
        state::{Outcome, Position, Status},
        stats::Statistics,
        trigger::*,
    },
    foods::names::FoodName,
    pets::pet::{Pet, MAX_PET_STATS, MIN_PET_STATS},
};

use std::{fmt::Display, ops::Sub};

/// The minimum damage any attack can do.
pub const MIN_DMG: isize = 1;
/// The maximum damage any attack can do.
pub const MAX_DMG: isize = 150;
const FULL_DMG_NEG_ITEMS: [FoodName; 2] = [FoodName::Coconut, FoodName::Melon];
const ALLOWED_FOOD_EFFECT_TRIGGER: [Status; 2] = [Status::AnyDmgCalc, Status::AttackDmgCalc];

/// Gets the maximum damage a pet can receive.
fn max_dmg_received(pet: &Pet) -> isize {
    // If has coconut, maximum dmg is 0. Otherwise, the normal 150.
    if pet.has_active_ability(&Action::Invincible) {
        0
    } else {
        MAX_DMG
    }
}

/// Calculate minimum damage that a pet can receive.
/// * `1` is the default.
fn min_dmg_received(pet: &Pet) -> isize {
    // If has melon or coconut, minimum dmg can be 0, Otherwise, should be 1.
    if pet
        .item
        .as_ref()
        .map_or(false, |food| FULL_DMG_NEG_ITEMS.contains(&food.name))
    {
        0
    } else {
        MIN_DMG
    }
}

/// Final damage calculation considering death's touch and endure actions.
fn final_dmg_calculation(pet: &Pet, dmg: isize, enemy: &Pet) -> isize {
    // Insta-kill if all apply:
    // * Any amount of damage is dealt.
    // * Enemy has death's touch.
    // * Pet being attacked has more health than 1.
    if dmg != 0 && enemy.has_active_ability(&Action::Kill) && pet.stats.health > 1 {
        0
    } else {
        let health = pet.stats.health.sub(dmg);
        // If has endure, stay alive at 1 health.
        // Otherwise do normal damage calculation.
        if pet.has_active_ability(&Action::Endure) {
            health.clamp(1, MAX_PET_STATS)
        } else {
            health.clamp(MIN_PET_STATS, MAX_PET_STATS)
        }
    }
}

/// Implements combat mechanics for a single [`Pet`].
pub trait PetCombat {
    /// Perform damage calculation for a direct [`attack`](crate::PetCombat::attack) returning new health for self and opponent.
    /// # Example
    /// ```
    /// use saptest::{Pet, PetName, PetCombat};
    /// let (ant_1, ant_2) = (
    ///     Pet::try_from(PetName::Ant).unwrap(),
    ///     Pet::try_from(PetName::Ant).unwrap()
    /// );
    /// let (new_ant_1_health, new_ant_2_health) = ant_1.calculate_new_health(&ant_2);
    /// assert!(new_ant_1_health == 0 && new_ant_2_health == 0)
    /// ```
    fn calculate_new_health(&self, enemy: &Pet) -> (isize, isize);

    /// Handle the logic of [`Pet`](crate::Pet) interaction during the battle phase.
    /// * Decrements a held [`Food`](crate::Food) uses.
    /// # Example
    /// ```
    /// use saptest::{Pet, PetName, Food, FoodName, PetCombat};
    ///
    /// let (mut ant_1, mut ant_2) = (
    ///     Pet::try_from(PetName::Ant).unwrap(),
    ///     Pet::try_from(PetName::Ant).unwrap()
    /// );
    /// // Give first ant melon.
    /// ant_1.item = Some(Food::try_from(FoodName::Melon).unwrap());
    ///
    /// // Original stats and effect uses.
    /// assert!(ant_1.stats.health == 1 && ant_2.stats.health == 1);
    /// assert_eq!(ant_1.item.as_ref().unwrap().ability.uses, Some(1));
    ///
    /// // Attack alters attack, health, and held item uses.
    /// ant_1.attack(&mut ant_2);
    /// assert!(ant_1.stats.health == 1 && ant_2.stats.health == 0);
    /// assert_eq!(ant_1.item.as_ref().unwrap().ability.uses, Some(0));
    /// ```
    fn attack(&mut self, enemy: &mut Pet) -> AttackOutcome;

    /// Perform a projectile/indirect attack on a [`Pet`](crate::Pet).
    /// * Health stat in [`Statistics`] is ignored.
    /// # Examples
    /// ```
    /// use saptest::{Pet, PetName, PetCombat, Statistics};
    ///
    /// let mut ant = Pet::try_from(PetName::Ant).unwrap();
    /// assert_eq!(ant.stats.health, 1);
    ///
    /// // Deal damage with attack value of 2.
    /// ant.indirect_attack(&Statistics {attack: 2, health: 0});
    ///
    /// assert_eq!(ant.stats.health, 0);
    /// ```
    fn indirect_attack(&mut self, dmg: &Statistics) -> AttackOutcome;

    /// Get triggers for both pets when health is altered.
    /// # Example
    /// ```
    /// use saptest::{Pet, PetName, PetCombat, effects::trigger::TRIGGER_SELF_UNHURT};
    ///
    /// let mut ant_1 = Pet::try_from(PetName::Ant).unwrap();
    /// // New health is identical.
    /// let outcome = ant_1.get_atk_outcomes(1);
    /// // Unhurt trigger for friends.
    /// assert_eq!(
    ///     outcome.friends.first().unwrap(),
    ///     &TRIGGER_SELF_UNHURT
    /// );
    /// ```
    fn get_atk_outcomes(&self, new_health: isize) -> AttackOutcome;

    /// Gets the [`Statistic`](crate::Statistics) modifiers of held foods that alter a pet's stats during battle.
    /// # Examples
    /// ---
    /// **Nothing** - Gives no additional stats in damage calculation.
    /// ```
    /// use saptest::{Pet, PetName, Statistics, PetCombat};
    ///
    /// let mut ant_1 = Pet::try_from(PetName::Ant).unwrap();
    /// assert_eq!(
    ///     ant_1.get_food_stat_modifier(),
    ///     None
    /// );
    /// ```
    /// ---
    /// **Melon** - Gives `20` additional health in damage calculation.
    /// ```
    /// use saptest::{Pet, PetName, Food, FoodName, Statistics, PetCombat};
    ///
    /// let mut ant_1 = Pet::try_from(PetName::Ant).unwrap();
    /// ant_1.item = Some(Food::try_from(FoodName::Melon).unwrap());
    /// assert_eq!(
    ///     ant_1.get_food_stat_modifier(),
    ///     Some(Statistics::new(0, 20).unwrap())
    /// );
    /// ```
    /// ---
    /// **MeatBone** - Gives `4` additional attack in damage calculation.
    /// ```
    /// use saptest::{Pet, PetName, Food, FoodName, Statistics, PetCombat};
    ///
    /// let mut ant_1 = Pet::try_from(PetName::Ant).unwrap();
    /// ant_1.item = Some(Food::try_from(FoodName::MeatBone).unwrap());
    /// assert_eq!(
    ///     ant_1.get_food_stat_modifier(),
    ///     Some(Statistics::new(4, 0).unwrap())
    /// );
    /// ```
    fn get_food_stat_modifier(&self) -> Option<Statistics>;

    /// Check if a [`Pet`]'s [`Food`](crate::Food) has this [`Action`].
    /// * Returns `false` if out of uses.
    /// # Example
    /// ```
    /// use saptest::{
    ///     Pet, PetName, PetCombat,
    ///     Food, FoodName, effects::actions::Action
    /// };
    /// let mut ant_1 = Pet::try_from(PetName::Ant).unwrap();
    /// ant_1.item = Some(Food::try_from(FoodName::Peanut).unwrap());
    ///
    /// assert!(ant_1.has_active_ability(&Action::Kill))
    /// ```
    fn has_active_ability(&self, ability: &Action) -> bool;
}

impl PetCombat for Pet {
    fn indirect_attack(&mut self, dmg: &Statistics) -> AttackOutcome {
        // If pet already dead, return early.
        if self.stats.health == 0 {
            return AttackOutcome::default();
        }
        // Get food status modifier. ex. Melon/Garlic
        let stat_modifier = self
            .get_food_stat_modifier()
            .unwrap_or(Statistics::default());

        let min_enemy_dmg = min_dmg_received(self);
        let max_enemy_dmg = max_dmg_received(self);
        let enemy_dmg = dmg
            .attack
            .sub(stat_modifier.health)
            // Must do a minimum of 1 damage.
            .clamp(min_enemy_dmg, max_enemy_dmg);

        let mut new_health = self.stats.health.sub(enemy_dmg);

        // Account for endure ability.
        new_health = if self.has_active_ability(&Action::Endure) {
            new_health.clamp(1, MAX_PET_STATS)
        } else {
            new_health.clamp(MIN_PET_STATS, MAX_PET_STATS)
        };

        // Reduce uses from ability if possible.
        self.item.as_mut().map(|item| item.ability.remove_uses(1));

        // Use health difference to determine outcome.
        let outcome = self.get_atk_outcomes(new_health);
        // Set new health.
        self.stats.health = new_health.clamp(MIN_PET_STATS, MAX_PET_STATS);
        outcome
    }

    fn get_atk_outcomes(&self, new_health: isize) -> AttackOutcome {
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
            let [self_faint, any_faint, ahead_faint] = get_self_faint_triggers(&health_diff_stats);
            let [enemy_faint, enemy_any_faint] = get_self_enemy_faint_triggers(&health_diff_stats);

            outcomes.extend([self_faint, any_faint, ahead_faint]);
            enemy_outcomes.extend([enemy_faint, enemy_any_faint]);
        } else if health_diff == 0 {
            // If original health - new health is 0, pet wasn't hurt.
            let mut self_unhurt = TRIGGER_SELF_UNHURT;
            self_unhurt.stat_diff = health_diff_stats;

            outcomes.push(self_unhurt)
        } else {
            // Otherwise, pet was hurt.
            let mut self_hurt = TRIGGER_SELF_HURT;
            let mut any_hurt = TRIGGER_ANY_HURT;
            self_hurt.stat_diff = health_diff_stats;
            any_hurt.stat_diff = health_diff_stats;

            let enemy_any_hurt = TRIGGER_ANY_ENEMY_HURT;

            outcomes.extend([self_hurt, any_hurt]);
            enemy_outcomes.push(enemy_any_hurt)
        };
        AttackOutcome {
            friends: outcomes,
            opponents: enemy_outcomes,
        }
    }

    fn get_food_stat_modifier(&self) -> Option<Statistics> {
        if let Some(food) = self.item.as_ref().filter(|food| {
            food.ability.position == Position::OnSelf
                && ALLOWED_FOOD_EFFECT_TRIGGER.contains(&food.ability.trigger.status)
        }) {
            let food_effect = if let Some(n_uses) = food.ability.uses {
                if n_uses > 0 {
                    // Return the food effect.
                    &food.ability.action
                } else {
                    return None;
                }
            } else {
                // None means unlimited uses.
                &food.ability.action
            };

            match food_effect {
                // Get stat modifiers from effects.
                Action::Add(stat_change) | Action::Remove(stat_change) => match stat_change {
                    StatChangeType::StaticValue(stats) => Some(*stats),
                    StatChangeType::SelfMultValue(stats_mult) => {
                        Some(self.stats.mult_perc(stats_mult))
                    }
                },
                Action::Negate(stats) => {
                    let mut mod_stats = *stats;
                    // Reverse values so that (2 atk, 0 health) -> (0 atk, 2 health).
                    mod_stats.invert();
                    Some(mod_stats)
                }
                Action::Critical(prob) => {
                    let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));
                    let prob = (*prob).clamp(0, 100) as f64 / 100.0;
                    // Deal double damage (Add attack twice) if probabilty yields true.
                    let dmg = if rng.gen_bool(prob) {
                        self.stats.attack
                    } else {
                        0
                    };

                    Some(Statistics {
                        attack: dmg,
                        health: 0,
                    })
                }
                // Otherwise, no change.
                _ => None,
            }
        } else {
            None
        }
    }

    fn has_active_ability(&self, ability: &Action) -> bool {
        if let Some(food) = self.item.as_ref() {
            &food.ability.action == ability && food.ability.uses != Some(0)
        } else {
            false
        }
    }

    fn calculate_new_health(&self, enemy: &Pet) -> (isize, isize) {
        // Get stat modifier from food.
        let stat_modifier = self
            .get_food_stat_modifier()
            .unwrap_or(Statistics::default());
        let enemy_stat_modifier = enemy
            .get_food_stat_modifier()
            .unwrap_or(Statistics::default());

        let min_enemy_dmg = min_dmg_received(self);
        let min_dmg = min_dmg_received(enemy);

        // If has coconut, maximum dmg is 0. Otherwise, the normal 150.
        let max_enemy_dmg = max_dmg_received(self);
        let max_dmg = max_dmg_received(enemy);

        // Any modifiers must apply to ATTACK as we want to only temporarily modify the health attribute of a pet.
        let enemy_dmg = (enemy.stats.attack + enemy_stat_modifier.attack)
            .sub(stat_modifier.health)
            .clamp(min_enemy_dmg, max_enemy_dmg);

        let dmg = (self.stats.attack + stat_modifier.attack)
            .sub(enemy_stat_modifier.health)
            .clamp(min_dmg, max_dmg);

        let new_health = final_dmg_calculation(self, enemy_dmg, enemy);
        let new_enemy_health = final_dmg_calculation(enemy, dmg, self);

        (new_health, new_enemy_health)
    }

    fn attack(&mut self, enemy: &mut Pet) -> AttackOutcome {
        let (new_health, new_enemy_health) = self.calculate_new_health(enemy);

        // Decrement number of uses on items, if any.
        self.item.as_mut().map(|item| item.ability.remove_uses(1));
        enemy.item.as_mut().map(|item| item.ability.remove_uses(1));

        // Get outcomes for both pets.
        // This doesn't factor in splash effects as pets outside of battle are affected.
        let mut outcome = self.get_atk_outcomes(new_health);
        let mut enemy_outcome = enemy.get_atk_outcomes(new_enemy_health);

        // Add specific trigger if directly knockout.
        if new_health == 0 {
            enemy_outcome.friends.push(TRIGGER_KNOCKOUT)
        }
        if new_enemy_health == 0 {
            outcome.friends.push(TRIGGER_KNOCKOUT)
        }

        // Add outcome for attacking pet.
        enemy_outcome.friends.insert(0, TRIGGER_SELF_ATTACK);
        outcome.friends.insert(0, TRIGGER_SELF_ATTACK);

        // Set the new health of a pet.
        self.stats.health = new_health;
        enemy.stats.health = new_enemy_health;

        // Extend outcomes from both sides.
        outcome.friends.extend(enemy_outcome.opponents);
        enemy_outcome.friends.extend(outcome.opponents);

        AttackOutcome {
            friends: Vec::from_iter(outcome.friends),
            opponents: Vec::from_iter(enemy_outcome.friends),
        }
    }
}

/// All [`Outcome`]s of a single attack for friends and enemies.
#[derive(Debug, PartialEq, Default)]
pub struct AttackOutcome {
    /// [`Outcome`] for friends.
    pub friends: Vec<Outcome>,
    /// [`Outcome`] for opponents.
    pub opponents: Vec<Outcome>,
}

impl Display for AttackOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Friends: {:?}\nOpponent: {:?}",
            self.friends, self.opponents,
        )
    }
}
