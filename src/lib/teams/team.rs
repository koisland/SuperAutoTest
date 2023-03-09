use crate::{
    effects::{
        state::{Outcome, Position, Target},
        trigger::*,
    },
    error::SAPTestError,
    pets::pet::{assign_effect_owner, Pet},
    shop::{store::ShopState, team_shopping::TeamShoppingHelpers},
    teams::history::History,
    teams::viewer::TeamViewer,
    wiki_scraper::parse_names::WordType,
    Food, Shop, CONFIG, SAPDB,
};

use itertools::Itertools;
use log::info;
use rand::{random, seq::IteratorRandom, SeedableRng};
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::VecDeque,
    fmt::Display,
    rc::{Rc, Weak},
};

const COPY_SUFFIX: &str = "_copy";

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

impl TeamFightOutcome {
    /// Opposite outcome.
    pub fn inverse(&self) -> Self {
        match self {
            TeamFightOutcome::Win => TeamFightOutcome::Loss,
            TeamFightOutcome::Loss => TeamFightOutcome::Win,
            TeamFightOutcome::Draw => TeamFightOutcome::Draw,
            TeamFightOutcome::None => TeamFightOutcome::None,
        }
    }
}

/// A Super Auto Pets team.
#[derive(Debug, Serialize, Deserialize)]
pub struct Team {
    /// Seed used to reproduce the outcome of events.
    pub seed: Option<u64>,
    /// Name of the team.
    pub(crate) name: String,
    /// Pets on the team.
    pub friends: Vec<Option<Rc<RefCell<Pet>>>>,
    /// Fainted pets.
    pub fainted: Vec<Option<Rc<RefCell<Pet>>>>,
    /// Sold pets.
    pub sold: Vec<Option<Rc<RefCell<Pet>>>>,
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
            sold: Default::default(),
            max_size: 5,
            triggers: VecDeque::new(),
            shop,
            history: History::default(),
            seed,
            curr_pet: None,
        }
    }
}

fn copy_rc_pets(
    slots: &[Option<Rc<RefCell<Pet>>>],
    team_name: Option<String>,
) -> Vec<Option<Rc<RefCell<Pet>>>> {
    slots
        .iter()
        .map(|slot| {
            if let Some(pet) = slot {
                let mut copied_pet = pet.borrow().clone();
                copied_pet.team = team_name.clone().or(copied_pet.team);
                Some(Rc::new(RefCell::new(copied_pet)))
            } else {
                None
            }
        })
        .collect_vec()
}

impl Clone for Team {
    fn clone(&self) -> Self {
        let mut copied_team_name = self.name.clone();
        copied_team_name.push_str(COPY_SUFFIX);

        // Because we use reference counted ptrs, default clone impl will just increase strong reference counts.
        // So we need to clone the inner values and reassign owners.
        let copied_friends = copy_rc_pets(&self.friends, Some(copied_team_name.clone()));
        let copied_fainted = copy_rc_pets(&self.fainted, Some(copied_team_name.clone()));
        let copied_sold = copy_rc_pets(&self.sold, Some(copied_team_name.clone()));
        let mut copied_stored_friends = self.stored_friends.clone();
        for friend in copied_stored_friends.iter_mut().flatten() {
            friend.team = Some(copied_team_name.clone())
        }
        // Change pet history to reflect name change.
        let mut updated_history = self.history.clone();
        if CONFIG.general.build_graph {
            updated_history
                .graph
                .update_nodes_with_team_name(&self.name, &copied_team_name);
        }

        // Copy triggers and update them if a pet is affected.
        let mut copied_triggers = self.triggers.clone();
        'trigger_loop: for trigger in copied_triggers.iter_mut() {
            let pet_id = trigger
                .affected_pet
                .as_ref()
                .and_then(|pet| pet.upgrade())
                .and_then(|pet| pet.borrow().id.clone());
            // If found id in trigger is same as pet then set trigger to copied friend.
            if let Some(pet_id) = pet_id.as_ref() {
                for friend in copied_friends.iter().flatten() {
                    if friend.borrow().id.as_ref() == Some(pet_id) {
                        trigger.affected_pet = Some(Rc::downgrade(friend));
                        continue 'trigger_loop;
                    }
                }
            }
        }

