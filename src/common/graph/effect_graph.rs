use petgraph::{stable_graph::NodeIndex, Directed, Graph};

use crate::common::{
    battle::state::{Action, Outcome, Position, Target},
    pets::pet::Pet,
};

#[derive(Debug, Clone)]
pub struct History {
    pub curr_turn: usize,
    pub dead: Vec<Option<Pet>>,
    pub curr_node: Option<NodeIndex>,
    pub prev_node: Option<NodeIndex>,
    pub effect_graph: Graph<Outcome, (Target, Position, Action, String), Directed>,
}
