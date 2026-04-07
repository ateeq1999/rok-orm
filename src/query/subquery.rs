//! Subquery-based WHERE conditions: `where_has`, `where_doesnt_have`, `where_has_raw`.

use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};
use super::condition::{Condition, JoinOp};

impl<T> QueryBuilder<T> {
    /// Add a `WHERE EXISTS (subquery)` clause.
    ///
    /// Use this to filter records that have at least one matching related record.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// // SELECT * FROM users WHERE EXISTS (SELECT 1 FROM posts WHERE posts.user_id = users.id)
    /// let (sql, _) = QueryBuilder::<()>::new("users")
    ///     .where_has_raw("SELECT 1 FROM posts WHERE posts.user_id = users.id")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("EXISTS"));
    /// ```
    pub fn where_has_raw(self, subquery: &str) -> Self {
        self.push(
            JoinOp::And,
            Condition::Raw(format!("EXISTS ({subquery})")),
        )
    }

    /// Add a `WHERE NOT EXISTS (subquery)` clause.
    pub fn where_doesnt_have_raw(self, subquery: &str) -> Self {
        self.push(
            JoinOp::And,
            Condition::Raw(format!("NOT EXISTS ({subquery})")),
        )
    }

    /// Add a `WHERE EXISTS` clause using a related model's HasMany relationship.
    ///
    /// Generates: `EXISTS (SELECT 1 FROM child_table WHERE child_table.fk = self_table.pk)`
    pub fn where_has<C: Model>(
        self,
        child_table: &str,
        foreign_key: &str,
        self_pk: &str,
    ) -> Self {
        let subquery = format!(
            "SELECT 1 FROM {child_table} WHERE {child_table}.{foreign_key} = {}.{self_pk}",
            self.table()
        );
        self.where_has_raw(&subquery)
    }

    /// Add a `WHERE NOT EXISTS` clause using a related model's HasMany relationship.
    pub fn where_doesnt_have<C: Model>(
        self,
        child_table: &str,
        foreign_key: &str,
        self_pk: &str,
    ) -> Self {
        let subquery = format!(
            "SELECT 1 FROM {child_table} WHERE {child_table}.{foreign_key} = {}.{self_pk}",
            self.table()
        );
        self.where_doesnt_have_raw(&subquery)
    }

    /// Add a `WHERE EXISTS` with an additional filter on the subquery.
    ///
    /// Generates:
    /// ```sql
    /// EXISTS (
    ///   SELECT 1 FROM child_table
    ///   WHERE child_table.fk = parent.pk
    ///   AND child_table.col = $N
    /// )
    /// ```
    pub fn where_has_with<C: Model>(
        self,
        child_table: &str,
        foreign_key: &str,
        self_pk: &str,
        filter_col: &str,
        filter_val: impl Into<SqlValue>,
    ) -> Self {
        let val = filter_val.into();
        let literal = match &val {
            SqlValue::Text(s) => format!("'{s}'"),
            SqlValue::Integer(n) => n.to_string(),
            SqlValue::Float(f) => f.to_string(),
            SqlValue::Bool(b) => if *b { "TRUE".into() } else { "FALSE".into() },
            SqlValue::Null => "NULL".into(),
        };
        let subquery = format!(
            "SELECT 1 FROM {child_table} WHERE {child_table}.{foreign_key} = {}.{self_pk} AND {child_table}.{filter_col} = {literal}",
            self.table()
        );
        self.where_has_raw(&subquery)
    }

    /// Access the underlying table name (used by subquery helpers).
    pub fn table(&self) -> &str {
        &self.table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn where_has_raw_generates_exists() {
        let (sql, _) = QueryBuilder::<()>::new("users")
            .where_has_raw("SELECT 1 FROM posts WHERE posts.user_id = users.id")
            .to_sql();
        assert!(sql.contains("EXISTS (SELECT 1 FROM posts WHERE posts.user_id = users.id)"));
    }

    #[test]
    fn where_doesnt_have_raw_generates_not_exists() {
        let (sql, _) = QueryBuilder::<()>::new("users")
            .where_doesnt_have_raw("SELECT 1 FROM posts WHERE posts.user_id = users.id")
            .to_sql();
        assert!(sql.contains("NOT EXISTS"));
    }
}
