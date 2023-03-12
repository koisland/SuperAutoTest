use log::info;
use std::sync::Arc;

use crate::{
    effects::{
        actions::{Action, SummonType},
        trigger::*,
    },
    error::SAPTestError,
    shop::store::ShopState,
    teams::team::TeamFightOutcome,
    teams::{effect_helpers::EffectApplyHelpers, history::TeamHistoryHelpers},
    PetCombat, PetName, Team, TeamEffects, TeamViewer, CONFIG,
};

const BATTLE_PHASE_COMPLETE_OUTCOMES: [TeamFightOutcome; 3] = [
    TeamFightOutcome::Win,
    TeamFightOutcome::Loss,
    TeamFightOutcome::Draw,
];

/// Enables combat between two [`Team`]s.
/// ```rust no_run
/// use saptest::TeamCombat;
/// ```
pub trait TeamCombat {
    /// Fight another team for a single battle phase.
    ///
    /// # Examples
    /// ---
    /// To complete the battle.
    /// ```rust
    /// use saptest::{
    ///     Team, TeamCombat, teams::team::TeamFightOutcome,
    ///     Pet, PetName
    /// };
    /// let mut team = Team::new(
    ///     &vec![Some(Pet::try_from(PetName::Cricket).unwrap()); 5],
    ///     5
    /// ).unwrap();
    /// let mut enemy_team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Hippo).unwrap())],
    ///     5
    /// ).unwrap();
    ///
    /// let mut outcome = team.fight(&mut enemy_team).unwrap();
    /// while let TeamFightOutcome::None = outcome {
    ///     outcome = team.fight(&mut enemy_team).unwrap();
    /// }
    ///
    /// assert!(outcome == TeamFightOutcome::Loss);
    /// ```
    /// ---
    /// To complete `n` turns.
    /// ```rust
    /// use saptest::{
    ///     Team, TeamCombat, teams::team::TeamFightOutcome,
    ///     Pet, PetName
    /// };
    /// let mut team = Team::new(
    ///     &vec![Some(Pet::try_from(PetName::Cricket).unwrap()); 5],
    ///     5
    /// ).unwrap();
    /// let mut enemy_team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Hippo).unwrap())],
    ///     5
    /// ).unwrap();
    ///
    /// let n = 2;
    /// let mut outcome = team.fight(&mut enemy_team).unwrap();
    /// for _ in 0..n-1 {
    ///     outcome = team.fight(&mut enemy_team).unwrap();
    /// }
    /// ```
    fn fight(&mut self, opponent: &mut Team) -> Result<TeamFightOutcome, SAPTestError>;

    /// Restore a team to its initial state.
    /// # Example
    /// ```rust no_run
    /// use saptest::{Pet, PetName, Team, TeamCombat, TeamEffects};
    ///
    /// // Empty team.
    /// let mut default_team = Team::default();
    /// default_team
    ///     .add_pet(Pet::try_from(PetName::Dog).unwrap(), 0, None).unwrap()
    ///     // Restore to previous state.
    ///     .restore();
    /// ```
    fn restore(&mut self) -> &mut Self;

    /// Clear team of empty slots and/or fainted pets moving up pets until the next healthiest pet.
    /// # Examples
    /// ```
    /// use saptest::{
    ///     Pet, PetName, Team,
    ///     TeamCombat, TeamViewer, TeamEffects,
    /// };
    ///
    /// let mut default_team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Dog).unwrap())],
    ///     5
    /// ).unwrap();
    ///
    /// assert_eq!(default_team.friends.len(), 1);
    ///
    /// default_team.first().unwrap().write().unwrap().stats.health = 0;
    /// default_team.clear_team();
    ///
    /// assert_eq!(default_team.friends.len(), 0);
    /// ```
    fn clear_team(&mut self) -> &mut Self;
}

impl TeamCombat for Team {
    fn fight(&mut self, opponent: &mut Team) -> Result<TeamFightOutcome, SAPTestError> {
        self.start_battle_phase(opponent)?
            .before_battle_phase(opponent)?
            .battle_phase(opponent)?
            .end_battle_phase(opponent)
    }

