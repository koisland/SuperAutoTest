use log::info;
use std::rc::Rc;

use crate::{
    effects::{
        state::{Outcome, Status},
        trigger::*,
    },
    error::SAPTestError,
    shop::store::ShopState,
    teams::team::TeamFightOutcome,
    PetCombat, Team, TeamEffects, TeamViewer,
};

/// Clear options for [`Team::clear_team`](crate::teams::combat::TeamCombat::clear_team)
pub enum ClearOption {
    /// Retain empty slots.
    KeepSlots,
    /// Remove empty slots.
    RemoveSlots,
}
impl From<&ClearOption> for bool {
    fn from(value: &ClearOption) -> Self {
        match value {
            ClearOption::KeepSlots => true,
            ClearOption::RemoveSlots => false,
        }
    }
}
/// Enables combat between two [`Team`]s.
/// ```rust no run
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
    /// ```rust no run
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
    ///     teams::combat::ClearOption
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
    /// default_team.clear_team(ClearOption::RemoveSlots);
    ///
    /// assert_eq!(default_team.friends.len(), 0);
    /// ```
    fn clear_team(&mut self, clear_opt: ClearOption) -> &mut Self;
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

        // Apply start of battle effects.
        self.clear_team(ClearOption::KeepSlots);
        opponent.clear_team(ClearOption::KeepSlots);

        // If current phase is 1, add start battle triggers.
        if self.history.curr_phase == 1 {
            self.triggers.push_front(TRIGGER_START_BATTLE)
        }
        if opponent.history.curr_phase == 1 {
            opponent.triggers.push_front(TRIGGER_START_BATTLE)
        }
        while !self.triggers.is_empty() || !opponent.triggers.is_empty() {
            self.trigger_effects(Some(opponent))?;
            opponent.trigger_effects(Some(self))?;
        }

        self.clear_team(ClearOption::KeepSlots);
        opponent.clear_team(ClearOption::KeepSlots);

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

        if let (Some(pet), Some(opponent_pet)) = (self.first(), opponent.first()) {
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

            self.clear_team(ClearOption::KeepSlots);
            opponent.clear_team(ClearOption::KeepSlots);
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
                self.trigger_effects(Some(opponent))?
                    .clear_team(ClearOption::RemoveSlots);
                opponent
                    .trigger_effects(Some(self))?
                    .clear_team(ClearOption::RemoveSlots);
            }
        }

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
                // Add end of battle node.
                let outcome_node = self.history.effect_graph.add_node(outcome_trigger);
                self.history.prev_node = self.history.curr_node;
                self.history.curr_node = Some(outcome_node);
                self.history.last_outcome = Some(outcome_node);
                // On outcome, increase turn count.
                self.history.curr_turn += 1;
                // Return outcome.
                outcome
            },
        )
    }

    fn restore(&mut self) -> &mut Self {
        self.friends = Team::create_rc_pets(&self.stored_friends);
        // Set current pet to first in line.
        self.curr_pet = self.friends.iter().flatten().next().map(Rc::downgrade);
        self.fainted.clear();
        // Set current battle phase to 1.
        self.history.curr_phase = 1;
        self.pet_count = self.stored_friends.len();
        self
    }

    fn clear_team(&mut self, clear_opt: ClearOption) -> &mut Self {
        let mut new_idx = 0;
        self.friends.retain(|slot| {
            if let Some(pet) = slot {
                // Check if not dead.
                if pet.borrow().stats.health != 0 {
                    pet.borrow_mut().pos = Some(new_idx);
                    new_idx += 1;
                    true
                } else {
                    // Pet is dead.
                    info!(target: "run", "(\"{}\")\n{} fainted.", self.name, pet.borrow());
                    self.fainted.push(Some(pet.clone()));
                    false
                }
            } else {
                (&clear_opt).into()
            }
        });
        self
    }
}