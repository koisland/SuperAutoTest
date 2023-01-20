use petgraph::{stable_graph::NodeIndex, Directed, Graph};

use crate::common::battle::{
    state::{Action, Outcome, Position, Target},
    trigger::TRIGGER_START_BATTLE,
};

/// Track history of a `Team`'s effects.
#[derive(Debug, Clone)]
pub struct History {
    pub curr_turn: usize,
    pub curr_node: Option<NodeIndex>,
    pub prev_node: Option<NodeIndex>,
    pub effect_graph: Graph<Outcome, (Target, Position, Action, String), Directed>,
}

impl History {
    pub fn new() -> Self {
        let mut history = History {
            curr_turn: 0,
            curr_node: None,
            prev_node: None,
            effect_graph: Graph::new(),
        };
        let starting_node = history.effect_graph.add_node(TRIGGER_START_BATTLE);
        (history.prev_node, history.curr_node) = (Some(starting_node), Some(starting_node));
        history
    }
}
