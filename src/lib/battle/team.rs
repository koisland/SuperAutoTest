use crate::{
    battle::{
        effect::Effect,
        state::{Action, Condition, Outcome, Position, Status, Target, TeamFightOutcome},
        team_effect_apply::EffectApply,
        trigger::*,
    },
    error::SAPTestError,
    graph::effect_graph::History,
    pets::{combat::PetCombat, pet::Pet},
    Food,
};

use itertools::Itertools;
use log::info;
use rand::{random, seq::IteratorRandom, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
    error::Error,
    fmt::Display,
    rc::{Rc, Weak},
};

use super::team_effect_apply::EffectApplyHelpers;

/// A Super Auto Pets team.
#[derive(Debug, Clone)]
pub struct Team {
    /// Name of the team.
    pub name: String,
    /// Pets on the team.
    pub friends: Vec<Rc<RefCell<Pet>>>,
    /// Fainted pets.
    pub fainted: Vec<Rc<RefCell<Pet>>>,
    /// Maximum number of pets that can be added.
    pub max_size: usize,
    /// Stored triggers used to invoke effects.
    ///
    /// Calling [`trigger_effects`](super::team_effect_apply::EffectApply::trigger_effects) will exhaust all stored triggers.
    /// * As a result, this will always be empty unless mutated.
    pub triggers: VecDeque<Outcome>,
    /// Effect history of a team.
    pub history: History,
    /// Seed used to reproduce the outcome of events.
    pub seed: u64,
    /// Clone of pets used for restoring team..
    pub(super) stored_friends: Vec<Rc<RefCell<Pet>>>,
    /// Count of all pets summoned on team.
    pub(super) pet_count: usize,
    pub(super) curr_pet: Option<Weak<RefCell<Pet>>>,
}

impl Default for Team {
    fn default() -> Self {
        Self {
            name: Default::default(),
            friends: Default::default(),
            stored_friends: Default::default(),
            fainted: Default::default(),
            max_size: 5,
            triggers: VecDeque::from_iter(ALL_TRIGGERS_START_BATTLE),
            history: History::new(),
            pet_count: Default::default(),
            seed: random(),
            curr_pet: None,
        }
    }
}

impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.friends == other.friends
            && self.stored_friends == other.stored_friends
            && self.fainted == other.fainted
            && self.max_size == other.max_size
            && self.triggers == other.triggers
            && self.pet_count == other.pet_count
    }
}

impl Team {
    /// Create a new team of pets of a given size.
    ///
    /// ```
    /// use sapt::{Pet, PetName, Team, EffectApply};
    ///
    /// let team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Dog).unwrap())],
    ///     5
    /// );
    ///
    /// assert!(team.is_ok());
    /// ```
    pub fn new(pets: &[Pet], max_size: usize) -> Result<Team, SAPTestError> {
        if pets.len() > max_size {
            Err(SAPTestError::InvalidTeamAction {
                subject: "Init Team".to_string(),
                indices: vec![pets.len()],
                reason: format!(
                    "Pets provided exceed specified max size. {} > {}",
                    pets.len(),
                    max_size
                ),
            })
        } else {
            // Index pets.
            let mut rc_pets: Vec<Rc<RefCell<Pet>>> = vec![];
            for (i, mut pet) in pets.iter().cloned().enumerate() {
                // Create id if one not assigned.
                pet.id = Some(pet.id.clone().unwrap_or(format!("{}_{}", pet.name, i)));
                pet.pos = Some(i);
                pet.update_missing_food_effects();

                let rc_pet = Rc::new(RefCell::new(pet));

                // Store weak reference to owner for all effects.
                for effect in rc_pet.borrow_mut().effect.iter_mut() {
                    effect.trigger.affected_pet = Some(Rc::downgrade(&rc_pet));
                    effect.owner = Some(Rc::downgrade(&rc_pet));
                }
                if let Some(item) = rc_pet.borrow_mut().item.as_mut() {
                    item.ability.owner = Some(Rc::downgrade(&rc_pet))
                }
                rc_pets.push(rc_pet)
            }
            let n_rc_pets = rc_pets.len();
            let curr_pet = rc_pets.first().map(|pet| Rc::downgrade(pet));
            Ok(Team {
                stored_friends: rc_pets.clone(),
                friends: rc_pets,
                max_size,
                pet_count: n_rc_pets,
                curr_pet,
                ..Default::default()
            })
        }
    }

