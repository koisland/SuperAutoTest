use crate::{
    effects::{
        actions::Action,
        effect::{Effect, EffectModify},
        state::{Outcome, Position, Status, Target},
        trigger::*,
    },
    error::SAPTestError,
    shop::trigger::*,
    teams::{
        effect_helpers::{is_pet_effect_exception, EffectApplyHelpers},
        team::Team,
        viewer::TeamViewer,
    },
    Pet, PetCombat,
};
use itertools::Itertools;

use std::{cell::RefCell, collections::VecDeque, rc::Rc};

const NON_COMBAT_TRIGGERS: [Outcome; 11] = [
    TRIGGER_ANY_LEVELUP,
    TRIGGER_START_TURN,
    TRIGGER_START_BATTLE,
    TRIGGER_BEFORE_FIRST_BATTLE,
    TRIGGER_END_TURN,
    TRIGGER_ANY_FOOD_BOUGHT,
    TRIGGER_ANY_FOOD_EATEN,
    TRIGGER_ANY_PET_BOUGHT,
    TRIGGER_ANY_PET_SOLD,
    TRIGGER_ROLL,
    TRIGGER_SHOP_TIER_UPGRADED,
];

fn knockout_pet_caused_knockout(team: &Team, pet: &Rc<RefCell<Pet>>) -> bool {
    team.triggers.iter().any(|trigger| {
        if trigger.status == Status::KnockOut
            && pet.borrow().has_effect_trigger(&Status::KnockOut, true)
        {
            if let Some(affected_pet) = &trigger.affected_pet {
                affected_pet.ptr_eq(&Rc::downgrade(pet))
            } else {
                false
            }
        } else {
            false
        }
    })
}

/// Enable applying [`Effect`]s to multiple [`Team`]s.
/// ```rust no_run
/// use saptest::TeamEffects;
/// ```
pub trait TeamEffects {
    /// Trigger all effects from both teams.
    /// * This exhausts all effect [`Outcome`] triggers.
    /// * Fainted [`Pet`]s are not removed.
    /// * Updates cycle for iteration needed to completely empty both teams.
    fn trigger_all_effects(&mut self, opponent: &mut Team) -> Result<&mut Self, SAPTestError>;

    /// Trigger the start of battle [`Pet`] [`Effect`]s for two [`Team`]s.
    /// * No [`Food`](crate::Food) abilities trigger with [`TRIGGER_START_BATTLE`](crate::effects::trigger::TRIGGER_START_BATTLE) so this functionality is not yet supported.
    /// * Invocation order does not matter with a [`Team`].
    ///     * `team.trigger_start_battle_effects(&mut enemy_team)` or its reverse will not alter the outcome.
    /// * This takes all [`Pet`]s into consideration unlike [`trigger_effects`](TeamEffects::trigger_effects) which only activates effects from a single [`Team`].
    /// * This exhausts all effect [`Outcome`] triggers.
    /// * Fainted [`Pet`]s are not removed.
    /// # Example
    /// ```
    /// use saptest::{
    ///     Team, TeamEffects, TeamViewer,
    ///     Pet, PetName, Statistics
    /// };
    ///
    /// // Dolphin at base (4,3).
    /// let dolphin = Pet::try_from(PetName::Dolphin).unwrap();
    /// // Caterpillar at level 3 with (3,4). Activates after dolphin.
    /// let mut caterpillar = Pet::try_from(PetName::Caterpillar).unwrap();
    /// caterpillar.set_level(3).unwrap();
    /// caterpillar.stats = Statistics::new(3,4).unwrap();
    ///
    /// let mut team = Team::new(&[Some(caterpillar)], 5).unwrap();
    /// let mut enemy_team = Team::new(&[Some(dolphin)], 5).unwrap();
    /// team.trigger_start_battle_effects(&mut enemy_team).unwrap();
    ///
    /// let butterfly = team.first().unwrap();
    /// assert!(
    ///     butterfly.borrow().stats == Statistics {attack: 1, health: 1} &&
    ///     butterfly.borrow().name == PetName::Butterfly
    /// )
    /// ```
    fn trigger_start_battle_effects(
        &mut self,
        opponent: &mut Team,
    ) -> Result<&mut Self, SAPTestError>;

