//! The SQL of a search, built from the query (spec 06).

use rusqlite::types::Value;

use super::MAX_RESULTS;
use crate::documents::index::COLUMNS;

/// A search statement under construction: `WHERE` conditions and `ORDER BY`
/// terms, each with its own parameters (`?`), bound in that order.
#[derive(Debug, Default)]
pub(super) struct Statement {
    conditions: Vec<String>,
    condition_params: Vec<Value>,
    order: Vec<String>,
    order_params: Vec<Value>,
}

impl Statement {
    /// Keeps only the documents matching `condition`.
    pub fn filter(&mut self, condition: &str, params: impl IntoIterator<Item = Value>) {
        self.conditions.push(condition.to_string());
        self.condition_params.extend(params);
    }

    /// Sorts by `term` after the terms added before it. A statement needs
    /// at least one.
    pub fn order_by(&mut self, term: &str, params: impl IntoIterator<Item = Value>) {
        self.order.push(term.to_string());
        self.order_params.extend(params);
    }

    /// The SQL text, over present documents only, capped at [`MAX_RESULTS`].
    /// `opened_at` of the recent documents is available to sort by.
    pub fn sql(&self) -> String {
        let conditions = ["missing_since IS NULL".to_string()]
            .iter()
            .chain(&self.conditions)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" AND ");
        let order = self.order.join(", ");
        format!(
            "SELECT {COLUMNS} FROM documents
             LEFT JOIN recent_documents ON document_id = id
             WHERE {conditions} ORDER BY {order} LIMIT {MAX_RESULTS}"
        )
    }

    /// The parameters, in the order the `?` appear in [`Self::sql`].
    pub fn params(self) -> Vec<Value> {
        let mut params = self.condition_params;
        params.extend(self.order_params);
        params
    }
}
