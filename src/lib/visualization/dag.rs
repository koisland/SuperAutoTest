use crate::Team;
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
pub fn to_dag(team: &Team) -> String {
    // Petgraph has dag.
    let dag = format!("{:?}", Dot::new(&team.history.phase_graph));

    // Use graphviz_rust to format.
    let mut ctx = PrinterContext::default();
    let mut dag_structure = parse(&dag).unwrap();
    dag_structure.add_stmt(Stmt::Attribute(GraphAttributes::rankdir(rankdir::LR)));
    dag_structure.add_stmt(Stmt::Attribute(GraphAttributes::bgcolor(color_name::azure)));

    dag_structure.print(&mut ctx)
}
