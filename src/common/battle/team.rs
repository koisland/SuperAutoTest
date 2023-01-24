use crate::common::{
    battle::{
        effect::Effect,
        state::{Condition, Outcome, Position, Status, Target, TeamFightOutcome},
        team_effect_apply::EffectApply,
        trigger::*,
    },
    error::TeamError,
    graph::effect_graph::History,
    pets::{combat::Combat, pet::Pet},
};

use itertools::Itertools;
use log::info;
use rand::{random, seq::IteratorRandom, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::{collections::VecDeque, error::Error, fmt::Display};

/// A Super Auto Pets team.
#[derive(Debug, Clone)]
pub struct Team {
    pub name: String,
    pub friends: Vec<Option<Pet>>,
    pub fainted: Vec<Option<Pet>>,
    pub max_size: usize,
    pub triggers: VecDeque<Outcome>,
    pub history: History,
    pub seed: u64,
    stored_friends: Vec<Option<Pet>>,
    pet_count: usize,
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
            seed: Default::default(),
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
    /// Create a new team of a given size.
    pub fn new(name: &str, pets: &[Option<Pet>], max_size: usize) -> Result<Team, TeamError> {
        if pets.len() > max_size {
            Err(TeamError {
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
                        pet_count += 1;
                    }
                    slot
                })
                .collect_vec();

            Ok(Team {
                name: name.to_string(),
                stored_friends: idx_pets.clone(),
                friends: idx_pets,
                max_size,
                triggers: VecDeque::from_iter(ALL_TRIGGERS_START_BATTLE),
                pet_count,
                seed: random(),
                ..Default::default()
            })
        }
    }

    #[allow(dead_code)]
    /// Restore the original `Team`.
    ///
    /// ```
    /// use sapdb::common::battle::team::Team;
    ///
    /// let team = Team::default();
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

    /// Clear `Team` of empty slots and/or fainted `Pet`s.
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

    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
        for pet in self.friends.iter_mut().flatten() {
            pet.seed = seed
        }
    }

    #[allow(dead_code)]
    pub fn get_effects(&self) -> Vec<(usize, Vec<Effect>)> {
        let mut effects: Vec<(usize, Vec<Effect>)> = vec![];
        for (i, friend) in self
            .friends
            .iter()
            .filter_map(|pet| pet.as_ref())
            .enumerate()
        {
            effects.push((i, friend.effect.clone()))
        }

        effects
    }

    /// Get `Pet`s by a given `Condition`.
    pub fn get_pets_by_cond(&mut self, cond: &Condition) -> Vec<&mut Pet> {
        let pets = self.get_all_pets().into_iter();
        let mut found_pets: Vec<&mut Pet> = vec![];

        match cond {
            Condition::Healthiest => {
                if let Some(pet) =
                    pets.max_by(|pet_1, pet_2| pet_1.stats.health.cmp(&pet_2.stats.health))
                {
                    found_pets.push(pet)
                }
            }
            Condition::Illest => {
                if let Some(pet) =
                    pets.min_by(|pet_1, pet_2| pet_1.stats.health.cmp(&pet_2.stats.health))
                {
                    found_pets.push(pet)
                }
            }
            Condition::Strongest => {
                if let Some(pet) =
                    pets.max_by(|pet_1, pet_2| pet_1.stats.attack.cmp(&pet_2.stats.attack))
                {
                    found_pets.push(pet)
                }
            }
            Condition::Weakest => {
                if let Some(pet) =
                    pets.min_by(|pet_1, pet_2| pet_1.stats.attack.cmp(&pet_2.stats.attack))
                {
                    found_pets.push(pet)
                }
            }
            Condition::HasFood(item_name) => {
                for pet in
                    pets.filter(|pet| pet.item.as_ref().map(|item| item.name) == Some(*item_name))
                {
                    found_pets.push(pet);
                }
            }
            Condition::TriggeredBy(trigger) => {
                for pet in pets.filter(|pet| {
                    pet.effect
                        .iter()
                        .any(|effect| &effect.trigger.status == trigger)
                }) {
                    found_pets.push(pet)
                }
            }
            // Allow all if condition is None.
            Condition::None => found_pets.extend(pets),
        };
        found_pets
    }

    /// Swap a `Pet`'s position with another on the `Team`.
    ///
    /// ```
    ///
    /// ```
    #[allow(dead_code)]
    pub fn swap_pets(&mut self, pos_1: usize, pos_2: usize) -> Result<&mut Self, TeamError> {
        if pos_1 > self.friends.len() || pos_2 > self.friends.len() {
            Err(TeamError {
                reason: format!("One or more positions (1: {pos_1}) (2: {pos_2}) is out of bounds"),
            })
        } else {
            self.friends.swap(pos_1, pos_2);
            // Clear team to reassign indices.
            self.clear_team();
            Ok(self)
        }
    }

    pub fn swap_pet_stats(
        &mut self,
        mut pos_1: usize,
        mut pos_2: usize,
    ) -> Result<&mut Self, TeamError> {
        // Swap idx so sorted.
        if pos_1 > pos_2 {
            std::mem::swap(&mut pos_1, &mut pos_2)
        }
        if pos_1 > self.friends.len() || pos_2 > self.friends.len() {
            return Err(TeamError {
                reason: format!("{pos_1} or {pos_2} larger than len of friends."),
            });
        }
        // Split and get two mut slices so can access elements at same time.
        let (mut_slice_1, mut_slice_2) = self.friends.split_at_mut(pos_1 + 1);
        let mut_slice_1_len = mut_slice_1.len();

        if let (Some(Some(pet_1)), Some(Some(pet_2))) = (
            mut_slice_1.get_mut(pos_1),
            mut_slice_2.get_mut(pos_2.saturating_sub(mut_slice_1_len)),
        ) {
            std::mem::swap(&mut pet_1.stats, &mut pet_2.stats);
            Ok(self)
        } else {
            Err(TeamError {
                reason: format!("Cannot access pets at {pos_1} and {pos_2} to swap stats."),
            })
        }
    }

    #[allow(dead_code)]
    pub fn push_pet(
        &mut self,
        pos: usize,
        by: isize,
        opponent: Option<&mut Team>,
    ) -> Result<&mut Self, Box<dyn Error>> {
        self.clear_team();

        if pos < self.friends.len() {
            let adj_pos: usize = if by.is_negative() {
                let pos_by: usize = (-by).try_into()?;
                (pos_by + pos).clamp(0, self.friends.len())
            } else {
                pos.saturating_sub(by.try_into()?)
            };
            let pet = self.friends.remove(pos);

            self.friends.insert(adj_pos, pet);

            // Add push trigger.
            let mut push_any_trigger = TRIGGER_ANY_PUSHED;
            push_any_trigger.idx = Some(adj_pos);
            self.triggers.push_back(push_any_trigger);

            // Reset indices.
            self.clear_team();

            // Add opponent triggers if provided.
            if let Some(opponent) = opponent {
                let mut push_trigger = TRIGGER_ANY_ENEMY_PUSHED;
                push_trigger.idx = Some(adj_pos);
                opponent.triggers.push_back(push_trigger)
            }
        } else {
            return Err(Box::new(TeamError {
                reason: format!("Invalid indices for given pos ({})", pos),
            }));
        }

        Ok(self)
    }

    /// Get a `Pet` at the specified index.
    /// * Fainted `Pet`s are ignored.
    pub fn get_idx_pet(&mut self, idx: usize) -> Option<&mut Pet> {
        if let Some(Some(pet)) = self.friends.get_mut(idx) {
            (pet.stats.health != 0).then_some(pet)
        } else {
            None
        }
    }
    /// Get the first `Pet` among friends.
    /// * Fainted `Pet`s are ignored.
    pub fn get_next_pet(&mut self) -> Option<&mut Pet> {
        if let Some(Some(pet)) = self.friends.first_mut() {
            (pet.stats.health != 0).then_some(pet)
        } else {
            None
        }
    }

    /// Get a random available `Pet`.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    #[allow(dead_code)]
    pub fn get_any_pet(&mut self) -> Option<&mut Pet> {
        let mut rng = ChaCha12Rng::seed_from_u64(self.seed);
        self.get_all_pets().into_iter().choose(&mut rng)
    }

    /// Get all available `Pet`s.
    /// * Fainted `Pet`s and/or empty slots are ignored.
    pub fn get_all_pets(&mut self) -> Vec<&mut Pet> {
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

    /// Add a `Pet` to a `Team`.
    /// * `:param pet:`
    ///     * `Pet` to be summoned.
    /// * `:param pos:`
    ///     * Index on `self.friends` to add `Pet` to.
    ///
    /// Raises `TeamError`:
    /// * If `self.friends` at specified size limit of `self.max_size`
    pub fn add_pet(
        &mut self,
        mut pet: Pet,
        pos: usize,
        opponent: Option<&mut Team>,
    ) -> Result<&mut Self, TeamError> {
        if self.get_all_pets().len() == self.max_size {
            let err = Err(TeamError {
                reason: format!(
                    "(\"{}\")\nMaximum number of pets reached. Cannot add {}.",
                    self.name, &pet
                ),
            });
            // Add overflow to dead pets.
            self.fainted.push(Some(pet));
            return err;
        }
        // Assign id to pet if not any.
        pet.id = Some(
            pet.id
                .clone()
                .unwrap_or(format!("{}_{}", pet.name, self.pet_count + 1)),
        );

        self.friends.insert(pos, Some(pet));
        info!(target: "dev", "(\"{}\")\nAdded pet to pos {pos}: {}.", self.name.to_string(), self.friends.get(pos).unwrap().as_ref().unwrap());

        // Set summon triggers.
        let mut self_trigger = TRIGGER_SELF_SUMMON;
        let mut any_trigger = TRIGGER_ANY_SUMMON;
        let mut any_enemy_trigger = TRIGGER_ANY_ENEMY_SUMMON;

        (self_trigger.idx, any_trigger.idx, any_enemy_trigger.idx) =
            (Some(pos), Some(pos), Some(pos));

        // Update old triggers and their positions that store a pet's idx after inserting new pet.
        // TODO: Look into more edge cases that may cause issue when triggers activate simultaneously.
        for trigger in self.triggers.iter_mut() {
            match (&mut trigger.position, &mut trigger.target) {
                (Position::Relative(orig_pos), Target::Friend)
                | (Position::Relative(orig_pos), Target::Enemy) => *orig_pos += 1,
                (Position::Trigger, Target::Friend)
                | (Position::Trigger, Target::Enemy)
                | (Position::OnSelf, Target::Friend)
                | (Position::OnSelf, Target::Enemy) => {
                    if let Some(idx) = trigger.idx.as_mut() {
                        *idx += 1
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

        // Check that both teams have a pet that is alive.
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
        if let (Some(pet), Some(opponent_pet)) = (self.get_next_pet(), opponent.get_next_pet()) {
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
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for friend in self.friends.iter().filter_map(|pet| pet.as_ref()) {
            writeln!(f, "{}", friend)?;
        }
        Ok(())
    }
}
