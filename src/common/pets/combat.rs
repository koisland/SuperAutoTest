use crate::common::{
    battle::{
        effect::Modify,
        state::{Action, Outcome, Position, Statistics, Target},
        team::Team,
        trigger::*,
    },
    foods::names::FoodName,
    pets::pet::Pet,
};
use std::{collections::VecDeque, fmt::Display};

const MIN_DMG: usize = 1;
const MAX_DMG: usize = 150;
const FULL_DMG_NEG_ITEMS: [Option<FoodName>; 2] = [Some(FoodName::Coconut), Some(FoodName::Melon)];

pub trait Combat {
    /// Perform damage calculation and get new health for self and opponent.
    fn calculate_damage(&self, enemy: &Pet) -> (usize, usize);
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
}

impl Combat for Pet {
    fn indirect_attack(&mut self, hit_stats: &Statistics) -> Vec<Outcome> {
        // If pet already dead, return early.
        if self.stats.health == 0 {
            return vec![];
        }
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
            let (mut self_faint, mut any_faint, mut ahead_faint) =
                (TRIGGER_SELF_FAINT, TRIGGER_ANY_FAINT, TRIGGER_AHEAD_FAINT);
            (self_faint.idx, any_faint.idx, ahead_faint.idx) =
                (self.pos, self.pos, self.pos.map(|pos| pos + 1));

            outcomes.extend([self_faint, any_faint, ahead_faint])
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
        self.item.as_ref().map_or(Statistics::default(), |food| {
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
                Action::Add(stats) | Action::Remove(stats) | Action::Negate(stats) => stats.clone(),
                // Otherwise, no change.
                _ => Statistics::default(),
            }
        })
    }

    fn calculate_damage(&self, enemy: &Pet) -> (usize, usize) {
        // Get stat modifier from food.
        let stat_modifier = self.get_food_stat_modifier();
        let enemy_stat_modifier = enemy.get_food_stat_modifier();

        // If has melon or coconut, minimum dmg can be 0, Otherwise, should be 1.
        let min_enemy_dmg =
            if FULL_DMG_NEG_ITEMS.contains(&self.item.as_ref().map(|food| food.name.clone())) {
                0
            } else {
                MIN_DMG
            };
        let min_dmg =
            if FULL_DMG_NEG_ITEMS.contains(&enemy.item.as_ref().map(|food| food.name.clone())) {
                0
            } else {
                MIN_DMG
            };

        // Any modifiers must apply to ATTACK as we want to only temporarily modify the health attribute of a pet.
        let enemy_atk = (enemy.stats.attack + enemy_stat_modifier.attack)
            .saturating_sub(stat_modifier.health)
            .clamp(min_enemy_dmg, MAX_DMG);

        let atk = (self.stats.attack + stat_modifier.attack)
            .saturating_sub(enemy_stat_modifier.health)
            .clamp(min_dmg, MAX_DMG);

        let new_health = self.stats.health.saturating_sub(enemy_atk);
        let new_enemy_health = enemy.stats.health.saturating_sub(atk);
        (new_health, new_enemy_health)
    }

    fn attack(&mut self, enemy: &mut Pet) -> BattleOutcome {
        let (new_health, new_enemy_health) = self.calculate_damage(enemy);

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
            opponents: VecDeque::from_iter(enemy_outcome),
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
