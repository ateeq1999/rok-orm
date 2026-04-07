//! withCount / withSum / withAvg / withMin / withMax — subquery aggregate columns.
//!
//! These methods inject correlated subquery expressions into the SELECT list,
//! enabling aggregate data about related records without a JOIN.
//!
//! # Example
//!
//! ```rust
//! use rok_orm::QueryBuilder;
//!
//! let (sql, _) = QueryBuilder::<()>::new("users")
//!     .with_count_col("posts", "user_id", "id", "posts_count")
//!     .with_sum_col("orders", "user_id", "id", "amount", "total_spent")
//!     .to_sql();
//!
//! assert!(sql.contains("SELECT COUNT(*) FROM posts"));
//! assert!(sql.contains("posts_count"));
//! assert!(sql.contains("SELECT SUM(amount) FROM orders"));
//! ```

use super::builder::QueryBuilder;

impl<T> QueryBuilder<T> {
    /// Inject `(SELECT COUNT(*) FROM child WHERE child.fk = self.pk) AS alias`.
    pub fn with_count_col(
        self,
        child_table: &str,
        foreign_key: &str,
        self_pk: &str,
        alias: &str,
    ) -> Self {
        let sub = format!(
            "(SELECT COUNT(*) FROM {child_table} WHERE {child_table}.{foreign_key} = {}.{self_pk}) AS {alias}",
            self.table
        );
        self.add_select_expr(&sub)
    }

    /// Inject `(SELECT SUM(col) FROM child WHERE ...) AS alias`.
    pub fn with_sum_col(
        self,
        child_table: &str,
        foreign_key: &str,
        self_pk: &str,
        col: &str,
        alias: &str,
    ) -> Self {
        let sub = format!(
            "(SELECT SUM({col}) FROM {child_table} WHERE {child_table}.{foreign_key} = {}.{self_pk}) AS {alias}",
            self.table
        );
        self.add_select_expr(&sub)
    }

    /// Inject `(SELECT AVG(col) FROM child WHERE ...) AS alias`.
    pub fn with_avg_col(
        self,
        child_table: &str,
        foreign_key: &str,
        self_pk: &str,
        col: &str,
        alias: &str,
    ) -> Self {
        let sub = format!(
            "(SELECT AVG({col}) FROM {child_table} WHERE {child_table}.{foreign_key} = {}.{self_pk}) AS {alias}",
            self.table
        );
        self.add_select_expr(&sub)
    }

    /// Inject `(SELECT MIN(col) FROM child WHERE ...) AS alias`.
    pub fn with_min_col(
        self,
        child_table: &str,
        foreign_key: &str,
        self_pk: &str,
        col: &str,
        alias: &str,
    ) -> Self {
        let sub = format!(
            "(SELECT MIN({col}) FROM {child_table} WHERE {child_table}.{foreign_key} = {}.{self_pk}) AS {alias}",
            self.table
        );
        self.add_select_expr(&sub)
    }

    /// Inject `(SELECT MAX(col) FROM child WHERE ...) AS alias`.
    pub fn with_max_col(
        self,
        child_table: &str,
        foreign_key: &str,
        self_pk: &str,
        col: &str,
        alias: &str,
    ) -> Self {
        let sub = format!(
            "(SELECT MAX({col}) FROM {child_table} WHERE {child_table}.{foreign_key} = {}.{self_pk}) AS {alias}",
            self.table
        );
        self.add_select_expr(&sub)
    }

    /// Append a raw expression to the SELECT list, prepending `*` if none set.
    pub(super) fn add_select_expr(mut self, expr: &str) -> Self {
        let cols = self.select_cols.get_or_insert_with(|| vec!["*".to_string()]);
        cols.push(expr.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_count_col_injects_subquery() {
        let (sql, _) = QueryBuilder::<()>::new("users")
            .with_count_col("posts", "user_id", "id", "posts_count")
            .to_sql();
        assert!(sql.contains("SELECT COUNT(*) FROM posts WHERE posts.user_id = users.id"), "sql: {sql}");
        assert!(sql.contains("posts_count"), "sql: {sql}");
    }

    #[test]
    fn with_sum_col_injects_subquery() {
        let (sql, _) = QueryBuilder::<()>::new("users")
            .with_sum_col("orders", "user_id", "id", "amount", "total_spent")
            .to_sql();
        assert!(sql.contains("SELECT SUM(amount) FROM orders"), "sql: {sql}");
        assert!(sql.contains("total_spent"), "sql: {sql}");
    }

    #[test]
    fn multiple_agg_cols_combined() {
        let (sql, _) = QueryBuilder::<()>::new("authors")
            .with_count_col("books", "author_id", "id", "book_count")
            .with_avg_col("books", "author_id", "id", "rating", "avg_rating")
            .to_sql();
        assert!(sql.contains("book_count"), "sql: {sql}");
        assert!(sql.contains("avg_rating"), "sql: {sql}");
    }
}
