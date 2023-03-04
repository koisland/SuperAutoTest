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
    let mut dag_structure = parse(&dag).unwrap();
    // LR rankdir differentiates team's better.
    dag_structure.add_stmt(Stmt::Attribute(GraphAttributes::rankdir(rankdir::TB)));
    // Light readable bg color.
    dag_structure.add_stmt(Stmt::Attribute(GraphAttributes::bgcolor(color_name::azure)));
    // Node separation, to declutter.
    dag_structure.add_stmt(Stmt::Attribute(GraphAttributes::nodesep(1.0)));
    Ok(dag_structure.print(&mut ctx))
}
