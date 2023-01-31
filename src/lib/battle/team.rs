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
};

use itertools::Itertools;
use log::info;
use rand::{random, seq::IteratorRandom, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::{
    collections::{HashSet, VecDeque},
    error::Error,
    fmt::Display,
};

/// A Super Auto Pets team.
#[derive(Debug, Clone)]
pub struct Team {
    /// Name of the team.
    pub name: String,
    /// Pets on the team.
    pub friends: Vec<Option<Pet>>,
    /// Fainted pets.
    pub fainted: Vec<Option<Pet>>,
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
    pub(super) stored_friends: Vec<Option<Pet>>,
    /// Count of all pets summoned on team.
    pub(super) pet_count: usize,
    /// Current pet effect being activated.
    pub(super) effect_idx: Option<usize>,
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
            effect_idx: None,
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
    pub fn new(pets: &[Option<Pet>], max_size: usize) -> Result<Team, SAPTestError> {
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
            let mut pet_count = 0;
            let idx_pets = pets
                .iter()
                .cloned()
                .map(|mut slot| {
                    if let Some(pet) = slot.as_mut() {
                        pet.set_pos(pet_count);
                        // Create id if one not assigned.
                        pet.id = Some(
                            pet.id
                                .clone()
                                .unwrap_or(format!("{}_{}", pet.name, pet_count)),
                        );
                        Team::update_missing_food_effects(pet);
                        pet_count += 1;
                    }
                    slot
                })
                .collect_vec();

            Ok(Team {
                stored_friends: idx_pets.clone(),
                friends: idx_pets,
                max_size,
                pet_count,
                ..Default::default()
            })
        }
    }

    /// Update which team owns effect.
    fn update_effect_team(&mut self, target: Target) {
        for friend in self.all() {
            for effect in friend.effect.iter_mut() {
                effect.owner_target = Some(target);
            }
        }
    }

    /// Updates missing food items from an [`Action::Gain`](crate::battle::state::Action::Gain) effect.
    /// * Specifically for [`Toucan`](crate::pets::names::PetName::Toucan).
    ///
    /// ```rust
    /// use sapt::{Pet, PetName, Food, FoodName, Team, EffectApply, battle::state::Action};
    ///
    /// let honey = Food::try_from(FoodName::Honey).unwrap();
    /// let mut toucan = Pet::try_from(PetName::Toucan).unwrap();
    /// toucan.item = Some(honey.clone());
    ///
    /// assert_eq!(
    ///     toucan.effect.first().unwrap().action,
    ///     Action::Gain(None)
    /// );
    ///
    /// let team = Team::new(&[Some(toucan)], 5).unwrap();
    ///
    /// assert_eq!(
    ///     team.friends
    ///         .first().unwrap().as_ref().unwrap()
    ///         .effect.first().unwrap()
    ///         .action,
    ///     Action::Gain(Some(Box::new(honey)))
    /// )
    /// ```
    fn update_missing_food_effects(pet: &mut Pet) {
        for effect in pet.effect.iter_mut() {
            let effect_missing_food = if let Action::Gain(food) = &effect.action {
                food.is_none()
            } else {
                false
            };
            if pet.item.as_ref().is_some() && effect_missing_food {
                effect.action = Action::Gain(Some(Box::new(pet.item.as_ref().unwrap().clone())))
            }
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
        self.pet_count = self
            .stored_friends
            .iter()
            .filter(|pet| pet.is_some())
            .count();
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
        let mut new_idx_cnt = 0;
        self.friends.retain_mut(|pet| {
            // Check if empty slot
            if pet.is_none() {
                false
            } else if pet.as_ref().map_or(false, |pet| pet.stats.health != 0) {
                // Pet is Some so safe to unwrap.
                // Set new pet index and increment
                pet.as_mut().unwrap().set_pos(new_idx_cnt);
                new_idx_cnt += 1;
                true
            } else {
                // Pet is dead.
                info!(target: "dev", "(\"{}\")\n{} fainted.", self.name, pet.as_ref().unwrap());
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
        for pet in self.friends.iter_mut().flatten() {
            pet.seed = seed
        }
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
            .filter_map(|pet| pet.as_ref().map(|pet| pet.effect.clone()))
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
    pub fn get_pets_by_cond(&mut self, cond: &Condition) -> Vec<&mut Pet> {
        let found_pets: HashSet<usize> = if let Condition::Multiple(conditions) = cond {
            conditions
                .iter()
                .flat_map(|condition| self.match_condition(condition))
                .collect()
        } else if let Condition::MultipleAll(conditions) = cond {
            conditions
                .iter()
                .map(|condition| self.match_condition(condition))
                .reduce(|idxs_1, idxs_2| {
                    idxs_1
                        .intersection(&idxs_2)
                        .cloned()
                        .collect::<HashSet<usize>>()
                })
                .unwrap_or_default()
        } else {
            self.match_condition(cond)
        };

        self.all()
            .into_iter()
            .filter(|pet| {
                if let Some(pos) = pet.pos {
                    found_pets.contains(&pos)
                } else {
                    false
                }
            })
            .collect_vec()
    }

    /// Match on a `Condition` and return indices.
    fn match_condition(&mut self, cond: &Condition) -> HashSet<usize> {
        let mut indices: HashSet<usize> = HashSet::new();
        let curr_pet_idx = self.effect_idx;
        let pets = self.all().into_iter();

        match cond {
            Condition::Healthiest => {
                if let Some(Some(pos)) = pets
                    .max_by(|pet_1, pet_2| pet_1.stats.health.cmp(&pet_2.stats.health))
                    .map(|pet| pet.pos)
                {
                    indices.insert(pos);
                }
            }
            Condition::Illest => {
                if let Some(Some(pos)) = pets
                    .min_by(|pet_1, pet_2| pet_1.stats.health.cmp(&pet_2.stats.health))
                    .map(|pet| pet.pos)
                {
                    indices.insert(pos);
                }
            }
            Condition::Strongest => {
                if let Some(Some(pos)) = pets
                    .max_by(|pet_1, pet_2| pet_1.stats.attack.cmp(&pet_2.stats.attack))
                    .map(|pet| pet.pos)
                {
                    indices.insert(pos);
                }
            }
            Condition::Weakest => {
                if let Some(Some(pos)) = pets
                    .min_by(|pet_1, pet_2| pet_1.stats.attack.cmp(&pet_2.stats.attack))
                    .map(|pet| pet.pos)
                {
                    indices.insert(pos);
                }
            }
            Condition::HasFood(item_name) => {
                for pos in pets.filter_map(|pet| {
                    if pet
                        .item
                        .as_ref()
                        .map(|food| food.name == *item_name)
                        .unwrap_or(false)
                    {
                        pet.pos
                    } else {
                        None
                    }
                }) {
                    indices.insert(pos);
                }
            }
            Condition::TriggeredBy(trigger) => {
                for pos in pets.filter_map(|pet| {
                    if pet
                        .effect
                        .iter()
                        .any(|effect| effect.trigger.status == *trigger)
                    {
                        pet.pos
                    } else {
                        None
                    }
                }) {
                    indices.insert(pos);
                }
            }
            // Allow all if condition is None.
            Condition::None => indices.extend(self.all().iter().filter_map(|pet| pet.pos)),
            Condition::IgnoreSelf => {
                indices.extend(self.all().iter().enumerate().filter_map(|(i, pet)| {
                    if curr_pet_idx == Some(i) {
                        None
                    } else {
                        pet.pos
                    }
                }))
            }
            Condition::HighestTier => {
                if let Some(Some(pos)) = pets
                    .max_by(|pet_1, pet_2| pet_1.tier.cmp(&pet_2.tier))
                    .map(|pet| pet.pos)
                {
                    indices.insert(pos);
                }
            }
            Condition::LowestTier => {
                if let Some(Some(pos)) = pets
                    .min_by(|pet_1, pet_2| pet_1.tier.cmp(&pet_2.tier))
                    .map(|pet| pet.pos)
                {
                    indices.insert(pos);
                }
            }
            _ => {}
        }
        indices
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
            // Clear team to reassign indices.
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
        if let (Some(Some(pet_1)), Some(Some(pet_2))) =
            (mut_slice_1.get_mut(pos_1), mut_slice_2.get_mut(adj_pos_2))
        {
            std::mem::swap(&mut pet_1.stats, &mut pet_2.stats);
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

            self.friends.insert(new_pos, pet);
            // Update indices.
            self.set_indices();

            // Update triggers.
            for trigger in self
                .triggers
                .iter_mut()
                .filter(|trigger| trigger.to_target == Target::Friend)
            {
                if let Some(idx) = trigger.to_idx.as_mut() {
                    if *idx < pos {
                        *idx += 1
                    }
                }
            }

            // Add push trigger.
            let mut push_any_trigger = TRIGGER_ANY_PUSHED;
            push_any_trigger.to_idx = Some(new_pos);
            self.triggers.push_back(push_any_trigger);

            // // Reset indices.
            // self.clear_team();

            // Add opponent triggers if provided.
            if let Some(opponent) = opponent {
                for trigger in opponent
                    .triggers
                    .iter_mut()
                    .filter(|trigger| trigger.to_target == Target::Enemy)
                {
                    if let Some(idx) = trigger.to_idx.as_mut() {
                        if *idx < pos {
                            *idx += 1
                        }
                    }
                }
                let mut push_trigger = TRIGGER_ANY_ENEMY_PUSHED;
                push_trigger.to_idx = Some(new_pos);
                opponent.triggers.push_back(push_trigger)
            }
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
    pub fn nth(&mut self, idx: usize) -> Option<&mut Pet> {
        if let Some(Some(pet)) = self.friends.get_mut(idx) {
            (pet.stats.health != 0).then_some(pet)
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
    pub fn first(&mut self) -> Option<&mut Pet> {
        if let Some(Some(pet)) = self.friends.first_mut() {
            (pet.stats.health != 0).then_some(pet)
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
    pub fn last(&mut self) -> Option<&mut Pet> {
        if let Some(Some(pet)) = self.friends.last_mut() {
            (pet.stats.health != 0).then_some(pet)
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
    pub fn any(&mut self) -> Option<&mut Pet> {
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
    pub fn all(&mut self) -> Vec<&mut Pet> {
        self.friends
            .iter_mut()
            .filter_map(|pet| {
                if let Some(pet) = pet.as_mut() {
                    (pet.stats.health != 0).then_some(pet)
                } else {
                    None
                }
            })
            .collect_vec()
    }

    fn set_indices(&mut self) -> &mut Self {
        for (i, friend) in self
            .friends
            .iter_mut()
            .filter_map(|pet| pet.as_mut())
            .enumerate()
        {
            friend.set_pos(i);
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
        if self.all().len() == self.max_size {
            // Add overflow to dead pets.
            self.fainted.push(Some(pet));

            return Err(Box::new(SAPTestError::InvalidTeamAction {
                subject: "Add Pet".to_string(),
                indices: vec![pos],
                reason: format!("Maximum number of pets ({}) reached.", self.max_size),
            }));
        }
        // Assign id to pet if not any.
        pet.id = Some(
            pet.id
                .clone()
                .unwrap_or(format!("{}_{}", pet.name, self.pet_count + 1)),
        );

        // Remove empty slot at position if there.
        if self.friends.get(pos).map(|slot| slot.is_none()) == Some(true) {
            self.friends.remove(pos);
        };

        self.friends.insert(pos, Some(pet));
        self.set_indices();

        info!(target: "dev", "(\"{}\")\nAdded pet to pos {pos}: {}.", self.name.to_string(), self.friends.get(pos).unwrap().as_ref().unwrap());

        // Set summon triggers.
        let mut self_trigger = TRIGGER_SELF_SUMMON;
        let mut any_trigger = TRIGGER_ANY_SUMMON;
        let mut any_enemy_trigger = TRIGGER_ANY_ENEMY_SUMMON;

        (
            self_trigger.to_idx,
            any_trigger.to_idx,
            any_enemy_trigger.to_idx,
        ) = (Some(pos), Some(pos), Some(pos));

        // Update old triggers and their positions that store a pet's idx after inserting new pet.
        // TODO: Look into more edge cases that may cause issue when triggers activate simultaneously.
        for trigger in self.triggers.iter_mut() {
            match &mut trigger.position {
                Position::Relative(orig_pos) => {
                    if *orig_pos >= pos.try_into()? {
                        *orig_pos += 1
                    }
                }
                Position::Trigger | Position::OnSelf => {
                    if let Some(idx) = trigger.to_idx.as_mut() {
                        if *idx >= pos {
                            *idx += 1
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(opponent) = opponent {
            opponent.triggers.push_back(any_enemy_trigger)
        }
        self.triggers.extend([
            // May run into issue with mushroomed scorpion.
            self_trigger,
            any_trigger,
        ]);
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

        // Update effects to reflect which pet and team it belongs to.
        self.update_effect_team(Target::Friend);
        opponent.update_effect_team(Target::Enemy);

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

        // Trigger Before Attack && Friend Ahead attack.
        self.triggers
            .extend([TRIGGER_SELF_ATTACK, TRIGGER_AHEAD_ATTACK]);
        opponent
            .triggers
            .extend([TRIGGER_SELF_ATTACK, TRIGGER_AHEAD_ATTACK]);

        self.clear_team();
        opponent.clear_team();

        while !self.triggers.is_empty() || !opponent.triggers.is_empty() {
            self.trigger_effects(opponent);
            opponent.trigger_effects(self);
        }

        self.clear_team();
        opponent.clear_team();

        // Check that two pets exist and attack.
        // Attack will result in triggers being added.
        if let (Some(pet), Some(opponent_pet)) = (self.first(), opponent.first()) {
            // Attack and get outcome of fight.
            info!(target: "dev", "Fight!\nPet: {}\nOpponent: {}", pet, opponent_pet);
            let outcome = pet.attack(opponent_pet);
            info!(target: "dev", "(\"{}\")\n{}", self.name, self);
            info!(target: "dev", "(\"{}\")\n{}", opponent.name, opponent);

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
        for friend in self.friends.iter().filter_map(|pet| pet.as_ref()) {
            writeln!(f, "{friend}")?;
        }
        Ok(())
    }
}
