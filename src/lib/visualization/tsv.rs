use crate::Team;

const DELIMITER: &str = "\t";
const TSV_HEADER: [&str; 11] = [
    "n_actions",
    "afflicting_id",
    "afflicting_team",
    "afflicting_stats",
    "affected_id",
    "affected_team",
    "affected_stats",
    "trigger",
    "action",
    "n_phases",
    "n_cycles",
];

/// Creates a dataframe string representing each step in a battle as a tab-separated row.
/// * Tabs (`\t`) are only supported as the formatted [`Action`](crate::effects::actions::Action)s contain commas.
///
/// # Example
/// ```
/// use saptest::{
///     Pet, PetName, Team, TeamCombat, create_battle_df
/// };
/// let mut team = Team::new(
///     &vec![Some(Pet::try_from(PetName::Ant).unwrap()); 5], 5
/// ).unwrap();
/// team.set_name("Ants").unwrap();
/// let mut enemy_team = team.clone();
///
/// team.set_seed(Some(25));
/// enemy_team.set_seed(Some(27));
///
/// // For a single battle phase.
/// team.fight(&mut enemy_team).unwrap();
///
/// let tsv = create_battle_df(&team);
/// let exp_tsv = "n_actions\tafflicting_id\tafflicting_team\tafflicting_stats\taffected_id\taffected_team\taffected_stats\ttrigger\taction\tn_phases\tn_cycles
/// 0\tAnt_0\tAnts_copy\t(2, 0)\tAnt_0\tAnts\t(2, 0)\tAttack\tDamage (0, 2)\t1\t15
/// 1\tAnt_0\tAnts\t(2, 0)\tAnt_0\tAnts_copy\t(2, 0)\tAttack\tDamage (0, 2)\t1\t15
/// 2\tAnt_0\tAnts\t(2, 0)\tAnt_3\tAnts\t(3, 3)\tFaint\tAdd (1, 1)\t1\t18
/// 3\tAnt_0\tAnts_copy\t(2, 0)\tAnt_2\tAnts_copy\t(3, 3)\tFaint\tAdd (1, 1)\t1\t27
/// ";
/// assert_eq!(tsv,  exp_tsv);
/// ```
///
/// # Fields:
/// 1. Number of actions performed.
///     * [`usize`]
/// 2. Afflicting pet's id.
///     * [`String`]
/// 3. Afflicting pet's team name.
///     * [`String`]
/// 4. Afflicting pet's stats after the action.
///     * [`String`]
/// 5. Affected pet's id.
///     * [`String`]
/// 6. Affected pet's team name.
///     * [`String`]
/// 7. Affected pet's stats after the action.
///     * [`String`]
/// 8. Action trigger [`Status`](crate::effects::state::Status).
///     * [`String`]
/// 9. [`Action`](crate::effects::actions::Action) taken.
///     * [`String`]
/// 10. Number of battle phases.
///     * [`usize`]
/// 11. Number of cycles.
///     * [`usize`]
///     * This represents the number of trigger [`Outcome`](crate::effects::state::Outcome)s iterated through.
pub fn create_battle_df(team: &Team) -> String {
    let graph = &team.history.graph.phase_graph;

    let mut tsv = String::new();
    // Add header.
    let header = TSV_HEADER.join(DELIMITER);
    tsv.push_str(&header);
    tsv.push('\n');

    // Iterate through the graph's edges.
    for (i, edge_idx) in graph.edge_indices().enumerate() {
        if let Some((status, action, (phase, cycle), affected_stats, afflicting_stats)) =
            graph.edge_weight(edge_idx)
        {
            // Find connected nodes
            if let Some((Some(afflicting_node), Some(affected_node))) =
                graph.edge_endpoints(edge_idx).map(|node_epts| {
                    (
                        graph.node_weight(node_epts.0),
                        graph.node_weight(node_epts.1),
                    )
                })
            {
                // Split id and team by tab.
                let afflicting_id_and_team = afflicting_node.to_string().replace(" - ", DELIMITER);
                let affected_id_and_team = affected_node.to_string().replace(" - ", DELIMITER);

                let mut line = [
                    i.to_string(),
                    afflicting_id_and_team,
                    afflicting_stats.to_string(),
                    affected_id_and_team,
                    affected_stats.to_string(),
                    status.to_string(),
                    action.to_string(),
                    phase.to_string(),
                    cycle.to_string(),
                ]
                .join(DELIMITER);

                line.push('\n');
                tsv.push_str(&line)
            }
        }
    }
    tsv
}

#[cfg(test)]
mod test {
    use super::create_battle_df;
    use crate::{Pet, PetName, Team, TeamCombat};

    #[test]
    fn test_create_tsv() {
        let mut team = Team::new(&vec![Some(Pet::try_from(PetName::Ant).unwrap()); 5], 5).unwrap();
        team.set_name("Ants").unwrap();

        let mut enemy_team = team.clone();

        team.set_seed(Some(25));
        enemy_team.set_seed(Some(27));

        // For a single battle phase.
        team.fight(&mut enemy_team).unwrap();

        let tsv = create_battle_df(&team);
        let exp_tsv = "n_actions\tafflicting_id\tafflicting_team\tafflicting_stats\taffected_id\taffected_team\taffected_stats\ttrigger\taction\tn_phases\tn_cycles
0\tAnt_0\tAnts_copy\t(2, 0)\tAnt_0\tAnts\t(2, 0)\tAttack\tDamage (0, 2)\t1\t15
1\tAnt_0\tAnts\t(2, 0)\tAnt_0\tAnts_copy\t(2, 0)\tAttack\tDamage (0, 2)\t1\t15
2\tAnt_0\tAnts\t(2, 0)\tAnt_3\tAnts\t(3, 3)\tFaint\tAdd (1, 1)\t1\t18
3\tAnt_0\tAnts_copy\t(2, 0)\tAnt_2\tAnts_copy\t(3, 3)\tFaint\tAdd (1, 1)\t1\t27
";
        assert_eq!(tsv, exp_tsv);
    }
}