    #[allow(dead_code)]
    /// Restore a team to its initial state.
    ///
    /// ```
    /// use sapt::{Pet, PetName, Team, EffectApply};
    ///
    /// let mut default_team = Team::default();
    /// default_team
    ///     .add_pet(Pet::try_from(PetName::Dog).unwrap(), 0, None).unwrap()
    ///     .restore();
    ///
    /// assert_eq!(default_team, Team::default());
    /// ```
    pub fn restore(&mut self) -> &mut Self {
        self.friends = self.stored_friends.clone();
        self.fainted.clear();
        self.history = History::new();
        self.triggers = VecDeque::from_iter(ALL_TRIGGERS_START_BATTLE);
        self.pet_count = self.stored_friends.len();
        self
    }

    /// Clear team of empty slots and/or fainted pets and reset indices.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Team, EffectApply};
    ///
    /// let mut default_team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Dog).unwrap())],
    ///     5
    /// ).unwrap();
    ///
    /// assert_eq!(default_team.friends.len(), 1);
    ///
    /// default_team.first().unwrap().stats.health = 0;
    /// default_team.clear_team();
    ///
    /// assert_eq!(default_team.friends.len(), 0);
    /// ```
    pub fn clear_team(&mut self) -> &mut Self {
        let mut new_idx = 0;
        self.friends.retain(|pet| {
            // Check if empty slot
            if pet.borrow().stats.health != 0 {
                pet.borrow_mut().pos = Some(new_idx);
                new_idx += 1;
                true
            } else {
                // Pet is dead.
                info!(target: "dev", "(\"{}\")\n{} fainted.", self.name, pet.borrow());
                self.fainted.push(pet.clone());
                false
            }
        });
        self
    }

