use log::info;
use std::rc::Rc;

use crate::{
    effects::{
        actions::{Action, SummonType},
        state::{Outcome, Status},
        trigger::*,
    },
    error::SAPTestError,
    shop::store::ShopState,
    teams::team::TeamFightOutcome,
    PetCombat, Team, TeamEffects, TeamViewer,
};

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

    /// Clear team of empty slots and/or fainted pets and reset indices.
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
    /// default_team.first().unwrap().borrow_mut().stats.health = 0;
    /// default_team.clear_team();
    ///
    /// assert_eq!(default_team.friends.len(), 0);
    /// ```
    fn clear_team(&mut self) -> &mut Self;
}

impl TeamCombat for Team {
    fn fight(&mut self, opponent: &mut Team) -> Result<TeamFightOutcome, SAPTestError> {
        // Exit while any shop is open.
        if self.shop.state == ShopState::Open || opponent.shop.state == ShopState::Open {
            return Err(SAPTestError::InvalidTeamAction {
                subject: "Shop Not Closed".to_string(),
                reason:
                    "Cannot fight while one or more teams has an open shop. Call shop.close_shop()"
                        .to_string(),
            });
        }

        info!(target: "run", "(\"{}\")\n{}", self.name, self);
        info!(target: "run", "(\"{}\")\n{}", opponent.name, opponent);

        // Clear empty slots at front. pushing pets forward.
        self.clear_team();
        opponent.clear_team();

        // If current phase is 1, perform start of battle.
        // Only one team is required to activate this.
        if self.history.curr_phase == 1 {
            self.trigger_start_battle_effects(opponent)?;
        }

        self.clear_team();
        opponent.clear_team();

        // If current phase is 1, add before first battle triggers.
        // Used for butterfly.
        if self.history.curr_phase == 1 {
            self.triggers.push_front(TRIGGER_BEFORE_FIRST_BATTLE)
        }
        if opponent.history.curr_phase == 1 {
            opponent.triggers.push_front(TRIGGER_BEFORE_FIRST_BATTLE)
        }

        // Increment battle phase counter.
        self.history.curr_phase += 1;
        opponent.history.curr_phase += 1;

        if let (Some(pet), Some(opponent_pet)) = (self.first(), opponent.first()) {
            self.curr_pet = Some(Rc::downgrade(&pet));
            opponent.curr_pet = Some(Rc::downgrade(&opponent_pet));

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

            while !self.triggers.is_empty() || !opponent.triggers.is_empty() {
                self.trigger_effects(Some(opponent))?;
                opponent.trigger_effects(Some(self))?;
            }
            // Turn will end prematurely if no pet at front.
            // Should not clear team yet.
        }

        // Check that two pets exist and attack.
        // Attack will result in triggers being added.
        if let (Some(pet), Some(opponent_pet)) = (self.first(), opponent.first()) {
            // Attack and get outcome of fight.
            info!(target: "run", "Fight!\nPet: {}\nOpponent: {}", pet.borrow(), opponent_pet.borrow());
            let mut outcome = pet.borrow_mut().attack(&mut opponent_pet.borrow_mut());
            info!(target: "run", "(\"{}\")\n{}", self.name, self);
            info!(target: "run", "(\"{}\")\n{}", opponent.name, opponent);

            // Update outcomes with weak references.
            for trigger in outcome.friends.iter_mut() {
                trigger.set_affected(&pet).set_afflicting(&opponent_pet);
            }
            for trigger in outcome.opponents.iter_mut() {
                trigger.set_affected(&opponent_pet).set_afflicting(&pet);
            }

            // Create node for hurt and attack status.
            if let Some(trigger) = outcome
                .friends
                .iter()
                .find(|trigger| trigger.status == Status::Hurt || trigger.status == Status::Attack)
            {
                self.create_node(trigger);
            }

            if let Some(trigger) = outcome
                .opponents
                .iter()
                .find(|trigger| trigger.status == Status::Hurt || trigger.status == Status::Attack)
            {
                opponent.create_node(trigger);
            }

            // Add triggers to team from outcome of battle.
            self.triggers.extend(outcome.friends.into_iter());
            opponent.triggers.extend(outcome.opponents.into_iter());

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
            while !self.triggers.is_empty() || !opponent.triggers.is_empty() {
                self.trigger_effects(Some(opponent))?;
                opponent.trigger_effects(Some(self))?;
            }

            self.clear_team();
            opponent.clear_team();
        }

        // Clear any fainted pets in case where first slot on either team is empty and battle phase interrupted.
        self.clear_team();
        opponent.clear_team();

        // Check if battle complete.
        Ok(
            if !self.friends.is_empty() && !opponent.friends.is_empty() {
                TeamFightOutcome::None
            } else {
                let outcome = if self.friends.is_empty() && opponent.friends.is_empty() {
                    info!(target: "run", "Draw!");
                    TeamFightOutcome::Draw
                } else if !opponent.friends.is_empty() {
                    info!(target: "run", "Enemy team won...");
                    TeamFightOutcome::Loss
                } else {
                    info!(target: "run", "Your team won!");
                    TeamFightOutcome::Win
                };
                let outcome_trigger: Outcome = (&outcome).into();
                let opponent_outcome_trigger: Outcome = (&outcome.inverse()).into();
                // Add end of battle node.
                let outcome_node = self.history.effect_graph.add_node(outcome_trigger);
                let opponent_outcome_node = opponent
                    .history
                    .effect_graph
                    .add_node(opponent_outcome_trigger);
                self.history.prev_node = self.history.curr_node;
                self.history.curr_node = Some(outcome_node);
                self.history.last_outcome = Some(outcome_node);

                opponent.history.prev_node = opponent.history.curr_node;
                opponent.history.curr_node = Some(opponent_outcome_node);
                opponent.history.last_outcome = Some(opponent_outcome_node);
                // On outcome, increase turn count.
                self.history.curr_turn += 1;
                opponent.history.curr_turn += 1;
                // Return outcome.
                outcome
            },
        )
    }

