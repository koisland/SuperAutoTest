use std::{collections::HashMap, fmt::Display};

use petgraph::{stable_graph::NodeIndex, Directed, Graph};

use crate::{
    effects::{
        actions::Action,
        state::{Status, Target},
    },
    teams::{team::TeamFightOutcome, viewer::TargetPet},
    Effect, Team,
};

pub type PetEffectGraph = Graph<PetNode, (Status, Action), Directed>;

/// Track history of a `Team`'s effects.
#[derive(Debug, Clone)]
pub struct History {
    pub curr_phase: usize,
    pub curr_turn: usize,
    pub pet_nodes: HashMap<PetNode, NodeIndex>,
    pub fight_outcomes: Vec<TeamFightOutcome>,
    pub phase_graph: PetEffectGraph,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PetNode {
    pub id: String,
    pub phase: usize,
    pub team: Target,
}

impl Display for PetNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?}) (Phase: {})", self.id, self.team, self.phase)
    }
}

pub(crate) trait TeamHistory {
    fn update_pet_nodes(&mut self, opponent: &mut Team);
    fn add_effect_edge(&mut self, affected: &TargetPet, afflicting: &TargetPet, effect: &Effect);
}

impl TeamHistory for Team {
    fn update_pet_nodes(&mut self, opponent: &mut Team) {
        for friend in self.friends.iter().flatten() {
            if let Some(id) = &friend.borrow().id {
                let mut node = PetNode {
                    id: id.clone(),
                    phase: self.history.curr_phase,
                    team: Target::Friend,
                };
                let node_idx = self.history.phase_graph.add_node(node.clone());
                self.history.pet_nodes.insert(node.clone(), node_idx);

                node.team = Target::Enemy;
                let enemy_node_idx = opponent.history.phase_graph.add_node(node.clone());
                opponent.history.pet_nodes.insert(node, enemy_node_idx);
            }
        }
    }

    fn add_effect_edge(&mut self, affected: &TargetPet, afflicting: &TargetPet, effect: &Effect) {
        let (affected_team, affected_pet) = affected;
        let (afflicting_team, afflicting_pet) = afflicting;

        if let Some(id) = &affected_pet.borrow().id {
            let node = PetNode {
                id: id.clone(),
                phase: self.history.curr_phase,
                team: *affected_team,
            };

            let affected_node_idx = *self
                .history
                .pet_nodes
                .entry(node)
                .or_insert_with_key(|node| self.history.phase_graph.add_node(node.clone()));

            if let Some(afflicting_pet_id) = &afflicting_pet.borrow().id {
                let other_node = PetNode {
                    id: afflicting_pet_id.clone(),
                    phase: self.history.curr_phase,
                    team: *afflicting_team,
                };
                let afflicting_node_idx = self.history.pet_nodes.get(&other_node).cloned();
                if let Some(afflicting_node_idx) = afflicting_node_idx {
                    self.history.phase_graph.add_edge(
                        afflicting_node_idx,
                        affected_node_idx,
                        (effect.trigger.status.clone(), effect.action.clone()),
                    );
                }
            }
        };
    }
}

impl Default for History {
    fn default() -> Self {
        Self {
            curr_phase: 1,
            curr_turn: 1,
            pet_nodes: Default::default(),
            fight_outcomes: Default::default(),
            phase_graph: Default::default(),
        }
    }
}
