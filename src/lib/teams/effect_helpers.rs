use crate::{
    db::record::{FoodRecord, PetRecord},
    effects::{
        actions::{
            Action, ConditionType, CopyType, GainType, LogicType, RandomizeType, StatChangeType,
            SummonType,
        },
        effect::{Effect, EffectModify, Entity},
        state::{ItemCondition, Outcome, Position, ShopCondition, Status, Target, TeamCondition},
        stats::Statistics,
        trigger::*,
    },
    error::SAPTestError,
    pets::{
        names::PetName,
        pet::{MAX_PET_STATS, MIN_PET_STATS},
    },
    shop::{
        store::{ItemSlot, ItemState, ShopState, MAX_SHOP_TIER},
        team_shopping::TeamShoppingHelpers,
    },
    teams::{history::TeamHistoryHelpers, team::Team, viewer::TeamViewer},
    Food, Pet, PetCombat, ShopItem, ShopItemViewer, ShopViewer, TeamEffects, TeamShopping, CONFIG,
    SAPDB,
};

use itertools::Itertools;
use log::info;
use rand::{
    random,
    seq::{IteratorRandom, SliceRandom},
    SeedableRng,
};
use rand_chacha::ChaCha12Rng;
use std::{cell::RefCell, rc::Rc};

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
        afflicting_pet: &Rc<RefCell<Pet>>,
        opponent: &mut Team,
    ) -> Result<(), SAPTestError>;

    /// Apply an [`Action`] to a target pet on a [`Team`].
    fn apply_single_effect(
        &mut self,
        affected_pet: &Rc<RefCell<Pet>>,
        afflicting_pet: &Rc<RefCell<Pet>>,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError>;

    /// Apply [`Shop`](crate::Shop) effects.
    fn apply_shop_effect(&mut self, effect: &Effect) -> Result<(), SAPTestError>;

    fn copy_effect(
        &self,
        attr_to_copy: &CopyType,
        targets: Vec<Rc<RefCell<Pet>>>,
        receiving_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError>;

    fn convert_gain_type_to_food(
        &self,
        gain_type: &GainType,
        target_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Option<Food>, SAPTestError>;

    fn convert_summon_type_to_pet(
        &self,
        summon_type: &SummonType,
        target_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Pet, SAPTestError>;

    fn swap_pets(
        &mut self,
        swap_type: &RandomizeType,
        effect: &Effect,
        trigger: &Outcome,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError>;

    fn shuffle_pets(
        &mut self,
        target_team: &Target,
        shuffle_type: &RandomizeType,
        effect: &Effect,
        trigger: &Outcome,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError>;

    fn summon_pet(
        &mut self,
        target_pet: &Rc<RefCell<Pet>>,
        summon_type: &SummonType,
        opponent: Option<&mut Team>,
    ) -> Result<Rc<RefCell<Pet>>, SAPTestError>;

    /// Get matching pets by [`ItemCondition`].
    fn get_matching_pets(
        &self,
        target: &Target,
        condition: &ItemCondition,
        opponent: &Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError>;

    /// Check if [`ConditionType`] is `true`.
    fn check_condition(
        &self,
        condition_type: &ConditionType,
        target_pet: &Rc<RefCell<Pet>>,
        opponent: &Option<&mut Team>,
    ) -> Result<bool, SAPTestError>;

    /// Apply a condition action.
    /// * action_set first arg is run if condition met. Otherwise, second arg run.
    fn apply_conditional_action(
        &mut self,
        affected_pet: &Rc<RefCell<Pet>>,
        afflicting_pet: &Rc<RefCell<Pet>>,
        condition_type: &LogicType,
        effect: &Effect,
        action_set: (&Action, &Action),
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError>;

    /// Hard-coded [`Tiger`](crate::PetName::Tiger) behavior.
    /// * Checks that pet behind current pet is a tiger.
    /// * Determines if [`Effect`] is valid by same methods in [`trigger_effects`](TeamEffects::trigger_effects).
    /// * Creates effects of `pet` at given tiger level.
    fn repeat_effects_if_tiger(
        &self,
        pet: &Rc<RefCell<Pet>>,
        trigger: &Outcome,
        trigger_petname: Option<&PetName>,
        same_pet_as_trigger: bool,
    ) -> Result<Vec<Effect>, SAPTestError>;

    /// Hard-coded [`Whale`](crate::PetName::Whale) behavior.
    fn evolve_pet(
        &mut self,
        affected_pet: &Rc<RefCell<Pet>>,
        afflicting_pet: &Rc<RefCell<Pet>>,
        lvl: usize,
        targets: Vec<Rc<RefCell<Pet>>>,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError>;

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
        afflicting_pet: &Rc<RefCell<Pet>>,
        opponent: &mut Team,
    ) -> Result<(), SAPTestError> {
        // Check for food uses.
        // Then copy food to avoid potential mut borrow.
        let item = afflicting_pet.borrow_mut().item.as_mut().and_then(|food| {
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
            let affected_pets = self.get_pets_by_effect(
                &valid_food.ability.trigger,
                &valid_food.ability,
                Some(opponent),
            )?;
            for affected_pet in affected_pets.iter() {
                if affected_pet.borrow().team.as_ref() == Some(&self.name) {
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
        trigger: &Outcome,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError> {
        let target_pets = if let Some(opponent) = opponent.as_ref() {
            self.get_pets_by_effect(trigger, effect, Some(opponent))?
        } else {
            self.get_pets_by_effect(trigger, effect, None)?
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
                pet_1.borrow_mut().swap(&mut pet_2.borrow_mut());
            }
            RandomizeType::Stats => {
                pet_1.borrow_mut().swap_stats(&mut pet_2.borrow_mut());
            }
        }
        Ok(vec![pet_1.clone(), pet_2.clone()])
    }

    fn shuffle_pets(
        &mut self,
        target_team: &Target,
        shuffle_type: &RandomizeType,
        effect: &Effect,
        trigger: &Outcome,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError> {
        let teams = match target_team {
            Target::Friend => vec![Some(self)],
            Target::Enemy => vec![opponent],
            Target::Either => vec![Some(self), opponent],
            _ => unimplemented!("Cannot shuffle on given target."),
        };
        let mut affected_pets = vec![];
        for team in teams.into_iter().flatten() {
            let mut rng = ChaCha12Rng::seed_from_u64(team.seed.unwrap_or_else(random));
            let curr_pet = team
                .curr_pet
                .as_ref()
                .map(|pet| pet.upgrade())
                .unwrap_or_else(|| team.first());
            let mut found_pets = team.get_pets_by_pos(
                curr_pet,
                // Find pets on current team.
                &Target::Friend,
                &effect.position,
                Some(trigger),
                None,
            )?;
            match shuffle_type {
                RandomizeType::Positions => {
                    // Shuffle to randomize found pets.
                    found_pets.shuffle(&mut rng);

                    // Then split into two pet chunks and swap pets.
                    for mut chunk in &found_pets.iter().chunks(2) {
                        let (Some(first_pet), Some(second_pet)) = (chunk.next(), chunk.next()) else {
                            continue;
                        };
                        first_pet.borrow_mut().swap(&mut second_pet.borrow_mut());
                        affected_pets.extend([first_pet.clone(), second_pet.clone()])
                    }
                    // Then reset indices in-place.
                    team.set_indices();
                }
                RandomizeType::Stats => {
                    // Invert stats. (2,1) -> (1,2)
                    for pet in found_pets.iter() {
                        pet.borrow_mut().stats.invert();
                        affected_pets.push(pet.clone())
                    }
                }
            }
        }

        Ok(affected_pets)
    }

    fn repeat_effects_if_tiger(
        &self,
        pet: &Rc<RefCell<Pet>>,
        trigger: &Outcome,
        trigger_petname: Option<&PetName>,
        same_pet_as_trigger: bool,
    ) -> Result<Vec<Effect>, SAPTestError> {
        let effect_pet_idx = pet.borrow().pos.ok_or(SAPTestError::InvalidTeamAction {
            subject: "No Pet Position Index.".to_string(),
            reason: format!("Pet {} must have an index set at this point.", pet.borrow()),
        })?;

        let mut tiger_doubled_effects = vec![];
        // For Tiger. Check if behind.
        if let Some(Some(pet_behind)) = self.friends.get(effect_pet_idx + 1) {
            if pet_behind.borrow().name == PetName::Tiger && self.shop.state == ShopState::Closed {
                // Get effect at level of tiger and repeat it.
                let pet_effect_at_tiger_lvl = pet.borrow().get_effect(pet_behind.borrow().lvl)?;
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

    fn convert_gain_type_to_food(
        &self,
        gain_type: &GainType,
        target_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Option<Food>, SAPTestError> {
        Ok(match gain_type {
            GainType::SelfItem => target_pet.borrow().item.clone(),
            GainType::DefaultItem(food_name) => Some(Food::try_from(food_name)?),
            GainType::StoredItem(food) => Some(*food.clone()),
            GainType::RandomShopItem => {
                let (sql, params) = self.shop.shop_query(Entity::Food, 1..self.shop.tier() + 1);
                self.convert_gain_type_to_food(&GainType::QueryItem(sql, params), target_pet)?
            }
            GainType::QueryItem(query, params) => {
                let food_records: Vec<FoodRecord> = SAPDB.execute_food_query(query, params)?;
                let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));
                // Only select one pet.
                let food_record =
                    food_records
                        .choose(&mut rng)
                        .ok_or(SAPTestError::QueryFailure {
                            subject: "Food Query".to_string(),
                            reason: format!("No record found for query: {query} with {params:?}"),
                        })?;
                Some(Food::try_from(food_record.name.clone())?)
            }
            GainType::NoItem => None,
        })
    }
    fn convert_summon_type_to_pet(
        &self,
        summon_type: &SummonType,
        target_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Pet, SAPTestError> {
        let mut new_pet = match summon_type {
            SummonType::QueryPet(sql, params, stats) => {
                let pet_records: Vec<PetRecord> = SAPDB.execute_pet_query(sql, params)?;
                let mut rng =
                    ChaCha12Rng::seed_from_u64(target_pet.borrow().seed.unwrap_or_else(random));
                // Only select one pet.
                let pet_record =
                    pet_records
                        .choose(&mut rng)
                        .ok_or(SAPTestError::QueryFailure {
                            subject: "Summon Query".to_string(),
                            reason: format!("No record found for query: {sql} with {params:?}"),
                        })?;
                // Give unique id.
                let mut pet = Pet::try_from(pet_record.clone())?;

                // Set stats if some value provided.
                if let Some(set_stats) = stats {
                    pet.stats = *set_stats;
                }
                pet
            }
            SummonType::StoredPet(box_pet) => *box_pet.clone(),
            SummonType::DefaultPet(default_pet) => Pet::try_from(default_pet.clone())?,
            SummonType::CustomPet(name, stat_types, lvl) => {
                let mut stats = match stat_types {
                    StatChangeType::StaticValue(stats) => *stats,
                    StatChangeType::SelfMultValue(stats) => {
                        target_pet.borrow().stats.mult_perc(stats)
                    }
                };
                Pet::new(
                    name.clone(),
                    Some(stats.clamp(1, MAX_PET_STATS).to_owned()),
                    *lvl,
                )?
            }
            SummonType::SelfPet(new_stats, new_level, keep_item) => {
                // Current pet. Remove item
                let mut pet = target_pet.borrow().clone();
                pet.item = if *keep_item {
                    target_pet.borrow().item.clone()
                } else {
                    None
                };
                pet.stats =
                    new_stats.map_or_else(|| target_pet.borrow().stats, |set_stats| set_stats);
                if let Some(new_level) = new_level {
                    pet.set_level(*new_level)?;
                }
                pet
            }
            SummonType::SelfTierPet(stats, level) => {
                let summon_query_type = SummonType::QueryPet(
                    "SELECT * FROM pets WHERE tier = ? AND lvl = ?".to_string(),
                    vec![
                        target_pet.borrow().tier.to_string(),
                        level.unwrap_or(1).to_string(),
                    ],
                    *stats,
                );
                self.convert_summon_type_to_pet(&summon_query_type, target_pet)?
            }
            SummonType::SelfTeamPet(stats, lvl, ignore_pet) => {
                let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));
                // Choose a pet on the current team that isn't the ignored pet.
                let chosen_friend_name = self
                    .friends
                    .iter()
                    .flatten()
                    .filter_map(|pet| {
                        let pet_name = pet.borrow().name.clone();
                        (pet_name != *ignore_pet).then_some(pet_name)
                    })
                    .choose(&mut rng);
                // NOTE: Allow to fail silently if no pet found.
                // Will only fail if friends empty or no valid friends found.
                if let Some(chosen_friend_name) = chosen_friend_name {
                    Pet::new(chosen_friend_name, *stats, lvl.unwrap_or(1))?
                } else {
                    return Err(SAPTestError::FallibleAction);
                }
            }
        };

        new_pet.id = Some(format!("{}_{}", new_pet.name, self.history.pet_count + 1));
        Ok(new_pet)
    }

    fn summon_pet(
        &mut self,
        target_pet: &Rc<RefCell<Pet>>,
        summon_type: &SummonType,
        opponent: Option<&mut Team>,
    ) -> Result<Rc<RefCell<Pet>>, SAPTestError> {
        // Can't impl TryFrom because requires target pet.
        let pet = self.convert_summon_type_to_pet(summon_type, target_pet)?;
        let target_idx = target_pet
            .borrow()
            .pos
            .ok_or(SAPTestError::InvalidTeamAction {
                subject: "Missing Summon Position".to_string(),
                reason: format!("Target pet {} has no position.", target_pet.borrow()),
            })?;

        // Handle case where pet in front faints and vector is empty.
        // Would panic attempting to insert at any position not at 0.
        // Also update position to be correct.
        let adj_target_idx = if target_idx > self.friends.len() {
            0
        } else {
            target_idx
        };

        self.add_pet(pet, adj_target_idx, opponent)?;
        // Should be safe to unwrap at this point.
        if let Some(Some(pet)) = self.friends.get(adj_target_idx) {
            Ok(pet.clone())
        } else {
            Err(SAPTestError::InvalidTeamAction {
                subject: "Missing Summoned Pet".to_string(),
                reason: "Something went wrong with added pet.".to_string(),
            })
        }
    }

    fn evolve_pet(
        &mut self,
        affected_pet: &Rc<RefCell<Pet>>,
        afflicting_pet: &Rc<RefCell<Pet>>,
        lvl: usize,
        targets: Vec<Rc<RefCell<Pet>>>,
        opponent: Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError> {
        let chosen_pet = targets.first().ok_or(SAPTestError::InvalidTeamAction {
            subject: "Evolve Pet".to_string(),
            reason: "No pet found to evolve.".to_string(),
        })?;

        // Clone the pet, upgrade the chosen pet's abilities, and remove its item.
        let mut leveled_pet = chosen_pet.borrow().clone();
        leveled_pet.set_level(lvl)?;
        leveled_pet.item = None;

        // Kill the original pet.
        let mut kill_effect = Effect {
            action: Action::Kill,
            ..Default::default()
        };
        kill_effect.assign_owner(Some(chosen_pet));
        self.apply_single_effect(chosen_pet, afflicting_pet, &kill_effect, opponent)?;

        // Set the target's pet ability to summon the pet.
        let target_pet_ref = Rc::downgrade(affected_pet);
        let mut target_pet_trigger = TRIGGER_SELF_FAINT;
        target_pet_trigger.affected_pet = Some(target_pet_ref.clone());
        affected_pet.borrow_mut().effect = vec![Effect {
            owner: Some(target_pet_ref),
            entity: Entity::Pet,
            trigger: target_pet_trigger,
            target: Target::Friend,
            position: Position::OnSelf,
            action: Action::Summon(SummonType::StoredPet(Box::new(leveled_pet.clone()))),
            uses: Some(1),
            temp: true,
        }];
        info!(target: "run", "(\"{}\")\nEvolving {}.", self.name, leveled_pet);
        info!(target: "run", "(\"{}\")\nSet pet {} to summon evolved pet on faint.", self.name, affected_pet.borrow());
        Ok(vec![chosen_pet.clone(), affected_pet.clone()])
    }

    fn copy_effect(
        &self,
        attr_to_copy: &CopyType,
        targets: Vec<Rc<RefCell<Pet>>>,
        receiving_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError> {
        let mut affected_pets = vec![];
        // Choose the first pet.
        let copied_attr = if let Some(pet_to_copy) = targets.first() {
            affected_pets.push(pet_to_copy.clone());

            match attr_to_copy.clone() {
                CopyType::Stats(replacement_stats) => Some(CopyType::Stats(
                    replacement_stats.map_or(Some(pet_to_copy.borrow().stats), Some),
                )),
                CopyType::PercentStats(perc_stats_mult) => {
                    // Multiply the stats of a chosen pet by some multiplier
                    let mut new_stats = pet_to_copy.borrow().stats.mult_perc(&perc_stats_mult);
                    new_stats.clamp(MIN_PET_STATS, MAX_PET_STATS);
                    info!(
                        target: "run", "(\"{}\")\nCopied {}% atk and {}% health from {}.",
                        self.name,
                        perc_stats_mult.attack,
                        perc_stats_mult.health,
                        receiving_pet.borrow()
                    );
                    Some(CopyType::Stats(Some(new_stats)))
                }
                CopyType::Effect(_, lvl) => Some(CopyType::Effect(
                    pet_to_copy.borrow().get_effect(lvl.unwrap_or(1))?,
                    lvl,
                )),
                CopyType::Item(_) => pet_to_copy
                    .borrow()
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
        match copied_attr.unwrap_or(CopyType::None) {
            CopyType::Stats(new_stats) => {
                // If some stats given use those as base.
                let new_stats = if let Some(mut new_stats) = new_stats {
                    // If any stat value is 0, use the target's original stats, otherwise, use the new stats.
                    *new_stats.comp_set_value(&receiving_pet.borrow().stats, 0)
                } else {
                    // Otherwise, copy stats from target.
                    receiving_pet.borrow().stats
                };

                receiving_pet.borrow_mut().stats = new_stats;

                info!(
                    target: "run", "(\"{}\")\nSet stats for {} to {}.",
                    self.name,
                    receiving_pet.borrow(),
                    receiving_pet.borrow().stats
                );
            }
            CopyType::Effect(mut effects, _) => {
                for effect in effects.iter_mut() {
                    effect.assign_owner(Some(receiving_pet));
                }
                receiving_pet.borrow_mut().effect = effects;

                info!(
                    target: "run", "(\"{}\")\nSet effect for {} to {:?}.",
                    self.name,
                    receiving_pet.borrow(),
                    receiving_pet.borrow().effect
                );
            }
            CopyType::Item(item) => {
                if let Some(mut food) = item {
                    // Assign ability owner to target_pet.
                    food.ability.assign_owner(Some(receiving_pet));

                    receiving_pet.borrow_mut().item = Some(*food);
                    info!(
                        target: "run", "(\"{}\")\nCopyied item for {} to {:?}.",
                        self.name,
                        receiving_pet.borrow(),
                        receiving_pet.borrow().item
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
        target_pet: &Rc<RefCell<Pet>>,
        opponent: &Option<&mut Team>,
    ) -> Result<bool, SAPTestError> {
        fn match_team_cond(team: &Team, cond: &TeamCondition) -> bool {
            match cond {
                TeamCondition::PreviousBattle(outcome) => {
                    // Get last battle outcome and if matches condition, apply effect.
                    if let Some(last_outcome) = team.history.fight_outcomes.last() {
                        last_outcome == outcome
                    } else {
                        false
                    }
                }
                TeamCondition::OpenSpaceEqual(des_num_open) => {
                    // Number of spaces open.
                    *des_num_open == team.open_slots()
                }
                TeamCondition::NumberPetsEqual(num_pets) => *num_pets == team.filled_slots(),
                TeamCondition::NumberPetsGreaterEqual(num_pets) => *num_pets <= team.filled_slots(),
                TeamCondition::NumberFaintedMultiple(multiple) => {
                    team.fainted.len() % *multiple == 0
                }
            }
        }
        match condition_type {
            ConditionType::Pet(target, cond) => Ok(self
                .get_matching_pets(target, cond, opponent)?
                .contains(target_pet)),
            ConditionType::Team(target, cond) => {
                if let Target::Friend = target {
                    Ok(match_team_cond(self, cond))
                } else if let (Target::Enemy, Some(opponent)) = (target, opponent) {
                    Ok(match_team_cond(opponent, cond))
                } else {
                    return Err(SAPTestError::InvalidTeamAction {
                        subject: "Invalid Target".to_string(),
                        reason: ("Target not a team or no team provided.").to_string(),
                    });
                }
            }
            ConditionType::Shop(cond) => match cond {
                ShopCondition::InState(state) => Ok(self.shop.state == *state),
                ShopCondition::GoldEqual(gold) => Ok(self.gold() == *gold),
                ShopCondition::GoldGreaterEqual(gold) => Ok(self.gold() >= *gold),
            },
        }
    }

    fn get_matching_pets(
        &self,
        target: &Target,
        condition: &ItemCondition,
        opponent: &Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError> {
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
        affected_pet: &Rc<RefCell<Pet>>,
        afflicting_pet: &Rc<RefCell<Pet>>,
        logic_type: &LogicType,
        effect: &Effect,
        action_set: (&Action, &Action),
        mut opponent: Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError> {
        let mut affected_pets = vec![];

        // /// TODO: Maybe change to execute difference from total?
        // /// ex. Conditional action for each open slot on team. if 0: execute if_action 0 times, else: execute else_action 5 times.
        // /// Get the number of pets at a given target.
        // fn num_pets(target: &Target, team: &Team, opponent: &Option<&mut Team>) -> usize {
        //     match target {
        //         Target::Friend => team.all().len(),
        //         Target::Enemy => opponent.as_ref().map(|opponent| opponent.all().len()).unwrap_or(0),
        //         Target::Shop => team.shop.pets.len(),
        //         Target::Either => team.all().len() + num_pets(&Target::Enemy, team, opponent),
        //         Target::None => 0,
        //     }
        // }

        /// Get the number of actions given a ConditionType.
        fn num_actions_for_each(
            cond_type: &ConditionType,
            team: &Team,
            opponent: &Option<&mut Team>,
        ) -> Result<usize, SAPTestError> {
            match cond_type {
                // Get number of pets matching condition
                ConditionType::Pet(target, cond) => {
                    Ok(team.get_matching_pets(target, cond, opponent)?.len())
                }
                ConditionType::Team(target, cond) => {
                    let selected_team = if *target == Target::Friend {
                        team
                    } else if let Some(opponent) = opponent.as_ref() {
                        opponent
                    } else {
                        return Err(SAPTestError::InvalidTeamAction {
                            subject: format!("Incompatible Target {target:?} or Missing Opponent"),
                            reason: format!("Opponent must be known for this action or invalid target {target:?} for {cond_type:?}."),
                        });
                    };
                    match cond {
                        TeamCondition::PreviousBattle(outcome) => {
                            let matching_outcomes = selected_team
                                .history
                                .fight_outcomes
                                .iter()
                                .filter(|fight_outcome| *fight_outcome == outcome)
                                .count();

                            Ok(matching_outcomes)
                        }
                        _ => {
                            return Err(SAPTestError::InvalidTeamAction {
                                subject: "Not Implemented".to_string(),
                                reason: format!("{cond:?} not implemented for {cond_type:?}."),
                            })
                        }
                    }
                }
                _ => {
                    return Err(SAPTestError::InvalidTeamAction {
                        subject: "Not Implemented".to_string(),
                        reason: format!(
                            "ConditionType {cond_type:?} not implemented for LogicType::ForEach."
                        ),
                    })
                }
            }
        }

        // Get number of times action should be executed for action and other action.
        let num_actions = match logic_type {
            LogicType::ForEach(cond_type) => num_actions_for_each(cond_type, self, &opponent)?,
            LogicType::If(cond_type) => {
                if self.check_condition(cond_type, affected_pet, &opponent)? {
                    1
                } else {
                    0
                }
            }
            LogicType::IfNot(cond_type) => {
                if !self.check_condition(cond_type, affected_pet, &opponent)? {
                    1
                } else {
                    0
                }
            }
            LogicType::IfAny(cond_type) => match cond_type {
                ConditionType::Pet(target, cond) => {
                    // If any pet matches condition, run action.
                    if !self.get_matching_pets(target, cond, &opponent)?.is_empty() {
                        1
                    } else {
                        0
                    }
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
                                        affected_pets: &mut Vec<Rc<RefCell<Pet>>>|
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
        let effect_owner: Rc<RefCell<Pet>> = effect.try_into()?;
        // let mut affected_team_pets = vec![];

        match &effect.action {
            Action::AddShopStats(stats) => {
                for pet_slot in self.shop.pets.iter() {
                    if let ItemSlot::Pet(pet) = &pet_slot.item {
                        pet.borrow_mut().stats += *stats;
                    } else {
                        unreachable!("Cannot have food item in pets.")
                    };
                }
                self.shop.perm_stats += *stats;
                info!(target: "run", "(\"{}\")\nAdded permanent shop {}.", self.name, stats)
            }
            Action::AddShopFood(gain_food_type) => {
                let new_shop_food = self
                    .convert_gain_type_to_food(gain_food_type, &effect_owner)?
                    .map(ShopItem::from);
                info!(target: "run", "(\"{}\")\nAdding shop item {:?}.", self.name, new_shop_food.as_ref());

                if let Some(Err(err)) = new_shop_food.map(|item| self.shop.add_item(item)) {
                    info!(target: "run", "(\"{}\")\n{err}.", self.name)
                }
            }
            Action::AddShopPet(summon_type) => {
                let new_shop_pet =
                    ShopItem::from(self.convert_summon_type_to_pet(summon_type, &effect_owner)?);
                info!(target: "run", "(\"{}\")\nAdding shop item {:?}.", self.name, &new_shop_pet);

                if let Err(err) = self.shop.add_item(new_shop_pet) {
                    info!(target: "run", "(\"{}\")\n{err}.", self.name)
                }
            }
            Action::ClearShop(item_type) => {
                match item_type {
                    Entity::Pet => self.shop.pets.clear(),
                    Entity::Food => self.shop.foods.clear(),
                }
                info!(target: "run", "(\"{}\")\nCleared shop {item_type:?}.", self.name)
            }
            Action::Profit(coins) => {
                for _ in 0..*coins {
                    self.shop.coins += 1;
                    info!(target: "run", "(\"{}\")\nIncreased shop coins by 1. New coin count: {}", self.name, self.shop.coins)
                }
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
                };
                for item in shop_items.filter(|item| affected_items_copy.contains(item)) {
                    item.cost = item.cost.saturating_sub(*discount)
                }
            }
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
                for pet in self.get_pets_by_effect(&TRIGGER_NONE, effect, None)? {
                    self.apply_single_effect(&pet, &effect_owner, effect, None)?;
                }
            }
        }

        Ok(())
    }
    fn apply_single_effect(
        &mut self,
        affected_pet: &Rc<RefCell<Pet>>,
        afflicting_pet: &Rc<RefCell<Pet>>,
        effect: &Effect,
        mut opponent: Option<&mut Team>,
    ) -> Result<Vec<Rc<RefCell<Pet>>>, SAPTestError> {
        let mut affected_pets: Vec<Rc<RefCell<Pet>>> = vec![];
        // Store copy of original effect as may be modified.
        let mut modified_effect = effect.clone();

        match &effect.action {
            Action::Add(stat_change) => {
                // Cannot add stats to fainted pets.
                // If owner is dead and the trigger for effect was not faint, ignore it.
                if affected_pet.borrow().stats.health == 0
                    || (afflicting_pet.borrow().stats.health == 0
                        && effect.trigger.status != Status::Faint)
                {
                    return Ok(affected_pets);
                }

                let added_stats = match stat_change {
                    StatChangeType::StaticValue(stats) => *stats,
                    StatChangeType::SelfMultValue(stats) => {
                        let mult_stats = afflicting_pet.borrow().stats.mult_perc(stats);
                        // Update action for dag with static value.
                        modified_effect.action =
                            Action::Add(StatChangeType::StaticValue(mult_stats));
                        mult_stats
                    }
                };

                // If effect is temporary, store stats to be removed from referenced pet on reopening shop.
                if effect.temp
                    && effect.target == Target::Friend
                    && self.shop.state == ShopState::Open
                {
                    self.shop.temp_stats.push((
                        affected_pet.borrow().id.as_ref().unwrap().clone(),
                        added_stats,
                    ));
                }
                affected_pet.borrow_mut().stats += added_stats;
                info!(target: "run", "(\"{}\")\nAdded {} to {}.", self.name, added_stats, affected_pet.borrow());
                affected_pets.push(affected_pet.clone());
            }
            Action::Remove(stat_change) => {
                let mut remove_stats = match stat_change {
                    StatChangeType::StaticValue(stats) => *stats,
                    StatChangeType::SelfMultValue(stats) => {
                        let mult_stats = afflicting_pet.borrow().stats.mult_perc(stats);
                        mult_stats
                    }
                };
                // Check for food on effect owner. Add any effect dmg modifiers. ex. Pineapple
                if let Some(item) = afflicting_pet
                    .borrow()
                    .item
                    .as_ref()
                    .filter(|item| Status::IndirectAttackDmgCalc == item.ability.trigger.status)
                {
                    if let Action::Add(modifier) = &item.ability.action {
                        remove_stats = match modifier {
                            StatChangeType::StaticValue(stats) => remove_stats + *stats,
                            StatChangeType::SelfMultValue(stats_mult) => {
                                remove_stats.mult_perc(stats_mult)
                            }
                        }
                    }
                }
                // Update with remaining modifiers.
                modified_effect.action = Action::Remove(StatChangeType::StaticValue(remove_stats));

                let mut atk_outcome = affected_pet.borrow_mut().indirect_attack(&remove_stats);
                info!(target: "run", "(\"{}\")\nRemoved {} health from {}.", self.name, remove_stats.attack, affected_pet.borrow());

                // Update for DAG to show health loss.
                modified_effect.action =
                    Action::Remove(StatChangeType::StaticValue(atk_outcome.friend_stat_change));

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
            Action::Gain(gain_food_type) => {
                let mut food = self.convert_gain_type_to_food(gain_food_type, afflicting_pet)?;

                if food.is_none() {
                    info!(target: "run", "(\"{}\")\nRemoved food from {}.", self.name, affected_pet.borrow());
                } else if let Some(food) = food.as_mut() {
                    info!(target: "run", "(\"{}\")\nGave {} to {}.", self.name, food, affected_pet.borrow());
                    food.ability.assign_owner(Some(affected_pet));
                }

                affected_pet.borrow_mut().item = food;
                affected_pets.push(affected_pet.clone());
            }
            Action::Moose(stats) => {
                // TODO: Separate into two different actions: Unfreeze shop + StatChangeType::MultShopTier
                for item in self.shop.foods.iter_mut() {
                    item.state = ItemState::Normal
                }
                let mut min_tier = MAX_SHOP_TIER;
                for pet in self.shop.pets.iter_mut() {
                    let pet_tier = pet.tier();
                    if pet_tier < min_tier {
                        min_tier = pet_tier
                    }
                    pet.state = ItemState::Normal
                }
                let buffed_stats = *stats * Statistics::new(min_tier, min_tier)?;
                modified_effect.action = Action::Add(StatChangeType::StaticValue(buffed_stats));
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
                };
                if let Some(stolen_item) = (0..possible_items.len())
                    .choose(&mut rng)
                    .map(|idx| possible_items.remove(idx))
                {
                    match stolen_item.item {
                        ItemSlot::Pet(pet) => {
                            // Multiply pet stats.
                            pet.borrow_mut().stats *= Statistics::new(*multiplier, *multiplier)?;
                            self.buy_pet_behavior(
                                pet,
                                Some(afflicting_pet.clone()),
                                &effect.position,
                            )?;
                        }
                        ItemSlot::Food(food) => {
                            // If stats adds some static value, multiply it by the mulitplier
                            if let Action::Add(StatChangeType::StaticValue(mut stats)) =
                                food.borrow_mut().ability.action
                            {
                                let stat_multiplier = Statistics::new(*multiplier, *multiplier)?;
                                stats *= stat_multiplier
                            }
                            self.buy_food_behavior(
                                food,
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
                    let prev_target_lvl = affected_pet.borrow().lvl;
                    affected_pet.borrow_mut().add_experience(1)?;
                    info!(target: "run", "(\"{}\")\nGave experience point to {}.", self.name, affected_pet.borrow());

                    // Target leveled up. Create trigger.
                    let pet_leveled_up = if affected_pet.borrow().lvl != prev_target_lvl {
                        info!(target: "run", "(\"{}\")\nPet {} leveled up.", self.name, affected_pet.borrow());
                        let mut lvl_trigger = TRIGGER_ANY_LEVELUP;
                        lvl_trigger.affected_pet = Some(Rc::downgrade(affected_pet));
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
                    .filter(|pet| pet.borrow().stats.health == 0)
                    .count();
                if let Some(position) = affected_pet.borrow().pos.map(|idx| idx + num_fainted) {
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
                if let Some(target_idx) = affected_pet.borrow().pos {
                    let mut transformed_pet = Pet::new(pet_name.clone(), *stats, *lvl)?;
                    transformed_pet.set_pos(target_idx);
                    transformed_pet.team = Some(self.name.to_owned());

                    if (0..self.friends.len()).contains(&target_idx) {
                        self.friends.remove(target_idx);
                        info!(target: "run", "(\"{}\")\nTransformed pet at position {} to {}.", self.name, target_idx, &transformed_pet);
                        let rc_transformed_pet = Rc::new(RefCell::new(transformed_pet));

                        for effect in rc_transformed_pet.borrow_mut().effect.iter_mut() {
                            effect.assign_owner(Some(&rc_transformed_pet));
                        }
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
                affected_pet.borrow_mut().stats.health = 0;
                info!(target: "run", "(\"{}\")\nKilled pet {}.", self.name, affected_pet.borrow());

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
                let debuff_stats = affected_pet.borrow().stats.mult_perc(perc_stats);
                modified_effect.action = Action::Debuff(debuff_stats);

                affected_pet.borrow_mut().stats -= debuff_stats;
                info!(target: "run", "(\"{}\")\nMultiplied stats of {} by {}.", self.name, affected_pet.borrow(), perc_stats);
                affected_pets.push(affected_pet.clone());
            }
            Action::Lynx => {
                let opponent = opponent.as_mut().ok_or(SAPTestError::InvalidTeamAction {
                    subject: format!("Missing Opponent for {:?}", &effect.action),
                    reason: "Opponent must be known for this action.".to_string(),
                })?;

                let opponent_lvls: usize = opponent.all().iter().map(|pet| pet.borrow().lvl).sum();
                let lvl_dmg_action = Action::Remove(StatChangeType::StaticValue(Statistics::new(
                    opponent_lvls,
                    0,
                )?));
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
                    self.evolve_pet(affected_pet, afflicting_pet, *lvl, targets, Some(*opponent))?
                } else {
                    self.evolve_pet(affected_pet, afflicting_pet, *lvl, targets, None)?
                };
                affected_pets.extend(pets);
            }
            Action::Stegosaurus(stats) => {
                let mut turn_mult_stats = *stats;
                // Multiply by turn number. Need to multiply raw values since mult op treats as percent.
                let turn_multiplier = TryInto::<isize>::try_into(self.history.curr_turn)?;
                turn_mult_stats.attack *= turn_multiplier;
                turn_mult_stats.health *= turn_multiplier;

                // Modify action to add turn-multiplied stats and apply effect.
                modified_effect.action = Action::Add(StatChangeType::StaticValue(turn_mult_stats));
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
                let mut pet_stats = affected_pet.borrow().stats;
                pet_stats.attack = TryInto::<isize>::try_into(self.shop.tier())? + 1;

                affected_pet.borrow_mut().stats = pet_stats;
                info!(target: "run", "(\"{}\")\nSet stats of {} by {}.", self.name, affected_pet.borrow(), pet_stats);
                affected_pets.push(affected_pet.clone());
            }
            Action::Copy(attr, target, pos) => {
                // Create effect to select a pet.
                modified_effect.position = pos.clone();
                modified_effect.target = *target;

                let targets = if let Some(opponent) = opponent.as_mut() {
                    self.get_pets_by_effect(&TRIGGER_NONE, &modified_effect, Some(opponent))?
                } else {
                    self.get_pets_by_effect(&TRIGGER_NONE, &modified_effect, None)?
                };
                affected_pets.extend(self.copy_effect(attr, targets, affected_pet)?);
            }
            Action::Swap(RandomizeType::Stats) => {
                affected_pet.borrow_mut().stats.invert();
                affected_pets.push(affected_pet.clone());
            }
            Action::None => {}
            _ => {
                return Err(SAPTestError::InvalidTeamAction {
                    subject: "Action Not Implemented".to_string(),
                    reason: format!("Single action ({:?}) not implemented yet.", &effect.action),
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
                pets.choose(&mut rng).and_then(|pet| pet.borrow().pos)
            }
            Position::First => (!self.friends.is_empty()).then_some(0),
            Position::Last => Some(self.friends.len().saturating_sub(1)),
            Position::Relative(idx) => {
                let Some((Target::Friend, adj_idx)) = self.cvt_rel_idx_to_adj_idx(0, *idx).ok() else {
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