    /// Apply [`Pet`](crate::pets::pet::Pet) [`Effect`]s that would be triggered by a given [`Outcome`].
    /// * This only applies effects on a **single** [`Team`] for the single [`Outcome`] trigger given.
    ///     * Start of battle effects should be handled by [`trigger_start_battle_effects`](TeamEffects::trigger_start_battle_effects).
    ///     * Non-opponent affecting effects can be activated by adding a [`TRIGGER_START_BATTLE`](crate::effects::trigger::TRIGGER_START_BATTLE).
    ///     * Use [`trigger_all_effects`](TeamEffects::trigger_all_effects) to completely expend any triggers.
    /// * This only applies [`Pet`] effects.
    ///     * Use [`trigger_items`](TeamEffects::trigger_items) for item effects.
    /// * Fainted [`Pet`]s are not removed.
    /// # Example
    /// ```rust
    /// use saptest::{
    ///     TeamEffects, Team, TeamViewer,
    ///     Pet, PetName,
    ///     effects::{state::Status, trigger::{TRIGGER_START_BATTLE, TRIGGER_SELF_HURT}}
    /// };
    ///
    /// let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// let mut team = Team::new(&vec![Some(mosquito); 5], 5).unwrap();
    /// let mut enemy_team = team.clone();
    /// team.set_seed(Some(12));
    /// enemy_team.set_seed(Some(12));
    ///
    /// // Trigger start of battle effects.
    /// team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team)).unwrap();
    ///
    /// // Triggers not exhausted.
    /// // Enemy team hurt by mosquito barrage.
    /// assert_eq!(team.triggers.len(), 9);
    /// assert!(enemy_team.triggers.iter().any(|trigger| matches!(trigger.status, Status::Hurt)));
    /// ```
    fn trigger_effects(
        &mut self,
        trigger: &Outcome,
        opponent: Option<&mut Team>,
    ) -> Result<&mut Self, SAPTestError>;

    /// Apply a [`Pet`](crate::pets::pet::Pet)'s [`Food`](crate::Food) [`Effect`]s that would be triggered by a given [`Outcome`].
    /// * This only applies effects on a **single** [`Team`] for the single [`Outcome`] trigger given.
    /// * Fainted [`Pet`]s are not removed.
    /// # Example
    /// ```rust
    /// use saptest::{
    ///     TeamEffects, Team, TeamViewer,
    ///     Pet, PetName, Position, Food, FoodName,
    ///     effects::{state::Status, trigger::TRIGGER_SELF_FAINT}
    /// };
    ///
    /// let mut mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// mosquito.item = Some(Food::try_from(FoodName::Honey).unwrap());
    /// let mut team = Team::new(&vec![Some(mosquito)], 5).unwrap();
    /// let mut faint_trigger = TRIGGER_SELF_FAINT;
    /// faint_trigger.set_affected(&team.first().unwrap());
    ///
    /// // Trigger item effects.
    /// team.trigger_items(&faint_trigger, None).unwrap();
    ///
    /// let first_pet = team.first().unwrap();
    /// assert_eq!(
    ///     first_pet.borrow().name,
    ///     PetName::Bee
    /// );
    /// ```
    fn trigger_items(
        &mut self,
        trigger: &Outcome,
        opponent: Option<&mut Team>,
    ) -> Result<&mut Self, SAPTestError>;

