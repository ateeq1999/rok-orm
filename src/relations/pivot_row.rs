//! [`PivotRow`] — a related model instance with accompanying pivot table data.

use std::collections::HashMap;
use crate::query::SqlValue;

/// A related model record paired with the pivot columns from the join table.
///
/// Use this as the return type when you need access to pivot data alongside the
/// related model (e.g., `assigned_at`, `expires_at` on a user-role pivot table).
///
/// # Example
///
/// ```rust,ignore
/// let roles: Vec<PivotRow<Role>> = user
///     .roles()
///     .with_pivot(&["assigned_at", "expires_at"])
///     .get(&pool)
///     .await?;
///
/// for pivot_row in &roles {
///     println!("role: {:?}", pivot_row.model);
///     println!("assigned: {:?}", pivot_row.pivot("assigned_at"));
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PivotRow<M> {
    /// The related model instance.
    pub model: M,
    /// Pivot column values keyed by column name.
    pub pivot_data: HashMap<String, SqlValue>,
}

impl<M> PivotRow<M> {
    /// Create a new `PivotRow` with the given model and pivot data.
    pub fn new(model: M, pivot_data: HashMap<String, SqlValue>) -> Self {
        Self { model, pivot_data }
    }

    /// Get a pivot column value by name.
    ///
    /// Returns `None` if the column wasn't included via `with_pivot`.
    pub fn pivot(&self, column: &str) -> Option<&SqlValue> {
        self.pivot_data.get(column)
    }

    /// Check whether a pivot column is present.
    pub fn has_pivot(&self, column: &str) -> bool {
        self.pivot_data.contains_key(column)
    }
}

impl<M> std::ops::Deref for PivotRow<M> {
    type Target = M;
    fn deref(&self) -> &M {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Role { pub id: i64, pub name: String }

    #[test]
    fn pivot_row_deref_to_model() {
        let mut pivot_data = HashMap::new();
        pivot_data.insert("assigned_at".into(), SqlValue::Text("2026-01-01".into()));

        let row = PivotRow::new(Role { id: 1, name: "admin".into() }, pivot_data);
        assert_eq!(row.id, 1);            // via Deref
        assert_eq!(row.name, "admin");    // via Deref
    }

    #[test]
    fn pivot_column_access() {
        let mut data = HashMap::new();
        data.insert("weight".into(), SqlValue::Integer(5));
        let row = PivotRow::new(Role { id: 2, name: "mod".into() }, data);
        assert_eq!(row.pivot("weight"), Some(&SqlValue::Integer(5)));
        assert_eq!(row.pivot("missing"), None);
        assert!(row.has_pivot("weight"));
        assert!(!row.has_pivot("missing"));
    }
}
