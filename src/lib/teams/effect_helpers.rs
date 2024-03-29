use crate::{
    effects::{
        actions::{
            Action, ConditionType, CopyType, LogicType, RandomizeType, StatChangeType, SummonType,
        },
        effect::{Effect, EffectModify, Entity},
        state::{ItemCondition, Outcome, Position, Status, Target},
        stats::Statistics,
        trigger::*,
    },
    error::SAPTestError,
    pets::{
        names::PetName,
        pet::{reassign_effects, MAX_PET_STATS, MIN_PET_STATS},
    },
    shop::{
        store::{ItemSlot, ItemState, ShopState},
        team_shopping::TeamShoppingHelpers,
    },
    teams::{history::TeamHistoryHelpers, team::Team, viewer::TeamViewer},
    Pet, PetCombat, ShopItem, ShopItemViewer, ShopViewer, TeamEffects, CONFIG,
};

use itertools::Itertools;
use log::info;
use rand::{
    random,
    seq::{IteratorRandom, SliceRandom},
    SeedableRng,
};
use rand_chacha::ChaCha12Rng;
use std::sync::{Arc, RwLock};

/// Used to ignore an effect if its trigger fits a set of conditions.
///  * Trigger for an [`Effect`] with an [`Action::Summon`] is a [`ZombieFly`](crate::PetName::ZombieFly).
///  * Trigger for an [`Effect`] with an [`Action::Summon`] is a [`Fly`](crate::PetName::Fly) and is also the current pet is that [`Fly`](crate::PetName::Fly).
///  * The pet causing the trigger is the same as the pet being checked for effects and the triggers targets [`Position::Any`](crate::Position::Any).
pub(crate) fn is_pet_effect_exception(
    trigger: &Outcome,
    trigger_petname: Option<&PetName>,
    pet_effect: &Effect,
    same_pet_as_trigger: bool,
) -> bool {
    if let Some(trigger_pet_name) = trigger_petname {
        if let Action::Summon(_) = pet_effect.action {
            *trigger_pet_name == PetName::ZombieFly
                || (*trigger_pet_name == PetName::Fly && same_pet_as_trigger)
        } else if let Action::Add(_) = pet_effect.action {
            trigger.position == Position::Any(ItemCondition::None) && same_pet_as_trigger
        } else {
            false
        }
    } else {
        false
    }
}

pub(crate) trait EffectApplyHelpers {
    /// Activate a [`Food`] effect that occurs at the same time as a battle.
    /// * ex. Chili
    fn apply_battle_food_effect(
        &mut self,
        afflicting_pet: &Arc<RwLock<Pet>>,
        opponent: &mut Team,
    ) -> Result<(), SAPTestError>;

