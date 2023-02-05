use petgraph::{stable_graph::NodeIndex, Directed, Graph};

use crate::battle::{
    state::{Action, Outcome, Position, Target},
    trigger::TRIGGER_START_BATTLE,
};

/// Track history of a `Team`'s effects.
#[derive(Debug, Clone, Default)]
pub struct History {
    pub curr_phase: usize,
    pub curr_turn: usize,
    pub curr_node: Option<NodeIndex>,
    pub prev_node: Option<NodeIndex>,
    pub effect_graph: Graph<Outcome, (Target, Position, Action, String), Directed>,
}

impl History {
    pub fn new() -> Self {
        let mut history = History::default();
        let starting_node = history.effect_graph.add_node(TRIGGER_START_BATTLE);
        (history.prev_node, history.curr_node) = (Some(starting_node), Some(starting_node));
        history
    }
}