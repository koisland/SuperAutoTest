use crate::{
    battle::{
        actions::{
            Action, ConditionType, CopyType, GainType, RandomizeType, StatChangeType, SummonType,
        },
        effect::{Effect, Entity, Modify},
        state::{Condition, Outcome, Position, Target},
        stats::Statistics,
        team::Team,
        trigger::*,
    },
    db::record::PetRecord,
    error::SAPTestError,
    pets::{
        names::PetName,
        pet::{MAX_PET_STATS, MIN_PET_STATS},
    },
    Food, Pet, PetCombat, SAPDB,
};

use itertools::Itertools;
use log::{error, info};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    SeedableRng,
};
use rand_chacha::ChaCha12Rng;
use std::{cell::RefCell, rc::Rc};

/// Pet doesn't store a reference to team so this was a workaround.
type TargetPet = Vec<(Target, Rc<RefCell<Pet>>)>;
const NONSPECIFIC_POSITIONS: [Position; 5] = [
    Position::None,
    Position::Any(Condition::None),
    Position::Any(Condition::NotSelf),
    Position::Relative(-1),
    Position::All(Condition::None),
];

/// Enable applying [`Effect`]s to multiple [`Team`]s.
/// # Example
/// ```
/// use saptest::EffectApply;
/// ```
pub trait EffectApply {
    /// Apply [`Pet`](crate::pets::pet::Pet) [`Effect`]s based on a team's stored [`Outcome`] triggers.
    /// # Examples
    /// ```rust
    /// use saptest::{EffectApply, Team, Pet, PetName};
    ///
    /// let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// let mut team = Team::new(&vec![mosquito; 5], 5).unwrap();
    /// let mut enemy_team = team.clone();
    ///
    /// // Start of battle triggers.
    /// assert_eq!(team.triggers.len(), 2);
    ///
    /// // Trigger effects.
    /// team.trigger_effects(&mut enemy_team);
    ///
    /// // Exhaust triggers.
    /// assert_eq!(team.triggers.len(), 0);
    /// ```
    fn trigger_effects(&mut self, opponent: &mut Team) -> &mut Self;
    /// Apply an [`Effect`] with an associated [`Outcome`] trigger to a [`Team`].
    /// * The `opponent` [`Team`] will get updated with additional [`Outcome`]s.
    /// * Effects and triggers should contain a Weak reference to the owning/affecting pet.
    /// # Examples
    /// ```rust
    /// use saptest::{EffectApply, Team, Pet, PetName, Statistics, battle::{state::Status, trigger::*}};
    /// // Get mosquito effect.
    /// let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// // Get effect with no reference.
    /// let no_ref_mosquito_effect = mosquito.effect.first().cloned().unwrap();
    ///
    /// // Init teams.
    /// let mut team = Team::new(&vec![mosquito.clone(); 5], 5).unwrap();
    /// let mut enemy_team = Team::new(&vec![mosquito; 5], 5).unwrap();
    /// enemy_team.set_seed(0);
    ///
    /// // Without a reference to the pet owning the effect, this will fail.
    /// assert!(team.apply_effect(&TRIGGER_START_BATTLE, &no_ref_mosquito_effect, &mut enemy_team).is_err());
    ///
    /// // Get mosquito_effect with reference.
    /// // Apply effect of mosquito at position 0 to a pet on team to enemy team.
    /// let mosquito_effect = team.first().unwrap().borrow().effect.first().cloned().unwrap();
    /// team.apply_effect(&TRIGGER_START_BATTLE, &mosquito_effect, &mut enemy_team).unwrap();
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
        opponent: &mut Team,
    ) -> Result<(), SAPTestError>;
}

