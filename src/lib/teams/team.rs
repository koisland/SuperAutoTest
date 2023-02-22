use crate::{
    effects::{
        effect::Effect,
        state::{Outcome, Position, Target},
        trigger::*,
    },
    error::SAPTestError,
    graph::effect_graph::History,
    pets::pet::{assign_effect_owner, Pet},
    shop::store::ShopState,
    teams::viewer::TeamViewer,
    Food, Shop,
};

use itertools::Itertools;
use log::info;
use rand::random;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::VecDeque,
    fmt::Display,
    rc::{Rc, Weak},
};

/// The outcome of a [`Team`](crate::teams::team::Team) fight.
///
/// # Examples
/// This can be used as an exit condition in a fight.
/// ```rust
/// use saptest::{
///     Team, TeamCombat, teams::team::TeamFightOutcome,
///     Pet, PetName, Statistics
/// };
///
/// let pet = Pet::try_from(PetName::Blowfish).unwrap();
/// let mut team = Team::new(&vec![Some(pet); 5], 5).unwrap();
/// let mut enemy_team = Team::clone(&team);
///
/// // Continue fighting while the winner of a fight is None.
/// let mut winner = team.fight(&mut enemy_team).unwrap();
/// while let TeamFightOutcome::None = winner {
///     winner = team.fight(&mut enemy_team).unwrap();
/// }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum TeamFightOutcome {
    /// Outcome of fight is a win.
    Win,
    /// Outcome of fight is a loss.
    Loss,
    /// Outcome of fight is a draw.
    Draw,
    /// No outcome for fight.
    None,
}

/// A Super Auto Pets team.
#[derive(Debug, Serialize, Deserialize)]
pub struct Team {
    /// Seed used to reproduce the outcome of events.
    pub seed: Option<u64>,
    /// Name of the team.
    pub name: String,
    /// Pets on the team.
    pub friends: Vec<Option<Rc<RefCell<Pet>>>>,
    /// Fainted pets.
    pub fainted: Vec<Option<Rc<RefCell<Pet>>>>,
    /// Maximum number of pets that can be added.
    pub max_size: usize,
    /// Stored triggers used to invoke effects.
    ///
    /// Calling [`trigger_effects`](super::effects::TeamEffects::trigger_effects) will exhaust all stored triggers.
    /// * As a result, this will always be empty unless mutated.
    pub triggers: VecDeque<Outcome>,
    /// Pet shop.
    #[serde(skip)]
    pub(crate) shop: Shop,
    /// Effect history of a team.
    #[serde(skip)]
    pub(crate) history: History,
    /// Current pet.
    #[serde(skip)]
    pub(crate) curr_pet: Option<Weak<RefCell<Pet>>>,
    /// Clone of pets used for restoring team.
    pub(crate) stored_friends: Vec<Option<Pet>>,
    /// Count of all pets summoned on team.
    pub(crate) pet_count: usize,
}

impl Default for Team {
    fn default() -> Self {
        let seed = random();
        let mut shop = Shop::default();
        // Random shop by default.
        shop.seed = None;

        Self {
            // TODO: Replace with auto generated names.
            name: Default::default(),
            friends: Default::default(),
            stored_friends: Default::default(),
            fainted: Default::default(),
            max_size: 5,
            triggers: VecDeque::new(),
            shop,
            history: History::new(),
            pet_count: Default::default(),
            seed,
            curr_pet: None,
        }
    }
}

