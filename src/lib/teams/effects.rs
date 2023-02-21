use crate::{
    db::record::PetRecord,
    effects::{
        actions::{
            Action, ConditionType, CopyType, GainType, RandomizeType, StatChangeType, SummonType,
        },
        effect::{Effect, Entity, Modify},
        state::{Condition, Outcome, Position, Status, Target},
        stats::Statistics,
        trigger::*,
    },
    error::SAPTestError,
    pets::{
        names::PetName,
        pet::{MAX_PET_STATS, MIN_PET_STATS},
    },
    shop::store::ShopState,
    teams::{
        team::Team,
        viewer::{TargetPets, TeamViewer},
    },
    Food, Pet, PetCombat, ShopItem, ShopViewer, SAPDB,
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

fn is_nonspecific_position(pos: &Position) -> bool {
    matches!(
        pos,
        Position::Any(_) | Position::All(_) | Position::None | Position::Relative(_)
    )
}

/// Enable applying [`Effect`]s to multiple [`Team`]s.
/// ```rust no run
/// use saptest::TeamEffects;
/// ```
pub trait TeamEffects {
    /// Apply [`Pet`](crate::pets::pet::Pet) [`Effect`]s based on a team's stored [`Outcome`] triggers.
    /// # Example
    /// ```rust
    /// use saptest::{
    ///     TeamEffects, Team, TeamViewer,
    ///     Pet, PetName,
    ///     effects::trigger::TRIGGER_START_BATTLE
    /// };
    ///
    /// let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// let mut team = Team::new(&vec![mosquito; 5], 5).unwrap();
    /// let mut enemy_team = team.clone();
    ///
    /// // Add a start of battle trigger.
    /// team.triggers.push_front(TRIGGER_START_BATTLE);
    /// // Trigger effects.
    /// team.trigger_effects(Some(&mut enemy_team)).unwrap();
    ///
    /// // Exhaust triggers.
    /// assert_eq!(team.triggers.len(), 0);
    /// ```
    fn trigger_effects(&mut self, opponent: Option<&mut Team>) -> Result<&mut Self, SAPTestError>;
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
    /// let mut team = Team::new(&vec![mosquito.clone(); 5], 5).unwrap();
    /// let mut enemy_team = Team::new(&vec![mosquito; 5], 5).unwrap();
    /// enemy_team.set_seed(Some(0));
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
    ///     enemy_team.friends[4].borrow().stats,
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
    ) -> Result<(), SAPTestError>;
}

