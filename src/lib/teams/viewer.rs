use crate::{
    effects::state::{EqualityCondition, Outcome, Target},
    error::SAPTestError,
    shop::store::ItemSlot,
    Effect, Entity, EntityName, FoodName, ItemCondition, Pet, Position, ShopViewer, Team,
};
use itertools::Itertools;
use rand::{
    random,
    seq::{IteratorRandom, SliceRandom},
    SeedableRng,
};
use rand_chacha::ChaCha12Rng;
use std::{cell::RefCell, rc::Rc};

use super::effects::EffectApplyHelpers;

/// [`Target`] team and reference to [`Pet`].
/// * Pet itself doesn't store a reference to [`Team`] so this was a workaround.
pub type TargetPets = Vec<(Target, Rc<RefCell<Pet>>)>;

/// Methods for viewing [`Team`]s.
pub trait TeamViewer {
    /// Get a pet at the specified index.
    /// * Fainted pets are ignored.
    /// # Examples
    /// ```
    /// use saptest::{Pet, PetName, Team, TeamViewer};
    ///
    /// let mut team = Team::new(&[
    ///     Some(Pet::try_from(PetName::Gorilla).unwrap()),
    ///     Some(Pet::try_from(PetName::Leopard).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    /// ], 5).unwrap();
    ///
    /// assert_eq!(
    ///     team.nth(1).unwrap().borrow().name,
    ///     PetName::Leopard
    /// )
    /// ```
    fn nth(&self, idx: usize) -> Option<Rc<RefCell<Pet>>>;

    /// Get the first pet on team.
    /// * Fainted pets are ignored.
    /// # Examples
    /// ```
    /// use saptest::{Pet, PetName, Team, TeamViewer};
    ///
    /// let mut team = Team::new(&[
    ///     Some(Pet::try_from(PetName::Gorilla).unwrap()),
    ///     Some(Pet::try_from(PetName::Leopard).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    /// ], 5).unwrap();
    ///
    /// assert_eq!(
    ///     team.first().unwrap().borrow().name,
    ///     PetName::Gorilla
    /// )
    /// ```
    fn first(&self) -> Option<Rc<RefCell<Pet>>>;

    /// Get the first pet on team.
    /// * Fainted pets are ignored.
    /// # Examples
    /// ```
    /// use saptest::{Pet, PetName, Team, TeamViewer};
    ///
    /// let mut team = Team::new(&[
    ///     Some(Pet::try_from(PetName::Gorilla).unwrap()),
    ///     Some(Pet::try_from(PetName::Leopard).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    /// ], 5).unwrap();
    ///
    /// assert_eq!(
    ///     team.last().unwrap().borrow().name,
    ///     PetName::Cat
    /// )
    /// ```
    fn last(&self) -> Option<Rc<RefCell<Pet>>>;

    /// Get a random available pet.
    /// * Fainted pets and/or empty slots are ignored.
    /// # Examples
    /// ```
    /// use saptest::{Pet, PetName, Team, TeamViewer};
    ///
    /// let mut team = Team::new(&[
    ///     Some(Pet::try_from(PetName::Dog).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    /// ], 5).unwrap();
    /// team.set_seed(Some(0));
    ///
    /// assert_eq!(
    ///     team.any().unwrap().borrow().name,
    ///     PetName::Cat
    /// )
    /// ```
    fn any(&self) -> Option<Rc<RefCell<Pet>>>;

