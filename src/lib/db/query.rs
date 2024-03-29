use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

use crate::{error::SAPTestError, Entity};

/// Query constructor for [`SapDB::execute_query`](crate::SapDB::execute_query).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SAPQuery {
    pub(crate) table: Option<Entity>,
    pub(crate) params: IndexMap<String, Vec<String>>,
}

impl std::fmt::Display for SAPQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Query ({:?}) {:?}",
            self.as_sql().ok(),
            self.flat_params()
        )
    }
}

impl<N, P> FromIterator<(N, Vec<P>)> for SAPQuery
where
    N: ToString,
    P: ToString,
{
    fn from_iter<T: IntoIterator<Item = (N, Vec<P>)>>(iter: T) -> Self {
        let mut query = SAPQuery::builder();
        for (param, value) in iter.into_iter() {
            query = query.set_param(param, value);
        }
        query
    }
}

impl SAPQuery {
    /// Construct a [`SAPDB`](struct@crate::SAPDB) query
    /// ```rust no_run
    /// use saptest::SAPQuery;
    ///
    /// // Construct an empty query.
    /// let mut query = SAPQuery::builder();
    /// ```
    pub fn builder() -> Self {
        SAPQuery {
            table: None,
            params: IndexMap::new(),
        }
    }

    /// Set table in [`SAPDB`](struct@crate::SAPDB) to query. See [here](crate::db) for more details.
    /// ```rust no_run
    /// use saptest::{Entity, SAPQuery};
    ///
    /// // Construct a query set to the "pets" table.
    /// let query = SAPQuery::builder().set_table(Entity::Pet);
    /// ```
    pub fn set_table(mut self, table: Entity) -> Self {
        self.table = Some(table);
        self
    }

    /// Set params in [`SAPDB`](struct@crate::SAPDB) to query. See [here](crate::db) for more details.
    /// * Prefixing the param name with `-` will select all values not in the given params.
    /// ---
    /// Ex. Query [`FoodRecord`](crate::db::record::FoodRecord)s where `name` is [`FoodName::Apple`](crate::FoodName::Apple) or [`FoodName::Coconut`](crate::FoodName::Coconut).
    /// ```rust no_run
    /// use saptest::{Entity, SAPQuery, FoodName};
    ///
    /// // Construct a query set to the "foods" table.
    /// let query = SAPQuery::builder()
    ///     .set_param("name", vec![FoodName::Apple, FoodName::Coconut])
    ///     .set_table(Entity::Food);
    /// ```
    /// ---
    /// Ex. Query [`PetRecord`](crate::db::record::PetRecord)s where name is **not** [`PetName::Ant`](crate::PetName::Ant) and [`PetName::Dog`](crate::PetName::Dog).
    /// ```rust no_run
    /// use saptest::{Entity, SAPQuery, PetName};
    ///
    /// // Construct a query set to the "pets" table.
    /// let query = SAPQuery::builder()
    ///     .set_param("-name", vec![PetName::Ant, PetName::Dog])
    ///     .set_table(Entity::Pet);
    /// ```
    pub fn set_param<N: ToString, V: ToString>(mut self, name: N, value: Vec<V>) -> Self {
        self.params
            .entry(name.to_string())
            .and_modify(|e| e.extend(value.iter().map(|val| val.to_string())))
            .or_insert(value.into_iter().map(|value| value.to_string()).collect());
        self
    }

    /// Get a flattened list of params in the order of insertion.
    /// ```
    /// use saptest::SAPQuery;
    ///
    /// let params = [
    ///     ("name", vec!["Turtle"]),
    ///     ("pack", vec!["Turtle", "Star"]),
    ///     ("tier", vec!["1", "2", "3"]),
    ///     ("lvl", vec!["1"]),
    /// ];
    /// let query = SAPQuery::from_iter(params);
    /// let params = query.flat_params();
    ///
    /// assert_eq!(
    ///     params,
    ///     vec!["Turtle", "Turtle", "Star", "1", "2", "3", "1"]
    /// )
    /// ```
    pub fn flat_params(&self) -> Vec<&String> {
        self.params.iter().flat_map(|(_, params)| params).collect()
    }