impl TeamEffects for Team {
    fn trigger_effects(
        &mut self,
        mut opponent: Option<&mut Team>,
    ) -> Result<&mut Self, SAPTestError> {
        info!(target: "run", "(\"{}\")\nTriggers:\n{}", self.name, self.triggers.iter().join("\n"));

        // Continue iterating until all triggers consumed.
        while let Some(trigger) = self.triggers.pop_front() {
            let mut applied_effects: Vec<(Outcome, Effect)> = vec![];

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

            // Iterate through pets in descending order by attack strength to collect valid effects.
            for (effect_pet_idx, pet) in self
                .friends
                .iter()
                .enumerate()
                .sorted_by(|(_, pet_1), (_, pet_2)| {
                    pet_1
                        .borrow()
                        .stats
                        .attack
                        .cmp(&pet_2.borrow().stats.attack)
                })
                .rev()
            {
                let same_pet_as_trigger = trigger
                    .clone()
                    .affected_pet
                    .map_or(false, |trigger_pet| trigger_pet.ptr_eq(&Rc::downgrade(pet)));

                // Get food and pet effect based on if its trigger is equal to current trigger, if any.
                if let Some(food) = pet.borrow_mut().item.as_mut().filter(|food| {
                    (food.ability.trigger == trigger
                        // This bottom condition allows triggers for effects that activate on any position/positions
                        || (is_nonspecific_position(&food.ability.trigger.position)
                            && food.ability.trigger.position == trigger.position
                            && food.ability.trigger.affected_team == trigger.affected_team
                            && food.ability.trigger.status == trigger.status))
                        && food.ability.uses != Some(0)
                }) {
                    // Drop uses by one if possible.
                    food.ability.remove_uses(1);
                    applied_effects.push((trigger.clone(), food.ability.clone()))
                };
                for pet_effect in pet.borrow_mut().effect.iter_mut().filter(|effect| {
                    (effect.trigger == trigger
                        // This bottom condition allows triggers for effects that activate on any position/positions. ex. Horse.
                        || (is_nonspecific_position(&effect.trigger.position)
                            && effect.trigger.position == trigger.position
                            && effect.trigger.affected_team == trigger.affected_team
                            && effect.trigger.status == trigger.status))
                        && effect.uses != Some(0)
                }) {
                    // Check the trigger name as final check before adding effect.
                    // Specific check for:
                    //  * If trigger for a summon action is a Zombie Fly, ignore it.
                    //  * If trigger for a summon action is a Fly and is also the current pet is that fly, ignore it.
                    if let Some(trigger_pet_name) = trigger_pet_name.as_ref() {
                        if let Action::Summon(_) = pet_effect.action {
                            if *trigger_pet_name == PetName::ZombieFly
                                || (*trigger_pet_name == PetName::Fly && same_pet_as_trigger)
                            {
                                continue;
                            }
                        } else if let Action::Add(_) = pet_effect.action {
                            // On self trigger and position any, ignore effect.
                            if trigger.position == Position::Any(Condition::None)
                                && same_pet_as_trigger
                            {
                                continue;
                            }
                        }
                    }
                    // For Tiger. Check if behind. Determines number of times effect applied.
                    let num_times_applied = self
                        .friends
                        .get(effect_pet_idx + 1)
                        .map(|pet| {
                            if pet.borrow().name == PetName::Tiger {
                                2
                            } else {
                                1
                            }
                        })
                        .unwrap_or(1);

                    // Drop uses by one if possible.
                    pet_effect.remove_uses(1);

                    for _ in 0..num_times_applied {
                        applied_effects.push((trigger.clone(), pet_effect.clone()))
                    }
                }
            }

            // Pet sold. Remove pet from friends.
            // Keep reference alive until rest of effects activated.
            let _sold_pet = if (trigger.status, trigger.position, trigger.affected_team)
                == (Status::Sell, Position::OnSelf, Target::Friend)
            {
                trigger_pet_pos.map(|pos| self.friends.remove(pos))
            } else {
                None
            };

            // Apply effects in reverse so proper order followed.
            for (trigger, effect) in applied_effects.into_iter().rev() {
                // Add node here for activated effect.
                let node_idx = self.history.effect_graph.add_node(trigger.clone());
                self.history.curr_node = Some(node_idx);
                if let Some(opponent) = opponent.as_mut() {
                    self.apply_effect(&trigger, &effect, Some(opponent))?
                } else {
                    self.apply_effect(&trigger, &effect, None)?
                }
            }

            // Set curr node to previous.
            self.history.prev_node = self.history.curr_node;
        }

        Ok(self)
    }