    /// Apply an [`Effect`] with an associated [`Outcome`] trigger to a [`Team`].
    /// * The `opponent` [`Team`] will get updated with additional [`Outcome`]s.
    /// * Effects and triggers should contain a Weak reference to the owning/affecting pet.
    /// # Examples
    /// ```rust
    /// use saptest::{
    ///     TeamEffects, Team, TeamViewer,
    ///     Pet, PetName,
    ///     Statistics, effects::{state::Status, trigger::*}
    /// };
    /// // Get mosquito effect.
    /// let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// // Get effect with no reference.
    /// let no_ref_mosquito_effect = mosquito.effect.first().cloned().unwrap();
    ///
    /// // Init teams.
    /// let mut team = Team::new(&vec![Some(mosquito.clone()); 5], 5).unwrap();
    /// let mut enemy_team = Team::new(&vec![Some(mosquito); 5], 5).unwrap();
    /// team.set_seed(Some(0));
    ///
    /// // Without a reference to the pet owning the effect, this will fail.
    /// assert!(team.apply_effect(&TRIGGER_START_BATTLE, &no_ref_mosquito_effect, Some(&mut enemy_team)).is_err());
    ///
    /// // Get mosquito_effect with reference.
    /// // Apply effect of mosquito at position 0 to a pet on team to enemy team.
    /// let mosquito_effect = team.first().unwrap().borrow().effect.first().cloned().unwrap();
    /// team.apply_effect(&TRIGGER_START_BATTLE, &mosquito_effect, Some(&mut enemy_team)).unwrap();
    ///
    /// // Last enemy mosquito takes one damage and opponent triggers gets updated.
    /// assert_eq!(
    ///     enemy_team.nth(1).unwrap().borrow().stats,
    ///     Statistics::new(2, 1).unwrap()
    /// );
    /// assert!(
    ///     enemy_team.triggers
    ///     .iter()
    ///     .find(|trigger| trigger.status == Status::Hurt)
    ///     .is_some()
    /// );
    /// ```
    fn apply_effect(
        &mut self,
        trigger: &Outcome,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError>;

    /// Get effect order for a single [`Team`].
    /// * Order is found by going from highest to lowest [`Pet`] attack.
    /// * If in battle:
    ///     * The first pet on the team is always first in effect priority.
    fn get_pet_effect_order(&self, in_battle: bool) -> Vec<Rc<RefCell<Pet>>>;
}

impl TeamEffects for Team {
    fn get_pet_effect_order(&self, in_battle: bool) -> Vec<Rc<RefCell<Pet>>> {
        let mut ordered_pets = self
            .friends
            .iter()
            .flatten()
            .sorted_by(|pet_1, pet_2| {
                pet_1
                    .borrow()
                    .stats
                    .attack
                    .cmp(&pet_2.borrow().stats.attack)
            })
            .rev()
            .cloned()
            .collect_vec();

        let curr_pet = self.curr_pet.as_ref().map(|pet| pet.upgrade());

        if let Some(Some(first_pet)) = curr_pet {
            if in_battle {
                // Remove first pet from ordered pets.
                ordered_pets.retain(|pet| !Rc::ptr_eq(pet, &first_pet));
                // And insert as first. Current pet always activates effect first.
                ordered_pets.insert(0, first_pet);
            }
        }
        ordered_pets
    }

    fn trigger_all_effects(&mut self, opponent: &mut Team) -> Result<&mut Self, SAPTestError> {
        // Get first pet. Teams should not be cleared at any step.
        let first_pet = self.friends.iter().flatten().next().cloned();
        let first_enemy_pet = opponent.friends.iter().flatten().next().cloned();

        // Find which team has the strongest friend at the front of their team.
        // Exception if pet at front with knockout effect caused knockout, that pet's team takes priority.
        let opponent_first =
            if let (Some(friend), Some(enemy)) = (first_pet.as_ref(), first_enemy_pet.as_ref()) {
                if knockout_pet_caused_knockout(self, friend) {
                    false
                } else if knockout_pet_caused_knockout(opponent, enemy) {
                    true
                } else {
                    friend.borrow().stats.attack > enemy.borrow().stats.attack
                }
            } else {
                return Err(SAPTestError::InvalidTeamAction {
                    subject: "No Pets Trigger Effects".to_string(),
                    reason: "One or more teams is missing pets to trigger effects.".to_string(),
                })?;
            };

        let mut friend_item_triggers = VecDeque::new();
        let mut opponent_item_triggers = VecDeque::new();

        // The team with a lower attack first pet goes first, otherwise it's reversed.
        // https://youtu.be/NSqjuA32AoA?t=426
        if opponent_first {
            loop {
                self.history.curr_cycle += 1;
                opponent.history.curr_cycle += 1;

                // Activate all pet effects until all triggers consumed. Then move on to items.
                // Opponent first, then friends.
                if let Some(trigger) = opponent.triggers.pop_front() {
                    opponent.trigger_effects(&trigger, Some(self))?;
                    opponent_item_triggers.push_back(trigger)
                } else if let Some(trigger) = self.triggers.pop_front() {
                    self.trigger_effects(&trigger, Some(opponent))?;
                    friend_item_triggers.push_back(trigger)
                } else if let Some(trigger) = opponent_item_triggers.pop_front() {
                    opponent.trigger_items(&trigger, Some(self))?;
                } else if let Some(trigger) = friend_item_triggers.pop_front() {
                    self.trigger_items(&trigger, Some(opponent))?;
                } else {
                    // Nothing left. All triggers consumed.
                    break;
                };
            }
        } else {
            loop {
                self.history.curr_cycle += 1;
                opponent.history.curr_cycle += 1;

                // Activate all pet effects until all triggers consumed. Then move on to items.
                // Opponent first, then friends.
                if let Some(trigger) = self.triggers.pop_front() {
                    self.trigger_effects(&trigger, Some(opponent))?;
                    friend_item_triggers.push_back(trigger)
                } else if let Some(trigger) = opponent.triggers.pop_front() {
                    opponent.trigger_effects(&trigger, Some(self))?;
                    opponent_item_triggers.push_back(trigger)
                } else if let Some(trigger) = friend_item_triggers.pop_front() {
                    self.trigger_items(&trigger, Some(opponent))?;
                } else if let Some(trigger) = opponent_item_triggers.pop_front() {
                    opponent.trigger_items(&trigger, Some(self))?;
                } else {
                    // Nothing left. All triggers consumed.
                    break;
                };
            }
        };

        Ok(self)
    }

