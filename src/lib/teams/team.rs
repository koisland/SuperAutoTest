use crate::{
    effects::{
        effect::Effect,
        state::{Outcome, Position, Status, Target},
        trigger::*,
    },
    error::SAPTestError,
    graph::effect_graph::History,
    pets::pet::{assign_effect_owner, Pet},
    shop::store::ShopState,
    teams::{effects::TeamEffects, viewer::TeamViewer},
    Food, PetCombat, Shop,
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
/// use saptest::{Team, Pet, PetName, Statistics, teams::team::TeamFightOutcome};
///
/// let pet = Pet::try_from(PetName::Blowfish).unwrap();
/// let mut team = Team::new(&vec![pet.clone(); 5], 5).unwrap();
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
    pub friends: Vec<Rc<RefCell<Pet>>>,
    /// Fainted pets.
    pub fainted: Vec<Rc<RefCell<Pet>>>,
    /// Maximum number of pets that can be added.
    pub max_size: usize,
    /// Stored triggers used to invoke effects.
    ///
    /// Calling [`trigger_effects`](super::effects::TeamEffects::trigger_effects) will exhaust all stored triggers.
    /// * As a result, this will always be empty unless mutated.
    pub triggers: VecDeque<Outcome>,
    /// Effect history of a team.
    #[serde(skip)]
    pub(crate) history: History,
    /// Pet shop.
    #[serde(skip)]
    pub(crate) shop: Shop,
    /// Current pet.
    #[serde(skip)]
    pub(crate) curr_pet: Option<Weak<RefCell<Pet>>>,
    /// Clone of pets used for restoring team.
    pub(crate) stored_friends: Vec<Pet>,
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
                .map(|pet| Rc::new(RefCell::new(pet.borrow().clone())))
                .collect_vec(),
            fainted: self
                .fainted
                .iter()
                .map(|pet| Rc::new(RefCell::new(pet.borrow().clone())))
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
    ///     &vec![Pet::try_from(PetName::Dog).unwrap(); 5],
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
    ///     &vec![Pet::try_from(PetName::Dog).unwrap(); 20],
    ///     20
    /// );
    /// assert!(team.is_ok());
    /// assert_eq!(team.unwrap().friends.len(), 20);
    /// ```
    pub fn new(pets: &[Pet], max_size: usize) -> Result<Team, SAPTestError> {
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
            let curr_pet = rc_pets.first().map(Rc::downgrade);
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
        for pet in self.friends.iter().chain(self.fainted.iter()) {
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
                .find(|pet| pet_and_trigger_pet_equal(pet, trigger.affected_pet.as_ref()))
            {
                trigger.set_affected(affected_pet);
            }
            if let Some(enemy_pets) = opponent.as_ref().map(|team| &team.friends) {
                if let Some(afflicting_pet) = self
                    .friends
                    .iter()
                    .chain(enemy_pets.iter())
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
                .find(|pet| current_pet.ptr_eq(&Rc::downgrade(pet)))
        }) {
            self.curr_pet = Some(Rc::downgrade(current_pet));
        }
        self
    }

    /// Create reference counted pets.
    fn create_rc_pets(pets: &[Pet]) -> Vec<Rc<RefCell<Pet>>> {
        // Index pets.
        let mut rc_pets: Vec<Rc<RefCell<Pet>>> = vec![];

        for (i, mut pet) in pets.iter().cloned().enumerate() {
            // Create id if one not assigned.
            pet.id = Some(pet.id.clone().unwrap_or(format!("{}_{}", pet.name, i)));
            pet.set_pos(i);

            let rc_pet = Rc::new(RefCell::new(pet));

            // Assign weak reference to owner for all effects.
            assign_effect_owner(&rc_pet);

            rc_pets.push(rc_pet)
        }
        rc_pets
    }

    /// Restore a team to its initial state prior to a battle.
    /// # Example
    /// ```rust no run
    /// use saptest::{Pet, PetName, Team, TeamEffects};
    ///
    /// let mut default_team = Team::default();
    /// default_team
    ///     .add_pet(Pet::try_from(PetName::Dog).unwrap(), 0, None).unwrap()
    ///     .restore();
    /// ```
    pub fn restore(&mut self) -> &mut Self {
        self.friends = Team::create_rc_pets(&self.stored_friends);
        // Set current pet to first in line.
        self.curr_pet = self.friends.first().map(Rc::downgrade);
        self.fainted.clear();
        // Set current battle phase to 1.
        self.history.curr_phase = 1;
        self.pet_count = self.stored_friends.len();
        self
    }

    /// Clear team of empty slots and/or fainted pets and reset indices.
    /// # Examples
    /// ```
    /// use saptest::{Pet, PetName, Team, TeamViewer, TeamEffects};
    ///
    /// let mut default_team = Team::new(
    ///     &[Pet::try_from(PetName::Dog).unwrap()],
    ///     5
    /// ).unwrap();
    ///
    /// assert_eq!(default_team.friends.len(), 1);
    ///
    /// default_team.first().unwrap().borrow_mut().stats.health = 0;
    /// default_team.clear_team();
    ///
    /// assert_eq!(default_team.friends.len(), 0);
    /// ```
    pub fn clear_team(&mut self) -> &mut Self {
        let mut new_idx = 0;
        self.friends.retain(|pet| {
            // Check if not dead.
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
    /// use saptest::{Pet, PetName, Team, TeamEffects, effects::trigger::TRIGGER_START_BATTLE};
    ///
    /// let mosquito = Pet::try_from(PetName::Mosquito).unwrap();
    /// let mut team = Team::new(&[mosquito.clone(), mosquito.clone()], 5).unwrap();
    /// let mut enemy_team = team.clone();
    ///
    /// // Set seed for enemy_team and trigger StartBattle effects.
    /// enemy_team.set_seed(Some(0));
    /// team.triggers.push_front(TRIGGER_START_BATTLE);
    /// team.trigger_effects(&mut enemy_team);
    ///
    /// // Mosquitoes always hit second pet with seed set to 0.
    /// assert!(
    ///     enemy_team.friends.get(1).map_or(false, |pet| pet.borrow().stats.health == 0),
    /// )
    /// ```
    pub fn set_seed(&mut self, seed: Option<u64>) {
        self.seed = seed;

        for pet in self.friends.iter().chain(self.fainted.iter()) {
            pet.borrow_mut().seed = seed
        }
        for stored_pet in self.stored_friends.iter_mut() {
            stored_pet.seed = seed
        }
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
    ///     &[Pet::try_from(PetName::Dog).unwrap()],
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
            .get_pets_by_effect(&TRIGGER_NONE, &null_effect, self)
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
    ///     &[Pet::try_from(PetName::Dog).unwrap()],
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
        let affected_pets = self.get_pets_by_effect(&TRIGGER_NONE, &null_effect, self)?;

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
    ///     Pet::try_from(PetName::Gorilla).unwrap(),
    ///     Pet::try_from(PetName::Leopard).unwrap(),
    ///     Pet::try_from(PetName::Cat).unwrap(),
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

            let pet = self.friends.remove(pos);

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
            return Err(SAPTestError::InvalidTeamAction {
                subject: "Push Pet".to_string(),
                reason: format!("Invalid indices ({pos})."),
            });
        }

        Ok(self)
    }

    pub(super) fn set_indices(&self) -> &Self {
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
    /// use saptest::{Pet, PetName, Team, TeamViewer};
    ///
    /// let mut team = Team::new(&[
    ///     Pet::try_from(PetName::Cat).unwrap(),
    ///     Pet::try_from(PetName::Cat).unwrap(),
    ///     Pet::try_from(PetName::Cat).unwrap(),
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
        mut pet: Pet,
        pos: usize,
        opponent: Option<&mut Team>,
    ) -> Result<&mut Self, SAPTestError> {
        // Assign id to pet if not any.
        let new_pet_id = format!("{}_{}", pet.name, self.pet_count + 1);
        pet.id = Some(pet.id.clone().unwrap_or(new_pet_id));
        pet.pos = Some(pos);

        let rc_pet = Rc::new(RefCell::new(pet));

        if self.all().len() == self.max_size {
            // Add overflow to dead pets.
            self.fainted.push(rc_pet);

            return Err(SAPTestError::InvalidPetAction {
                subject: "Add Pet".to_string(),
                reason: format!("Maximum number of pets ({}) reached.", self.max_size),
            });
        }

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
        self.friends.insert(pos, rc_pet);

        // Set current pet to always be first in line.
        self.curr_pet = Some(Rc::downgrade(self.friends.first().unwrap()));
        // And reset indices.
        self.set_indices();

        Ok(self)
    }

    /// Fight another team for a single battle phase.
    ///
    /// # Examples
    /// ---
    /// To complete the battle.
    /// ```rust
    /// use saptest::{Team, Pet, PetName, teams::team::TeamFightOutcome};
    ///
    /// let mut team = Team::new(
    ///     &vec![Pet::try_from(PetName::Cricket).unwrap(); 5],
    ///     5
    /// ).unwrap();
    /// let mut enemy_team = Team::new(
    ///     &[Pet::try_from(PetName::Hippo).unwrap()],
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
    /// use saptest::{Team, Pet, PetName, teams::team::TeamFightOutcome};
    ///
    /// let mut team = Team::new(
    ///     &vec![Pet::try_from(PetName::Cricket).unwrap(); 5],
    ///     5
    /// ).unwrap();
    /// let mut enemy_team = Team::new(
    ///     &[Pet::try_from(PetName::Hippo).unwrap()],
    ///     5
    /// ).unwrap();
    ///
    /// let n = 2;
    /// let mut outcome = team.fight(&mut enemy_team).unwrap();
    /// for _ in 0..n-1 {
    ///     outcome = team.fight(&mut enemy_team).unwrap();
    /// }
    /// ```
    pub fn fight(&mut self, opponent: &mut Team) -> Result<TeamFightOutcome, SAPTestError> {
        // Exit while any shop is open.
        if self.shop.state == ShopState::Open || opponent.shop.state == ShopState::Open {
            return Err(SAPTestError::InvalidTeamAction {
                subject: "Shop Not Closed".to_string(),
                reason:
                    "Cannot fight while one or more teams has an open shop. Call shop.close_shop()"
                        .to_string(),
            });
        }

        info!(target: "dev", "(\"{}\")\n{}", self.name, self);
        info!(target: "dev", "(\"{}\")\n{}", opponent.name, opponent);

        // Apply start of battle effects.
        self.clear_team();
        opponent.clear_team();

        // If current phase is 1, add start battle triggers.
        if self.history.curr_phase == 1 {
            self.triggers.push_front(TRIGGER_START_BATTLE)
        }
        if opponent.history.curr_phase == 1 {
            opponent.triggers.push_front(TRIGGER_START_BATTLE)
        }
        while !self.triggers.is_empty() || !opponent.triggers.is_empty() {
            self.trigger_effects(opponent)?;
            opponent.trigger_effects(self)?;
        }

        self.clear_team();
        opponent.clear_team();

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
                self.trigger_effects(opponent)?;
                opponent.trigger_effects(self)?;
            }

            self.clear_team();
            opponent.clear_team();
        }

        // Check that two pets exist and attack.
        // Attack will result in triggers being added.
        if let (Some(pet), Some(opponent_pet)) = (self.first(), opponent.first()) {
            // Attack and get outcome of fight.
            info!(target: "dev", "Fight!\nPet: {}\nOpponent: {}", pet.borrow(), opponent_pet.borrow());
            let mut outcome = pet.borrow_mut().attack(&mut opponent_pet.borrow_mut());
            info!(target: "dev", "(\"{}\")\n{}", self.name, self);
            info!(target: "dev", "(\"{}\")\n{}", opponent.name, opponent);

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
                self.trigger_effects(opponent)?.clear_team();
                opponent.trigger_effects(self)?.clear_team();
            }
        }

        // Check if battle complete.
        Ok(
            if !self.friends.is_empty() && !opponent.friends.is_empty() {
                TeamFightOutcome::None
            } else {
                // Add end of battle node.
                self.history.prev_node = self.history.curr_node;
                self.history.curr_node =
                    Some(self.history.effect_graph.add_node(TRIGGER_END_BATTLE));
                // On outcome, increase turn count.
                self.history.curr_turn += 1;

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
            },
        )
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