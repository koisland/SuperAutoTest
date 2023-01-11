use crate::common::{
    battle::{
        effect::Modify,
        state::{Action, Outcome, Position, Statistics, Target},
        team::Team,
        trigger::*,
    },
    pets::pet::Pet,
};
use std::{collections::VecDeque, fmt::Display};

const MIN_DMG: usize = 1;
const MAX_DMG: usize = 150;

pub trait Combat {
    /// Handle the logic of a single pet interaction during the battle phase.
    /// * Alters a `Pet`'s `stats.attack` and `stats.health`
    /// * Decrements any held `Food`'s `uses` attribute.
    /// * Returns `BattleOutcome` showing status of pets.
    ///
    /// ```
    ///let ant_1 = Pet::new(PetName::Ant,
    ///    Statistics {attack: 2, health: 1}, 1, None).unwrap();
    ///let ant_2 = Pet::new(PetName::Ant,
    ///    Statistics {attack: 2, health: 3}, 1, None).unwrap();
    ///
    ///let outcome = ant_t1.attack(&mut ant_t2);
    /// ```
    fn attack(&mut self, enemy: &mut Pet) -> BattleOutcome;
    /// Perform a projectile/indirect attack on a `Pet`.
    /// * `:param hit_stats:`
    ///     * Amount of `Statistics` to reduce a `Pet`'s stats by.
    ///
    /// Returns:
    /// * `Outcome`(s) of attack.
    fn indirect_attack(&mut self, hit_stats: &Statistics) -> Vec<Outcome>;
    /// Apply an `Effect` of a `Food` that:
    ///  * Targets a `Pet` other than itself during combat
    ///  * Does damage remove some `Statistics`.
    ///  * And affects a specific `Position`.
    fn apply_food_effect(&mut self, opponent: &mut Team);
    /// Get `Outcome` when health is altered.
    fn get_outcome(&self, new_health: usize) -> Vec<Outcome>;
    /// Gets the `Statistic` modifiers of held foods that alter a pet's stats during battle.
    ///
    /// If a pet has no held food, no `Statistics` are provided.
    fn get_food_stat_modifier(&self) -> Statistics;
    /// Check if pet is alive.
    fn is_alive(&self) -> bool;
}

impl Combat for Pet {
    fn is_alive(&self) -> bool {
        self.stats.health != 0
    }

    fn indirect_attack(&mut self, hit_stats: &Statistics) -> Vec<Outcome> {
        // Get food status modifier. ex. Melon/Garlic
        let stat_modifier = self.get_food_stat_modifier();
        // Subtract stat_modifer (150/2) from indirect attack.
        let enemy_atk = hit_stats
            .attack
            .saturating_sub(stat_modifier.health)
            // Must do a minimum of 1 damage.
            .clamp(MIN_DMG, MAX_DMG);
        let new_health = self.stats.health.saturating_sub(enemy_atk);

        // Use health difference to determine outcome.
        let outcome = self.get_outcome(new_health);
        // Set new health.
        self.stats.health = new_health;
        outcome
    }

    fn apply_food_effect(&mut self, opponent: &mut Team) {
        if let Some(food) = self.item.as_mut() {
            let food_effect = &food.ability;
            // Food targets an enemy at a specific position and removes stats.
            if let (Target::Enemy, Action::Remove(stats), Position::Specific(idx)) = (
                &food_effect.target,
                &food_effect.action,
                &food_effect.position,
            ) {
                let adj_idx =
                    usize::try_from(*idx).expect("Unable to coerce specific idx to usize.");
                if let Some(target) = opponent
                    .friends
                    .borrow_mut()
                    .get_mut(adj_idx)
                    .map(|pet| pet.as_mut().unwrap())
                {
                    let indir_atk_outcome = target.borrow_mut().indirect_attack(stats);

                    opponent.triggers.borrow_mut().extend(indir_atk_outcome);
                    food.ability.remove_uses(1);
                }
            }
        }
    }

    fn get_outcome(&self, new_health: usize) -> Vec<Outcome> {
        let health_diff = self.stats.health.saturating_sub(new_health);
        let mut outcomes: Vec<Outcome> = vec![];
        if health_diff == self.stats.health {
            // If difference between health before and after battle is equal the before battle health,
            // pet lost all health during fight and has fainted.
            let (mut self_faint, mut any_faint) = (TRIGGER_SELF_FAINT, TRIGGER_ANY_FAINT);
            (self_faint.idx, any_faint.idx) = (self.pos, self.pos);

            outcomes.extend([self_faint, any_faint])
        } else if health_diff == 0 {
            // If original health - new health is 0, pet wasn't hurt.
            let mut self_unhurt = TRIGGER_SELF_UNHURT;
            self_unhurt.idx = self.pos;

            outcomes.push(self_unhurt)
        } else {
            // Otherwise, pet was hurt.
            let mut self_hurt = TRIGGER_SELF_HURT;
            self_hurt.idx = self.pos;

            outcomes.push(self_hurt)
        };
        outcomes
    }

    fn get_food_stat_modifier(&self) -> Statistics {
        // If a pet has an item that alters stats...
        // Otherwise, no stat modifier added.
        self.item.as_ref().map_or(
            Statistics {
                attack: 0,
                health: 0,
            },
            |food| {
                let food_effect = food.ability.uses.as_ref().map_or(&Action::None, |uses| {
                    if *uses > 0 && food.ability.position == Position::OnSelf {
                        // Return the food effect.
                        &food.ability.action
                    } else {
                        &Action::None
                    }
                });

                match food_effect {
                    // Get stat modifiers from effects.
                    Action::Add(stats) | Action::Remove(stats) | Action::Negate(stats) => {
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

    fn attack(&mut self, enemy: &mut Pet) -> BattleOutcome {
        // Get stat modifier from food.
        let stat_modifier = self.get_food_stat_modifier();
        let enemy_stat_modifier = enemy.get_food_stat_modifier();

        // Any modifiers must apply to ATTACK as we want to only temporarily modify the health attribute of a pet.
        let enemy_atk = (enemy.stats.attack + enemy_stat_modifier.attack)
            .saturating_sub(stat_modifier.health)
            .clamp(MIN_DMG, MAX_DMG);
        let new_health = self.stats.health.saturating_sub(enemy_atk);

        let atk = (self.stats.attack + stat_modifier.attack)
            .saturating_sub(enemy_stat_modifier.health)
            .clamp(MIN_DMG, MAX_DMG);
        let new_enemy_health = enemy.stats.health.saturating_sub(atk);

        // Decrement number of uses on items, if any.
        self.item.as_mut().map(|item| item.ability.remove_uses(1));
        enemy.item.as_mut().map(|item| item.ability.remove_uses(1));

        // Get outcomes for both pets.
        // This doesn't factor in splash effects as pets outside of battle are affected.
        let outcome = self.get_outcome(new_health);
        let enemy_outcome = enemy.get_outcome(new_enemy_health);

        // Set the new health of a pet.
        self.stats.health = new_health;
        enemy.stats.health = new_enemy_health;

        BattleOutcome {
            friends: VecDeque::from_iter(outcome),
            opponents: VecDeque::from_iter(
                enemy_outcome,
                // outcome,
            ),
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