    fn trigger_start_battle_effects(
        &mut self,
        opponent: &mut Team,
    ) -> Result<&mut Self, SAPTestError> {
        let self_pets = self
            .friends
            .iter()
            .flatten()
            .map(|pet| (Target::Friend, pet));
        let opponent_pets = opponent
            .friends
            .iter()
            .flatten()
            .map(|pet| (Target::Enemy, pet));

        let mut activated_effects: Vec<(Target, Effect)> = vec![];
        for (team, pet) in self_pets
            .chain(opponent_pets)
            .sorted_by(|(_, pet_1), (_, pet_2)| {
                pet_1
                    .borrow()
                    .stats
                    .attack
                    .cmp(&pet_2.borrow().stats.attack)
            })
            .rev()
        {
            // Do not need to mutate to reduce uses as start of battle should only occur once.
            let start_of_battle_effects = pet
                .borrow()
                .effect
                .iter()
                .filter_map(|effect| {
                    if effect.trigger.status == Status::StartOfBattle {
                        Some((team, effect.clone()))
                    } else {
                        None
                    }
                })
                .collect_vec();

            // Check for tiger effects.
            let tiger_effects = match team {
                Target::Friend => {
                    self.repeat_effects_if_tiger(pet, &TRIGGER_START_BATTLE, None, false)?
                }
                Target::Enemy => {
                    opponent.repeat_effects_if_tiger(pet, &TRIGGER_START_BATTLE, None, false)?
                }
                _ => unreachable!("Not possible to get other targets."),
            };

            activated_effects.extend(start_of_battle_effects);
            activated_effects.extend(tiger_effects.into_iter().map(|effect| (team, effect)))
        }
        for (team, effect) in activated_effects.iter() {
            match team {
                Target::Friend => {
                    self.apply_effect(&TRIGGER_START_BATTLE, effect, Some(opponent))?;
                }
                Target::Enemy => {
                    opponent.apply_effect(&TRIGGER_START_BATTLE, effect, Some(self))?;
                }
                _ => unreachable!("Not possible to have other targets."),
            }
        }

        // Exhaust any produced triggers from start of battle.
        self.trigger_all_effects(opponent)?;

        Ok(self)
    }

    fn trigger_items(
        &mut self,
        trigger: &Outcome,
        mut opponent: Option<&mut Team>,
    ) -> Result<&mut Self, SAPTestError> {
        let mut applied_effects: Vec<Effect> = vec![];
        let ordered_pets = self.get_pet_effect_order(!NON_COMBAT_TRIGGERS.contains(trigger));

        for pet in ordered_pets.iter() {
            // Get food and pet effect based on if its trigger is equal to current trigger, if any.
            if let Some(food) = pet
                .borrow_mut()
                .item
                .as_mut()
                .filter(|food| food.ability.check_activates(trigger))
            {
                // Drop uses by one if possible.
                food.ability.remove_uses(1);
                applied_effects.push(food.ability.clone())
            };
        }

        for effect in applied_effects.into_iter() {
            if let Some(opponent) = opponent.as_mut() {
                self.apply_effect(trigger, &effect, Some(opponent))?;
            } else {
                self.apply_effect(trigger, &effect, None)?;
            }
        }

        Ok(self)
    }

