use petgraph::{stable_graph::NodeIndex, Directed, Graph};
use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    effects::{
        actions::{Action, StatChangeType},
        state::Status,
    },
    error::SAPTestError,
    pets::combat::AttackOutcome,
    teams::team::TeamFightOutcome,
    Pet, Statistics, Team,
};

pub type PhaseCycle = (usize, usize);
pub type PetEffectGraph =
    Graph<PetNode, (Status, Action, PhaseCycle, Statistics, Statistics), Directed>;

/// Track history of a `Team`'s effects.
#[derive(Debug, Clone)]
pub struct History {
    // Whether or not the history of the team is the primary one.
    pub primary_team: bool,
    pub curr_phase: usize,
    pub curr_cycle: usize,
    pub curr_turn: usize,
    pub pet_count: usize,
    pub fight_outcomes: Vec<TeamFightOutcome>,
    pub graph: BattleGraph,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PetNode {
    pub id: String,
    pub team: String,
}

impl Default for History {
    fn default() -> Self {
        Self {
            primary_team: true,
            curr_phase: 1,
            curr_turn: 1,
            curr_cycle: 1,
            pet_count: 0,
            fight_outcomes: Default::default(),
            graph: BattleGraph::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BattleGraph {
    pub pet_nodes: HashMap<PetNode, NodeIndex>,
    pub phase_graph: PetEffectGraph,
}

impl BattleGraph {
    pub(crate) fn update(
        &mut self,
        friends: &[Option<Rc<RefCell<Pet>>>],
        enemies: &[Option<Rc<RefCell<Pet>>>],
    ) {
        for pet in friends.iter().chain(enemies.iter()).flatten() {
            if let (Some(id), Some(team)) = (&pet.borrow().id, &pet.borrow().team) {
                let node = PetNode {
                    id: id.to_owned(),
                    team: team.to_owned(),
                };
                let node_idx = self.phase_graph.add_node(node.clone());
                self.pet_nodes.insert(node, node_idx);
            }
        }
    }

    pub(crate) fn update_nodes_with_team_name(&mut self, original_name: &str, new_name: &str) {
        // Update nodes in graph if any.
        let mut updated_nodes = vec![];
        for idx in self.phase_graph.node_indices() {
            if let Some(node) = self.phase_graph.node_weight_mut(idx) {
                if node.team == original_name {
                    node.team = new_name.to_owned();
                }
                updated_nodes.push((node.clone(), idx));
            }
        }
        // Replace old pet_nodes with ones with updated names.
        self.pet_nodes = updated_nodes.into_iter().collect();
    }
}

pub(crate) trait TeamHistoryHelpers {
    fn add_hurt_and_attack_edges(
        &mut self,
        affected_pet: &Rc<RefCell<Pet>>,
        afflicting_pet: &Rc<RefCell<Pet>>,
        atk_outcome: &AttackOutcome,
    ) -> Result<(), SAPTestError>;

    fn add_action_edge(
        &mut self,
        affected: &Rc<RefCell<Pet>>,
        afflicting: &Rc<RefCell<Pet>>,
        status: &Status,
        action: &Action,
    ) -> Result<(), SAPTestError>;
}
impl TeamHistoryHelpers for Team {
    fn add_hurt_and_attack_edges(
        &mut self,
        affected_pet: &Rc<RefCell<Pet>>,
        afflicting_pet: &Rc<RefCell<Pet>>,
        atk_outcome: &AttackOutcome,
    ) -> Result<(), SAPTestError> {
        let mut outcomes = if Some(&self.name) == affected_pet.borrow().team.as_ref() {
            atk_outcome.friends.iter()
        } else {
            atk_outcome.opponents.iter()
        };
        if let Some(trigger) = outcomes
            .find(|trigger| trigger.status == Status::Hurt || trigger.status == Status::Attack)
        {
            self.add_action_edge(
                affected_pet,
                afflicting_pet,
                &trigger.status,
                &Action::Remove(StatChangeType::StaticValue(atk_outcome.friend_stat_change)),
            )?;
            self.add_action_edge(
                afflicting_pet,
                affected_pet,
                &trigger.status,
                &Action::Remove(StatChangeType::StaticValue(atk_outcome.enemy_stat_change)),
            )?;
        }

        Ok(())
    }
    fn add_action_edge(
        &mut self,
        affected: &Rc<RefCell<Pet>>,
        afflicting: &Rc<RefCell<Pet>>,
        status: &Status,
        action: &Action,
    ) -> Result<(), SAPTestError> {
        if let (Some(affected_team), Some(afflicting_team)) = (
            affected.borrow().team.as_ref(),
            afflicting.borrow().team.as_ref(),
        ) {
            if let Some(id) = &affected.borrow().id {
                let node = PetNode {
                    id: id.clone(),
                    team: affected_team.to_owned(),
                };

                let affected_node_idx = *self
                    .history
                    .graph
                    .pet_nodes
                    .entry(node)
                    .or_insert_with_key(|node| {
                        self.history.graph.phase_graph.add_node(node.clone())
                    });

                if let Some(afflicting_pet_id) = &afflicting.borrow().id {
                    let other_node = PetNode {
                        id: afflicting_pet_id.clone(),
                        team: afflicting_team.to_owned(),
                    };
                    let afflicting_node_idx =
                        self.history.graph.pet_nodes.get(&other_node).cloned();
                    if let Some(afflicting_node_idx) = afflicting_node_idx {
                        self.history.graph.phase_graph.add_edge(
                            afflicting_node_idx,
                            affected_node_idx,
                            (
                                status.clone(),
                                action.clone(),
                                (self.history.curr_phase, self.history.curr_cycle),
                                affected.borrow().stats,
                                afflicting.borrow().stats,
                            ),
                        );
                    }
                }
            };
        }
        Ok(())
    }
}

impl Display for PetNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.id, self.team)
    }
}