        let mut copied_team = Self {
            name: copied_team_name,
            friends: copied_friends,
            fainted: copied_fainted,
            sold: copied_sold,
            max_size: self.max_size,
            triggers: copied_triggers,
            history: updated_history,
            seed: self.seed,
            stored_friends: copied_stored_friends,
            curr_pet: None,
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
            return Err(SAPTestError::InvalidTeamAction {
                subject: "Init Team".to_string(),
                reason: format!(
                    "Pets provided exceed specified max size. {} > {max_size}",
                    pets.len(),
                ),
            });
        };
        let seed = random();
        let name = Team::get_random_name(seed)?;
        // Save a copy as reference.
        let mut pets_copy = pets.to_vec();
        let rc_pets = Team::create_rc_pets(&mut pets_copy, &name);
        let curr_pet = if let Some(Some(first_pet)) = rc_pets.first() {
            Some(Rc::downgrade(first_pet))
        } else {
            None
        };
        // Create reference counted clone passing mut reference to assign ids.
        let mut team = Team {
            name,
            stored_friends: pets_copy,
            friends: rc_pets,
            max_size,
            curr_pet,
            ..Default::default()
        };

        // Update pet count.
        team.history.pet_count = team.all().len();
        // By default shop is closed when team created using new().
        team.shop.state = ShopState::Closed;
        Ok(team)
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
    pub(crate) fn create_rc_pets(
        pets: &mut [Option<Pet>],
        team_name: &str,
    ) -> Vec<Option<Rc<RefCell<Pet>>>> {
        // Index pets.
        let mut rc_pets: Vec<Option<Rc<RefCell<Pet>>>> = vec![];

        for (i, slot) in pets.iter_mut().enumerate() {
            let rc_pet = if let Some(pet) = slot {
                // Create id if one not assigned.
                pet.team = Some(team_name.to_owned());
                pet.id = Some(pet.id.clone().unwrap_or(format!("{}_{}", pet.name, i)));
                pet.set_pos(i);

                let rc_pet = Rc::new(RefCell::new(pet.clone()));

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
    /// team.set_seed(Some(0));
    /// team.trigger_effects(&TRIGGER_START_BATTLE, Some(&mut enemy_team));
    ///
    /// // Mosquitoes always hit first pet with seed set to 0.
    /// assert!(
    ///     enemy_team.friends.first().map_or(false, |pet| pet.as_ref().unwrap().borrow().stats.health == 0),
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

    /// Get the name of the team.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Gets a random [`Team`] name.
    /// * This pulls a random adjective and noun from the `names` table in [`SapDB`](crate::SapDB).
    /// ```
    /// use saptest::Team;
    /// let name = Team::get_random_name(5).unwrap();
    /// assert_eq!(&name, "The Submissive Stickers");
    /// ```
    pub fn get_random_name(seed: u64) -> Result<String, SAPTestError> {
        let conn = SAPDB.pool.get()?;
        let mut rng = ChaCha12Rng::seed_from_u64(seed);

        let mut prefix_stmt = conn.prepare("SELECT word FROM names WHERE word_category = ?")?;
        let mut noun_stmt = conn.prepare("SELECT word FROM names WHERE word_category = ?")?;
        let prefix: Option<String> = prefix_stmt
            .query([WordType::Prefix.to_string()])?
            .mapped(|row| row.get(0))
            .flatten()
            .choose(&mut rng);
        let noun: Option<String> = noun_stmt
            .query([WordType::Noun.to_string()])?
            .mapped(|row| row.get(0))
            .flatten()
            .choose(&mut rng);

        if let (Some(mut prefix), Some(noun)) = (prefix, noun) {
            prefix.insert_str(0, "The ");
            Ok(prefix + " " + &noun)
        } else {
            Err(SAPTestError::QueryFailure {
                subject: "No Name Generated".to_string(),
                reason: "A prefix or noun was missing.".to_string(),
            })
        }
    }

    /// Assign a name for the team and all of its pets.
    /// * By default, team names are constructed randomly.
    /// * Names must not be empty and cannot contain non-alphanumeric characters.
    /// # Example
    /// ```
    /// use saptest::Team;
    /// // This constructs a random team name.
    /// let mut team = Team::default();
    /// let new_name = "The Super Auto Pets";
    /// // New name is set for the team and its contained pets.
    /// assert!(team.set_name(new_name).is_ok());
    /// assert_eq!(
    ///     team.get_name(),
    ///     new_name
    /// );
    /// ```
    pub fn set_name(&mut self, name: &str) -> Result<&mut Self, SAPTestError> {
        if name.trim().is_empty()
            || !name
                .chars()
                .all(|char| char.is_alphanumeric() || char == ' ')
        {
            return Err(SAPTestError::InvalidTeamAction {
                subject: format!("Invalid Name ({name})"),
                reason: "Team must have a non-empty, alphanumeric name.".to_string(),
            });
        };

        for friend in self.stored_friends.iter_mut().flatten() {
            friend.team = Some(name.to_owned())
        }
        for friend in self
            .friends
            .iter()
            .chain(self.fainted.iter().chain(self.sold.iter()))
            .flatten()
        {
            friend.borrow_mut().team = Some(name.to_owned())
        }
        self.name = name.to_owned();

        Ok(self)
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
    ///     &Position::Relative(0),
    ///     Some(Food::try_from(&FoodName::Garlic).unwrap())
    /// ).unwrap();
    ///
    /// let dog = team.first().unwrap();
    /// assert_eq!(dog.borrow().item.as_ref().unwrap().name, FoodName::Garlic);
    /// ```
    pub fn set_item(
        &mut self,
        pos: &Position,
        item: Option<Food>,
    ) -> Result<&mut Self, SAPTestError> {
        // If item, assign/activate item. Otherwise, set to None.
        if let Some(food) = item.as_ref() {
            self.buy_food_behavior(
                Rc::new(RefCell::new(food.clone())),
                self.first(),
                pos,
                false,
            )?;
        } else {
            let affected_pets = self
                .get_pets_by_pos(self.first(), &Target::Friend, pos, None, None)
                .map_err(|_| SAPTestError::InvalidTeamAction {
                    subject: "Item Pet Position".to_string(),
                    reason: format!("Position is not valid: {pos:?}"),
                })?;

            for pet in affected_pets.into_iter() {
                pet.borrow_mut().item = None
            }
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
        let affected_pets = self.get_pets_by_pos(self.first(), &Target::Friend, pos, None, None)?;

        for pet in affected_pets.iter() {
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
        let new_pet_id = format!("{}_{}", pet.name, self.history.pet_count + 1);
        let pet_id = pet.id.clone();
        let rc_pet = Rc::new(RefCell::new(pet));
        let alive_pets = self.all().len();

        if alive_pets == self.max_size {
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
        rc_pet.borrow_mut().team = Some(self.name.clone());

        // Assign effects to new pet.
        for effect in rc_pet.borrow_mut().effect.iter_mut() {
            effect.assign_owner(Some(&rc_pet));
        }
        if let Some(food_item) = rc_pet.borrow_mut().item.as_mut() {
            food_item.ability.assign_owner(Some(&rc_pet));
        }

        // Set summon triggers.
        let weak_ref_pet = Rc::downgrade(&rc_pet);
        let [self_trigger, any_trigger, any_enemy_trigger] = get_summon_triggers(weak_ref_pet);

        if let Some(opponent) = opponent {
            opponent.triggers.push_back(any_enemy_trigger)
        }
        self.triggers.extend([self_trigger, any_trigger]);

        info!(target: "run", "(\"{}\")\nAdded pet to pos {pos}: {}.", self.name.to_string(), rc_pet.borrow());

        // Empty slot. Remove and replace with pet.
        let curr_slot = self.friends.get(pos);
        if curr_slot.map_or(false, |slot| slot.is_none()) {
            self.friends.remove(pos);
        }

        self.friends.insert(pos, Some(rc_pet));
        self.history.pet_count += 1;

        // Set current pet to always be first in line.
        if let Some(Some(pet)) = self.friends.first() {
            self.curr_pet = Some(Rc::downgrade(pet));
        }

        Ok(self)
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
