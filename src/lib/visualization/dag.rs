use crate::{error::SAPTestError, Team};
use graphviz_rust::{
    attributes::{color_name, rankdir, GraphAttributes},
    dot_structures::Stmt,
    parse,
    printer::{DotPrinter, PrinterContext},
};
use petgraph::dot::Dot;

/// Generate [`Team`] history as a directed acrylic graph.
/// * Pets are nodes
/// * Edges are the trigger and the action performed.
pub fn battle_to_dag(team: &Team) -> Result<String, SAPTestError> {
    let dag = format!("{:?}", Dot::new(&team.history.graph.phase_graph));

    let mut ctx = PrinterContext::default();
    let mut dot_digraph = parse(&dag).map_err(|err| SAPTestError::InvalidTeamAction {
        subject: "DAG Parse Failed".to_string(),
        reason: err,
    })?;
    // LR rankdir differentiates team's better.
    dot_digraph.add_stmt(Stmt::Attribute(GraphAttributes::rankdir(rankdir::TB)));
    // Light readable bg color.
    dot_digraph.add_stmt(Stmt::Attribute(GraphAttributes::bgcolor(color_name::azure)));
    // Node separation, to declutter.
    dot_digraph.add_stmt(Stmt::Attribute(GraphAttributes::nodesep(1.0)));
    Ok(dot_digraph.print(&mut ctx))
}

#[cfg(test)]
mod tests {
    use super::battle_to_dag;
    use crate::{tests::common::test_mammoth_team, TeamCombat};

    #[test]
    fn test_multi_phase_dag() {
        let mut team = test_mammoth_team();
        team.set_name("The Super Auto Pets").unwrap();
        let mut enemy_team = team.clone();

        team.fight(&mut enemy_team).unwrap();
        team.fight(&mut enemy_team).unwrap();
        team.fight(&mut enemy_team).unwrap();
        team.fight(&mut enemy_team).unwrap();

        let dag = battle_to_dag(&team).unwrap();

        assert_eq!(
            dag,
            String::from(
                r#"digraph  {
    0[label="PetNode { id: \"Mammoth_0\", team: \"The Super Auto Pets\" }"]
    1[label="PetNode { id: \"Dog_1\", team: \"The Super Auto Pets\" }"]
    2[label="PetNode { id: \"Dog_2\", team: \"The Super Auto Pets\" }"]
    3[label="PetNode { id: \"Dog_3\", team: \"The Super Auto Pets\" }"]
    4[label="PetNode { id: \"Dog_4\", team: \"The Super Auto Pets\" }"]
    5[label="PetNode { id: \"Mammoth_0\", team: \"The Super Auto Pets_copy\" }"]
    6[label="PetNode { id: \"Dog_1\", team: \"The Super Auto Pets_copy\" }"]
    7[label="PetNode { id: \"Dog_2\", team: \"The Super Auto Pets_copy\" }"]
    8[label="PetNode { id: \"Dog_3\", team: \"The Super Auto Pets_copy\" }"]
    9[label="PetNode { id: \"Dog_4\", team: \"The Super Auto Pets_copy\" }"]
    5 -> 0 [label="(Attack, (1, 1), Remove(StaticValue(Statistics { attack: 0, health: 3 })))"]
    0 -> 5 [label="(Attack, (1, 1), Remove(StaticValue(Statistics { attack: 0, health: 3 })))"]
    5 -> 0 [label="(Attack, (2, 1), Remove(StaticValue(Statistics { attack: 0, health: 3 })))"]
    0 -> 5 [label="(Attack, (2, 1), Remove(StaticValue(Statistics { attack: 0, health: 3 })))"]
    5 -> 0 [label="(Attack, (3, 1), Remove(StaticValue(Statistics { attack: 0, health: 3 })))"]
    0 -> 5 [label="(Attack, (3, 1), Remove(StaticValue(Statistics { attack: 0, health: 3 })))"]
    5 -> 0 [label="(Attack, (4, 1), Remove(StaticValue(Statistics { attack: 0, health: 1 })))"]
    0 -> 5 [label="(Attack, (4, 1), Remove(StaticValue(Statistics { attack: 0, health: 1 })))"]
    0 -> 1 [label="(Faint, (4, 4), Add(StaticValue(Statistics { attack: 2, health: 2 })))"]
    0 -> 2 [label="(Faint, (4, 4), Add(StaticValue(Statistics { attack: 2, health: 2 })))"]
    0 -> 3 [label="(Faint, (4, 4), Add(StaticValue(Statistics { attack: 2, health: 2 })))"]
    0 -> 4 [label="(Faint, (4, 4), Add(StaticValue(Statistics { attack: 2, health: 2 })))"]
    5 -> 6 [label="(Faint, (4, 12), Add(StaticValue(Statistics { attack: 2, health: 2 })))"]
    5 -> 7 [label="(Faint, (4, 12), Add(StaticValue(Statistics { attack: 2, health: 2 })))"]
    5 -> 8 [label="(Faint, (4, 12), Add(StaticValue(Statistics { attack: 2, health: 2 })))"]
    5 -> 9 [label="(Faint, (4, 12), Add(StaticValue(Statistics { attack: 2, health: 2 })))"]
    rankdir=TB
    bgcolor=azure
    nodesep=1
}"#
            )
        )
    }
}