impl EffectApply for Team {
    fn trigger_effects(&mut self, opponent: &mut Team) -> &mut Self {
        info!(target: "dev", "(\"{}\")\nTriggers:\n{}", self.name, self.triggers.iter().join("\n"));

        // Continue iterating until all triggers consumed.
        while let Some(trigger) = self.triggers.pop_front() {
            let mut applied_effects: Vec<(Outcome, Effect)> = vec![];

            // Get petname of trigger.
            let trigger_pet_name = if let Some(Some(trigger_pet)) =
                trigger.affected_pet.as_ref().map(|pet| pet.upgrade())
            {
                Some(trigger_pet.borrow().name.clone())
            } else {
                None
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
                        || (NONSPECIFIC_POSITIONS.contains(&food.ability.trigger.position)
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
                        || (NONSPECIFIC_POSITIONS.contains(&effect.trigger.position)
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
            // Apply effects in reverse so proper order followed.
            for (trigger, effect) in applied_effects.into_iter().rev() {
                // Add node here for activated effect.
                let node_idx = self.history.effect_graph.add_node(trigger.clone());
                self.history.curr_node = Some(node_idx);

                if let Err(err) = self.apply_effect(&trigger, &effect, opponent) {
                    error!(target: "dev", "(\"{}\")\nSomething went wrong. {:?}", self.name, err)
                };
            }

            // Set curr node to previous.
            self.history.prev_node = self.history.curr_node;
        }

        self
    }

    fn apply_effect(
        &mut self,
        trigger: &Outcome,
        effect: &Effect,
        opponent: &mut Team,
    ) -> Result<(), SAPTestError> {
        // Set current pet.
        self.curr_pet = effect.owner.clone();

        let target_pets = self.get_pets_by_effect(trigger, effect, opponent)?;
        match (&effect.target, &effect.action) {
            (_, Action::Swap(swap_type)) => {
                if target_pets.len() != 2 {
                    return Err(SAPTestError::InvalidTeamAction {
                        subject: format!("Swap {swap_type:?}"),
                        indices: vec![],
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
                        self.swap_pets(&mut pet_1.borrow_mut(), &mut pet_2.borrow_mut())
                    }
                    RandomizeType::Stats => {
                        self.swap_pet_stats(&mut pet_1.borrow_mut(), &mut pet_2.borrow_mut())
                    }
                };
            }
            // Must be here to only activate once.
            (target_team, Action::Shuffle(shuffle_by)) => {
                let teams = match target_team {
                    Target::Friend => vec![self],
                    Target::Enemy => vec![opponent],
                    Target::Either => vec![self, opponent],
                    _ => unimplemented!("Cannot shuffle on given target."),
                };
                for team in teams {
                    let mut rng = ChaCha12Rng::seed_from_u64(team.seed);
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

            _ => {
                for (team, pet) in target_pets.into_iter() {
                    match team {
                        Target::Friend => self.apply_single_effect(pet, effect, opponent)?,
                        Target::Enemy => opponent.apply_single_effect(pet, effect, self)?,
                        _ => unimplemented!(),
                    };
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
        opponent: &mut Team,
    ) -> Result<(), SAPTestError>;
    /// Get pets by Outcome trigger and a Position.
    fn get_pets_by_effect(
        &self,
        trigger: &Outcome,
        effect: &Effect,
        opponent: &Team,
    ) -> Result<TargetPet, SAPTestError>;
    fn copy_effect(
        &self,
        attr_to_copy: &CopyType,
        targets: TargetPet,
        receiving_pet: &Rc<RefCell<Pet>>,
    ) -> Result<(), SAPTestError>;
    fn summon_pet(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        summon_type: &SummonType,
        opponent: &mut Team,
    ) -> Result<Option<String>, SAPTestError>;
    fn evolve_pet(
        &mut self,
        lvl: usize,
        targets: TargetPet,
        target_pet: Rc<RefCell<Pet>>,
        opponent: &mut Team,
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
    fn summon_pet(
        &mut self,
        target_pet: Rc<RefCell<Pet>>,
        summon_type: &SummonType,
        opponent: &mut Team,
    ) -> Result<Option<String>, SAPTestError> {
        let pet = match summon_type {
            SummonType::QueryPet(sql, params, stats) => {
                let pet_records: Vec<PetRecord> = SAPDB.execute_pet_query(sql, params)?;
                let mut rng = ChaCha12Rng::seed_from_u64(target_pet.borrow().seed);
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
                pet
            }
            SummonType::StoredPet(box_pet) => *box_pet.clone(),
            SummonType::DefaultPet(default_pet) => Pet::try_from(default_pet.clone())?,
            SummonType::CustomPet(name, stat_types, lvl) => {
                let mut stats = match stat_types {
                    StatChangeType::StaticValue(stats) => *stats,
                    StatChangeType::SelfMultValue(stats) => target_pet.borrow().stats * *stats,
                };
                Pet::new(
                    name.clone(),
                    None,
                    Some(stats.clamp(1, MAX_PET_STATS).to_owned()),
                    *lvl,
                )?
            }
            SummonType::SelfPet(stats) => {
                // Current pet. Remove item
                let mut pet = target_pet.borrow().clone();
                pet.item = None;
                pet.stats = *stats;
                pet
            }
        };

        let target_idx = target_pet
            .borrow()
            .pos
            .ok_or(SAPTestError::InvalidTeamAction {
                subject: "Missing Summon Position".to_string(),
                indices: vec![],
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

        self.add_pet(pet, adj_target_idx, Some(opponent))?;
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
        targets: TargetPet,
        target_pet: Rc<RefCell<Pet>>,
        opponent: &mut Team,
    ) -> Result<(), SAPTestError> {
        let (_, chosen_pet) = targets.first().ok_or(SAPTestError::InvalidTeamAction {
            subject: "Evolve Pet".to_string(),
            indices: vec![],
            reason: "No pet found to evolve.".to_string(),
        })?;

        // Clone the pet, upgrade the chosen pet's abilities, and remove its item.
        let mut leveled_pet = chosen_pet.borrow().clone();
        leveled_pet.set_level(lvl)?;
        leveled_pet.item = None;

        // Kill the original pet.
        chosen_pet.borrow_mut().stats.health = 0;
        info!(target: "dev", "(\"{}\")\nKilled pet {}.", self.name, chosen_pet.borrow());

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
        opponent.triggers.extend(enemy_self_faint_triggers);

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
        info!(target: "dev", "(\"{}\")\nEvolving {}.", self.name, leveled_pet);
        info!(target: "dev", "(\"{}\")\nSet pet {} to summon evolved pet on faint.", self.name, target_pet.borrow());
        Ok(())
    }

    fn copy_effect(
        &self,
        attr_to_copy: &CopyType,
        targets: TargetPet,
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
                    let mut new_stats = pet_to_copy.borrow().stats * perc_stats_mult;
                    new_stats.clamp(MIN_PET_STATS, MAX_PET_STATS);
                    info!(
                        target: "dev", "(\"{}\")\nCopied {}% atk and {}% health from {}.",
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
                    target: "dev", "(\"{}\")\nSet stats for {} to {}.",
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
                    target: "dev", "(\"{}\")\nSet effect for {} to {:?}.",
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
                        target: "dev", "(\"{}\")\nCopyied item for {} to {:?}.",
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
        opponent: &mut Team,
    ) -> Result<(), SAPTestError> {
        let mut target_ids: Vec<Option<String>> = vec![];
        let effect_owner = effect
            .owner
            .clone()
            .ok_or(SAPTestError::InvalidTeamAction {
                subject: "Missing Effect Owner".to_string(),
                indices: vec![],
                reason: format!("{effect:?} has no owner."),
            })?
            .upgrade()
            .ok_or(SAPTestError::InvalidTeamAction {
                subject: "Dropped Owner".to_string(),
                indices: vec![],
                reason: "Pet reference dropped.".to_string(),
            })?;

        match &effect.action {
            Action::Add(stat_change) => {
                let added_stats = match stat_change {
                    StatChangeType::StaticValue(stats) => *stats,
                    StatChangeType::SelfMultValue(stats) => effect_owner.borrow().stats * *stats,
                };
                target_pet.borrow_mut().stats += added_stats;
                info!(target: "dev", "(\"{}\")\nAdded {} to {}.", self.name, added_stats, target_pet.borrow());
                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Remove(stat_change) => {
                let remove_stats = match stat_change {
                    StatChangeType::StaticValue(stats) => *stats,
                    StatChangeType::SelfMultValue(stats) => effect_owner.borrow().stats * *stats,
                };
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
                info!(target: "dev", "(\"{}\")\nRemoved {} health from {}.", self.name, remove_stats.attack, target_pet.borrow());
                self.triggers.extend(atk_outcome.friends);
                opponent.triggers.extend(atk_outcome.opponents);

                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Vulture(stats) => {
                // Should only target enemies for this to work as we are only checking the opposing team the effect comes from.
                // Add 1 because faint triggers and only after does pet move into fainted.
                let num_fainted = opponent.fainted.len() + 1;
                // If num fainted pets is even, do damage.
                if num_fainted % 2 == 0 {
                    info!(target: "dev", "(\"{}\")\nTwo pets fainted.", self.name);
                    let mut remove_effect = effect.clone();
                    remove_effect.action = Action::Remove(StatChangeType::StaticValue(*stats));
                    self.apply_single_effect(target_pet, &remove_effect, opponent)?;
                }
            }
            Action::Gain(gain_food_type) => {
                let food = match gain_food_type {
                    GainType::SelfItem => effect_owner.borrow().item.clone(),
                    GainType::DefaultItem(food_name) => Some(Food::try_from(food_name)?),
                    GainType::StoredItem(food) => Some(*food.clone()),
                };

                if let Some(mut food) = food {
                    info!(target: "dev", "(\"{}\")\nGave {} to {}.", self.name, food, target_pet.borrow());
                    food.ability.assign_owner(Some(&target_pet));
                    target_pet.borrow_mut().item = Some(food);
                    target_ids.push(target_pet.borrow().id.clone())
                }
            }
            Action::Experience => {
                let prev_target_lvl = target_pet.borrow().lvl;
                target_pet.borrow_mut().add_experience(1)?;
                info!(target: "dev", "(\"{}\")\nGave experience point to {}.", self.name, target_pet.borrow());

                // Target leveled up. Create trigger.
                let pet_leveled_up = if target_pet.borrow().lvl != prev_target_lvl {
                    info!(target: "dev", "(\"{}\")\nPet {} leveled up.", self.name, target_pet.borrow());
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
                    info!(target: "dev", "(\"{}\")\nPushed pet at position {} by {}.", self.name, position, pos_change);
                    self.push_pet(position, pos_change, Some(opponent))?;
                }
            }
            Action::Transform(pet_name, stats, lvl) => {
                if let Some(target_idx) = target_pet.borrow().pos {
                    let mut transformed_pet = Pet::new(pet_name.clone(), None, *stats, *lvl)?;
                    transformed_pet.set_pos(target_idx);

                    if (0..self.friends.len()).contains(&target_idx) {
                        self.friends.remove(target_idx);
                        info!(target: "dev", "(\"{}\")\nTransformed pet at position {} to {}.", self.name, target_idx, &transformed_pet);
                        let rc_transformed_pet = Rc::new(RefCell::new(transformed_pet));

                        for effect in rc_transformed_pet.borrow_mut().effect.iter_mut() {
                            effect.assign_owner(Some(&rc_transformed_pet));
                        }
                        self.friends.insert(target_idx, rc_transformed_pet);
                    }
                }
            }
            Action::Summon(summon_type) => {
                let summoned_pet_id = self.summon_pet(target_pet, summon_type, opponent)?;
                target_ids.push(summoned_pet_id);
            }
            Action::Multiple(actions) => {
                for action in actions {
                    // Create new effect with single action.
                    let mut effect_copy = effect.clone();
                    effect_copy.action = action.clone();
                    self.apply_single_effect(target_pet.clone(), &effect_copy, opponent)?
                }
            }
            Action::Conditional(condition_type, action) => {
                match condition_type {
                    ConditionType::ForEach(target, condition) => {
                        // Get number of pets matching condition
                        let num_matches = if *target == Target::Friend {
                            self.get_pets_by_cond(condition).len()
                        } else {
                            opponent.get_pets_by_cond(condition).len()
                        };
                        // Create new effect with action.
                        let mut effect_copy = effect.clone();
                        effect_copy.action = *action.clone();
                        // For each pet that matches the condition, execute the action.
                        for _ in 0..num_matches {
                            self.apply_single_effect(target_pet.clone(), &effect_copy, opponent)?
                        }
                    }
                    ConditionType::If(target, condition) => {
                        let matching_pets = if *target == Target::Friend {
                            self.get_pets_by_cond(condition)
                        } else {
                            opponent.get_pets_by_cond(condition)
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
                info!(target: "dev", "(\"{}\")\nKilled pet {}.", self.name, target_pet.borrow());

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
                opponent.triggers.extend(enemy_faint_triggers);
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
                    opponent.triggers.push_front(knockout_trigger)
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
                opponent.triggers.extend(atk_outcome.opponents);

                info!(target: "dev", "(\"{}\")\nRemoved {} health from {}.", self.name, tier_spec_stats.attack, target_pet.borrow());
                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::Debuff(perc_stats) => {
                let debuff_stats = target_pet.borrow().stats * *perc_stats;
                target_pet.borrow_mut().stats -= debuff_stats;
                info!(target: "dev", "(\"{}\")\nMultiplied stats of {} by {}.", self.name, target_pet.borrow(), perc_stats)
            }
            Action::Lynx => {
                let opponent_lvls: usize = opponent.all().iter().map(|pet| pet.borrow().lvl).sum();
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

                let targets = self.get_pets_by_effect(&TRIGGER_NONE, &copy_effect, opponent)?;

                self.evolve_pet(*lvl, targets, target_pet, opponent)?;
            }
            Action::Stegosaurus(stats) => {
                let mut turn_mult_stats = *stats;
                // Multiply by turn number. Need to multiply raw values since mult op treats as percent.
                let turn_multiplier = TryInto::<isize>::try_into(self.history.curr_turn + 1)?;
                turn_mult_stats.attack *= turn_multiplier;
                turn_mult_stats.health *= turn_multiplier;

                // Modify action to add turn-multiplied stats and apply effect.
                let mut effect_copy = effect.clone();
                effect_copy.action = Action::Add(StatChangeType::StaticValue(turn_mult_stats));
                self.apply_single_effect(target_pet, &effect_copy, opponent)?;
            }
            Action::Copy(attr, target, pos) => {
                // Create effect to select a pet.
                let mut copy_effect = effect.clone();
                copy_effect.position = pos.clone();
                copy_effect.target = *target;

                let targets = self.get_pets_by_effect(&TRIGGER_NONE, &copy_effect, opponent)?;
                self.copy_effect(attr, targets, &target_pet)?;

                target_ids.push(target_pet.borrow().id.clone())
            }
            Action::None => {}
            _ => unimplemented!("Action not implemented"),
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

    fn get_pets_by_effect(
        &self,
        trigger: &Outcome,
        effect: &Effect,
        opponent: &Team,
    ) -> Result<Vec<(Target, Rc<RefCell<Pet>>)>, SAPTestError> {
        let curr_pet = if let Some(effect_pet) = &effect.owner {
            effect_pet.upgrade()
        } else {
            return Err(SAPTestError::InvalidTeamAction {
                subject: "Current Pet Reference".to_string(),
                indices: vec![],
                reason: "Doesn't exist.".to_string(),
            });
        };

        let mut pets = vec![];
        match (effect.target, &effect.position) {
            (Target::Friend | Target::Enemy, Position::Any(condition)) => {
                let team = if effect.target == Target::Friend {
                    self
                } else {
                    opponent
                };
                let mut rng = ChaCha12Rng::seed_from_u64(team.seed);
                if let Some(random_pet) = team
                    .get_pets_by_cond(condition)
                    .into_iter()
                    .choose(&mut rng)
                {
                    pets.push((effect.target, random_pet))
                }
            }
            (Target::Either, Position::Any(condition)) => {
                let mut rng = ChaCha12Rng::seed_from_u64(self.seed);
                let self_pets = self.get_pets_by_cond(condition);
                let opponent_pets = opponent.get_pets_by_cond(condition);
                if let Some(random_pet) = vec![Target::Friend; self_pets.len()]
                    .into_iter()
                    .zip_eq(self_pets)
                    .chain(
                        vec![Target::Enemy; opponent_pets.len()]
                            .into_iter()
                            .zip_eq(opponent_pets),
                    )
                    .choose(&mut rng)
                {
                    pets.push(random_pet)
                }
            }
            (Target::Friend | Target::Enemy, Position::All(condition)) => {
                let team = if effect.target == Target::Friend {
                    self
                } else {
                    opponent
                };
                for pet in team.get_pets_by_cond(condition) {
                    pets.push((effect.target, pet))
                }
            }
            (Target::Either, Position::All(condition)) => {
                for (target_team, team) in [(Target::Friend, self), (Target::Enemy, opponent)] {
                    for pet in team.get_pets_by_cond(condition) {
                        pets.push((target_team, pet))
                    }
                }
            }
            (Target::Friend | Target::Enemy, Position::Opposite) => {
                let team = if effect.target == Target::Friend {
                    self
                } else {
                    opponent
                };
                if let Some(Some(pos)) = &curr_pet.map(|pet| pet.borrow().pos) {
                    if let Some(opposite_pet) = team.nth(*pos) {
                        pets.push((effect.target, opposite_pet))
                    }
                }
            }
            (_, Position::OnSelf) => {
                if let Some(self_pet) = &curr_pet {
                    pets.push((effect.target, self_pet.clone()))
                }
            }
            (_, Position::TriggerAffected) => {
                if let Some(Some(affected_pet)) = trigger
                    .affected_pet
                    .as_ref()
                    .map(|pet_ref| pet_ref.upgrade())
                {
                    pets.push((trigger.affected_team, affected_pet))
                }
            }
            (_, Position::TriggerAfflicting) => {
                if let Some(Some(afflicting_pet)) = trigger
                    .afflicting_pet
                    .as_ref()
                    .map(|pet_ref| pet_ref.upgrade())
                {
                    pets.push((trigger.affected_team, afflicting_pet))
                }
            }
            (Target::Friend | Target::Enemy, Position::Relative(rel_pos)) => {
                let team = if effect.target == Target::Friend {
                    self
                } else {
                    opponent
                };
                if let Some(Some(effect_pet_idx)) = &curr_pet.as_ref().map(|pet| pet.borrow().pos) {
                    let (target_team, adj_idx) = team
                        ._cvt_rel_idx_to_adj_idx(*effect_pet_idx, *rel_pos)
                        .unwrap();
                    // Pet can only be on same team.
                    if target_team == Target::Friend {
                        if let Some(rel_pet) = team.friends.get(adj_idx) {
                            pets.push((effect.target, rel_pet.clone()))
                        }
                    }
                }
            }
            (Target::Either, Position::Relative(rel_pos)) => {
                if let Some(Some(effect_pet_idx)) = &curr_pet.as_ref().map(|pet| pet.borrow().pos) {
                    let (target_team, adj_idx) = self
                        ._cvt_rel_idx_to_adj_idx(*effect_pet_idx, *rel_pos)
                        .unwrap();
                    let team = if target_team == Target::Friend {
                        self
                    } else {
                        opponent
                    };
                    if let Some(rel_pet) = team.friends.get(adj_idx) {
                        pets.push((target_team, rel_pet.clone()))
                    }
                }
            }
            (Target::Friend | Target::Enemy, Position::Range(effect_range)) => {
                let team = if effect.target == Target::Friend {
                    self
                } else {
                    opponent
                };
                for idx in effect_range.clone() {
                    if let Some(Some(effect_pet_idx)) =
                        curr_pet.as_ref().map(|pet| pet.borrow().pos)
                    {
                        let (target_team, adj_idx) =
                            team._cvt_rel_idx_to_adj_idx(effect_pet_idx, idx).unwrap();
                        if target_team == Target::Friend {
                            if let Some(rel_pet) = team.friends.get(adj_idx) {
                                pets.push((target_team, rel_pet.clone()))
                            }
                        }
                    }
                }
            }
            (Target::Either, Position::Range(effect_range)) => {
                for idx in effect_range.clone() {
                    if let Some(Some(effect_pet_idx)) =
                        &curr_pet.as_ref().map(|pet| pet.borrow().pos)
                    {
                        let (target_team, adj_idx) =
                            self._cvt_rel_idx_to_adj_idx(*effect_pet_idx, idx).unwrap();
                        let team = if target_team == Target::Friend {
                            self
                        } else {
                            opponent
                        };
                        if let Some(rel_pet) = team.friends.get(adj_idx) {
                            pets.push((target_team, rel_pet.clone()))
                        }
                    }
                }
            }
            (Target::Friend | Target::Enemy, Position::First) => {
                let team = if effect.target == Target::Friend {
                    self
                } else {
                    opponent
                };
                if let Some(first_pet) = team.all().first() {
                    pets.push((effect.target, first_pet.clone()))
                }
            }
            (Target::Friend | Target::Enemy, Position::Last) => {
                let team = if effect.target == Target::Friend {
                    self
                } else {
                    opponent
                };
                if let Some(last_pet) = team.all().last() {
                    pets.push((effect.target, last_pet.clone()))
                }
            }
            (_, Position::Multiple(positions)) => {
                let mut effect_copy = effect.clone();
                for pos in positions {
                    effect_copy.position = pos.clone();
                    pets.extend(self.get_pets_by_effect(trigger, &effect_copy, opponent)?)
                }
            }
            (Target::Either, Position::N(condition, num_pets)) => {
                let mut self_pets = self.get_pets_by_cond(condition).into_iter();
                let mut opponent_pets = opponent.get_pets_by_cond(condition).into_iter();
                // Get n values of indices.
                for n in 0..*num_pets {
                    // Alternate between teams.
                    if n % 2 == 0 {
                        if let Some(pet) = self_pets.next() {
                            pets.push((Target::Friend, pet))
                        }
                    } else if let Some(pet) = opponent_pets.next() {
                        pets.push((Target::Enemy, pet))
                    }
                }
            }
            (Target::Friend | Target::Enemy, Position::N(condition, n)) => {
                let team = if effect.target == Target::Friend {
                    self
                } else {
                    opponent
                };
                let mut found_pets = team.get_pets_by_cond(condition).into_iter();
                // Get n values of indices.
                for _ in 0..*n {
                    if let Some(pet) = found_pets.next() {
                        pets.push((effect.target, pet))
                    }
                }
            }
            (Target::Friend | Target::Enemy, Position::Adjacent) => {
                let team = if effect.target == Target::Friend {
                    self
                } else {
                    opponent
                };
                // Get pet ahead and behind.
                let mut effect_copy = effect.clone();
                for rel_pos in [-1, 1].into_iter() {
                    effect_copy.position = Position::Relative(rel_pos);
                    pets.extend(team.get_pets_by_effect(trigger, &effect_copy, opponent)?)
                }
            }
            _ => unimplemented!(),
        }

        Ok(pets)
    }
}