    /// Apply an [`Action`] to a target pet on a [`Team`].
    fn apply_single_effect(
        &mut self,
        affected_pet: &Arc<RwLock<Pet>>,
        afflicting_pet: &Arc<RwLock<Pet>>,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError>;

    /// Apply [`Shop`] effects.
    fn apply_shop_effect(&mut self, effect: &Effect) -> Result<(), SAPTestError>;

    fn copy_effect(
        &self,
        attr_to_copy: &CopyType,
        targets: Vec<Arc<RwLock<Pet>>>,
        receiving_pet: &Arc<RwLock<Pet>>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError>;

    fn swap_pets(
        &mut self,
        swap_type: &RandomizeType,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError>;

    fn shuffle_pets(
        &mut self,
        target_team: &Target,
        shuffle_type: &RandomizeType,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError>;

    fn summon_pet(
        &mut self,
        target_pet: &Arc<RwLock<Pet>>,
        summon_type: &SummonType,
        opponent: Option<&mut Team>,
    ) -> Result<Arc<RwLock<Pet>>, SAPTestError>;

    /// Get matching pets by [`ItemCondition`].
    fn get_matching_pets(
        &self,
        target: &Target,
        condition: &ItemCondition,
        opponent: &Option<&mut Team>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError>;

    /// Check if [`ConditionType`] is `true`.
    fn check_condition(
        &self,
        condition_type: &ConditionType,
        effect: &Effect,
        target_pet: &Arc<RwLock<Pet>>,
        opponent: &Option<&mut Team>,
    ) -> Result<bool, SAPTestError>;

    /// Apply a condition action.
    /// * action_set first arg is run if condition met. Otherwise, second arg run.
    fn apply_conditional_action(
        &mut self,
        affected_pet: &Arc<RwLock<Pet>>,
        afflicting_pet: &Arc<RwLock<Pet>>,
        condition_type: &LogicType,
        effect: &Effect,
        action_set: (&Action, &Action),
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError>;

    /// Hard-coded [`Tiger`](crate::PetName::Tiger) behavior.
    /// * Checks that pet behind current pet is a tiger.
    /// * Determines if [`Effect`] is valid by same methods in [`trigger_effects`](TeamEffects::trigger_effects).
    /// * Creates effects of `pet` at given tiger level.
    fn repeat_effects_if_tiger(
        &self,
        pet: &Arc<RwLock<Pet>>,
        trigger: &Outcome,
        trigger_petname: Option<&PetName>,
        same_pet_as_trigger: bool,
    ) -> Result<Vec<Effect>, SAPTestError>;

    /// Hard-coded [`Whale`](crate::PetName::Whale) behavior.
    fn evolve_pet(
        &mut self,
        affected_pet: &Arc<RwLock<Pet>>,
        afflicting_pet: &Arc<RwLock<Pet>>,
        lvl: usize,
        targets: Vec<Arc<RwLock<Pet>>>,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError>;

    /// Calculates an adjusted index based on the current index and a relative index.
    /// * `:param curr_idx:` The current index.
    /// * `:param rel_idx:` Number of positions relative to the current index.
    ///     * If *negative*, the index is **behind** the current index.
    ///     * If *positive*, the index is **ahead** of the current index.
    ///
    /// Output:
    /// * Value of the new index on a team represented by a variant in the enum `Target`.
    fn cvt_rel_idx_to_adj_idx(
        &self,
        curr_idx: usize,
        rel_idx: isize,
    ) -> Result<(Target, usize), SAPTestError>;
    fn cvt_pos_to_idx(&self, pos: &Position) -> Option<usize>;
}

impl EffectApplyHelpers for Team {
    fn apply_battle_food_effect(
        &mut self,
        afflicting_pet: &Arc<RwLock<Pet>>,
        opponent: &mut Team,
    ) -> Result<(), SAPTestError> {
        // Check for food uses.
        // Then copy food to avoid potential mut borrow.
        let item = afflicting_pet
            .write()
            .unwrap()
            .item
            .as_mut()
            .and_then(|food| {
                if food.ability.uses != Some(0)
                    && food.ability.trigger.status == Status::BattleFoodEffect
                {
                    food.ability.remove_uses(1);
                    Some(food.to_owned())
                } else {
                    None
                }
            });
        if let Some(valid_food) = item {
            // Use effect on affected pets.
            let affected_pets = self.get_pets_by_effect(&valid_food.ability, Some(opponent))?;
            for affected_pet in affected_pets.iter() {
                if affected_pet.read().unwrap().team.as_ref() == Some(&self.name) {
                    self.apply_single_effect(
                        affected_pet,
                        afflicting_pet,
                        &valid_food.ability,
                        Some(opponent),
                    )?;
                } else {
                    opponent.apply_single_effect(
                        affected_pet,
                        afflicting_pet,
                        &valid_food.ability,
                        Some(self),
                    )?;
                }
            }
        }

        Ok(())
    }

    fn swap_pets(
        &mut self,
        swap_type: &RandomizeType,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError> {
        let target_pets = if let Some(opponent) = opponent.as_ref() {
            self.get_pets_by_effect(effect, Some(opponent))?
        } else {
            self.get_pets_by_effect(effect, None)?
        };

        if target_pets.len() != 2 {
            return Err(SAPTestError::InvalidTeamAction {
                subject: format!("Swap {swap_type:?}"),
                reason: format!(
                    "Only two friends allowed for swapping. Given: {}",
                    target_pets.len()
                ),
            });
        }

        // Safe to unwrap.
        let (pet_1, pet_2) = (target_pets.first().unwrap(), target_pets.get(1).unwrap());
        match swap_type {
            RandomizeType::Positions => {
                pet_1.write().unwrap().swap(&mut pet_2.write().unwrap());
            }
            RandomizeType::Stats => {
                pet_1
                    .write()
                    .unwrap()
                    .swap_stats(&mut pet_2.write().unwrap());
            }
        }
        Ok(vec![pet_1.clone(), pet_2.clone()])
    }

    fn shuffle_pets(
        &mut self,
        target_team: &Target,
        shuffle_type: &RandomizeType,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError> {
        let teams = match target_team {
            Target::Friend => vec![Some(self)],
            Target::Enemy => vec![opponent],
            Target::Either => vec![Some(self), opponent],
            _ => unimplemented!("Cannot shuffle on given target."),
        };
        let mut affected_pets = vec![];
        for team in teams.into_iter().flatten() {
            let mut rng = ChaCha12Rng::seed_from_u64(team.seed.unwrap_or_else(random));
            let curr_pet =
                if let Some(Some(curr_pet)) = team.curr_pet.as_ref().map(|pet| pet.upgrade()) {
                    Some(curr_pet)
                } else {
                    team.first()
                };
            let mut found_pets = team.get_pets_by_pos(
                curr_pet,
                // Find pets on current team.
                &Target::Friend,
                &effect.position,
                Some(&effect.trigger),
                None,
            )?;
            match shuffle_type {
                RandomizeType::Positions => {
                    // Shuffle to randomize found pets.
                    found_pets.shuffle(&mut rng);

                    // Then split into two pet chunks and swap pets.
                    for mut chunk in &found_pets.iter().chunks(2) {
                        let (Some(first_pet), Some(second_pet)) = (chunk.next(), chunk.next())
                        else {
                            continue;
                        };
                        first_pet
                            .write()
                            .unwrap()
                            .swap(&mut second_pet.write().unwrap());
                        affected_pets.extend([first_pet.clone(), second_pet.clone()])
                    }
                    // Then reset indices in-place.
                    team.set_indices();
                }
                RandomizeType::Stats => {
                    // Invert stats. (2,1) -> (1,2)
                    for pet in found_pets.iter() {
                        pet.write().unwrap().stats.invert();
                        affected_pets.push(pet.clone())
                    }
                }
            }
        }

        Ok(affected_pets)
    }

    fn repeat_effects_if_tiger(
        &self,
        pet: &Arc<RwLock<Pet>>,
        trigger: &Outcome,
        trigger_petname: Option<&PetName>,
        same_pet_as_trigger: bool,
    ) -> Result<Vec<Effect>, SAPTestError> {
        let effect_pet_idx = {
            let pet = pet.read().unwrap();
            pet.pos.ok_or(SAPTestError::InvalidTeamAction {
                subject: "No Pet Position Index.".to_string(),
                reason: format!("Pet {} must have an index set at this point.", pet),
            })?
        };

        let mut tiger_doubled_effects = vec![];
        // For Tiger. Check if behind.
        if let Some(Some(pet_behind)) = self.friends.get(effect_pet_idx + 1) {
            if pet_behind.read().unwrap().name == PetName::Tiger
                && self.shop.state == ShopState::Closed
            {
                // Get effect at level of tiger and repeat it.
                let pet_effect_at_tiger_lvl = pet
                    .read()
                    .unwrap()
                    .get_effect(pet_behind.read().unwrap().lvl)?;
                for mut effect in pet_effect_at_tiger_lvl {
                    // Assign owner so new lvled effect matches owner.
                    effect.assign_owner(Some(pet));

                    let valid_effect = !is_pet_effect_exception(
                        trigger,
                        trigger_petname,
                        &effect,
                        same_pet_as_trigger,
                    );
                    if effect.check_activates(trigger) && valid_effect {
                        tiger_doubled_effects.push(effect)
                    }
                }
            }
        };
        Ok(tiger_doubled_effects)
    }

    fn summon_pet(
        &mut self,
        target_pet: &Arc<RwLock<Pet>>,
        summon_type: &SummonType,
        opponent: Option<&mut Team>,
    ) -> Result<Arc<RwLock<Pet>>, SAPTestError> {
        // Can't impl TryFrom because requires target pet.
        let summoned_pet = summon_type.to_pet(self, target_pet)?;
        let target_pet = target_pet.read().unwrap();
        let target_idx = target_pet.pos.ok_or(SAPTestError::InvalidTeamAction {
            subject: "Missing Summon Position".to_string(),
            reason: format!("Target pet {} has no position.", target_pet),
        })?;

        // Handle case where pet in front faints and vector is empty.
        // Would panic attempting to insert at any position not at 0.
        // Also update position to be correct.
        let adj_target_idx = if target_idx > self.friends.len() {
            0
        } else {
            target_idx
        };

        self.add_pet(summoned_pet, adj_target_idx, opponent)?;
        // Should be safe to unwrap at this point.
        if let Some(Some(summoned_pet)) = self.friends.get(adj_target_idx) {
            Ok(summoned_pet.clone())
        } else {
            Err(SAPTestError::InvalidTeamAction {
                subject: "Missing Summoned Pet".to_string(),
                reason: "Something went wrong with added pet.".to_string(),
            })
        }
    }

    fn evolve_pet(
        &mut self,
        affected_pet: &Arc<RwLock<Pet>>,
        afflicting_pet: &Arc<RwLock<Pet>>,
        lvl: usize,
        targets: Vec<Arc<RwLock<Pet>>>,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError> {
        let chosen_pet = targets.first().ok_or(SAPTestError::FallibleAction)?;

        // Create a default pet, upgrade its level, and scale its health and attack.
        let mut leveled_pet = Pet::try_from(chosen_pet.read().unwrap().name.clone())?;
        leveled_pet.set_level(lvl)?;
        leveled_pet.stats.attack *= TryInto::<isize>::try_into(lvl)?;
        leveled_pet.stats.health *= TryInto::<isize>::try_into(lvl)?;

        // Kill the original pet.
        let mut kill_effect = Effect {
            action: Action::Kill,
            ..Default::default()
        };
        kill_effect.assign_owner(Some(chosen_pet));
        self.apply_single_effect(chosen_pet, afflicting_pet, &kill_effect, opponent)?;

        // Set the target's pet ability to summon the pet.
        let target_pet_ref = Arc::downgrade(affected_pet);
        let mut target_pet_trigger = TRIGGER_SELF_FAINT;
        target_pet_trigger.affected_pet = Some(target_pet_ref.clone());
        let mut affected_pet_guard = affected_pet.write().unwrap();
        affected_pet_guard.effect = vec![Effect {
            owner: Some(target_pet_ref),
            trigger: target_pet_trigger,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Summon(SummonType::StoredPet(Box::new(leveled_pet.clone()))),
            uses: Some(1),
            temp: true,
        }];
        info!(target: "run", "(\"{}\")\nEvolving {}.", self.name, leveled_pet);
        info!(target: "run", "(\"{}\")\nSet pet {} to summon evolved pet on faint.", self.name, affected_pet_guard);
        Ok(vec![chosen_pet.clone(), affected_pet.clone()])
    }

    fn copy_effect(
        &self,
        attr_to_copy: &CopyType,
        targets: Vec<Arc<RwLock<Pet>>>,
        receiving_pet: &Arc<RwLock<Pet>>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError> {
        let mut affected_pets = vec![];
        // Choose the first pet.
        let copied_attr = if let Some(pet_to_copy) = targets.first() {
            affected_pets.push(pet_to_copy.clone());

            match attr_to_copy.clone() {
                CopyType::Stats(replacement_stats) => Some(CopyType::Stats(
                    replacement_stats.map_or(Some(pet_to_copy.read().unwrap().stats), Some),
                )),
                CopyType::PercentStats(perc_stats_mult) => {
                    // Multiply the stats of a chosen pet by some multiplier
                    let mut new_stats = pet_to_copy
                        .read()
                        .unwrap()
                        .stats
                        .mult_perc(&perc_stats_mult);
                    new_stats.clamp(MIN_PET_STATS, MAX_PET_STATS);
                    info!(
                        target: "run", "(\"{}\")\nCopied {}% atk and {}% health from {}.",
                        self.name,
                        perc_stats_mult.attack,
                        perc_stats_mult.health,
                        receiving_pet.read().unwrap()
                    );
                    Some(CopyType::Stats(Some(new_stats)))
                }
                CopyType::Effect(_, lvl) => Some(CopyType::Effect(
                    pet_to_copy.read().unwrap().get_effect(lvl.unwrap_or(1))?,
                    lvl,
                )),
                CopyType::Item(_) => pet_to_copy
                    .read()
                    .unwrap()
                    .item
                    .as_ref()
                    .map(|food| CopyType::Item(Some(Box::new(food.clone())))),
                _ => None,
            }
        } else {
            None
        };

        // Chose the target of recipient of copied pet stats/effect.
        // Calculate stats or set ability.
        let mut receiving_pet_guard = receiving_pet.write().unwrap();
        match copied_attr.unwrap_or(CopyType::None) {
            CopyType::Stats(new_stats) => {
                // If some stats given use those as base.
                let new_stats = if let Some(mut new_stats) = new_stats {
                    // If any stat value is 0, use the target's original stats, otherwise, use the new stats.
                    *new_stats.comp_set_value(&receiving_pet_guard.stats, 0)
                } else {
                    // Otherwise, copy stats from target.
                    receiving_pet_guard.stats
                };

                receiving_pet_guard.stats = new_stats;

                info!(
                    target: "run", "(\"{}\")\nSet stats for {} to {}.",
                    self.name,
                    receiving_pet_guard,
                    receiving_pet_guard.stats
                );
            }
            CopyType::Effect(mut effects, _) => {
                for effect in effects.iter_mut() {
                    effect.assign_owner(Some(receiving_pet));
                }
                receiving_pet_guard.effect = effects;

                info!(
                    target: "run", "(\"{}\")\nSet effect for {} to {:?}.",
                    self.name,
                    receiving_pet_guard,
                    receiving_pet_guard.effect
                );
            }
            CopyType::Item(item) => {
                if let Some(mut food) = item {
                    // Assign ability owner to target_pet.
                    food.ability.assign_owner(Some(receiving_pet));

                    receiving_pet_guard.item = Some(*food);
                    info!(
                        target: "run", "(\"{}\")\nCopyied item for {} to {:?}.",
                        self.name,
                        receiving_pet_guard,
                        receiving_pet_guard.item
                    );
                }
            }
            CopyType::None => {}
            // Only used to store stats. Gets converted to CopyType::Stats.
            CopyType::PercentStats(_) => {}
        }
        affected_pets.push(receiving_pet.clone());
        Ok(affected_pets)
    }

    fn check_condition(
        &self,
        condition_type: &ConditionType,
        effect: &Effect,
        target_pet: &Arc<RwLock<Pet>>,
        opponent: &Option<&mut Team>,
    ) -> Result<bool, SAPTestError> {
        match condition_type {
            ConditionType::Pet(target, cond) => Ok(self
                .get_matching_pets(target, cond, opponent)?
                .iter()
                .any(|pet| Arc::ptr_eq(pet, target_pet))),
            ConditionType::Team(target, cond) => {
                if let Target::Friend = target {
                    Ok(cond.matches_team(self))
                } else if let (Target::Enemy, Some(opponent)) = (target, opponent) {
                    Ok(cond.matches_team(opponent))
                } else {
                    return Err(SAPTestError::InvalidTeamAction {
                        subject: "Invalid Target".to_string(),
                        reason: ("Target not a team or no team provided.").to_string(),
                    });
                }
            }
            ConditionType::Shop(cond) => Ok(cond.matches_shop(self)),
            ConditionType::Trigger(entity, cond) => match entity {
                Entity::Pet => {
                    let pets = self.all();

                    Ok(self
                        .filter_matching_pets(pets, cond)
                        .into_iter()
                        .any(|pet| Arc::ptr_eq(&pet, target_pet)))
                }
                Entity::Food => {
                    let Some(food) = effect
                        .trigger
                        .afflicting_food
                        .as_ref()
                        .and_then(|food_ref| food_ref.upgrade())
                    else {
                        return Ok(false);
                    };
                    let food = food.read().unwrap();
                    Ok(cond.matches_food(&food))
                }
                Entity::Toy => todo!(),
            },
        }
    }

    fn get_matching_pets(
        &self,
        target: &Target,
        condition: &ItemCondition,
        opponent: &Option<&mut Team>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError> {
        Ok(if *target == Target::Friend {
            self.get_pets_by_cond(condition)
        } else if *target == Target::Shop {
            let items = self.shop.get_shop_items_by_cond(condition, &Entity::Pet)?;
            items
                .into_iter()
                .filter_map(|item| {
                    if let ItemSlot::Pet(pet) = &item.item {
                        Some(pet.clone())
                    } else {
                        None
                    }
                })
                .collect_vec()
        } else {
            opponent
                .as_ref()
                .ok_or(SAPTestError::InvalidTeamAction {
                    subject: format!("Missing Opponent for {:?}", &condition),
                    reason: "Opponent must be known for this action.".to_string(),
                })?
                .get_pets_by_cond(condition)
        })
    }

    fn apply_conditional_action(
        &mut self,
        affected_pet: &Arc<RwLock<Pet>>,
        afflicting_pet: &Arc<RwLock<Pet>>,
        logic_type: &LogicType,
        effect: &Effect,
        action_set: (&Action, &Action),
        mut opponent: Option<&mut Team>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError> {
        let mut affected_pets = vec![];

        // Get number of times action should be executed for action and other action.
        let num_actions = match logic_type {
            LogicType::ForEach(cond_type) => {
                cond_type.num_actions_for_each(self, &opponent, Some(&effect.trigger))?
            }
            LogicType::If(cond_type) => {
                usize::from(self.check_condition(cond_type, effect, affected_pet, &opponent)?)
            }
            LogicType::IfNot(cond_type) => {
                usize::from(!self.check_condition(cond_type, effect, affected_pet, &opponent)?)
            }
            LogicType::IfAny(cond_type) => match cond_type {
                ConditionType::Pet(target, cond) => {
                    // If any pet matches condition, run action.
                    usize::from(!self.get_matching_pets(target, cond, &opponent)?.is_empty())
                }
                _ => {
                    return Err(SAPTestError::InvalidTeamAction {
                        subject: "Not Implemented".to_string(),
                        reason: "ConditionType not implemented for logic type.".to_string(),
                    })
                }
            },
        };

        // Create new effect with action.
        // For each condition met, execute the action.
        let mut effect_copy = effect.clone();
        let mut execute_actions = move |num_actions: usize,
                                        effect_copy: &Effect,
                                        affected_pets: &mut Vec<Arc<RwLock<Pet>>>|
              -> Result<(), SAPTestError> {
            for _ in 0..num_actions {
                match effect_copy.target {
                    Target::Friend | Target::Enemy | Target::Either => {
                        if let Some(opponent) = opponent.as_mut() {
                            affected_pets.extend(self.apply_single_effect(
                                affected_pet,
                                afflicting_pet,
                                effect_copy,
                                Some(opponent),
                            )?)
                        } else {
                            self.apply_single_effect(
                                affected_pet,
                                afflicting_pet,
                                effect_copy,
                                None,
                            )?;
                        }
                    }
                    Target::Shop => self.apply_shop_effect(effect_copy)?,
                    _ => {}
                }
            }
            Ok(())
        };

        // Execute actions if condition met.
        effect_copy.action = action_set.0.clone();
        execute_actions(num_actions, &effect_copy, &mut affected_pets)?;

        // And the other action ONCE if the condition not met.
        if num_actions == 0 {
            effect_copy.action = action_set.1.clone();
            execute_actions(1, &effect_copy, &mut affected_pets)?;
        }
        Ok(affected_pets)
    }

    fn apply_shop_effect(&mut self, effect: &Effect) -> Result<(), SAPTestError> {
        let effect_owner: Arc<RwLock<Pet>> = effect.try_into()?;

        match &effect.action {
            Action::AddShopStats(stats) => {
                for pet_slot in self.shop.pets.iter() {
                    if let ItemSlot::Pet(pet) = &pet_slot.item {
                        pet.write().unwrap().stats += *stats;
                    } else {
                        unreachable!("Cannot have food item in pets.")
                    };
                }
                self.shop.perm_stats += *stats;
                info!(target: "run", "(\"{}\")\nAdded permanent shop {}.", self.name, stats)
            }
            Action::AddShopFood(gain_food_type) => {
                let new_shop_food = gain_food_type
                    .to_food(self, &effect_owner)?
                    .map(ShopItem::from);
                info!(target: "run", "(\"{}\")\nAdding shop item {:?}.", self.name, new_shop_food.as_ref());

                if let Some(Err(err)) = new_shop_food.map(|item| self.shop.add_item(item)) {
                    info!(target: "run", "(\"{}\")\n{err}.", self.name)
                }
            }
            Action::AddShopPet(summon_type) => {
                let new_shop_pet = ShopItem::from(summon_type.to_pet(self, &effect_owner)?);
                info!(target: "run", "(\"{}\")\nAdding shop item {:?}.", self.name, &new_shop_pet);

                if let Err(err) = self.shop.add_item(new_shop_pet) {
                    info!(target: "run", "(\"{}\")\n{err}.", self.name)
                }
            }
            Action::ClearShop(item_type) => {
                match item_type {
                    Entity::Pet => self.shop.pets.clear(),
                    Entity::Food => self.shop.foods.clear(),
                    _ => {
                        return Err(SAPTestError::InvalidShopAction {
                            subject: String::from("Invalid Shop Entity"),
                            reason: format!("{item_type} cannot be cleared."),
                        })
                    }
                }
                info!(target: "run", "(\"{}\")\nCleared shop {item_type:?}.", self.name)
            }
            Action::AlterGold(coins) => {
                if coins.is_negative() {
                    let coin_change: usize = (-*coins).try_into()?;
                    self.shop.coins = self.shop.coins.saturating_sub(coin_change);
                } else {
                    let coin_change: usize = (*coins).try_into()?;
                    self.shop.coins += coin_change;
                }
                info!(target: "run", "(\"{}\")\nAltered shop gold by {}. New coin count: {}", self.name, coins, self.shop.coins)
            }
            Action::Discount(entity, discount) => {
                // Method only gets immutable refs.
                let affected_items_copy = self
                    .shop
                    .get_shop_items_by_pos(&effect.position, entity)?
                    .into_iter()
                    .cloned()
                    .collect_vec();
                let shop_items = match entity {
                    Entity::Pet => self.shop.pets.iter_mut(),
                    Entity::Food => self.shop.foods.iter_mut(),
                    _ => {
                        return Err(SAPTestError::InvalidShopAction {
                            subject: String::from("Invalid Shop Entity"),
                            reason: format!("{entity} cannot be discounted."),
                        })
                    }
                };
                for item in shop_items.filter(|item| affected_items_copy.contains(item)) {
                    item.cost = item.cost.saturating_sub(*discount)
                }
            }
            Action::GetToy(toy_type) => {
                if let Some(toy) = toy_type.to_toy(self)? {
                    self.toys.push(toy)
                }
            }
            Action::SaveGold { limit } => self.shop.saved_coins = self.shop.coins.clamp(0, *limit),
            Action::FreeRoll(n_rolls) => {
                for _ in 0..*n_rolls {
                    self.shop.free_rolls += 1;
                    info!(target: "run", "(\"{}\")\nIncreased free rolls by 1. New free rolls: {}", self.name, self.shop.free_rolls)
                }
            }
            Action::Multiple(actions) => {
                let mut effect_copy = effect.clone();
                for action in actions {
                    effect_copy.action = action.clone();
                    self.apply_shop_effect(&effect_copy)?;
                }
            }
            Action::Conditional(logic_type, if_action, else_action) => {
                self.apply_conditional_action(
                    &effect_owner,
                    &effect_owner,
                    logic_type,
                    effect,
                    (if_action, else_action),
                    None,
                )?;
            }
            _ => {
                for pet in self.get_pets_by_effect(effect, None)? {
                    self.apply_single_effect(&pet, &effect_owner, effect, None)?;
                }
            }
        }

        Ok(())
    }

    fn apply_single_effect(
        &mut self,
        affected_pet: &Arc<RwLock<Pet>>,
        afflicting_pet: &Arc<RwLock<Pet>>,
        effect: &Effect,
        mut opponent: Option<&mut Team>,
    ) -> Result<Vec<Arc<RwLock<Pet>>>, SAPTestError> {
        let mut affected_pets: Vec<Arc<RwLock<Pet>>> = vec![];
        // Store copy of original effect as may be modified.
        let mut modified_effect = effect.clone();

        match &effect.action {
            Action::Add(stat_change) => {
                let affected_pet_stats = affected_pet.read().unwrap().stats;
                let afflicting_pet_stats = afflicting_pet.read().unwrap().stats;

                // Cannot add stats to fainted pets.
                // If owner is dead and the trigger for effect was not faint, ignore it.
                let afflicting_pet_w_faint_trigger_has_fainted =
                    afflicting_pet_stats.health == 0 && effect.trigger.status != Status::Faint;
                if affected_pet_stats.health == 0 || afflicting_pet_w_faint_trigger_has_fainted {
                    return Ok(affected_pets);
                }
                // Convert stat change to stats with afflicting pet stats.
                let added_stats = stat_change.to_stats(
                    Some(afflicting_pet_stats),
                    Some(&self.counters),
                    false,
                )?;

                // Update action for digraph with static value.
                modified_effect.action = Action::Add(StatChangeType::Static(added_stats));

                // If effect is temporary, store stats to be removed from referenced pet on reopening shop.
                if effect.temp
                    && effect.target == Target::Friend
                    && self.shop.state == ShopState::Open
                {
                    self.shop
                        .temp_stats
                        .push((affected_pet.read().unwrap().id.unwrap(), added_stats));
                }
                affected_pet.write().unwrap().stats += added_stats;
                {
                    let pet = affected_pet.read().unwrap();
                    info!(target: "run", "(\"{}\")\nAdded {} to {}.", self.name, added_stats, pet);
                }
                affected_pets.push(affected_pet.clone());
            }
            Action::Remove(stat_change) => {
                let afflicting_pet_stats = afflicting_pet.read().unwrap().stats;

                let mut remove_stats = stat_change.to_stats(
                    Some(afflicting_pet_stats),
                    Some(&self.counters),
                    false,
                )?;

                // Check for food on effect owner. Add any effect dmg modifiers. ex. Pineapple
                if let Some(item) = afflicting_pet
                    .read()
                    .unwrap()
                    .item
                    .as_ref()
                    .filter(|item| Status::IndirectAttackDmgCalc == item.ability.trigger.status)
                {
                    if let Action::Add(modifier) = &item.ability.action {
                        let modifier_stats = modifier.to_stats(
                            Some(afflicting_pet_stats),
                            Some(&self.counters),
                            false,
                        )?;
                        remove_stats += modifier_stats
                    }
                }
                // Update with remaining modifiers.
                modified_effect.action = Action::Remove(StatChangeType::Static(remove_stats));

                let mut atk_outcome = affected_pet.write().unwrap().indirect_attack(&remove_stats);
                {
                    let pet = affected_pet.read().unwrap();
                    info!(target: "run", "(\"{}\")\nRemoved {} health from {}.", self.name, remove_stats.attack, pet);
                }

                // Update for digraph to show health loss.
                modified_effect.action =
                    Action::Remove(StatChangeType::Static(atk_outcome.friend_stat_change));

                // Update triggers from where they came from.
                if let Some(opponent) = opponent.as_mut() {
                    atk_outcome.unload_atk_outcomes(
                        self,
                        Some(opponent),
                        affected_pet,
                        effect.owner.clone(),
                    );
                } else {
                    atk_outcome.unload_atk_outcomes(self, None, affected_pet, effect.owner.clone());
                }

                affected_pets.push(affected_pet.clone());
            }
            Action::Set(stat_change) => {
                let new_stats = {
                    let pet = affected_pet.read().unwrap();
                    let team_counters = if pet.team.as_ref() == Some(&self.name) {
                        Some(&self.counters)
                    } else {
                        opponent.as_ref().map(|team| &team.counters)
                    };
                    stat_change.to_stats(Some(pet.stats), team_counters, true)?
                };
                affected_pet.write().unwrap().stats = new_stats;
            }
            Action::Gain(gain_food_type) => {
                let mut food = gain_food_type.to_food(self, afflicting_pet)?;
                {
                    let pet = affected_pet.read().unwrap();
                    if food.is_none() {
                        info!(target: "run", "(\"{}\")\nRemoved food from {}.", self.name, pet);
                    } else if let Some(food) = food.as_mut() {
                        info!(target: "run", "(\"{}\")\nGave {} to {}.", self.name, food, pet);
                        food.ability.assign_owner(Some(affected_pet));

                        // Check if given an ailment.
                        if food.is_ailment {
                            let mut trigger_ailment = TRIGGER_ANY_GAIN_AILMENT;
                            trigger_ailment.set_affected(affected_pet);
                            self.triggers.push_back(trigger_ailment)
                        }
                    }
                }
                affected_pet.write().unwrap().item = food;
                affected_pets.push(affected_pet.clone());
            }
            Action::AlterCost(cost_change) => {
                let affected_pet_cost = affected_pet.read().unwrap().cost;
                if cost_change.is_negative() {
                    let coin_change: usize = (-*cost_change).try_into()?;
                    affected_pet.write().unwrap().cost =
                        affected_pet_cost.saturating_sub(coin_change);
                } else {
                    let coin_change: usize = (*cost_change).try_into()?;
                    affected_pet.write().unwrap().cost += coin_change;
                }
                info!(
                    target: "run",
                    "(\"{}\")\nAltered cost of {:?} by {}. New coin count: {}",
                    self.name,
                    affected_pet.read().unwrap().id,
                    cost_change,
                    affected_pet.read().unwrap().cost
                )
            }
            Action::Moose { stats, tier } => {
                // TODO: Separate into two different actions: Unfreeze shop + StatChangeType::MultShopTier
                for item in self.shop.foods.iter_mut() {
                    item.state = ItemState::Normal
                }
                let mut num_tier = 0;
                for pet in self.shop.pets.iter_mut() {
                    if pet.tier() == *tier {
                        num_tier += 1
                    }
                    pet.state = ItemState::Normal
                }
                let buffed_stats = *stats * Statistics::new(num_tier, num_tier)?;
                modified_effect.action = Action::Add(StatChangeType::Static(buffed_stats));
                affected_pets.extend(self.apply_single_effect(
                    affected_pet,
                    afflicting_pet,
                    &modified_effect,
                    None,
                )?);
            }
            Action::Fox(item_type, multiplier) => {
                let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));

                let possible_items = match item_type {
                    Entity::Pet => &mut self.shop.pets,
                    Entity::Food => &mut self.shop.foods,
                    _ => {
                        return Err(SAPTestError::InvalidPetAction {
                            subject: String::from("Invalid Fox Free Entity"),
                            reason: format!("Fox cannot obtain {item_type}."),
                        })
                    }
                };
                if let Some(stolen_item) = (0..possible_items.len())
                    .choose(&mut rng)
                    .map(|idx| possible_items.remove(idx))
                {
                    match stolen_item.item {
                        ItemSlot::Pet(pet) => {
                            // Multiply pet stats.
                            pet.write().unwrap().stats *=
                                Statistics::new(*multiplier, *multiplier)?;
                            self.buy_pet_behavior(
                                &pet,
                                Some(afflicting_pet.clone()),
                                &effect.position,
                            )?;
                        }
                        ItemSlot::Food(food) => {
                            // If stats adds some static value, multiply it by the mulitplier
                            if let Action::Add(StatChangeType::Static(mut stats)) =
                                food.read().unwrap().ability.action
                            {
                                let stat_multiplier = Statistics::new(*multiplier, *multiplier)?;
                                stats *= stat_multiplier
                            }
                            self.buy_food_behavior(
                                &food,
                                Some(afflicting_pet.clone()),
                                &effect.position,
                                false,
                            )?;
                        }
                    }
                }

                // Exhaust any triggers produced.
                while let Some(trigger) = self.triggers.pop_front() {
                    self.trigger_effects(&trigger, None)?;
                    self.trigger_items(&trigger, None)?;
                }
            }
            Action::Experience(exp) => {
                for _ in 0..*exp {
                    let mut pet = affected_pet.write().unwrap();
                    let prev_target_lvl = pet.lvl;
                    pet.add_experience(1)?;
                    info!(target: "run", "(\"{}\")\nGave experience point to {}.", self.name, pet);

                    // Target leveled up. Create trigger.
                    let pet_leveled_up = if pet.lvl != prev_target_lvl {
                        info!(target: "run", "(\"{}\")\nPet {} leveled up.", self.name, pet);
                        let mut lvl_trigger = TRIGGER_ANY_LEVELUP;
                        lvl_trigger.affected_pet = Some(Arc::downgrade(affected_pet));
                        Some(lvl_trigger)
                    } else {
                        None
                    };

                    if let Some(level_trigger) = pet_leveled_up {
                        self.triggers.push_back(level_trigger)
                    }
                }

                affected_pets.push(affected_pet.clone());
            }
            Action::Push(position) => {
                let pos_change = match position {
                    Position::First => {
                        let friends_len: isize = self.max_size.try_into()?;
                        friends_len
                    }
                    Position::Last => {
                        let friends_len: isize = self.max_size.try_into()?;
                        -friends_len
                    }
                    Position::Relative(rel_idx) => *rel_idx,
                    _ => unimplemented!("Position not implemented for push."),
                };
                // Adjust position by number of fainted pets.
                let num_fainted = self
                    .friends
                    .iter()
                    .flatten()
                    .filter(|pet| pet.read().unwrap().stats.health == 0)
                    .count();
                if let Some(position) = affected_pet
                    .read()
                    .unwrap()
                    .pos
                    .map(|idx| idx + num_fainted)
                {
                    info!(target: "run", "(\"{}\")\nPushed pet at position {} by {}.", self.name, position, pos_change);
                    if let Some(opponent) = opponent.as_mut() {
                        self.push_pet(position, pos_change, Some(opponent))?;
                    } else {
                        self.push_pet(position, pos_change, None)?;
                    }
                    affected_pets.push(affected_pet.clone());
                }
            }
            Action::Transform(pet_name, stats, lvl) => {
                if let Some(target_idx) = affected_pet.read().unwrap().pos {
                    let mut transformed_pet = Pet::new(pet_name.clone(), *stats, *lvl)?;
                    transformed_pet.set_pos(target_idx);
                    transformed_pet.team = Some(self.name.to_owned());

                    if (0..self.friends.len()).contains(&target_idx) {
                        self.friends.remove(target_idx);
                        info!(target: "run", "(\"{}\")\nTransformed pet at position {} to {}.", self.name, target_idx, &transformed_pet);
                        let rc_transformed_pet = Arc::new(RwLock::new(transformed_pet));
                        reassign_effects(&rc_transformed_pet);

                        affected_pets.extend([rc_transformed_pet.clone(), affected_pet.clone()]);
                        self.friends.insert(target_idx, Some(rc_transformed_pet));
                    }
                }
            }
            Action::Summon(summon_type) => {
                let summon_result = if let Some(opponent) = opponent.as_mut() {
                    self.summon_pet(affected_pet, summon_type, Some(opponent))
                } else {
                    self.summon_pet(affected_pet, summon_type, None)
                };

                if let Ok(pet) = summon_result {
                    affected_pets.push(pet)
                } else if let Err(err) = summon_result {
                    // Fallible error. Attempted to add too many pets or no space/pets on team.
                    if matches!(
                        err,
                        SAPTestError::InvalidPetAction { .. } | SAPTestError::FallibleAction
                    ) {
                    } else {
                        // Otherwise, something actually went wrong.
                        return Err(err);
                    }
                }
            }
            Action::Multiple(actions) => {
                // Create new effect with single action.
                let mut effect_copy = effect.clone();
                if let Some(opponent) = opponent.as_mut() {
                    for action in actions {
                        effect_copy.action = action.clone();
                        affected_pets.extend(self.apply_single_effect(
                            affected_pet,
                            afflicting_pet,
                            &effect_copy,
                            Some(opponent),
                        )?)
                    }
                } else {
                    for action in actions {
                        effect_copy.action = action.clone();
                        affected_pets.extend(self.apply_single_effect(
                            affected_pet,
                            afflicting_pet,
                            &effect_copy,
                            None,
                        )?)
                    }
                }
            }
            Action::Conditional(condition_type, if_action, else_action) => {
                let pets = if let Some(opponent) = opponent.as_mut() {
                    self.apply_conditional_action(
                        affected_pet,
                        afflicting_pet,
                        condition_type,
                        effect,
                        (if_action, else_action),
                        Some(opponent),
                    )?
                } else {
                    self.apply_conditional_action(
                        affected_pet,
                        afflicting_pet,
                        condition_type,
                        effect,
                        (if_action, else_action),
                        None,
                    )?
                };
                // Return early as ^ calls apply_single_effects() and will duplicate graph edges.
                return Ok(pets);
            }
            Action::Kill => {
                let mut pet = affected_pet.write().unwrap();
                pet.stats.health = 0;
                info!(target: "run", "(\"{}\")\nKilled pet {}.", self.name, pet);

                let mut self_faint_triggers = get_self_faint_triggers(&None);
                let mut enemy_faint_triggers = get_self_enemy_faint_triggers(&None);

                for trigger in self_faint_triggers
                    .iter_mut()
                    .chain(enemy_faint_triggers.iter_mut())
                {
                    trigger.set_affected(affected_pet);
                }
                // Add death triggers.
                self.triggers.extend(self_faint_triggers);
                if let Some(opponent) = opponent.as_mut() {
                    opponent.triggers.extend(enemy_faint_triggers);
                }
                affected_pets.push(affected_pet.clone());
            }
            Action::Debuff(perc_stats) => {
                let mut pet = affected_pet.write().unwrap();
                // TODO: Change so modifier can be on afflicting or affected pet. Current only affected.
                let debuff_stats =
                    perc_stats.to_stats(Some(pet.stats), Some(&self.counters), false)?;
                modified_effect.action = Action::Debuff(StatChangeType::Static(debuff_stats));

                pet.stats -= debuff_stats;
                info!(target: "run", "(\"{}\")\nMultiplied stats of {} by {}.", self.name, pet, perc_stats);
                affected_pets.push(affected_pet.clone());
            }
            Action::Lynx => {
                let opponent = opponent.as_mut().ok_or(SAPTestError::InvalidTeamAction {
                    subject: format!("Missing Opponent for {:?}", &effect.action),
                    reason: "Opponent must be known for this action.".to_string(),
                })?;

                let opponent_lvls: usize = opponent
                    .all()
                    .iter()
                    .map(|pet| pet.read().unwrap().lvl)
                    .sum();
                let lvl_dmg_action =
                    Action::Remove(StatChangeType::Static(Statistics::new(opponent_lvls, 0)?));
                modified_effect.action = lvl_dmg_action;

                self.apply_single_effect(
                    affected_pet,
                    afflicting_pet,
                    &modified_effect,
                    Some(*opponent),
                )?;
                affected_pets.push(affected_pet.clone());
            }
            Action::Whale(lvl, pos) => {
                let targets = if let Some(opponent) = opponent.as_ref() {
                    self.get_pets_by_pos(
                        Some(affected_pet.clone()),
                        &effect.target,
                        pos,
                        None,
                        Some(opponent),
                    )?
                } else {
                    self.get_pets_by_pos(
                        Some(affected_pet.clone()),
                        &effect.target,
                        pos,
                        None,
                        None,
                    )?
                };

                let pets = if let Some(opponent) = opponent.as_mut() {
                    self.evolve_pet(affected_pet, afflicting_pet, *lvl, targets, Some(*opponent))
                } else {
                    self.evolve_pet(affected_pet, afflicting_pet, *lvl, targets, None)
                };

                // Allow to fail if no pets found.
                if let Err(SAPTestError::FallibleAction) = pets {
                } else if let Ok(pets) = pets {
                    affected_pets.extend(pets);
                } else {
                    pets?;
                }
            }
            Action::Stegosaurus(stats) => {
                let mut turn_mult_stats = *stats;
                // Multiply by turn number. Need to multiply raw values since mult op treats as percent.
                let turn_multiplier = TryInto::<isize>::try_into(self.history.curr_turn)?;
                turn_mult_stats.attack *= turn_multiplier;
                turn_mult_stats.health *= turn_multiplier;

                // Modify action to add turn-multiplied stats and apply effect.
                modified_effect.action = Action::Add(StatChangeType::Static(turn_mult_stats));
                let pets = if let Some(opponent) = opponent.as_mut() {
                    self.apply_single_effect(
                        affected_pet,
                        afflicting_pet,
                        &modified_effect,
                        Some(*opponent),
                    )?
                } else {
                    self.apply_single_effect(affected_pet, afflicting_pet, &modified_effect, None)?
                };
                affected_pets.extend(pets);
            }
            Action::Cockroach => {
                let mut pet = affected_pet.write().unwrap();
                pet.stats.attack = TryInto::<isize>::try_into(self.shop.tier())? + 1;

                info!(target: "run", "(\"{}\")\nSet stats of {}.", self.name, pet);
                affected_pets.push(affected_pet.clone());
            }
            Action::Copy(attr, target, pos) => {
                // Create effect to select a pet.
                modified_effect.position = pos.clone();
                modified_effect.target = *target;

                let targets = if let Some(opponent) = opponent.as_mut() {
                    self.get_pets_by_effect(&modified_effect, Some(opponent))?
                } else {
                    self.get_pets_by_effect(&modified_effect, None)?
                };
                affected_pets.extend(self.copy_effect(attr, targets, affected_pet)?);
            }
            Action::Swap(RandomizeType::Stats) => {
                affected_pet.write().unwrap().stats.invert();
                affected_pets.push(affected_pet.clone());
            }
            Action::AddToCounter(counter, count_change) => {
                let modify_count = |count: &mut usize| {
                    if count_change.is_negative() {
                        let count_change = (-*count_change).try_into().unwrap_or(0);
                        *count = count.saturating_sub(count_change)
                    } else {
                        // TODO: Should probably be clamped.
                        *count = count.saturating_add((*count_change).try_into().unwrap_or(0))
                    }
                };
                let modify_w_new_count = || {
                    // Create new count and modify it. Closure ensures always clamped to 0.
                    let mut new_cnt = 0;
                    modify_count(&mut new_cnt);
                    new_cnt
                };
                match effect.target {
                    Target::Friend => {
                        self.counters
                            .entry(counter.to_owned())
                            .and_modify(modify_count)
                            .or_insert_with(modify_w_new_count);
                    }
                    Target::Enemy => {
                        if let Some(opponent) = opponent.as_mut() {
                            opponent
                                .counters
                                .entry(counter.to_owned())
                                .and_modify(modify_count)
                                .or_insert_with(modify_w_new_count);
                        } else {
                            return Err(SAPTestError::InvalidTeamAction {
                                subject: "Opponent Counter Modification".to_owned(),
                                reason: format!("Opponent required to modify counter, {counter}."),
                            });
                        };
                    }
                    _ => {
                        return Err(SAPTestError::InvalidTeamAction {
                            subject: "No Counter".to_string(),
                            reason: format!("Target ({:?}) has no counter.", effect.target),
                        })
                    }
                }
            }
            Action::None => {}
            _ => {
                return Err(SAPTestError::InvalidTeamAction {
                    subject: "Action Not Implemented".to_string(),
                    reason: format!("Single action ({:?}) not implemented.", &effect.action),
                })
            }
        }

        // Build graph edges if toggled.
        if CONFIG.general.build_graph {
            if self.history.primary_team {
                for pet in affected_pets.iter() {
                    self.add_action_edge(
                        pet,
                        afflicting_pet,
                        &modified_effect.trigger.status,
                        &modified_effect.action,
                    )?;
                }
            } else if let Some(opponent) = opponent.as_mut() {
                for pet in affected_pets.iter() {
                    opponent.add_action_edge(
                        pet,
                        afflicting_pet,
                        &modified_effect.trigger.status,
                        &modified_effect.action,
                    )?;
                }
            }
        }

        Ok(affected_pets)
    }

    fn cvt_pos_to_idx(&self, pos: &Position) -> Option<usize> {
        match pos {
            Position::Any(cond) => {
                let pets = self.get_pets_by_cond(cond);
                let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));
                pets.choose(&mut rng)
                    .and_then(|pet| pet.read().unwrap().pos)
            }
            Position::First => (!self.friends.is_empty()).then_some(0),
            Position::Last => Some(self.friends.len().saturating_sub(1)),
            Position::Relative(idx) => {
                let Some((Target::Friend, adj_idx)) = self.cvt_rel_idx_to_adj_idx(0, *idx).ok()
                else {
                    return None;
                };
                Some(adj_idx)
            }
            _ => None,
        }
    }
    fn cvt_rel_idx_to_adj_idx(
        &self,
        curr_idx: usize,
        rel_idx: isize,
    ) -> Result<(Target, usize), SAPTestError> {
        let effect_pet_idx = isize::try_from(curr_idx)?;
        // Negative idx means behind.
        // Positive idx mean ahead.
        // We adjust so within bounds of team.
        let adj_idx = if rel_idx.is_negative() {
            -rel_idx + effect_pet_idx
        } else {
            let new_idx = effect_pet_idx - rel_idx;
            // On the other team.
            if new_idx.is_negative() {
                return Ok((Target::Enemy, (-new_idx - 1).try_into()?));
            } else {
                new_idx
            }
        };
        Ok((
            Target::Friend,
            adj_idx.clamp(0, self.max_size.try_into()?).try_into()?,
        ))
    }
}