    fn trigger_effects(
        &mut self,
        trigger: &Outcome,
        mut opponent: Option<&mut Team>,
    ) -> Result<&mut Self, SAPTestError> {
        let mut applied_effects: Vec<Effect> = vec![];

        // Get petname and position of trigger.
        let (trigger_pet_name, trigger_pet_pos) = if let Some(Some(trigger_pet)) =
            trigger.affected_pet.as_ref().map(|pet| pet.upgrade())
        {
            (
                Some(trigger_pet.borrow().name.clone()),
                trigger_pet.borrow().pos,
            )
        } else {
            (None, None)
        };

        // Determine pet order of effects on team.
        let ordered_pets = self.get_pet_effect_order(!NON_COMBAT_TRIGGERS.contains(trigger));

        // Iterate through pets in descending order by attack strength to collect valid effects.
        for pet in ordered_pets.iter() {
            let same_pet_as_trigger = trigger
                .clone()
                .affected_pet
                .map_or(false, |trigger_pet| trigger_pet.ptr_eq(&Rc::downgrade(pet)));

            let valid_effects = pet
                .borrow_mut()
                .effect
                .iter_mut()
                .filter_map(|effect| {
                    if !effect.check_activates(trigger)
                        || is_pet_effect_exception(
                            trigger,
                            trigger_pet_name.as_ref(),
                            effect,
                            same_pet_as_trigger,
                        )
                    {
                        None
                    } else {
                        // Drop uses by one if possible.
                        effect.remove_uses(1);
                        Some(effect.clone())
                    }
                })
                .collect_vec();

            // Check if tiger should activate.
            // Also checks if effects are valid.
            let tiger_effects = self.repeat_effects_if_tiger(
                pet,
                trigger,
                trigger_pet_name.as_ref(),
                same_pet_as_trigger,
            )?;

            applied_effects.extend(valid_effects);
            applied_effects.extend(tiger_effects);
        }

        // Pet sold. Remove pet from friends and add to sold pet.
        if (&trigger.status, &trigger.position, &trigger.affected_team)
            == (&Status::Sell, &Position::OnSelf, &Target::Friend)
        {
            if let Some(pet_pos) = trigger_pet_pos {
                self.sold.push(self.friends.remove(pet_pos));
                self.friends.insert(pet_pos, None);
            }
        };

        for effect in applied_effects.into_iter() {
            if let Some(opponent) = opponent.as_mut() {
                self.apply_effect(trigger, &effect, Some(opponent))?;
            } else {
                self.apply_effect(trigger, &effect, None)?;
            }
        }
        Ok(self)
    }

    fn apply_effect(
        &mut self,
        trigger: &Outcome,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError> {
        // Set current pet.
        self.curr_pet = effect.owner.clone();
        let mut affected_pets = vec![];

        match (&effect.target, &effect.action) {
            // Swapping pets only possible between two pets so place here where only activates once.
            (_, Action::Swap(swap_type)) => {
                affected_pets.extend(self.swap_pets(swap_type, effect, trigger, opponent)?)
            }
            // Shuffle effects act on sets of pets so must be here to only activate once.
            (target_team, Action::Shuffle(shuffle_by)) => affected_pets.extend(self.shuffle_pets(
                target_team,
                shuffle_by,
                effect,
                trigger,
                opponent,
            )?),
            // All shop actions go here.
            (Target::Shop, _) => self.apply_shop_effect(effect)?,
            // Effects applied to individual pets are here.
            _ => {
                let target_pets = if let Some(opponent) = opponent.as_ref() {
                    self.get_pets_by_effect(trigger, effect, Some(opponent))?
                } else {
                    self.get_pets_by_effect(trigger, effect, None)?
                };
                let afflicting_pet = effect.try_into()?;
                if let Some(opponent) = opponent {
                    for target_pet in target_pets.into_iter() {
                        // Match against team name.
                        let pets = if target_pet.borrow().team.as_ref() == Some(&self.name) {
                            self.apply_single_effect(
                                &target_pet,
                                &afflicting_pet,
                                effect,
                                Some(opponent),
                            )?
                        } else if target_pet.borrow().team.as_ref() == Some(&opponent.name) {
                            opponent.apply_single_effect(
                                &target_pet,
                                &afflicting_pet,
                                effect,
                                Some(self),
                            )?
                        } else {
                            return Err(SAPTestError::InvalidTeamAction {
                                subject: "Invalid Team Name".to_string(),
                                reason: format!(
                                    "Pet {} has a team name that doesn't match the current fighting teams ({} vs. {})",
                                    target_pet.borrow(), self.name, opponent.name
                                )
                            });
                        };
                        affected_pets.extend(pets)
                    }
                } else {
                    for target_pet in target_pets.into_iter() {
                        affected_pets.extend(self.apply_single_effect(
                            &target_pet,
                            &target_pet,
                            effect,
                            None,
                        )?)
                    }
                }
            }
        }
        Ok(affected_pets)
    }
}