    /// Get all available pets.
    /// * Fainted pets and/or empty slots are ignored.
    /// # Examples
    /// ```
    /// use saptest::{Pet, PetName, Team, TeamViewer};
    ///
    /// let mut team = Team::new(&[
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap())
    /// ], 5).unwrap();
    ///
    /// assert_eq!(
    ///     team.all().len(),
    ///     3
    /// )
    /// ```
    fn all(&self) -> Vec<Rc<RefCell<Pet>>>;

    /// Filter pets that match an [`EqualityCondition`](crate::effects::state::EqualityCondition).
    /// * Used by [`TeamViewer::get_pets_by_cond`].
    /// * Will [`panic`] if used using a condition specific to a [`Shop`](crate::Shop) like [`EqualityCondition::Frozen`].
    fn check_eq_cond<T>(&self, all_pets: T, eq_cond: &EqualityCondition) -> Vec<Rc<RefCell<Pet>>>
    where
        T: IntoIterator<Item = Rc<RefCell<Pet>>>;

    /// Get pets by a given [`ItemCondition`].
    /// * Will [`panic`] if using:
    ///     * A [`ItemCondition::Equal`] or [`ItemCondition::NotEqual`] specific to a [`Shop`](crate::Shop) like [`EqualityCondition::Frozen`].
    ///         * Use the [`ShopViewer::get_shop_items_by_cond`] method instead.
    ///     * Nested [`ItemCondition::Multiple`] or [`ItemCondition::MultipleAll`].
    /// # Examples
    /// ---
    /// Pets with a [`StartOfBattle`](crate::effects::state::Status::StartOfBattle) [`Effect`](crate::Effect) trigger.
    /// ```
    /// use saptest::{
    ///     Pet, PetName, Team, TeamViewer, ItemCondition,
    ///     effects::state::{Status, EqualityCondition}
    /// };
    ///
    /// let pets = [
    ///     Some(Pet::try_from(PetName::Gorilla).unwrap()),
    ///     Some(Pet::try_from(PetName::Leopard).unwrap()),
    ///     Some(Pet::try_from(PetName::Mosquito).unwrap())
    /// ];
    /// let mut team = Team::new(&pets, 5).unwrap();
    /// // Get pets with a start of battle effect trigger.
    /// let matching_pets = team.get_pets_by_cond(
    ///     &ItemCondition::Equal(EqualityCondition::Trigger(Status::StartOfBattle))
    /// );
    /// assert_eq!(
    ///     matching_pets.len(),
    ///     2
    /// );
    /// ```
    /// ---
    /// Pets with [`Honey`](crate::FoodName::Honey) as [`Food`](crate::Food).
    /// ```
    /// use saptest::{
    ///     Pet, PetName, Food, FoodName,
    ///     Team, TeamViewer,
    ///     ItemCondition, EntityName, Position,
    ///     effects::state::{Status, EqualityCondition}
    /// };
    /// let pets = [
    ///     Some(Pet::try_from(PetName::Gorilla).unwrap()),
    ///     Some(Pet::try_from(PetName::Leopard).unwrap()),
    ///     Some(Pet::try_from(PetName::Mosquito).unwrap())
    /// ];
    /// let mut team = Team::new(&pets, 5).unwrap();
    /// // Give two random pets honey.
    /// team.set_item(
    ///     &Position::N(ItemCondition::None, 2, true),
    ///     Some(Food::try_from(FoodName::Honey).unwrap())
    /// );
    /// let matching_pets = team.get_pets_by_cond(
    ///     &ItemCondition::Equal(
    ///         EqualityCondition::Name(
    ///             EntityName::Food(FoodName::Honey)
    ///         )
    ///    )
    /// );
    /// assert_eq!(matching_pets.len(), 2);
    /// ```
    fn get_pets_by_cond(&self, cond: &ItemCondition) -> Vec<Rc<RefCell<Pet>>>;

    /// Get all pet effects on the team.
    /// # Examples
    /// ```
    /// use saptest::{Pet, PetName, Team, TeamViewer};
    ///
    /// let team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Dog).unwrap())],
    ///     5
    /// ).unwrap();
    ///
    /// assert_eq!(team.get_effects().len(), 1);
    /// ```
    fn get_effects(&self) -> Vec<Vec<Effect>>;

    /// Get pets affected by an effect and a trigger.
    /// # Example
    /// ```
    /// use saptest::{
    ///     Team, TeamViewer, Pet, PetName,
    ///     Effect, Position, Entity, Statistics,
    ///     effects::{
    ///         trigger::*,
    ///         state::Target,
    ///         actions::{Action, StatChangeType}
    ///     }
    /// };
    /// let team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Ant).unwrap()), Some(Pet::try_from(PetName::Dog).unwrap())],
    ///     5
    /// ).unwrap();
    /// let enemy_team = team.clone();
    /// // Define the crocodile pet effect.
    /// let croc_effect = Effect::new(
    ///     Entity::Pet,
    ///     TRIGGER_START_BATTLE,
    ///     Target::Enemy,
    ///     Position::Last,
    ///     Action::Remove(StatChangeType::StaticValue(Statistics { attack: 8, health: 0 })),
    ///     Some(1),
    ///     true
    /// );
    /// // Search for pets.
    /// let pets_found = team.get_pets_by_effect(&TRIGGER_NONE, &croc_effect, Some(&enemy_team)).unwrap();
    /// // As expected, the last enemy pet is the target of the effect.
    /// let (target, pet_found) = pets_found.first().unwrap();
    /// assert_eq!(Target::Enemy, *target);
    /// assert!(
    ///     pets_found.len() == 1 &&
    ///     &enemy_team.last().unwrap() == pet_found
    /// )
    /// ```
    fn get_pets_by_effect(
        &self,
        trigger: &Outcome,
        effect: &Effect,
        opponent: Option<&Team>,
    ) -> Result<TargetPets, SAPTestError>;

    /// Get a pet by a [`Position`].
    /// * Specific [`Position`] variants like [`Position::Relative`] and [`Position::Range`] require a starting pet hence the optional `curr_pet`.
    /// * [`TargetPets`] is a tuple with the belonging Target group ([`Target::Shop`], [`Target::Friend`], [`Target::Enemy`]) and the pets found.
    /// * May [`panic`] under certain [`ItemCondition`]s.
    ///     * See [`TeamViewer::get_pets_by_cond`].
    /// # Example
    /// ```rust
    /// use saptest::{
    ///     Team, TeamViewer, Pet, PetName,
    ///     Position, ItemCondition, effects::state::Target
    /// };
    /// let team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Ant).unwrap()), Some(Pet::try_from(PetName::Dog).unwrap())],
    ///     5
    /// ).unwrap();
    /// // Find all pets.
    /// let found_pets = team.get_pets_by_pos(None, &Target::Friend, &Position::All(ItemCondition::None), None, None);
    /// assert!(
    ///     found_pets.is_ok() &&
    ///     found_pets.unwrap().len() == 2
    /// );
    /// ```
    fn get_pets_by_pos(
        &self,
        curr_pet: Option<Rc<RefCell<Pet>>>,
        target: &Target,
        pos: &Position,
        trigger: Option<&Outcome>,
        opponent: Option<&Team>,
    ) -> Result<TargetPets, SAPTestError>;

    /// Get the number of open [`Pet`] slots on the [`Team`].
    /// # Example
    /// ```
    /// use saptest::{Team, TeamViewer};
    ///
    /// let team = Team::default();
    /// assert_eq!(team.open_slots(), 5);
    /// ```
    fn open_slots(&self) -> usize;

    /// Get the number of filled [`Pet`] slots on the [`Team`].
    /// # Example
    /// ```
    /// use saptest::{Team, TeamViewer};
    ///
    /// let team = Team::default();
    /// assert_eq!(team.filled_slots(), 0);
    /// ```
    fn filled_slots(&self) -> usize;
}