    fn restore(&mut self) -> &mut Self {
        // Move fainted and sold pets to friends.
        self.friends.append(&mut self.fainted);
        self.friends.append(&mut self.sold);

        // Keep pet if friend exists in stored friend.
        self.friends.retain(|slot| {
            if let Some(friend) = slot {
                self.stored_friends.iter().flatten().any(|stored_friend| {
                    if friend.borrow().eq(stored_friend) {
                        true
                    } else if stored_friend.id == friend.borrow().id {
                        // Replace current attr with stored friends.
                        friend.borrow_mut().stats = stored_friend.stats;
                        friend.borrow_mut().exp = stored_friend.exp;
                        friend.borrow_mut().effect = stored_friend.effect.clone();
                        friend.borrow_mut().lvl = stored_friend.lvl;
                        friend.borrow_mut().item = stored_friend.item.clone();
                        friend.borrow_mut().pos = stored_friend.pos;
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
        // Sort by idx.
        self.friends
            .sort_by(|slot_1, slot_2| match (slot_1, slot_2) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (Some(pet_1), Some(pet_2)) => pet_1.borrow().pos.cmp(&pet_2.borrow().pos),
            });
        // Fill empty slots at specific positions accounting for slots already added.
        for slot_idx in empty_slots {
            self.friends.insert(slot_idx, None)
        }
        self.reset_pet_references(None);

        // Set current pet to first in line.
        self.curr_pet = self.friends.iter().flatten().next().map(Rc::downgrade);

        // Set current battle phase to 1.
        self.history.curr_phase = 1;
        self.pet_count = self.stored_friends.len();
        self
    }

    fn clear_team(&mut self) -> &mut Self {
        let mut idx = 0;
        self.friends.retain_mut(|slot| {
            // Pet in slot.
            if let Some(pet) = slot.as_ref().filter(|pet| pet.borrow().stats.health == 0) {
                // Pet is dead, remove from slot.
                info!(target: "run", "(\"{}\")\n{} fainted.", self.name, pet.borrow());
                // Check if pet summons a pet. Remove slot if does.
                // The argument of summon action is not checked only action variant.
                let summon_action = Action::Summon(SummonType::SelfTierPet);
                let summons_pets = pet.borrow().has_food_ability(&summon_action, false)
                    || pet.borrow().has_effect_ability(&summon_action, false);

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
                pet.borrow_mut().pos = Some(idx);
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
        self
    }
}