    fn restore(&mut self) -> &mut Self {
        // Move fainted and sold pets to friends.
        self.friends.append(&mut self.fainted);
        self.friends.append(&mut self.sold);

        // Keep pet if friend exists in stored friend.
        self.friends.retain(|slot| {
            if let Some(friend) = slot {
                self.stored_friends.iter().flatten().any(|stored_friend| {
                    let mut friend = friend.write().unwrap();
                    if friend.eq(stored_friend) {
                        true
                    } else if stored_friend.id == friend.id {
                        // Replace current attr with stored friends.
                        friend.stats = stored_friend.stats;
                        friend.exp = stored_friend.exp;
                        friend.effect = stored_friend.effect.clone();
                        friend.lvl = stored_friend.lvl;
                        friend.item = stored_friend.item.clone();
                        friend.pos = stored_friend.pos;
                        true
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        });

        let empty_slots = self
            .stored_friends
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| slot.is_none().then_some(i));
        // Sort by idx to ensure slots and pets are separated.
        self.friends
            .sort_by(|slot_1, slot_2| match (slot_1, slot_2) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (Some(pet_1), Some(pet_2)) => {
                    pet_1.read().unwrap().pos.cmp(&pet_2.read().unwrap().pos)
                }
            });
        // Fill empty slots at specific positions accounting for slots already added.
        for slot_idx in empty_slots {
            self.friends.insert(slot_idx, None)
        }
        self.reset_pet_references(None);

        // Set current pet to first in line.
        self.curr_pet = self.friends.iter().flatten().next().map(Arc::downgrade);

        // Set current battle phase to 1.
        self.history.curr_phase = 1;
        self.history.pet_count = self.stored_friends.len();
        self
    }

    fn clear_team(&mut self) -> &mut Self {
        let mut idx = 0;
        self.friends.retain_mut(|slot| {
            // Pet in slot.
            if let Some(pet) = slot
                .as_ref()
                .filter(|pet| pet.read().unwrap().stats.health == 0)
            {
                // Pet is dead, remove from slot.
                info!(target: "run", "(\"{}\")\n{} fainted.", self.name, pet.read().unwrap());
                // Check if pet summons a pet. Remove slot if does.
                // The argument of summon action is not checked only action variant.
                let summon_action = Action::Summon(SummonType::DefaultPet(PetName::None));
                let summons_pets = pet.read().unwrap().has_food_ability(&summon_action, false)
                    || pet
                        .read()
                        .unwrap()
                        .has_effect_ability(&summon_action, false);

                self.fainted.push(slot.take());
                // Remove slot if at first position and not in shop.
                // Keep slot if not at first or in shop.
                if idx == 0 && self.shop.state != ShopState::Open || summons_pets {
                    false
                } else {
                    idx += 1;
                    true
                }
            } else if let Some(pet) = slot {
                // Otherwise, reindex pet.
                pet.write().unwrap().pos = Some(idx);
                idx += 1;
                true
            } else {
                // If no indices (idx == 0) assigned and not in shop, still haven't reached a valid pet.
                // Otherwise, keep incrementing idx to maintain order.
                if idx == 0 && self.shop.state != ShopState::Open {
                    false
                } else {
                    idx += 1;
                    true
                }
            }
        });
        // Trim any empty slots at the end.
        let last_idx = self.friends.iter().enumerate().rev().find_map(|(i, slot)| {
            if slot.is_some() {
                Some(i)
            } else {
                None
            }
        });
        if let Some(last_idx) = last_idx {
            self.friends.truncate(last_idx + 1)
        }
        self
    }
}

trait BattlePhases {
    fn start_battle_phase(&mut self, opponent: &mut Team) -> Result<&mut Self, SAPTestError>;
    fn before_battle_phase(&mut self, opponent: &mut Team) -> Result<&mut Self, SAPTestError>;
    fn battle_phase(&mut self, opponent: &mut Team) -> Result<&mut Self, SAPTestError>;
    fn end_battle_phase(&mut self, opponent: &mut Team) -> Result<TeamFightOutcome, SAPTestError>;
    fn get_battle_outcome(&self, opponent: &Team) -> TeamFightOutcome;
}

impl BattlePhases for Team {
    fn start_battle_phase(&mut self, opponent: &mut Team) -> Result<&mut Self, SAPTestError> {
        // Exit while any shop is open.
        if self.shop.state == ShopState::Open || opponent.shop.state == ShopState::Open {
            return Err(SAPTestError::InvalidTeamAction {
                subject: "Shop Not Closed".to_string(),
                reason:
                    "Cannot fight while one or more teams has an open shop. Call shop.close_shop()"
                        .to_string(),
            });
        }
        if self.name == opponent.name {
            return Err(SAPTestError::InvalidTeamAction {
                subject: "Duplicate Team Name".to_string(),
                reason: "Team names cannot be identical.".to_string(),
            });
        }
        // If already complete, allow to continue to next phases. Nothing will happen.
        if BATTLE_PHASE_COMPLETE_OUTCOMES.contains(&self.get_battle_outcome(opponent)) {
            return Ok(self);
        }

        // Set which team's history should be the primary one.
        self.history.primary_team = true;
        opponent.history.primary_team = false;

        // Move up if not start of battle.
        if self.history.curr_phase != 1 {
            self.clear_team();
            opponent.clear_team();
        }

        info!(target: "run", "(\"{}\")\n{}", self.name, self);
        info!(target: "run", "(\"{}\")\n{}", opponent.name, opponent);

        // If current phase is 1, perform start of battle and update graph (if enabled).
        // Only one team is required to activate this.
        if self.history.curr_phase == 1 {
            if CONFIG.general.build_graph {
                self.history.graph.update(&self.friends, &opponent.friends);
            }
            self.trigger_start_battle_effects(opponent)?;
        }

        // If current phase is 1, add before first battle triggers.
        // Used for butterfly.
        if self.history.curr_phase == 1 {
            self.triggers.push_front(TRIGGER_BEFORE_FIRST_BATTLE)
        }
        if opponent.history.curr_phase == 1 {
            opponent.triggers.push_front(TRIGGER_BEFORE_FIRST_BATTLE)
        }

        Ok(self)
    }

    fn before_battle_phase(&mut self, opponent: &mut Team) -> Result<&mut Self, SAPTestError> {
        // Trigger before attack.
        if let (Some(pet), Some(opponent_pet)) = (self.first(), opponent.first()) {
            self.curr_pet = Some(Arc::downgrade(&pet));
            opponent.curr_pet = Some(Arc::downgrade(&opponent_pet));

            self.triggers.extend([
                TRIGGER_SELF_BEFORE_ATTACK
                    .clone()
                    .set_affected(&pet)
                    .to_owned(),
                TRIGGER_ANY_BEFORE_ATTACK
                    .clone()
                    .set_affected(&pet)
                    .to_owned(),
            ]);
            opponent.triggers.extend([
                TRIGGER_SELF_BEFORE_ATTACK
                    .clone()
                    .set_affected(&opponent_pet)
                    .to_owned(),
                TRIGGER_ANY_BEFORE_ATTACK
                    .clone()
                    .set_affected(&opponent_pet)
                    .to_owned(),
            ]);

            self.trigger_all_effects(opponent)?;
        }

        Ok(self)
    }

    fn battle_phase(&mut self, opponent: &mut Team) -> Result<&mut Self, SAPTestError> {
        // Check that two pets exist and attack.
        // * Turn will end prematurely if no pet at front.
        // * Should not clear/move up teams yet.
        if let (Some(pet), Some(opponent_pet)) = (self.first(), opponent.first()) {
            // Attack and get outcome of fight.
            info!(target: "run", "Fight!\nPet: {}\nOpponent: {}", pet.read().unwrap(), opponent_pet.read().unwrap());
            let mut atk_outcome = pet
                .write()
                .unwrap()
                .attack(&mut opponent_pet.write().unwrap());

            // Check for battle food effects like chili.
            self.apply_battle_food_effect(&pet, opponent)?;
            opponent.apply_battle_food_effect(&opponent_pet, self)?;

            info!(target: "run", "(\"{}\")\n{}", self.name, self);
            info!(target: "run", "(\"{}\")\n{}", opponent.name, opponent);

            // Update outcomes with weak references.
            for trigger in atk_outcome.friends.iter_mut() {
                trigger.set_affected(&pet).set_afflicting(&opponent_pet);
            }
            for trigger in atk_outcome.opponents.iter_mut() {
                trigger.set_affected(&opponent_pet).set_afflicting(&pet);
            }

            if CONFIG.general.build_graph {
                self.add_hurt_and_attack_edges(&pet, &opponent_pet, &atk_outcome)?;
            }

            // Add triggers to team from outcome of battle.
            self.triggers.extend(atk_outcome.friends.into_iter());
            opponent.triggers.extend(atk_outcome.opponents.into_iter());

            // Add triggers for pet behind.
            if let Some(pet_behind) = opponent.nth(1) {
                opponent.triggers.push_back(
                    TRIGGER_AHEAD_ATTACK
                        .clone()
                        .set_affected(&pet_behind)
                        .to_owned(),
                )
            }
            if let Some(pet_behind) = self.nth(1) {
                self.triggers.push_back(
                    TRIGGER_AHEAD_ATTACK
                        .clone()
                        .set_affected(&pet_behind)
                        .to_owned(),
                )
            }

            // Apply effect triggers from combat phase.
            self.trigger_all_effects(opponent)?;
        }

        Ok(self)
    }

    fn end_battle_phase(&mut self, opponent: &mut Team) -> Result<TeamFightOutcome, SAPTestError> {
        // Increment battle phase counter.
        // A battle phase is a single direct attack between pets.
        self.history.curr_phase += 1;
        opponent.history.curr_phase += 1;

        // Clear any fainted pets in case where first slot on either team is empty or battle phase interrupted.
        self.clear_team();
        opponent.clear_team();

        // Replace opponent graph.
        if CONFIG.general.build_graph {
            opponent.history.graph = self.history.graph.clone();
        }

        // Check outcome.
        let outcome = self.get_battle_outcome(opponent);
        // Update history.
        if BATTLE_PHASE_COMPLETE_OUTCOMES.contains(&outcome) {
            opponent.history.fight_outcomes.push(outcome.inverse());
            self.history.fight_outcomes.push(outcome.clone());

            // On outcome, increase turn count.
            self.history.curr_turn += 1;
            opponent.history.curr_turn += 1;
        };

        Ok(outcome)
    }

    fn get_battle_outcome(&self, opponent: &Team) -> TeamFightOutcome {
        let alive_friends = self.all();
        let alive_opponents = opponent.all();

        // Still friends alive, battle outcome not decided.
        if !alive_friends.is_empty() && !alive_opponents.is_empty() {
            TeamFightOutcome::None
        } else if alive_friends.is_empty() && alive_opponents.is_empty() {
            TeamFightOutcome::Draw
        } else if !opponent.friends.is_empty() {
            TeamFightOutcome::Loss
        } else {
            TeamFightOutcome::Win
        }
    }
}