    /// Set a `u64` seed for a team allowing for reproducibility of events.
    /// * **Note:** For abilities that select a random pet on the enemy team, the seed must be set for the opposing team.
    /// # Examples
    ///  ```
    /// use sapt::{Pet, PetName, Team, EffectApply};
    ///
    /// let mosquito = Some(Pet::try_from(PetName::Mosquito).unwrap());
    /// let pets = [mosquito.clone(), mosquito.clone()];
    /// let mut team = Team::new(&pets, 5).unwrap();
    /// let mut enemy_team = team.clone();
    ///
    /// // Set seed for enemy_team and trigger StartBattle effects.
    /// enemy_team.set_seed(0);
    /// team.trigger_effects(&mut enemy_team);
    ///
    /// // Mosquitoes always hit first pet with seed set to 0.
    /// assert_eq!(enemy_team.friends[0].as_ref().unwrap().stats.health, 0)
    /// ```
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
        for pet in self.friends.iter() {
            pet.borrow_mut().seed = seed
        }
    }

    /// Assign an item to a team member.
    pub fn set_item(
        &mut self,
        pos: Position,
        item: Option<Food>,
    ) -> Result<&mut Self, SAPTestError> {
        let affected_pets = self._match_position_one_team(&TRIGGER_NONE, &pos);

        for pet in affected_pets?.iter() {
            let mut item_copy = item.clone();
            if let Some(item) = item_copy.as_mut() {
                item.ability.owner = Some(Rc::downgrade(pet));
            }
            pet.borrow_mut().item = item_copy;
        }
        Ok(self)
    }

    #[allow(dead_code)]
    /// Get all pet effects on the team.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Team};
    ///
    /// let team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Dog).unwrap())],
    ///     5
    /// ).unwrap();
    ///
    /// assert_eq!(team.get_effects().len(), 1);
    /// ```
    pub fn get_effects(&self) -> Vec<Vec<Effect>> {
        self.friends
            .iter()
            .map(|pet| pet.borrow().effect.clone())
            .collect_vec()
    }

    /// Get pets by a given [`Condition`].
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Team, battle::state::{Condition, Status}};
    ///
    /// let pets = [
    ///     Some(Pet::try_from(PetName::Gorilla).unwrap()),
    ///     Some(Pet::try_from(PetName::Leopard).unwrap()),
    ///     Some(Pet::try_from(PetName::Mosquito).unwrap())
    /// ];
    /// let mut team = Team::new(&pets, 5).unwrap();
    ///
    /// assert_eq!(
    ///     team.get_pets_by_cond(
    ///         &Condition::TriggeredBy(Status::StartOfBattle)
    ///     ).len(),
    ///     2
    /// );
    /// ```
    pub fn get_pets_by_cond(&mut self, cond: &Condition) -> Vec<Rc<RefCell<Pet>>> {
        if let Condition::Multiple(conditions) = cond {
            conditions
                .iter()
                .flat_map(|condition| self.match_condition(condition))
                .collect()
        } else if let Condition::MultipleAll(conditions) = cond {
            let matching_pets = vec![];
            let all_matches = conditions
                .iter()
                .filter_map(|cond| {
                    let matching_pets = self.match_condition(cond);
                    (!matching_pets.is_empty()).then_some(matching_pets)
                })
                .collect_vec();
            // Take first set of matches.
            if let Some(mut matching_pets) = all_matches.first().cloned() {
                // Remove any pets not within.
                for matches in all_matches.iter() {
                    matching_pets.retain(|pet| matches.contains(pet))
                }
            }
            matching_pets
        } else {
            self.match_condition(cond)
        }
    }

    /// Match on a `Condition` and return indices.
    fn match_condition(&mut self, cond: &Condition) -> Vec<Rc<RefCell<Pet>>> {
        let mut matching_pets: Vec<Rc<RefCell<Pet>>> = vec![];
        let pets = self.all().into_iter();

        match cond {
            Condition::Healthiest => {
                if let Some(pet) = pets.max_by(|pet_1, pet_2| {
                    pet_1
                        .borrow()
                        .stats
                        .health
                        .cmp(&pet_2.borrow().stats.health)
                }) {
                    matching_pets.push(pet);
                }
            }
            Condition::Illest => {
                if let Some(pet) = pets.min_by(|pet_1, pet_2| {
                    pet_1
                        .borrow()
                        .stats
                        .health
                        .cmp(&pet_2.borrow().stats.health)
                }) {
                    matching_pets.push(pet);
                }
            }
            Condition::Strongest => {
                if let Some(pet) = pets.max_by(|pet_1, pet_2| {
                    pet_1
                        .borrow()
                        .stats
                        .attack
                        .cmp(&pet_2.borrow().stats.attack)
                }) {
                    matching_pets.push(pet);
                }
            }
            Condition::Weakest => {
                if let Some(pet) = pets.min_by(|pet_1, pet_2| {
                    pet_1
                        .borrow()
                        .stats
                        .attack
                        .cmp(&pet_2.borrow().stats.attack)
                }) {
                    matching_pets.push(pet);
                }
            }
            Condition::HasFood(item_name) => {
                for pet in pets.filter(|pet| {
                    pet.borrow()
                        .item
                        .as_ref()
                        .map_or(false, |food| food.name == *item_name)
                }) {
                    matching_pets.push(pet);
                }
            }
            Condition::TriggeredBy(trigger) => {
                for pet in pets.filter(|pet| {
                    pet.borrow()
                        .effect
                        .iter()
                        .any(|effect| effect.trigger.status == *trigger)
                }) {
                    matching_pets.push(pet);
                }
            }
            // Allow all if condition is None.
            Condition::None => matching_pets.extend(self.all().into_iter()),
            // Condition::IgnoreSelf => {
            //     indices.extend(self.all().iter().enumerate().filter_map(|(i, pet)| {
            //         if curr_pet_idx == Some(i) {
            //             None
            //         } else {
            //             pet.pos
            //         }
            //     }))
            // }
            Condition::HighestTier => {
                if let Some(pet) =
                    pets.max_by(|pet_1, pet_2| pet_1.borrow().tier.cmp(&pet_2.borrow().tier))
                {
                    matching_pets.push(pet);
                }
            }
            Condition::LowestTier => {
                if let Some(pet) =
                    pets.min_by(|pet_1, pet_2| pet_1.borrow().tier.cmp(&pet_2.borrow().tier))
                {
                    matching_pets.push(pet);
                }
            }
            _ => {}
        }
        matching_pets
    }

    /// Swap a pets position with another on the team.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Team};
    ///
    /// let pets = [
    ///     Some(Pet::try_from(PetName::Gorilla).unwrap()),
    ///     Some(Pet::try_from(PetName::Leopard).unwrap()),
    /// ];
    /// let mut team = Team::new(&pets, 5).unwrap();
    ///
    /// team.swap_pets(0, 1).unwrap();
    /// assert!(
    ///     team.nth(0).unwrap().name == PetName::Leopard &&
    ///     team.nth(1).unwrap().name == PetName::Gorilla
    /// )
    /// ```
    pub fn swap_pets(&mut self, pos_1: usize, pos_2: usize) -> Result<&mut Self, SAPTestError> {
        if pos_1 > self.friends.len() || pos_2 > self.friends.len() {
            Err(SAPTestError::InvalidTeamAction {
                subject: "Swap Pets".to_string(),
                indices: vec![pos_1, pos_2],
                reason: "One or more positions are out of bounds".to_string(),
            })
        } else {
            self.friends.swap(pos_1, pos_2);
            self.set_indices();
            Ok(self)
        }
    }

    /// Swap a pets stats with another on the team.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Team, Statistics};
    ///
    /// let mut team = Team::new(&[
    ///     Some(Pet::try_from(PetName::Gorilla).unwrap()),
    ///     Some(Pet::try_from(PetName::Leopard).unwrap()),
    /// ], 5).unwrap();
    /// assert!(
    ///     team.nth(0).unwrap().stats == Statistics::new(6, 9).unwrap() &&
    ///     team.nth(1).unwrap().stats == Statistics::new(10, 4).unwrap()
    /// );
    ///
    /// team.swap_pet_stats(0, 1).unwrap();
    /// assert!(
    ///     team.nth(0).unwrap().stats == Statistics::new(10, 4).unwrap() &&
    ///     team.nth(1).unwrap().stats == Statistics::new(6, 9).unwrap()
    /// )
    /// ```
    pub fn swap_pet_stats(
        &mut self,
        mut pos_1: usize,
        mut pos_2: usize,
    ) -> Result<&mut Self, SAPTestError> {
        // Swap idx so sorted.
        if pos_1 > pos_2 {
            std::mem::swap(&mut pos_1, &mut pos_2)
        }
        if pos_1 > self.friends.len() || pos_2 > self.friends.len() {
            return Err(SAPTestError::InvalidTeamAction {
                subject: "Swap Pet Stats".to_string(),
                indices: vec![pos_1, pos_2],
                reason: format!("{pos_1} or {pos_2} larger than len of friends."),
            });
        }
        // Split and get two mut slices so can access elements at same time.
        let (mut_slice_1, mut_slice_2) = self.friends.split_at_mut(pos_1 + 1);
        let mut_slice_1_len = mut_slice_1.len();
        let adj_pos_2 = pos_2.saturating_sub(mut_slice_1_len);
        if let (Some(pet_1), Some(pet_2)) = (mut_slice_1.get(pos_1), mut_slice_2.get(adj_pos_2)) {
            std::mem::swap(&mut pet_1.borrow_mut().stats, &mut pet_2.borrow_mut().stats);
            Ok(self)
        } else {
            Err(SAPTestError::InvalidTeamAction {
                subject: "Swap Pet Stats (Access)".to_string(),
                indices: vec![pos_1, adj_pos_2],
                reason: "Cannot access pets to swap stats.".to_string(),
            })
        }
    }

    /// Push a pet to another position on the team.
    /// * `by` is relative to current position.
    /// * An `opponent` can be provided optionally to update their `triggers`.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Team, Statistics};
    ///
    /// let mut team = Team::new(&[
    ///     Some(Pet::try_from(PetName::Gorilla).unwrap()),
    ///     Some(Pet::try_from(PetName::Leopard).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    /// ], 5).unwrap();
    ///
    /// // Push Gorilla two slots back.
    /// team.push_pet(0, -2, None).unwrap();
    /// assert!(
    ///     team.nth(0).unwrap().name == PetName::Leopard &&
    ///     team.nth(1).unwrap().name == PetName::Cat &&
    ///     team.nth(2).unwrap().name == PetName::Gorilla
    /// )
    /// ```
    pub fn push_pet(
        &mut self,
        pos: usize,
        by: isize,
        opponent: Option<&mut Team>,
    ) -> Result<&mut Self, Box<dyn Error>> {
        if pos < self.friends.len() {
            let new_pos: usize = if by.is_negative() {
                let pos_by: usize = (-by).try_into()?;
                (pos_by + pos).clamp(0, self.friends.len())
            } else {
                pos.saturating_sub(by.try_into()?)
            };

            let pet = self.friends.remove(pos);
            pet.borrow_mut().pos = Some(new_pos);

            // Add push trigger.
            let mut push_any_trigger = TRIGGER_ANY_PUSHED;
            push_any_trigger.affected_pet = Some(Rc::downgrade(&pet));
            self.triggers.push_back(push_any_trigger);

            // Add opponent triggers if provided.
            if let Some(opponent) = opponent {
                let mut push_trigger = TRIGGER_ANY_ENEMY_PUSHED;
                push_trigger.affected_pet = Some(Rc::downgrade(&pet));
                opponent.triggers.push_back(push_trigger)
            }

            self.friends.insert(new_pos, pet);
            self.set_indices();
        } else {
            return Err(Box::new(SAPTestError::InvalidTeamAction {
                subject: "Push Pet".to_string(),
                indices: vec![pos],
                reason: "Invalid indices.".to_string(),
            }));
        }

        Ok(self)
    }

    /// Get a pet at the specified index.
    /// * Fainted pets are ignored.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Team};
    ///
    /// let mut team = Team::new(&[
    ///     Some(Pet::try_from(PetName::Gorilla).unwrap()),
    ///     Some(Pet::try_from(PetName::Leopard).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    /// ], 5).unwrap();
    ///
    /// assert_eq!(
    ///     team.nth(1).unwrap().name,
    ///     PetName::Leopard
    /// )
    /// ```
    pub fn nth(&mut self, idx: usize) -> Option<Rc<RefCell<Pet>>> {
        if let Some(pet) = self
            .friends
            .get(idx)
            .filter(|pet| pet.borrow().stats.health != 0)
        {
            Some(pet.clone())
        } else {
            None
        }
    }

    /// Get the first pet on team.
    /// * Fainted pets are ignored.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Team};
    ///
    /// let mut team = Team::new(&[
    ///     Some(Pet::try_from(PetName::Gorilla).unwrap()),
    ///     Some(Pet::try_from(PetName::Leopard).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    /// ], 5).unwrap();
    ///
    /// assert_eq!(
    ///     team.first().unwrap().name,
    ///     PetName::Gorilla
    /// )
    /// ```
    pub fn first(&self) -> Option<Rc<RefCell<Pet>>> {
        if let Some(pet) = self
            .friends
            .first()
            .filter(|pet| pet.borrow().stats.health != 0)
        {
            Some(pet.clone())
        } else {
            None
        }
    }

    /// Get the first pet on team.
    /// * Fainted pets are ignored.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Team};
    ///
    /// let mut team = Team::new(&[
    ///     Some(Pet::try_from(PetName::Gorilla).unwrap()),
    ///     Some(Pet::try_from(PetName::Leopard).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    /// ], 5).unwrap();
    ///
    /// assert_eq!(
    ///     team.last().unwrap().name,
    ///     PetName::Cat
    /// )
    /// ```
    pub fn last(&mut self) -> Option<Rc<RefCell<Pet>>> {
        if let Some(pet) = self
            .friends
            .last()
            .filter(|pet| pet.borrow().stats.health != 0)
        {
            Some(pet.clone())
        } else {
            None
        }
    }

    /// Get a random available pet.
    /// * Fainted pets and/or empty slots are ignored.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Team};
    ///
    /// let mut team = Team::new(&[
    ///     Some(Pet::try_from(PetName::Dog).unwrap()),
    ///     None,
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    ///     None,
    ///     None
    /// ], 5).unwrap();
    /// team.set_seed(0);
    ///
    /// assert_eq!(
    ///     team.any().unwrap().name,
    ///     PetName::Cat
    /// )
    /// ```
    #[allow(dead_code)]
    pub fn any(&mut self) -> Option<Rc<RefCell<Pet>>> {
        let mut rng = ChaCha12Rng::seed_from_u64(self.seed);
        self.all().into_iter().choose(&mut rng)
    }

    /// Get all available pets.
    /// * Fainted pets and/or empty slots are ignored.
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Team};
    ///
    /// let mut team = Team::new(&[
    ///     None,
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    ///     None,
    ///     Some(Pet::try_from(PetName::Cat).unwrap())
    /// ], 5).unwrap();
    ///
    /// assert_eq!(
    ///     team.all().len(),
    ///     3
    /// )
    /// ```
    pub fn all(&mut self) -> Vec<Rc<RefCell<Pet>>> {
        self.friends
            .iter()
            .filter_map(|pet| {
                if pet.borrow().stats.health != 0 {
                    Some(pet.clone())
                } else {
                    None
                }
            })
            .collect_vec()
    }

    pub(super) fn set_indices(&mut self) -> &mut Self {
        for (i, friend) in self.friends.iter().enumerate() {
            if let Ok(mut unborrowed_pet) = friend.try_borrow_mut() {
                unborrowed_pet.pos = Some(i);
            }
        }
        self
    }

    /// Add a pet to position on a team.
    /// * An `opponent` can be provided to update its effect triggers.
    ///
    /// # Examples
    /// ```
    /// use sapt::{Pet, PetName, Team};
    ///
    /// let mut team = Team::new(&[
    ///     None,
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    ///     None,
    ///     Some(Pet::try_from(PetName::Cat).unwrap())
    /// ], 5).unwrap();
    ///
    /// team.add_pet(Pet::try_from(PetName::Turtle).unwrap(), 0, None);
    /// assert_eq!(
    ///     team.first().unwrap().name,
    ///     PetName::Turtle
    /// )
    pub fn add_pet(
        &mut self,
        mut pet: Pet,
        pos: usize,
        opponent: Option<&mut Team>,
    ) -> Result<&mut Self, Box<dyn Error>> {
        // Assign id to pet if not any.
        let new_pet_id = format!("{}_{}", pet.name, self.pet_count + 1);
        pet.id = Some(pet.id.clone().unwrap_or(new_pet_id));
        pet.pos = Some(pos);

        let rc_pet = Rc::new(RefCell::new(pet));

        if self.all().len() == self.max_size {
            // Add overflow to dead pets.
            self.fainted.push(rc_pet);

            return Err(Box::new(SAPTestError::InvalidTeamAction {
                subject: "Add Pet".to_string(),
                indices: vec![pos],
                reason: format!("Maximum number of pets ({}) reached.", self.max_size),
            }));
        }

        // Set summon triggers.
        let mut self_trigger = TRIGGER_SELF_SUMMON;
        let mut any_trigger = TRIGGER_ANY_SUMMON;
        let mut any_enemy_trigger = TRIGGER_ANY_ENEMY_SUMMON;

        let weak_ref_pet = Rc::downgrade(&rc_pet);
        (
            self_trigger.affected_pet,
            any_trigger.affected_pet,
            any_enemy_trigger.affected_pet,
        ) = (
            Some(weak_ref_pet.clone()),
            Some(weak_ref_pet.clone()),
            Some(weak_ref_pet.clone()),
        );

        if let Some(opponent) = opponent {
            opponent.triggers.push_back(any_enemy_trigger)
        }
        self.triggers.extend([
            // May run into issue with mushroomed scorpion.
            self_trigger,
            any_trigger,
        ]);

        info!(target: "dev", "(\"{}\")\nAdded pet to pos {pos}: {}.", self.name.to_string(), rc_pet.borrow());
        self.friends.insert(pos, rc_pet);
        self.set_indices();

        Ok(self)
    }

    /// Fight another team for a single battle phase.
    ///
    /// # Examples
    /// #### To complete the battle.
    /// ```rust
    /// use sapt::{Team, Pet, PetName, battle::state::TeamFightOutcome};
    ///
    /// let mut team = Team::new(
    ///     &vec![Some(Pet::try_from(PetName::Cricket).unwrap()); 5],
    ///     5
    /// ).unwrap();
    /// let mut enemy_team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Hippo).unwrap())],
    ///     5
    /// ).unwrap();
    ///
    /// let mut outcome = team.fight(&mut enemy_team);
    /// while let TeamFightOutcome::None = outcome {
    ///     outcome = team.fight(&mut enemy_team);
    /// }
    ///
    /// assert!(outcome == TeamFightOutcome::Loss);
    /// ```
    /// #### To complete `n` turns.
    /// ```rust
    /// use sapt::{Team, Pet, PetName, battle::state::TeamFightOutcome};
    ///
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
    /// let mut outcome = team.fight(&mut enemy_team);
    /// for _ in 0..n-1 {
    ///     outcome = team.fight(&mut enemy_team);
    /// }
    pub fn fight(&mut self, opponent: &mut Team) -> TeamFightOutcome {
        info!(target: "dev", "(\"{}\")\n{}", self.name, self);
        info!(target: "dev", "(\"{}\")\n{}", opponent.name, opponent);

        // Apply start of battle effects.
        self.clear_team();
        opponent.clear_team();

        while !self.triggers.is_empty() || !opponent.triggers.is_empty() {
            self.trigger_effects(opponent);
            opponent.trigger_effects(self);
        }

        // If current phase is 0, add before first battle triggers.
        // Used for butterfly.
        if self.history.curr_phase == 0 {
            self.triggers.push_back(TRIGGER_BEFORE_FIRST_BATTLE)
        }
        if opponent.history.curr_phase == 0 {
            opponent.triggers.push_back(TRIGGER_BEFORE_FIRST_BATTLE)
        }

        // Increment battle phase counter.
        self.history.curr_phase += 1;

        // Check that two pets exist and attack.
        // Attack will result in triggers being added.
        if let (Some(pet), Some(opponent_pet)) = (self.first(), opponent.first()) {
            // Trigger Before Attack && Friend Ahead attack.
            self.triggers.extend(get_atk_triggers(&pet));
            opponent.triggers.extend(get_atk_triggers(&opponent_pet));

            self.clear_team();
            opponent.clear_team();

            while !self.triggers.is_empty() || !opponent.triggers.is_empty() {
                self.trigger_effects(opponent);
                opponent.trigger_effects(self);
            }

            self.clear_team();
            opponent.clear_team();

            // Attack and get outcome of fight.
            info!(target: "dev", "Fight!\nPet: {}\nOpponent: {}", pet.borrow(), opponent_pet.borrow());
            let mut outcome = pet.borrow_mut().attack(&mut opponent_pet.borrow_mut());
            info!(target: "dev", "(\"{}\")\n{}", self.name, self);
            info!(target: "dev", "(\"{}\")\n{}", opponent.name, opponent);

            // Update outcomes with weak references.
            for trigger in outcome.friends.iter_mut() {
                trigger.affected_pet = Some(Rc::downgrade(&pet));
                trigger.afflicting_pet = Some(Rc::downgrade(&opponent_pet));
            }
            for trigger in outcome.opponents.iter_mut() {
                trigger.affected_pet = Some(Rc::downgrade(&opponent_pet));
                trigger.afflicting_pet = Some(Rc::downgrade(&pet));
            }
            // Create attack node.
            self.create_node(&TRIGGER_SELF_ATTACK);
            opponent.create_node(&TRIGGER_SELF_ATTACK);

            if let Some(hurt_trigger) = outcome
                .friends
                .iter()
                .find(|trigger| trigger.status == Status::Hurt)
            {
                self.create_node(hurt_trigger);
            }

            if let Some(opponent_hurt_trigger) = outcome
                .opponents
                .iter()
                .find(|trigger| trigger.status == Status::Hurt)
            {
                opponent.create_node(opponent_hurt_trigger);
            }

            // Add triggers to team from outcome of battle.
            self.triggers.extend(outcome.friends.into_iter());
            opponent.triggers.extend(outcome.opponents.into_iter());

            // Apply effect triggers from combat phase.
            while !self.triggers.is_empty() || !opponent.triggers.is_empty() {
                self.trigger_effects(opponent).clear_team();
                opponent.trigger_effects(self).clear_team();
            }
        }
        if !self.friends.is_empty() && !opponent.friends.is_empty() {
            TeamFightOutcome::None
        } else {
            // Add end of battle node.
            self.history.prev_node = self.history.curr_node;
            self.history.curr_node = Some(self.history.effect_graph.add_node(TRIGGER_END_BATTLE));

            if self.friends.is_empty() && opponent.friends.is_empty() {
                info!(target: "dev", "Draw!");
                TeamFightOutcome::Draw
            } else if !opponent.friends.is_empty() {
                info!(target: "dev", "Enemy team won...");
                TeamFightOutcome::Loss
            } else {
                info!(target: "dev", "Your team won!");
                TeamFightOutcome::Win
            }
        }
    }

    /// Create a node logging an effect's result for a [`Team`]'s history.
    fn create_node(&mut self, trigger: &Outcome) -> &mut Self {
        let node_idx = self.history.effect_graph.add_node(trigger.clone());
        self.history.prev_node = self.history.curr_node;
        self.history.curr_node = Some(node_idx);
        self
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for friend in self.friends.iter() {
            writeln!(f, "{}", friend.borrow())?;
        }
        Ok(())
    }
}