    fn apply_effect(
        &mut self,
        trigger: &Outcome,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError> {
        // Set current pet.
        self.curr_pet = effect.owner.clone();

        let target_pets = if let Some(opponent) = opponent.as_ref() {
            self.get_pets_by_effect(trigger, effect, Some(opponent))?
        } else {
            self.get_pets_by_effect(trigger, effect, None)?
        };

        match (&effect.target, &effect.action) {
            (_, Action::Swap(swap_type)) => {
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
                let ((_, pet_1), (_, pet_2)) =
                    (target_pets.first().unwrap(), target_pets.get(1).unwrap());
                match swap_type {
                    RandomizeType::Positions => {
                        pet_1.borrow_mut().swap(&mut pet_2.borrow_mut());
                        self
                    }
                    RandomizeType::Stats => {
                        pet_1.borrow_mut().swap_stats(&mut pet_2.borrow_mut());
                        self
                    }
                };
            }
            // Must be here to only activate once.
            (target_team, Action::Shuffle(shuffle_by)) => {
                let teams = match target_team {
                    Target::Friend => vec![Some(self)],
                    Target::Enemy => vec![opponent],
                    Target::Either => vec![Some(self), opponent],
                    _ => unimplemented!("Cannot shuffle on given target."),
                };
                for team in teams.into_iter().flatten() {
                    let mut rng = ChaCha12Rng::seed_from_u64(team.seed.unwrap_or_else(random));
                    match shuffle_by {
                        RandomizeType::Positions => {
                            team.friends.shuffle(&mut rng);
                            team.set_indices();
                        }
                        RandomizeType::Stats => {
                            for pet in team.friends.iter() {
                                pet.borrow_mut().stats.invert();
                            }
                        }
                    }
                }
            }
            // All shop actions go here.
            (Target::Shop, _) => {
                let Some(Some(_effect_pet)) = self.curr_pet.as_ref().map(|pet| pet.upgrade()) else {
                    return Err(SAPTestError::InvalidTeamAction {
                        subject: "No Effect Owner (Shop)".to_string(),
                        reason: "Reference to pet should still live for its shop effect to activate.".to_string()
                    });
                };
                // If target pets empty, effect doesn't affect pet. Use effect pet as placeholder.
                if target_pets.is_empty() {
                    self.apply_single_effect(_effect_pet, effect, None)?;
                } else {
                    for (_, pet) in target_pets {
                        self.apply_single_effect(pet.clone(), effect, None)?;
                    }
                }
            }
            _ => {
                if let Some(opponent) = opponent {
                    for (team, pet) in target_pets.into_iter() {
                        match team {
                            Target::Friend => {
                                self.apply_single_effect(pet, effect, Some(opponent))?
                            }
                            Target::Enemy => {
                                opponent.apply_single_effect(pet, effect, Some(self))?
                            }
                            _ => unimplemented!(),
                        };
                    }
                } else {
                    for (_, pet) in target_pets.into_iter() {
                        self.apply_single_effect(pet, effect, None)?
                    }
                }
            }
        }

        Ok(())
    }
}
pub(crate) trait EffectApplyHelpers {
    /// Apply an `Action` to a target idx on a `Team`.
    fn apply_single_effect(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        effect: &Effect,
        opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError>;
    fn copy_effect(
        &self,
        attr_to_copy: &CopyType,
        targets: TargetPets,
        receiving_pet: &Rc<RefCell<Pet>>,
    ) -> Result<(), SAPTestError>;
    fn gain_type_to_food(
        &self,
        gain_type: &GainType,
        target_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Option<Food>, SAPTestError>;
    fn summon_type_to_pet(
        &self,
        summon_type: &SummonType,
        target_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Pet, SAPTestError>;
    fn summon_pet(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        summon_type: &SummonType,
        opponent: Option<&mut Team>,
    ) -> Result<Option<String>, SAPTestError>;
    fn evolve_pet(
        &mut self,
        lvl: usize,
        targets: TargetPets,
        target_pet: Rc<RefCell<Pet>>,
        opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError>;
    /// Calculates an adjusted index based on the current index and a relative index.
    /// * `:param curr_idx:` The current index.
    /// * `:param rel_idx:` Number of positions relative to the current index.
    ///     * If *negative*, the index is **behind** the current index.
    ///     * If *positive*, the index is **ahead** of the current index.
    ///
    /// Output:
    /// * Value of the new index on a team represented by a variant in the enum `Target`.
    fn _cvt_rel_idx_to_adj_idx(
        &self,
        curr_idx: usize,
        rel_idx: isize,
    ) -> Result<(Target, usize), SAPTestError>;
}

impl EffectApplyHelpers for Team {
    fn gain_type_to_food(
        &self,
        gain_type: &GainType,
        target_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Option<Food>, SAPTestError> {
        Ok(match gain_type {
            GainType::SelfItem => target_pet.borrow().item.clone(),
            GainType::DefaultItem(food_name) => Some(Food::try_from(food_name)?),
            GainType::StoredItem(food) => Some(*food.clone()),
            GainType::NoItem => None,
        })
    }
    fn summon_type_to_pet(
        &self,
        summon_type: &SummonType,
        target_pet: &Rc<RefCell<Pet>>,
    ) -> Result<Pet, SAPTestError> {
        match summon_type {
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
                let mut pet = Pet::try_from(pet_record.clone())?;
                // Set stats if some value provided.
                if let Some(set_stats) = stats {
                    pet.stats = *set_stats;
                }
                Ok(pet)
            }
            SummonType::StoredPet(box_pet) => Ok(*box_pet.clone()),
            SummonType::DefaultPet(default_pet) => Pet::try_from(default_pet.clone()),
            SummonType::CustomPet(name, stat_types, lvl) => {
                let mut stats = match stat_types {
                    StatChangeType::StaticValue(stats) => *stats,
                    StatChangeType::SelfMultValue(stats) => {
                        target_pet.borrow().stats.mult_perc(stats)
                    }
                };
                Pet::new(
                    name.clone(),
                    None,
                    Some(stats.clamp(1, MAX_PET_STATS).to_owned()),
                    *lvl,
                )
            }
            SummonType::SelfPet(stats) => {
                // Current pet. Remove item
                let mut pet = target_pet.borrow().clone();
                pet.item = None;
                pet.stats = *stats;
                Ok(pet)
            }
            SummonType::SelfTierPet => {
                let summon_query_type = SummonType::QueryPet(
                    "SELECT * FROM pets where tier = ?".to_string(),
                    vec![target_pet.borrow().tier.to_string()],
                    None,
                );
                self.summon_type_to_pet(&summon_query_type, target_pet)
            }
        }
    }
    fn summon_pet(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        summon_type: &SummonType,
        opponent: Option<&mut Team>,
    ) -> Result<Option<String>, SAPTestError> {
        // Can't impl TryFrom because requires target pet.
        let pet = self.summon_type_to_pet(summon_type, &target_pet)?;

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
        let new_pet_id = self
            .friends
            .get(adj_target_idx)
            .unwrap()
            .borrow()
            .id
            .clone();
        Ok(new_pet_id)
    }

    fn evolve_pet(
        &mut self,
        lvl: usize,
        targets: TargetPets,
        target_pet: Rc<RefCell<Pet>>,
        opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError> {
        let (_, chosen_pet) = targets.first().ok_or(SAPTestError::InvalidTeamAction {
            subject: "Evolve Pet".to_string(),
            reason: "No pet found to evolve.".to_string(),
        })?;

        // Clone the pet, upgrade the chosen pet's abilities, and remove its item.
        let mut leveled_pet = chosen_pet.borrow().clone();
        leveled_pet.set_level(lvl)?;
        leveled_pet.item = None;

        // Kill the original pet.
        chosen_pet.borrow_mut().stats.health = 0;
        info!(target: "run", "(\"{}\")\nKilled pet {}.", self.name, chosen_pet.borrow());

        // Add death triggers.
        let mut self_faint_triggers = get_self_faint_triggers(&None);
        let mut enemy_self_faint_triggers = get_self_enemy_faint_triggers(&None);
        for trigger in self_faint_triggers
            .iter_mut()
            .chain(enemy_self_faint_triggers.iter_mut())
        {
            trigger.set_affected(chosen_pet);
        }
        self.triggers.extend(self_faint_triggers);
        if let Some(opponent) = opponent {
            opponent.triggers.extend(enemy_self_faint_triggers);
        }

        // Set the target's pet ability to summon the pet.
        let target_pet_ref = Rc::downgrade(&target_pet);
        let mut target_pet_trigger = TRIGGER_SELF_FAINT;
        target_pet_trigger.affected_pet = Some(target_pet_ref.clone());
        target_pet.borrow_mut().effect = vec![Effect {
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
        info!(target: "run", "(\"{}\")\nSet pet {} to summon evolved pet on faint.", self.name, target_pet.borrow());
        Ok(())
    }

    fn copy_effect(
        &self,
        attr_to_copy: &CopyType,
        targets: TargetPets,
        receiving_pet: &Rc<RefCell<Pet>>,
    ) -> Result<(), SAPTestError> {
        // Choose the first pet.
        let copied_attr = if let Some((_, pet_to_copy)) = targets.first() {
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
            CopyType::PercentStats(_) => {}
        }
        Ok(())
    }
    fn apply_single_effect(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        effect: &Effect,
        mut opponent: Option<&mut Team>,
    ) -> Result<(), SAPTestError> {
        let mut target_ids: Vec<Option<String>> = vec![];
        let effect_owner = effect
            .owner
            .clone()
            .ok_or(SAPTestError::InvalidTeamAction {
                subject: "Missing Effect Owner".to_string(),
                reason: format!("{effect:?} has no owner."),
            })?
            .upgrade()
            .ok_or(SAPTestError::InvalidTeamAction {
                subject: "Dropped Owner".to_string(),
                reason: "Pet reference dropped.".to_string(),
            })?;

        match &effect.action {
            Action::Add(stat_change) => {
                let added_stats = match stat_change {
                    StatChangeType::StaticValue(stats) => *stats,
                    StatChangeType::SelfMultValue(stats) => {
                        effect_owner.borrow().stats.mult_perc(stats)
                    }
                };
                // If effect is temporary, store stats to be removed from referenced pet on reopening shop.
                if effect.temp
                    && effect.target == Target::Friend
                    && self.shop.state == ShopState::Open
                {
                    self.shop.temp_stats.push((
                        target_pet.borrow().id.as_ref().unwrap().clone(),
                        added_stats,
                    ));
                }
                target_pet.borrow_mut().stats += added_stats;
                info!(target: "run", "(\"{}\")\nAdded {} to {}.", self.name, added_stats, target_pet.borrow());
                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Remove(stat_change) => {
                let mut remove_stats = match stat_change {
                    StatChangeType::StaticValue(stats) => *stats,
                    StatChangeType::SelfMultValue(stats) => {
                        effect_owner.borrow().stats.mult_perc(stats)
                    }
                };
                // Check for food. Add any effect dmg modifiers.
                if let Some(item) = effect_owner
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
                let mut atk_outcome = target_pet.borrow_mut().indirect_attack(&remove_stats);

                // Update triggers from where they came from.
                for trigger in atk_outcome
                    .friends
                    .iter_mut()
                    .chain(atk_outcome.opponents.iter_mut())
                {
                    trigger.set_affected(&target_pet);
                    trigger.afflicting_pet = effect.owner.clone();
                }
                // Collect triggers for both teams.
                info!(target: "run", "(\"{}\")\nRemoved {} health from {}.", self.name, remove_stats.attack, target_pet.borrow());
                self.triggers.extend(atk_outcome.friends);
                if let Some(opponent) = opponent {
                    opponent.triggers.extend(atk_outcome.opponents);
                }

                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Vulture(stats) => {
                let opponent = opponent.ok_or(SAPTestError::InvalidTeamAction {
                    subject: format!("Missing Opponent for {:?}", &effect.action),
                    reason: "Opponent must be known for this action.".to_string(),
                })?;
                // Should only target enemies for this to work as we are only checking the opposing team the effect comes from.
                // Add 1 because faint triggers and only after does pet move into fainted.
                let num_fainted = opponent.fainted.len() + 1;
                // If num fainted pets is even, do damage.
                if num_fainted % 2 == 0 {
                    info!(target: "run", "(\"{}\")\nTwo pets fainted.", self.name);
                    let mut remove_effect = effect.clone();
                    remove_effect.action = Action::Remove(StatChangeType::StaticValue(*stats));
                    self.apply_single_effect(target_pet, &remove_effect, Some(opponent))?;
                }
            }
            Action::Gain(gain_food_type) => {
                let food = self.gain_type_to_food(gain_food_type, &effect_owner)?;

                if food.is_none() {
                    info!(target: "run", "(\"{}\")\nRemoved food from {}.", self.name, target_pet.borrow());
                    target_pet.borrow_mut().item = None;
                    target_ids.push(target_pet.borrow().id.clone())
                } else if let Some(mut food) = food {
                    info!(target: "run", "(\"{}\")\nGave {} to {}.", self.name, food, target_pet.borrow());
                    food.ability.assign_owner(Some(&target_pet));
                    target_pet.borrow_mut().item = Some(food);
                    target_ids.push(target_pet.borrow().id.clone())
                }
            }
            Action::Experience => {
                let prev_target_lvl = target_pet.borrow().lvl;
                target_pet.borrow_mut().add_experience(1)?;
                info!(target: "run", "(\"{}\")\nGave experience point to {}.", self.name, target_pet.borrow());

                // Target leveled up. Create trigger.
                let pet_leveled_up = if target_pet.borrow().lvl != prev_target_lvl {
                    info!(target: "run", "(\"{}\")\nPet {} leveled up.", self.name, target_pet.borrow());
                    let mut lvl_trigger = TRIGGER_ANY_LEVELUP;
                    lvl_trigger.affected_pet = Some(Rc::downgrade(&target_pet));
                    Some(lvl_trigger)
                } else {
                    None
                };

                if let Some(level_trigger) = pet_leveled_up {
                    self.triggers.push_back(level_trigger)
                }
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
                if let Some(position) = target_pet.borrow().pos {
                    info!(target: "run", "(\"{}\")\nPushed pet at position {} by {}.", self.name, position, pos_change);
                    self.push_pet(position, pos_change, opponent)?;
                }
            }
            Action::Transform(pet_name, stats, lvl) => {
                if let Some(target_idx) = target_pet.borrow().pos {
                    let mut transformed_pet = Pet::new(pet_name.clone(), None, *stats, *lvl)?;
                    transformed_pet.set_pos(target_idx);

                    if (0..self.friends.len()).contains(&target_idx) {
                        self.friends.remove(target_idx);
                        info!(target: "run", "(\"{}\")\nTransformed pet at position {} to {}.", self.name, target_idx, &transformed_pet);
                        let rc_transformed_pet = Rc::new(RefCell::new(transformed_pet));

                        for effect in rc_transformed_pet.borrow_mut().effect.iter_mut() {
                            effect.assign_owner(Some(&rc_transformed_pet));
                        }
                        self.friends.insert(target_idx, rc_transformed_pet);
                    }
                }
            }
            Action::Summon(summon_type) => {
                let summon_result = self.summon_pet(target_pet, summon_type, opponent);
                if let Err(err) = summon_result {
                    // Fallible error. Attempted to add too many pets.
                    if let SAPTestError::InvalidPetAction { .. } = err {
                    } else {
                        // Otherwise, something actually went wrong.
                        return Err(err);
                    }
                } else if let Ok(summoned_pet_id) = summon_result {
                    target_ids.push(summoned_pet_id)
                };
            }
            Action::Multiple(actions) => {
                // Create new effect with single action.
                let mut effect_copy = effect.clone();
                if let Some(opponent) = opponent {
                    for action in actions {
                        effect_copy.action = action.clone();
                        self.apply_single_effect(target_pet.clone(), &effect_copy, Some(opponent))?
                    }
                } else {
                    for action in actions {
                        effect_copy.action = action.clone();
                        self.apply_single_effect(target_pet.clone(), &effect_copy, None)?
                    }
                }
            }
            Action::Conditional(condition_type, action) => {
                match condition_type {
                    ConditionType::ForEach(target, condition) => {
                        // Get number of pets matching condition
                        let num_matches = if *target == Target::Friend {
                            self.get_pets_by_cond(condition).len()
                        } else {
                            opponent
                                .as_ref()
                                .ok_or(SAPTestError::InvalidTeamAction {
                                    subject: format!("Missing Opponent for {:?}", &effect.action),
                                    reason: "Opponent must be known for this action.".to_string(),
                                })?
                                .get_pets_by_cond(condition)
                                .len()
                        };
                        // Create new effect with action.
                        let mut effect_copy = effect.clone();
                        effect_copy.action = *action.clone();
                        // For each pet that matches the condition, execute the action.

                        if let Some(opponent) = opponent {
                            for _ in 0..num_matches {
                                self.apply_single_effect(
                                    target_pet.clone(),
                                    &effect_copy,
                                    Some(opponent),
                                )?
                            }
                        } else {
                            for _ in 0..num_matches {
                                self.apply_single_effect(target_pet.clone(), &effect_copy, None)?
                            }
                        }
                    }
                    ConditionType::If(target, condition) => {
                        let matching_pets = if *target == Target::Friend {
                            self.get_pets_by_cond(condition)
                        } else {
                            opponent
                                .as_ref()
                                .ok_or(SAPTestError::InvalidTeamAction {
                                    subject: format!("Missing Opponent for {:?}", &effect.action),
                                    reason: "Opponent must be known for this action.".to_string(),
                                })?
                                .get_pets_by_cond(condition)
                        };

                        // If a pet matches condition, run action.
                        if matching_pets.contains(&target_pet) {
                            let mut effect_copy = effect.clone();
                            effect_copy.action = *action.clone();
                            self.apply_single_effect(target_pet, &effect_copy, opponent)?;
                        }
                    }
                }
            }
            Action::Kill => {
                target_pet.borrow_mut().stats.health = 0;
                info!(target: "run", "(\"{}\")\nKilled pet {}.", self.name, target_pet.borrow());

                let mut self_faint_triggers = get_self_faint_triggers(&None);
                let mut enemy_faint_triggers = get_self_enemy_faint_triggers(&None);

                for trigger in self_faint_triggers
                    .iter_mut()
                    .chain(enemy_faint_triggers.iter_mut())
                {
                    trigger.set_affected(&target_pet);
                }
                // Add death triggers.
                self.triggers.extend(self_faint_triggers);
                if let Some(opponent) = opponent {
                    opponent.triggers.extend(enemy_faint_triggers);
                }
            }
            Action::Rhino(stats) => {
                // Double damage against tier 1 pets.
                let tier_spec_stats = if target_pet.borrow().tier == 1 {
                    Statistics {
                        attack: stats.attack * 2,
                        health: stats.health,
                    }
                } else {
                    *stats
                };
                let mut atk_outcome = target_pet.borrow_mut().indirect_attack(&tier_spec_stats);

                // If kill by indirect, still counts as knockout.
                if target_pet.borrow().stats.health == 0 {
                    let mut knockout_trigger = TRIGGER_KNOCKOUT;
                    knockout_trigger.set_afflicting(&target_pet);
                    knockout_trigger.affected_pet = effect.owner.clone();
                    if let Some(opponent) = opponent.as_mut() {
                        opponent.triggers.push_front(knockout_trigger);
                    }
                }

                for trigger in atk_outcome
                    .friends
                    .iter_mut()
                    .chain(atk_outcome.opponents.iter_mut())
                {
                    trigger.set_affected(&target_pet);
                    trigger.afflicting_pet = effect.owner.clone();
                }

                // Collect triggers for both teams.
                self.triggers.extend(atk_outcome.friends);
                if let Some(opponent) = opponent {
                    opponent.triggers.extend(atk_outcome.opponents);
                }

                info!(target: "run", "(\"{}\")\nRemoved {} health from {}.", self.name, tier_spec_stats.attack, target_pet.borrow());
                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Debuff(perc_stats) => {
                let debuff_stats = target_pet.borrow().stats.mult_perc(perc_stats);
                target_pet.borrow_mut().stats -= debuff_stats;
                info!(target: "run", "(\"{}\")\nMultiplied stats of {} by {}.", self.name, target_pet.borrow(), perc_stats)
            }
            Action::Tapir => {
                let mut rng =
                    ChaCha12Rng::seed_from_u64(effect_owner.borrow().seed.unwrap_or_else(random));
                // Choose a pet on the current team that isn't a tapir.
                let chosen_friend = self
                    .friends
                    .iter()
                    .filter_map(|pet| {
                        let pet_name = pet.borrow().name.clone();
                        (pet_name != PetName::Tapir).then_some(pet_name)
                    })
                    .choose(&mut rng);

                if let Some(pet_name) = chosen_friend {
                    let summon =
                        Box::new(Pet::new(pet_name, None, None, effect_owner.borrow().lvl)?);
                    self.summon_pet(target_pet, &SummonType::StoredPet(summon), opponent)?;
                }
            }
            Action::Lynx => {
                let opponent_lvls: usize = opponent
                    .as_ref()
                    .ok_or(SAPTestError::InvalidTeamAction {
                        subject: format!("Missing Opponent for {:?}", &effect.action),
                        reason: "Opponent must be known for this action.".to_string(),
                    })?
                    .all()
                    .iter()
                    .map(|pet| pet.borrow().lvl)
                    .sum();
                let lvl_dmg_action = Action::Remove(StatChangeType::StaticValue(Statistics::new(
                    opponent_lvls,
                    0,
                )?));
                let mut effect_copy = effect.clone();
                effect_copy.action = lvl_dmg_action;

                self.apply_single_effect(target_pet, &effect_copy, opponent)?
            }
            Action::Whale(lvl, rel_pos) => {
                let mut copy_effect = effect.clone();
                // Based on a specific relative position, select the pet to 'swallow' and remove.
                copy_effect.position = rel_pos.clone();
                copy_effect.target = Target::Friend;

                let targets = self.get_pets_by_effect(&TRIGGER_NONE, &copy_effect, None)?;

                self.evolve_pet(*lvl, targets, target_pet, opponent)?;
            }
            Action::Stegosaurus(stats) => {
                let mut turn_mult_stats = *stats;
                // Multiply by turn number. Need to multiply raw values since mult op treats as percent.
                let turn_multiplier = TryInto::<isize>::try_into(self.history.curr_turn)?;
                turn_mult_stats.attack *= turn_multiplier;
                turn_mult_stats.health *= turn_multiplier;

                // Modify action to add turn-multiplied stats and apply effect.
                let mut effect_copy = effect.clone();
                effect_copy.action = Action::Add(StatChangeType::StaticValue(turn_mult_stats));
                self.apply_single_effect(target_pet, &effect_copy, opponent)?;
            }
            Action::Cockroach => {
                let mut pet_stats = target_pet.borrow().stats;
                pet_stats.attack = TryInto::<isize>::try_into(self.shop.tier())? + 1;

                target_pet.borrow_mut().stats = pet_stats;
                info!(target: "run", "(\"{}\")\nSet stats of {} by {}.", self.name, target_pet.borrow(), pet_stats)
            }
            Action::Copy(attr, target, pos) => {
                // Create effect to select a pet.
                let mut copy_effect = effect.clone();
                copy_effect.position = pos.clone();
                copy_effect.target = *target;

                let targets = if let Some(opponent) = opponent {
                    self.get_pets_by_effect(&TRIGGER_NONE, &copy_effect, Some(opponent))?
                } else {
                    self.get_pets_by_effect(&TRIGGER_NONE, &copy_effect, None)?
                };
                self.copy_effect(attr, targets, &target_pet)?;

                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::AddShopStats(stats) => {
                self.shop.perm_stats += *stats;
                info!(target: "run", "(\"{}\")\nAdded permanent shop {}.", self.name, stats)
            }
            Action::AddShopFood(gain_food_type) => {
                let new_shop_food = self
                    .gain_type_to_food(gain_food_type, &effect_owner)?
                    .map(ShopItem::from);
                info!(target: "run", "(\"{}\")\nAdding shop item {:?}.", self.name, new_shop_food.as_ref());

                if let Some(Err(err)) = new_shop_food.map(|item| self.shop.add_item(item)) {
                    info!(target: "run", "(\"{}\")\n{err}.", self.name)
                }
            }
            Action::AddShopPet(summon_type) => {
                let new_shop_pet =
                    ShopItem::from(self.summon_type_to_pet(summon_type, &effect_owner)?);
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
            Action::Profit => {
                self.shop.coins += 1;
                info!(target: "run", "(\"{}\")\nIncreased shop coins by 1. New coin count: {}", self.name, self.shop.coins)
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
                self.shop.refresh_costs();
            }
            Action::FreeRoll => {
                self.shop.free_rolls += 1;
                info!(target: "run", "(\"{}\")\nIncreased free rolls by 1. New free rolls: {}", self.name, self.shop.free_rolls)
            }
            Action::Swap(swap_type) => {
                // Only allow stat swap here.
                if let RandomizeType::Stats = swap_type {
                    target_pet.borrow_mut().stats.invert();
                }
            }
            Action::None => {}
            _ => {
                return Err(SAPTestError::InvalidTeamAction {
                    subject: "Action Not Implemented".to_string(),
                    reason: format!("Single action ({:?}) not implemented yet.", &effect.action),
                })
            }
        }
        // Create edge by iterating over all targets affected.
        if let (Some(prev_node), Some(curr_node)) = (self.history.prev_node, self.history.curr_node)
        {
            for target_id in target_ids {
                self.history.effect_graph.add_edge(
                    prev_node,
                    curr_node,
                    (
                        effect.target,
                        effect.position.clone(),
                        effect.action.clone(),
                        // If added, may not have and id. Default to 'None'.
                        target_id.unwrap_or_else(|| "None".to_string()),
                    ),
                );
            }
        };
        Ok(())
    }

    fn _cvt_rel_idx_to_adj_idx(
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