impl TeamViewer for Team {
    fn filled_slots(&self) -> usize {
        self.friends.iter().flatten().count()
    }
    fn open_slots(&self) -> usize {
        self.max_size - self.filled_slots()
    }

    fn get_effects(&self) -> Vec<Vec<Effect>> {
        self.friends
            .iter()
            .flatten()
            .map(|pet| pet.borrow().effect.clone())
            .collect_vec()
    }

    fn nth(&self, idx: usize) -> Option<Rc<RefCell<Pet>>> {
        self.friends.get(idx).and_then(|pet| {
            pet.as_ref()
                .filter(|pet| pet.borrow().stats.health != 0)
                .map(Rc::clone)
        })
    }

    fn first(&self) -> Option<Rc<RefCell<Pet>>> {
        self.friends.first().and_then(|pet| {
            pet.as_ref()
                .filter(|pet| pet.borrow().stats.health != 0)
                .map(Rc::clone)
        })
    }

    fn last(&self) -> Option<Rc<RefCell<Pet>>> {
        self.friends.last().and_then(|pet| {
            pet.as_ref()
                .filter(|pet| pet.borrow().stats.health != 0)
                .map(Rc::clone)
        })
    }

    fn any(&self) -> Option<Rc<RefCell<Pet>>> {
        let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));
        self.all().into_iter().choose(&mut rng)
    }

    fn all(&self) -> Vec<Rc<RefCell<Pet>>> {
        self.friends
            .iter()
            .filter_map(|pet| {
                pet.as_ref()
                    .filter(|pet| pet.borrow().stats.health != 0)
                    .map(Rc::clone)
            })
            .collect_vec()
    }

    fn check_eq_cond<T>(&self, all_pets: T, eq_cond: &EqualityCondition) -> Vec<Rc<RefCell<Pet>>>
    where
        T: IntoIterator<Item = Rc<RefCell<Pet>>>,
    {
        let all_pets = all_pets.into_iter();
        match eq_cond {
            EqualityCondition::IsSelf => all_pets
                .filter(|pet| Rc::downgrade(pet).ptr_eq(self.curr_pet.as_ref().unwrap()))
                .collect_vec(),
            EqualityCondition::Tier(tier) => all_pets
                .filter(|pet| pet.borrow().tier == *tier)
                .collect_vec(),
            EqualityCondition::Name(name) => all_pets
                .filter(|pet| match name {
                    EntityName::Pet(pet_name) => &pet.borrow().name == pet_name,
                    EntityName::Food(item_name) => {
                        // If item_name is None. Means check pet has no food.
                        if item_name == &FoodName::None {
                            pet.borrow().item.is_none()
                        } else {
                            pet.borrow()
                                .item
                                .as_ref()
                                .map_or(false, |food| &food.name == item_name)
                        }
                    }
                })
                .collect_vec(),
            EqualityCondition::Level(lvl) => all_pets
                .filter(|pet| pet.borrow().lvl.eq(lvl))
                .collect_vec(),
            EqualityCondition::Trigger(trigger) => all_pets
                .filter(|pet| {
                    pet.borrow()
                        .effect
                        .iter()
                        .any(|effect| effect.trigger.status == *trigger)
                })
                .collect_vec(),
            EqualityCondition::Action(action) => all_pets
                .filter(|pet| {
                    pet.borrow()
                        .effect
                        .iter()
                        .any(|effect| effect.action == **action)
                })
                .collect_vec(),
            _ => unimplemented!("ItemCondition not implemented for Team pets."),
        }
    }

    fn get_pets_by_cond(&self, cond: &ItemCondition) -> Vec<Rc<RefCell<Pet>>> {
        if let ItemCondition::Multiple(conditions) = cond {
            conditions
                .iter()
                .flat_map(|condition| self.get_pets_by_cond(condition))
                .collect()
        } else if let ItemCondition::MultipleAll(conditions) = cond {
            let mut matching_pets = vec![];
            let all_matches = conditions
                .iter()
                .map(|cond| self.get_pets_by_cond(cond))
                .collect_vec();
            // Take smallest set of matches.
            if let Some(mut first_matching_pets) = all_matches
                .iter()
                .min_by(|matches_1, matches_2| matches_1.len().cmp(&matches_2.len()))
                .cloned()
            {
                // Remove any pets not within.
                for matches in all_matches.iter() {
                    first_matching_pets.retain(|pet| matches.contains(pet))
                }
                matching_pets.extend(first_matching_pets.iter().cloned())
            }
            matching_pets
        } else {
            let all_pets = self.all().into_iter();
            match cond {
                ItemCondition::Healthiest => all_pets
                    .max_by(|pet_1, pet_2| {
                        pet_1
                            .borrow()
                            .stats
                            .health
                            .cmp(&pet_2.borrow().stats.health)
                    })
                    .map_or(vec![], |found| vec![found]),
                ItemCondition::Illest => all_pets
                    .min_by(|pet_1, pet_2| {
                        pet_1
                            .borrow()
                            .stats
                            .health
                            .cmp(&pet_2.borrow().stats.health)
                    })
                    .map_or(vec![], |found| vec![found]),
                ItemCondition::Strongest => all_pets
                    .max_by(|pet_1, pet_2| {
                        pet_1
                            .borrow()
                            .stats
                            .attack
                            .cmp(&pet_2.borrow().stats.attack)
                    })
                    .map_or(vec![], |found| vec![found]),
                ItemCondition::Weakest => all_pets
                    .min_by(|pet_1, pet_2| {
                        pet_1
                            .borrow()
                            .stats
                            .attack
                            .cmp(&pet_2.borrow().stats.attack)
                    })
                    .map_or(vec![], |found| vec![found]),
                ItemCondition::Equal(eq_cond) => self.check_eq_cond(all_pets, eq_cond),
                ItemCondition::NotEqual(eq_cond) => {
                    let eqiv_pets = self.check_eq_cond(all_pets.clone(), eq_cond);
                    all_pets
                        .into_iter()
                        .filter(|pet| !eqiv_pets.contains(pet))
                        .collect_vec()
                }
                // Allow all if condition is None.
                ItemCondition::None => all_pets.collect_vec(),
                ItemCondition::HighestTier => all_pets
                    .max_by(|pet_1, pet_2| pet_1.borrow().tier.cmp(&pet_2.borrow().tier))
                    .map_or(vec![], |found| vec![found]),
                ItemCondition::LowestTier => all_pets
                    .min_by(|pet_1, pet_2| pet_1.borrow().tier.cmp(&pet_2.borrow().tier))
                    .map_or(vec![], |found| vec![found]),
                _ => unimplemented!("ItemCondition not implemented for Team pets or attempted to nest multiple ItemCondition::Multiple*s."),
            }
        }
    }
    fn get_pets_by_pos(
        &self,
        curr_pet: Option<Rc<RefCell<Pet>>>,
        target: &Target,
        pos: &Position,
        trigger: Option<&Outcome>,
        opponent: Option<&Team>,
    ) -> Result<TargetPets, SAPTestError> {
        let mut pets = vec![];

        let opponent = match &target {
            // Set opponent to be self as target opponent will never be used.
            Target::Friend | Target::Shop => self,
            Target::Enemy | Target::Either => {
                let Some(enemy_team) = opponent else {
                    return Err(SAPTestError::InvalidTeamAction {
                        subject: "No Enemy Team Provided".to_string(),
                        reason: format!("Enemy team is required for finding pets by target {target:?}")
                    })
                };
                enemy_team
            }
            Target::None => {
                return Err(SAPTestError::InvalidTeamAction {
                    subject: "No Target Provided".to_string(),
                    reason: "A target is required for finding pets.".to_string(),
                })
            }
        };
        let team = if matches!(*target, Target::Friend | Target::Either) {
            self
        } else {
            opponent
        };

        match (target, &pos) {
            (Target::Friend | Target::Enemy, Position::Any(condition)) => {
                let mut rng = ChaCha12Rng::seed_from_u64(team.seed.unwrap_or_else(random));
                if let Some(random_pet) = team
                    .get_pets_by_cond(condition)
                    .into_iter()
                    .choose(&mut rng)
                {
                    pets.push((*target, random_pet))
                }
            }
            (Target::Either, Position::Any(condition)) => {
                let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));
                let self_pets = self.get_pets_by_cond(condition);
                let opponent_pets = opponent.get_pets_by_cond(condition);
                if let Some((target, random_pet)) = vec![Target::Friend; self_pets.len()]
                    .into_iter()
                    .zip_eq(self_pets)
                    .chain(
                        vec![Target::Enemy; opponent_pets.len()]
                            .into_iter()
                            .zip_eq(opponent_pets),
                    )
                    .choose(&mut rng)
                {
                    pets.push((target, random_pet))
                }
            }
            (Target::Friend | Target::Enemy, Position::All(condition)) => {
                for pet in team.get_pets_by_cond(condition) {
                    pets.push((*target, pet))
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
                if let Some(Some(pos)) = &curr_pet.map(|pet| pet.borrow().pos) {
                    if let Some(opposite_pet) = team.nth(*pos) {
                        pets.push((*target, opposite_pet))
                    }
                }
            }
            (_, Position::OnSelf) => {
                if let Some(self_pet) = &curr_pet {
                    pets.push((*target, self_pet.clone()))
                }
            }
            (_, Position::TriggerAfflicting) | (_, Position::TriggerAffected) => {
                let Some(trigger) = trigger else {
                    let pos = pos.clone();
                    return Err(SAPTestError::InvalidTeamAction {
                        subject: "No Trigger Provided".to_string(),
                        reason: format!("Trigger required for finding pets by {pos:?}")
                    })
                };
                let trigger_pet = if let Position::TriggerAffected = pos {
                    trigger.affected_pet.as_ref()
                } else {
                    trigger.afflicting_pet.as_ref()
                };
                if let Some(Some(trigger_pet)) = trigger_pet.map(|pet_ref| pet_ref.upgrade()) {
                    pets.push((trigger.affected_team, trigger_pet))
                }
            }
            (Target::Friend | Target::Enemy, Position::Relative(rel_pos)) => {
                if let Some(Some(effect_pet_idx)) = &curr_pet.as_ref().map(|pet| pet.borrow().pos) {
                    let (target_team, adj_idx) = team
                        .cvt_rel_idx_to_adj_idx(*effect_pet_idx, *rel_pos)
                        .unwrap();
                    // Pet can only be on same team.
                    if target_team == Target::Friend {
                        if let Some(rel_pet) = team.nth(adj_idx) {
                            pets.push((*target, rel_pet))
                        }
                    }
                }
            }
            (Target::Either, Position::Relative(rel_pos)) => {
                if let Some(Some(effect_pet_idx)) = &curr_pet.as_ref().map(|pet| pet.borrow().pos) {
                    let (target_team, adj_idx) = self
                        .cvt_rel_idx_to_adj_idx(*effect_pet_idx, *rel_pos)
                        .unwrap();
                    let team = if target_team == Target::Friend {
                        self
                    } else {
                        opponent
                    };
                    if let Some(rel_pet) = team.nth(adj_idx) {
                        pets.push((target_team, rel_pet))
                    }
                }
            }
            (Target::Friend | Target::Enemy, Position::Nearest(n_pets_directional)) => {
                if let Some(Some(effect_pet_idx)) = curr_pet.as_ref().map(|pet| pet.borrow().pos) {
                    // Negative ranges have to work for both teams hence matching on target.
                    // When matching on opponent, always set to first pet.
                    // (o = curr, x = dest)
                    // Ex. -2 behind
                    //      [o][ ][x][ ][ ]
                    // * If friend: 1..5 (We don't want the first position.)
                    // * If enemy: 0..5 (We want the first position.)
                    // Ex. 2 ahead
                    //      [x][ ][o][ ][ ]
                    // * If at pos other than 0: 0..3
                    // * If at pos 0: 0..0 (We don't want the first position.)
                    let (num_pets, pet_range) = if n_pets_directional.is_negative() {
                        // If target is enemy include the first pet in pet range.
                        let start_pos = if *target == Target::Enemy {
                            effect_pet_idx
                        } else {
                            effect_pet_idx + 1
                        };
                        (
                            TryInto::<usize>::try_into(-*n_pets_directional)?,
                            start_pos..team.friends.len(),
                        )
                    } else {
                        let num_pets = TryInto::<usize>::try_into(*n_pets_directional)?;
                        let start_pos = effect_pet_idx.saturating_sub(num_pets);
                        (
                            num_pets,
                            // Cover first position to the position of pet.
                            start_pos..effect_pet_idx,
                        )
                    };
                    // Get pets from range.
                    if let Some(slots_in_range) = team.friends.get(pet_range) {
                        let mut pets_in_range = slots_in_range.iter().flatten();

                        for _ in 0..num_pets {
                            if let Some(pet) = pets_in_range.next() {
                                pets.push((*target, pet.clone()))
                            }
                        }
                    }
                }
            }
            (Target::Either, Position::Nearest(n_pets_directional)) => {
                let num_pets = if n_pets_directional.is_negative() {
                    TryInto::<usize>::try_into(-*n_pets_directional)?
                } else {
                    TryInto::<usize>::try_into(*n_pets_directional)?
                };
                let pets_in_range = team.get_pets_by_pos(
                    curr_pet.clone(),
                    &Target::Friend,
                    &Position::Nearest(*n_pets_directional),
                    trigger,
                    Some(opponent),
                )?;
                // If less than expected on current team found. Get the other team's pets.
                let num_in_range = pets_in_range.len();
                if num_in_range < num_pets {
                    let missing_n_pets: isize = (num_pets - num_in_range).try_into()?;
                    // Set reference pet to first opponent pet and target to enemy
                    let opponents_pets = team.get_pets_by_pos(
                        curr_pet,
                        &Target::Enemy,
                        // Set to negative as we are looking at the pets behind the first opponent pet.
                        &Position::Nearest(-(missing_n_pets)),
                        None,
                        Some(opponent),
                    )?;
                    pets.extend(
                        opponents_pets
                            .into_iter()
                            .map(|(_, pet)| (Target::Enemy, pet)),
                    );
                }
                // Add pets found on team to opponent's pets.
                pets.extend(pets_in_range);
            }
            (Target::Friend | Target::Enemy, Position::Range(effect_range)) => {
                for idx in effect_range.clone() {
                    if let Some(Some(effect_pet_idx)) =
                        curr_pet.as_ref().map(|pet| pet.borrow().pos)
                    {
                        let (target_team, adj_idx) =
                            team.cvt_rel_idx_to_adj_idx(effect_pet_idx, idx)?;
                        if target_team == Target::Friend {
                            if let Some(rel_pet) = team.nth(adj_idx) {
                                pets.push((target_team, rel_pet))
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
                            self.cvt_rel_idx_to_adj_idx(*effect_pet_idx, idx).unwrap();
                        let team = if target_team == Target::Friend {
                            self
                        } else {
                            opponent
                        };
                        if let Some(rel_pet) = team.nth(adj_idx) {
                            pets.push((target_team, rel_pet.clone()))
                        }
                    }
                }
            }
            (Target::Friend | Target::Enemy, Position::First) => {
                if let Some(first_pet) = team.all().first() {
                    pets.push((*target, first_pet.clone()))
                }
            }
            (Target::Friend | Target::Enemy, Position::Last) => {
                if let Some(last_pet) = team.all().last() {
                    pets.push((*target, last_pet.clone()))
                }
            }
            (_, Position::Multiple(positions)) => {
                for pos in positions {
                    pets.extend(self.get_pets_by_pos(
                        curr_pet.clone(),
                        target,
                        pos,
                        trigger,
                        Some(opponent),
                    )?)
                }
            }
            (Target::Either, Position::N(condition, num_pets, randomize)) => {
                let mut self_pets = self.get_pets_by_cond(condition);
                let mut opponent_pets = opponent.get_pets_by_cond(condition);

                if *randomize {
                    let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));
                    self_pets.shuffle(&mut rng);
                    opponent_pets.shuffle(&mut rng);
                }

                let (mut self_pets, mut opponent_pets) =
                    (self_pets.into_iter(), opponent_pets.into_iter());

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
            (Target::Friend | Target::Enemy, Position::N(condition, n, randomize)) => {
                let mut found_pets = team.get_pets_by_cond(condition);
                if *randomize {
                    let mut rng = ChaCha12Rng::seed_from_u64(self.seed.unwrap_or_else(random));
                    found_pets.shuffle(&mut rng);
                }
                let mut found_pets = found_pets.into_iter();

                // Get n values of indices.
                for _ in 0..*n {
                    if let Some(pet) = found_pets.next() {
                        pets.push((*target, pet))
                    }
                }
            }
            (Target::Friend | Target::Enemy, Position::Adjacent) => {
                let friends = if *target == Target::Friend {
                    &self.friends
                } else {
                    &opponent.friends
                };
                let Some(Some(pos)) = curr_pet.map(|pet| pet.borrow().pos) else {
                    return Err(SAPTestError::InvalidTeamAction {
                        subject: "No Pet Position Idx".to_string(),
                        reason: format!("Pet position required for finding pets by {pos:?}")
                    })
                };
                // Get pet ahead and behind.
                if let Some(Some(prev_pet)) = pos.checked_sub(1).map(|idx| {
                    friends
                        .iter()
                        .flatten()
                        .find(|friend| friend.borrow().pos == Some(idx))
                }) {
                    pets.push((*target, prev_pet.clone()))
                };
                if let Some(ahead_pet) = friends
                    .iter()
                    .flatten()
                    .find(|friend| friend.borrow().pos == Some(pos + 1))
                {
                    pets.push((*target, ahead_pet.clone()))
                }
            }
            (Target::Shop, pos) => {
                let items = self.shop.get_shop_items_by_pos(pos, &Entity::Pet)?;
                for item in items {
                    if let ItemSlot::Pet(shop_pet) = &item.item {
                        pets.push((*target, shop_pet.clone()))
                    }
                }
            }
            _ => {
                return Err(SAPTestError::InvalidTeamAction {
                    subject: "Unimplemented Target/Position".to_string(),
                    reason: format!("Cannot find any valid pets with {target:?} and {pos:?}"),
                })
            }
        }

        Ok(pets)
    }

    fn get_pets_by_effect(
        &self,
        trigger: &Outcome,
        effect: &Effect,
        opponent: Option<&Team>,
    ) -> Result<TargetPets, SAPTestError> {
        let curr_pet = if let Some(effect_pet) = &effect.owner {
            effect_pet.upgrade()
        } else {
            // Otherwise, use first pet on team.
            self.first()
        };
        self.get_pets_by_pos(
            curr_pet,
            &effect.target,
            &effect.position,
            Some(trigger),
            opponent,
        )
    }
}