    /// Generate a `SQL` string from the query.
    /// * Raises [`SAPTestError::QueryFailure`] when no table is set.
    /// ```
    /// use saptest::{Entity, SAPQuery, FoodName};
    ///
    /// let stmt = SAPQuery::builder()
    ///     .set_table(Entity::Food)
    ///     .set_param("name", vec![FoodName::Apple, FoodName::Coconut])
    ///     .as_sql()
    ///     .unwrap();
    ///
    /// assert_eq!("SELECT * FROM foods WHERE name IN (?, ?)", &stmt)
    /// ```
    pub fn as_sql(&self) -> Result<String, SAPTestError> {
        let Some(table) = self.table.map(|table| {
            let mut table_name = table.to_string().to_lowercase();
            table_name.push('s');
            table_name
        }) else {
            return Err(SAPTestError::QueryFailure {
                subject: "No Table".to_string(),
                reason: "Query requires a table.".to_string(),
            });
        };
        let mut sql_stmt = format!("SELECT * FROM {}", table);
        // If params.
        if !self.params.is_empty() {
            sql_stmt.push_str(" WHERE ")
        }

        // Iterate through params and set up SQL statement.
        // No user param values are inserted.
        for (i, (param_name, param_value)) in self.params.iter().enumerate() {
            // If param_name starts with '-', use NOT IN to get all other params.
            let mut param_name = &param_name[..];
            let sql_in = if let Some(neg_param_name) = param_name.strip_prefix('-') {
                param_name = neg_param_name;
                "NOT IN"
            } else {
                "IN"
            };
            // Set number of query params.
            let n_elems = param_value.len();
            let params_string = vec!["?"; n_elems].join(", ");

            // If at end of params, don't include AND.
            if i + 1 == self.params.len() {
                let _ = write!(sql_stmt, "{param_name} {sql_in} ({params_string})");
            } else {
                let _ = write!(sql_stmt, "{param_name} {sql_in} ({params_string}) AND ",);
            }
        }
        Ok(sql_stmt)
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use super::SAPQuery;
    use crate::{db::pack::Pack, Entity};

    #[test]
    fn test_build_query_from_iter() {
        let params = [
            ("name", vec!["Turtle"]),
            ("pack", vec!["Turtle", "Star"]),
            ("tier", vec!["1", "2", "3"]),
            ("lvl", vec!["1"]),
        ];
        let query = SAPQuery::from_iter(params.clone());
        assert!(query.table.is_none());
        assert_eq!(
            query.params.clone().into_iter().collect_vec(),
            params
                .iter()
                .map(|(param, values)| (
                    String::from(*param),
                    values.iter().map(|val| String::from(*val)).collect_vec()
                ))
                .collect_vec()
        );
    }

    #[test]
    fn test_query_flat_params() {
        let params = [
            ("name", vec!["Turtle"]),
            ("pack", vec!["Turtle", "Star"]),
            ("tier", vec!["1", "2", "3"]),
            ("lvl", vec!["1"]),
        ];
        let query = SAPQuery::from_iter(params);
        let params = query.flat_params();

        assert_eq!(params, vec!["Turtle", "Turtle", "Star", "1", "2", "3", "1"])
    }

    #[test]
    fn test_build_param_query() {
        let stmt = SAPQuery::builder()
            .set_table(Entity::Food)
            .set_param("name", vec!["apple", "coconut"])
            .as_sql()
            .unwrap();

        assert_eq!("SELECT * FROM foods WHERE name IN (?, ?)", &stmt)
    }

    #[test]
    fn test_build_neg_param_query() {
        let stmt = SAPQuery::builder()
            .set_table(Entity::Food)
            .set_param("name", vec!["apple", "coconut"])
            .set_param("-pack", vec![Pack::Turtle])
            .as_sql()
            .unwrap();
        assert_eq!(
            "SELECT * FROM foods WHERE name IN (?, ?) AND pack NOT IN (?)",
            &stmt
        )
    }
}
