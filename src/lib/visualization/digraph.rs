use crate::{effects::state::Status, Team};
use petgraph::{dot::Dot, Graph};
use std::collections::HashMap;

type SimpleBattleGraph = Graph<String, (Status, String, String)>;

const DOT_PARAMS: &str = r#"
    rankdir=LR
    node [shape=box, style="rounded, filled", fontname="Arial"]
    edge [fontname="Arial"]"#;

/// Generate [`Team`]'s battle history as a [directed graph](https://en.wikipedia.org/wiki/Directed_graph).
/// * Structure:
///     * Pets and teams are nodes
///     * Triggers, battle phases + triggers consumed, and the action performed are edges.
/// * Use the `verbose` argument to print out the entire unformatted [`Action`](crate::effects::actions::Action) enum.
///
/// # Example
/// ```
/// use saptest::{
///     Pet, PetName, Team, TeamCombat, create_battle_digraph
/// };
/// let mut team = Team::new(
///     &vec![Some(Pet::try_from(PetName::Ant).unwrap()); 5], 5
/// ).unwrap();
/// team.set_name("Ants").unwrap();
/// let mut enemy_team = team.clone();
///
/// team.set_seed(Some(25));
/// enemy_team.set_seed(Some(25));
///
/// team.fight(&mut enemy_team).unwrap();
/// let digraph = create_battle_digraph(&team, false);
/// let exp_digraph = r#"digraph {
///     rankdir=LR
///     node [shape=box, style="rounded, filled", fontname="Arial"]
///     edge [fontname="Arial"]
///     0 [ label = "Ant_0 - Ants_copy" ]
///     1 [ label = "Ant_0 - Ants", fillcolor = "yellow" ]
///     2 [ label = "Ant_3 - Ants", fillcolor = "yellow" ]
///     3 [ label = "Ant_3 - Ants_copy" ]
///     0 -> 1 [ label = "(Attack, Damage (0, 1), Phase: 1)" ]
///     1 -> 0 [ label = "(Attack, Damage (0, 1), Phase: 1)" ]
///     1 -> 2 [ label = "(Faint, Add (2, 1), Phase: 1)" ]
///     0 -> 3 [ label = "(Faint, Add (2, 1), Phase: 1)" ]
///}
/// "#;
/// assert_eq!(digraph, exp_digraph);
/// ```
pub fn create_battle_digraph(team: &Team, verbose: bool) -> String {
    let mut raw_digraph = if verbose {
        format!("{:?}", Dot::new(&team.history.graph.phase_graph))
    } else {
        simple_digraph(team)
    };

    // Find start of graph and insert dot params.
    if let Some(digraph_start_idx) = raw_digraph.find(|chr| chr == '{') {
        raw_digraph.insert_str(digraph_start_idx + 1, DOT_PARAMS);
    };

    // Add fillcolor to distinguish teams.
    let team_name = &team.name;
    let (from, to) = if verbose {
        // Verbose has struct field end character '}'
        (
            format!("{team_name} }}\""),
            format!("{team_name} }}\", fillcolor = \"yellow\""),
        )
    } else {
        (
            format!("{team_name}\""),
            format!("{team_name}\", fillcolor = \"yellow\""),
        )
    };

    // Remove \\\ from debug printing.
    raw_digraph = raw_digraph.replace("\\\"", "").replace(&from, &to);
    raw_digraph
}

/// Create a simplified digraph diagram from a PetGraph DiGraph data structure.
/// * Normal conversion to string uses the Debug representation of the PetNode and Action structs.
/// * This makes the graph difficult to read and cluttered.
/// * Here, we reconstruct the graph where these structs are replaced by formatted strings.
fn simple_digraph(team: &Team) -> String {
    let graph = &team.history.graph.phase_graph;
    let mut new_graph = SimpleBattleGraph::new();
    let mut new_string_nodes = HashMap::new();

    // Iterate through edges
    for edge_idx in graph.edge_indices() {
        if let Some(edge_weight) = graph.edge_weight(edge_idx) {
            // Find connected nodes
            if let Some((node_1, node_2)) = graph.edge_endpoints(edge_idx) {
                if let (Some(pet_node_1), Some(pet_node_2)) =
                    (graph.node_weight(node_1), graph.node_weight(node_2))
                {
                    // Convert pet_node to neater string.
                    let node_1_str = pet_node_1.to_string();
                    let node_2_str = pet_node_2.to_string();

                    // Only add node to new graph if already exists.
                    if !new_string_nodes.contains_key(&node_1_str) {
                        let idx = new_graph.add_node(node_1_str.clone());
                        new_string_nodes.insert(node_1_str.clone(), idx);
                    }
                    if !new_string_nodes.contains_key(&node_2_str) {
                        let idx = new_graph.add_node(node_2_str.clone());
                        new_string_nodes.insert(node_2_str.clone(), idx);
                    }
                    // Add edge converting actions to neater string.
                    if let (Some(idx_1), Some(idx_2)) = (
                        new_string_nodes.get(&node_1_str),
                        new_string_nodes.get(&node_2_str),
                    ) {
                        let (status, action, phase_cycle, _, _) = edge_weight;
                        new_graph.add_edge(
                            *idx_1,
                            *idx_2,
                            (
                                status.clone(),
                                action.to_string(),
                                format!("Phase: {}", phase_cycle.0),
                            ),
                        );
                    }
                }
            }
        }
    }
    format!("{:?}", Dot::new(&new_graph))
}

#[cfg(test)]
mod tests {
    use super::create_battle_digraph;
    use crate::{tests::common::test_mammoth_team, TeamCombat};