impl Clone for Team {
    fn clone(&self) -> Self {
        // Because we use reference counted ptrs, default clone impl will just increase strong reference counts.
        // This will result in a panic as borrowing the original pet as mut multiple times.
        // So we need to clone the inner values and reassign owners.
        let mut copied_team = Self {
            name: self.name.clone(),
            friends: self
                .friends
                .iter()
                .map(|pet| {
                    pet.as_ref()
                        .map(|pet| Rc::new(RefCell::new(pet.borrow().clone())))
                })
                .collect_vec(),
            fainted: self
                .fainted
                .iter()
                .map(|pet| {
                    pet.as_ref()
                        .map(|pet| Rc::new(RefCell::new(pet.borrow().clone())))
                })
                .collect_vec(),
            max_size: self.max_size,
            triggers: self.triggers.clone(),
            history: self.history.clone(),
            seed: self.seed,
            stored_friends: self.stored_friends.clone(),
            pet_count: self.pet_count,
            curr_pet: self.curr_pet.clone(),
            shop: self.shop.clone(),
        };
        // Reassign references.
        copied_team.reset_pet_references(None);
        copied_team
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
    /// Create a new team of [`Pet`]s of a given size.
    /// # Examples
    /// ---
    /// Standard 5-pet team.
    /// ```
    /// use saptest::{Pet, PetName, Team, TeamEffects};
    /// let team = Team::new(
    ///     &vec![Some(Pet::try_from(PetName::Dog).unwrap()); 5],
    ///     5
    /// );
    /// assert!(team.is_ok());
    /// assert_eq!(team.unwrap().friends.len(), 5);
    /// ```
    /// ---
    /// Team of 20 pets.
    /// ```
    /// use saptest::{Pet, PetName, Team, TeamEffects};
    /// let team = Team::new(
    ///     &vec![Some(Pet::try_from(PetName::Dog).unwrap()); 20],
    ///     20
    /// );
    /// assert!(team.is_ok());
    /// assert_eq!(team.unwrap().friends.len(), 20);
    /// ```
    pub fn new(pets: &[Option<Pet>], max_size: usize) -> Result<Team, SAPTestError> {
        if pets.len() > max_size {
            Err(SAPTestError::InvalidTeamAction {
                subject: "Init Team".to_string(),
                reason: format!(
                    "Pets provided exceed specified max size. {} > {}",
                    pets.len(),
                    max_size
                ),
            })
        } else {
            let rc_pets = Team::create_rc_pets(pets);
            let n_rc_pets = rc_pets.len();
            let curr_pet = if let Some(Some(first_pet)) = rc_pets.first() {
                Some(Rc::downgrade(first_pet))
            } else {
                None
            };

            let mut team = Team {
                stored_friends: pets.to_vec(),
                friends: rc_pets,
                max_size,
                pet_count: n_rc_pets,
                curr_pet,
                ..Default::default()
            };
            // By default shop is closed when team created using new().
            team.shop.state = ShopState::Closed;
            Ok(team)
        }
    }

    /// Reassign owners for pets.
    pub(crate) fn reset_pet_references(&mut self, opponent: Option<&mut Team>) -> &mut Self {
        // Assign references.
        for pet in self.friends.iter().chain(self.fainted.iter()).flatten() {
            assign_effect_owner(pet)
        }
        // Assign affected pets for triggers.
        let pet_and_trigger_pet_equal =
            |rc_pet: &&Rc<RefCell<Pet>>, weak_pet: Option<&Weak<RefCell<Pet>>>| {
                weak_pet.map_or(false, |aff_pet| aff_pet.ptr_eq(&Rc::downgrade(rc_pet)))
            };
        for trigger in self.triggers.iter_mut() {
            if let Some(affected_pet) = self
                .friends
                .iter()
                .flatten()
                .find(|pet| pet_and_trigger_pet_equal(pet, trigger.affected_pet.as_ref()))
            {
                trigger.set_affected(affected_pet);
            }
            if let Some(enemy_pets) = opponent.as_ref().map(|team| &team.friends) {
                if let Some(afflicting_pet) = self
                    .friends
                    .iter()
                    .chain(enemy_pets.iter())
                    .flatten()
                    .find(|pet| pet_and_trigger_pet_equal(pet, trigger.afflicting_pet.as_ref()))
                {
                    trigger.set_afflicting(afflicting_pet);
                }
            }
        }
        // Set current pet.
        if let Some(Some(current_pet)) = self.curr_pet.as_ref().map(|current_pet| {
            self.friends
                .iter()
                .flatten()
                .find(|pet| current_pet.ptr_eq(&Rc::downgrade(pet)))
        }) {
            self.curr_pet = Some(Rc::downgrade(current_pet));
        }
        self
    }

    /// Create reference counted pets.
    pub(crate) fn create_rc_pets(pets: &[Option<Pet>]) -> Vec<Option<Rc<RefCell<Pet>>>> {
        // Index pets.
        let mut rc_pets: Vec<Option<Rc<RefCell<Pet>>>> = vec![];

        for (i, slot) in pets.iter().cloned().enumerate() {
            let rc_pet = if let Some(mut pet) = slot {
                // Create id if one not assigned.
                pet.id = Some(pet.id.clone().unwrap_or(format!("{}_{}", pet.name, i)));
                pet.set_pos(i);

                let rc_pet = Rc::new(RefCell::new(pet));

                // Assign weak reference to owner for all effects.
                assign_effect_owner(&rc_pet);
                Some(rc_pet)
            } else {
                None
            };

            rc_pets.push(rc_pet)
        }
        rc_pets
    }

    /// Set a `u64` seed for a team allowing for reproducibility of events.
    /// * **Note:** For abilities that select a random pet on the enemy team, the seed must be set for the opposing team.
    /// # Examples
    ///  ```
    /// use saptest::{Pet, PetName, Team, TeamEffects, effects::trigger::TRIGGER_START_BATTLE};
    ///
    /// let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// let mut team = Team::new(&vec![Some(mosquito); 2], 5).unwrap();
    /// let mut enemy_team = team.clone();
    ///
    /// // Set seed for enemy_team and trigger StartBattle effects.
    /// enemy_team.set_seed(Some(0));
    /// team.triggers.push_front(TRIGGER_START_BATTLE);
    /// team.trigger_effects(Some(&mut enemy_team));
    ///
    /// // Mosquitoes always hit second pet with seed set to 0.
    /// assert!(
    ///     enemy_team.friends.get(1).map_or(false, |pet| pet.as_ref().unwrap().borrow().stats.health == 0),
    /// )
    /// ```
    pub fn set_seed(&mut self, seed: Option<u64>) -> &mut Self {
        self.seed = seed;

        for pet in self.friends.iter().chain(self.fainted.iter()).flatten() {
            pet.borrow_mut().seed = seed
        }
        for stored_pet in self.stored_friends.iter_mut().flatten() {
            stored_pet.seed = seed
        }
        self
    }

    /// Assign an item to a team member.
    /// # Example
    /// ```
    /// use saptest::{
    ///     Pet, PetName, Food, FoodName,
    ///     Team, TeamViewer, effects::state::Position
    /// };
    ///
    /// let mut team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Dog).unwrap())],
    ///     5
    /// ).unwrap();
    /// team.set_item(
    ///     Position::Relative(0),
    ///     Some(Food::try_from(&FoodName::Garlic).unwrap())
    /// ).unwrap();
    ///
    /// let dog = team.first().unwrap();
    /// assert_eq!(dog.borrow().item.as_ref().unwrap().name, FoodName::Garlic);
    /// ```
    pub fn set_item(
        &mut self,
        pos: Position,
        item: Option<Food>,
    ) -> Result<&mut Self, SAPTestError> {
        // Create a temporary effect to grab all desired pets to give items to.
        let null_effect = Effect {
            target: Target::Friend,
            position: pos.clone(),
            owner: self.curr_pet.clone(),
            ..Default::default()
        };
        let affected_pets = self
            .get_pets_by_effect(&TRIGGER_NONE, &null_effect, None)
            .map_err(|_| SAPTestError::InvalidTeamAction {
                subject: "Item Pet Position".to_string(),
                reason: format!("Position is not valid: {pos:?}"),
            })?;

        for (_, pet) in affected_pets.iter() {
            let mut item_copy = item.clone();
            if let Some(item) = item_copy.as_mut() {
                item.ability.owner = Some(Rc::downgrade(pet));
                item.ability.trigger.affected_pet = Some(Rc::downgrade(pet));
            }
            pet.borrow_mut().item = item_copy;
        }
        Ok(self)
    }

    /// Set level of a team member.
    /// # Example
    /// ```
    /// use saptest::{
    ///     Pet, PetName, Food, FoodName,
    ///     Team, TeamViewer, effects::state::Position};
    ///
    /// let mut team = Team::new(
    ///     &[Some(Pet::try_from(PetName::Dog).unwrap())],
    ///     5
    /// ).unwrap();
    /// team.set_level(&Position::First, 2).unwrap();
    ///
    /// let dog = team.first().unwrap();
    /// assert_eq!(dog.borrow().get_level(), 2);
    /// ```
    pub fn set_level(&mut self, pos: &Position, lvl: usize) -> Result<&mut Self, SAPTestError> {
        // Create a temporary effect to grab all desired pets to give items to.
        let null_effect = Effect {
            target: Target::Friend,
            position: pos.clone(),
            owner: self.curr_pet.clone(),
            ..Default::default()
        };
        let affected_pets = self.get_pets_by_effect(&TRIGGER_NONE, &null_effect, None)?;

        for (_, pet) in affected_pets.iter() {
            pet.borrow_mut().set_level(lvl)?;

            let mut levelup_trigger = TRIGGER_SELF_LEVELUP;
            let mut levelup_any_trigger = TRIGGER_ANY_LEVELUP;
            levelup_trigger.set_affected(pet);
            levelup_any_trigger.set_affected(pet);

            self.triggers.extend([levelup_trigger, levelup_any_trigger]);

            for effect in pet.borrow_mut().effect.iter_mut() {
                effect.assign_owner(Some(pet));
            }
        }

        Ok(self)
    }

    /// Push a pet to another position on the team.
    /// * `by` is relative to current position.
    /// * An `opponent` can be provided optionally to update their `triggers`.
    /// # Examples
    /// ```
    /// use saptest::{Pet, PetName, Team, TeamViewer, Statistics};
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
    ///     team.nth(0).unwrap().borrow().name == PetName::Leopard &&
    ///     team.nth(1).unwrap().borrow().name == PetName::Cat &&
    ///     team.nth(2).unwrap().borrow().name == PetName::Gorilla
    /// )
    /// ```
    pub fn push_pet(
        &mut self,
        pos: usize,
        by: isize,
        opponent: Option<&mut Team>,
    ) -> Result<&mut Self, SAPTestError> {
        if pos < self.friends.len() {
            let new_pos: usize = if by.is_negative() {
                let pos_by: usize = (-by).try_into()?;
                (pos_by + pos).clamp(0, self.friends.len())
            } else {
                pos.saturating_sub(by.try_into()?)
            };

            if self.friends.get(pos).map_or(true, |slot| slot.is_none()) {
                return Err(SAPTestError::InvalidTeamAction {
                    subject: "No Pet at Push Position".to_string(),
                    reason: format!("Position ({pos}) has no pet."),
                });
            }
            let pet = self.friends.remove(pos).unwrap();

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

            self.friends.insert(new_pos, Some(pet));
            self.set_indices();
        } else {
            return Err(SAPTestError::InvalidTeamAction {
                subject: "Push Pet".to_string(),
                reason: format!("Invalid indices ({pos})."),
            });
        }

        Ok(self)
    }

    pub(super) fn set_indices(&self) -> &Self {
        for (i, slot) in self.friends.iter().enumerate() {
            if let Some(Ok(mut unborrowed_pet)) =
                slot.as_ref().map(|friend| friend.try_borrow_mut())
            {
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
    /// use saptest::{Pet, PetName, Team, TeamViewer};
    ///
    /// let mut team = Team::new(&[
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    ///     Some(Pet::try_from(PetName::Cat).unwrap()),
    /// ], 5).unwrap();
    ///
    /// team.add_pet(Pet::try_from(PetName::Turtle).unwrap(), 0, None);
    /// assert_eq!(
    ///     team.first().unwrap().borrow().name,
    ///     PetName::Turtle
    /// )
    /// ```
    pub fn add_pet(
        &mut self,
        pet: Pet,
        pos: usize,
        opponent: Option<&mut Team>,
    ) -> Result<&mut Self, SAPTestError> {
        let new_pet_id = format!("{}_{}", pet.name, self.pet_count + 1);
        let pet_id = pet.id.clone();
        let rc_pet = Rc::new(RefCell::new(pet));

        if self.all().len() == self.max_size {
            // Add overflow to dead pets.
            self.fainted.push(Some(rc_pet));

            return Err(SAPTestError::InvalidPetAction {
                subject: "Max Pets".to_string(),
                reason: format!("Maximum number of pets ({}) reached.", self.max_size),
            });
        }
        if pos > self.max_size {
            return Err(SAPTestError::InvalidPetAction {
                subject: "Invalid Position".to_string(),
                reason: format!(
                    "Position ({pos}) greater than maximum number of pets ({}).",
                    self.max_size
                ),
            });
        }
        // Add additional slots if greater than current number of slots..
        if pos > self.friends.len() {
            for _ in 0..pos - self.friends.len() {
                self.friends.push(None)
            }
        }

        // Assign id to pet if not any.
        rc_pet.borrow_mut().id = Some(pet_id.unwrap_or(new_pet_id));
        rc_pet.borrow_mut().pos = Some(pos);

        // Assign effects to new pet.
        for effect in rc_pet.borrow_mut().effect.iter_mut() {
            effect.assign_owner(Some(&rc_pet));
        }
        if let Some(food_item) = rc_pet.borrow_mut().item.as_mut() {
            food_item.ability.assign_owner(Some(&rc_pet));
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
            Some(weak_ref_pet),
        );

        if let Some(opponent) = opponent {
            opponent.triggers.push_back(any_enemy_trigger)
        }
        self.triggers.extend([self_trigger, any_trigger]);

        info!(target: "dev", "(\"{}\")\nAdded pet to pos {pos}: {}.", self.name.to_string(), rc_pet.borrow());

        // Empty slot. Remove and replace with pet.
        let curr_slot = self.friends.get(pos);
        if curr_slot.map_or(false, |slot| slot.is_none()) {
            self.friends.remove(pos);
        }

        self.friends.insert(pos, Some(rc_pet));

        // Set current pet to always be first in line.
        if let Some(Some(pet)) = self.friends.first() {
            self.curr_pet = Some(Rc::downgrade(pet));
        }
        // And reset indices.
        self.set_indices();

        Ok(self)
    }

    /// Create a node logging an effect's result for a [`Team`]'s history.
    pub(crate) fn create_node(&mut self, trigger: &Outcome) -> &mut Self {
        let node_idx = self.history.effect_graph.add_node(trigger.clone());
        self.history.prev_node = self.history.curr_node;
        self.history.curr_node = Some(node_idx);
        self
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for friend in self.friends.iter() {
            match friend {
                Some(friend) => writeln!(f, "{}", friend.borrow())?,
                None => writeln!(f, "[]")?,
            }
        }
        Ok(())
    }
}
