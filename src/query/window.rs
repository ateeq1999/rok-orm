//! 10.4 Window function helpers for [`QueryBuilder`].
//!
//! Window functions are built on top of `select_raw` / `from_subquery` (Phase 8 / 10.3).
//!
//! # Pattern
//!
//! ```rust
//! use rok_orm::QueryBuilder;
//!
//! // Latest post per user: ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at)
//! let (sql, _) = QueryBuilder::<()>::new("posts")
//!     .window_rank_by("user_id", "created_at", "row_num")
//!     .having_rank(1)
//!     .to_sql();
//!
//! // Generates:
//! // SELECT * FROM (
//! //   SELECT *, ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at) AS row_num
//! //   FROM posts
//! // ) AS __ranked WHERE row_num = 1
//! assert!(sql.contains("ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at) AS row_num"));
//! assert!(sql.contains("__ranked WHERE row_num = 1"));
//! ```

use super::builder::QueryBuilder;

impl<T> QueryBuilder<T> {
    /// Inject `ROW_NUMBER() OVER (PARTITION BY … ORDER BY …) AS alias` into SELECT.
    ///
    /// The alias is stored so that a subsequent [`having_rank`](Self::having_rank)
    /// call can reference it in the outer subquery wrapper.
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("posts")
    ///     .window_rank_by("user_id", "created_at", "row_num")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at) AS row_num"));
    /// ```
    pub fn window_rank_by(
        mut self,
        partition_col: &str,
        order_col: &str,
        alias: &str,
    ) -> Self {
        let expr = format!(
            "ROW_NUMBER() OVER (PARTITION BY {partition_col} ORDER BY {order_col}) AS {alias}"
        );
        self = self.add_select_expr(&expr);
        self.window_rank_alias = Some(alias.to_string());
        self
    }

    /// Wrap the query in `SELECT * FROM (...) AS __ranked WHERE alias = n`.
    ///
    /// Must be called after [`window_rank_by`](Self::window_rank_by).
    /// The `LIMIT` / `OFFSET` on the inner query are suppressed so the full
    /// ranked result set is available to the outer filter.
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("posts")
    ///     .window_rank_by("user_id", "created_at", "row_num")
    ///     .having_rank(1)
    ///     .to_sql();
    ///
    /// assert!(sql.contains("__ranked WHERE row_num = 1"));
    /// ```
    pub fn having_rank(mut self, n: i64) -> Self {
        let alias = self
            .window_rank_alias
            .clone()
            .unwrap_or_else(|| "row_num".to_string());
        self.having_rank_n = Some((alias, n));
        self
    }
}