    #[test]
    fn test_simple_digraph() {
        let mut team = test_mammoth_team();
        team.set_name("The Super Auto Pets").unwrap();
        let mut enemy_team = team.clone();

        team.fight(&mut enemy_team).unwrap();
        team.fight(&mut enemy_team).unwrap();
        team.fight(&mut enemy_team).unwrap();
        team.fight(&mut enemy_team).unwrap();

        let simple_digraph = create_battle_digraph(&team, false);
        let exp_digraph = r#"digraph {
    rankdir=LR
    node [shape=box, style="rounded, filled", fontname="Arial"]
    edge [fontname="Arial"]
    0 [ label = "Mammoth_0 - The Super Auto Pets_copy" ]
    1 [ label = "Mammoth_0 - The Super Auto Pets", fillcolor = "yellow" ]
    2 [ label = "Dog_1 - The Super Auto Pets", fillcolor = "yellow" ]
    3 [ label = "Dog_2 - The Super Auto Pets", fillcolor = "yellow" ]
    4 [ label = "Dog_3 - The Super Auto Pets", fillcolor = "yellow" ]
    5 [ label = "Dog_4 - The Super Auto Pets", fillcolor = "yellow" ]
    6 [ label = "Dog_1 - The Super Auto Pets_copy" ]
    7 [ label = "Dog_2 - The Super Auto Pets_copy" ]
    8 [ label = "Dog_3 - The Super Auto Pets_copy" ]
    9 [ label = "Dog_4 - The Super Auto Pets_copy" ]
    0 -> 1 [ label = "(Attack, Damage (0, 3), Phase: 1)" ]
    1 -> 0 [ label = "(Attack, Damage (0, 3), Phase: 1)" ]
    0 -> 1 [ label = "(Attack, Damage (0, 3), Phase: 2)" ]
    1 -> 0 [ label = "(Attack, Damage (0, 3), Phase: 2)" ]
    0 -> 1 [ label = "(Attack, Damage (0, 3), Phase: 3)" ]
    1 -> 0 [ label = "(Attack, Damage (0, 3), Phase: 3)" ]
    0 -> 1 [ label = "(Attack, Damage (0, 1), Phase: 4)" ]
    1 -> 0 [ label = "(Attack, Damage (0, 1), Phase: 4)" ]
    1 -> 2 [ label = "(Faint, Add (2, 2), Phase: 4)" ]
    1 -> 3 [ label = "(Faint, Add (2, 2), Phase: 4)" ]
    1 -> 4 [ label = "(Faint, Add (2, 2), Phase: 4)" ]
    1 -> 5 [ label = "(Faint, Add (2, 2), Phase: 4)" ]
    0 -> 6 [ label = "(Faint, Add (2, 2), Phase: 4)" ]
    0 -> 7 [ label = "(Faint, Add (2, 2), Phase: 4)" ]
    0 -> 8 [ label = "(Faint, Add (2, 2), Phase: 4)" ]
    0 -> 9 [ label = "(Faint, Add (2, 2), Phase: 4)" ]
}
"#;
        assert_eq!(exp_digraph, format!("{simple_digraph}"))
    }
}

#[test]
fn test_verbose_digraph() {
    use crate::{create_battle_digraph, Pet, PetName, Team, TeamCombat};
    let mut team = Team::new(&vec![Some(Pet::try_from(PetName::Ant).unwrap()); 5], 5).unwrap();
    team.set_name("Ants").unwrap();

    let mut enemy_team = team.clone();

    team.set_seed(Some(25));
    enemy_team.set_seed(Some(25));

    team.fight(&mut enemy_team).unwrap();
    let digraph = create_battle_digraph(&team, true);
    let exp_digraph = r#"digraph {
    rankdir=LR
    node [shape=box, style="rounded, filled", fontname="Arial"]
    edge [fontname="Arial"]
    0 [ label = "PetNode { id: Ant_0, team: Ants }", fillcolor = "yellow" ]
    1 [ label = "PetNode { id: Ant_1, team: Ants }", fillcolor = "yellow" ]
    2 [ label = "PetNode { id: Ant_2, team: Ants }", fillcolor = "yellow" ]
    3 [ label = "PetNode { id: Ant_3, team: Ants }", fillcolor = "yellow" ]
    4 [ label = "PetNode { id: Ant_4, team: Ants }", fillcolor = "yellow" ]
    5 [ label = "PetNode { id: Ant_0, team: Ants_copy }" ]
    6 [ label = "PetNode { id: Ant_1, team: Ants_copy }" ]
    7 [ label = "PetNode { id: Ant_2, team: Ants_copy }" ]
    8 [ label = "PetNode { id: Ant_3, team: Ants_copy }" ]
    9 [ label = "PetNode { id: Ant_4, team: Ants_copy }" ]
    5 -> 0 [ label = "(Attack, Remove(Static(Statistics { attack: 0, health: 1 })), (1, 15), Statistics { attack: 2, health: 0 }, Statistics { attack: 2, health: 0 })" ]
    0 -> 5 [ label = "(Attack, Remove(Static(Statistics { attack: 0, health: 1 })), (1, 15), Statistics { attack: 2, health: 0 }, Statistics { attack: 2, health: 0 })" ]
    0 -> 3 [ label = "(Faint, Add(Static(Statistics { attack: 2, health: 1 })), (1, 18), Statistics { attack: 4, health: 2 }, Statistics { attack: 2, health: 0 })" ]
    5 -> 8 [ label = "(Faint, Add(Static(Statistics { attack: 2, health: 1 })), (1, 26), Statistics { attack: 4, health: 2 }, Statistics { attack: 2, health: 0 })" ]
}
"#;
    assert_eq!(digraph, exp_digraph)
}
