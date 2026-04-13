//! 10.3 Sub-queries, CTEs, and `FROM` overrides for [`QueryBuilder`].
//!
//! # SubQueryBuilder
//!
//! [`SubQueryBuilder`] is a thin wrapper around `QueryBuilder<()>` that exposes
//! the subset of builder methods that make sense inside a sub-select.  Its
//! [`to_inner_sql`](SubQueryBuilder::to_inner_sql) method returns the rendered
//! SQL (without a trailing `;`) together with any bound params.
//!
//! # Param numbering
//!
//! For PostgreSQL, inner `$1`, `$2`, … placeholders are rewritten to the
//! correct outer offset by [`Condition::RawExpr`] — no manual renumbering
//! needed.  For SQLite / MySQL the inner SQL uses `?` and is returned as-is.
//!
//! # CTE limitation
//!
//! CTEs created with [`QueryBuilder::with_cte`] do **not** support bound
//! parameters in the initial implementation.  Use `where_raw` with inlined
//! literal values inside the CTE closure, or use a subquery instead.

use super::builder::{Dialect, QueryBuilder};
use super::condition::{Condition, JoinOp, SqlValue};

// ── SubQueryBuilder ───────────────────────────────────────────────────────────

/// A lightweight query builder for use inside subquery / CTE closures.
///
/// Exposes a subset of [`QueryBuilder`] methods — enough to build a `SELECT`
/// without the outer context of a full ORM query.
pub struct SubQueryBuilder {
    inner: QueryBuilder<()>,
}

impl SubQueryBuilder {
    pub(super) fn new(dialect: Dialect) -> Self {
        let mut inner = QueryBuilder::new("");
        inner.dialect = dialect;
        SubQueryBuilder { inner }
    }

    /// Set the `FROM` table.
    pub fn table(mut self, t: &str) -> Self {
        self.inner.table = t.to_string();
        self
    }

    /// Select specific columns.
    pub fn select(mut self, cols: &[&str]) -> Self {
        self.inner = self.inner.select(cols);
        self
    }

    /// Override the SELECT list with a raw expression.
    pub fn select_raw(mut self, expr: &str) -> Self {
        self.inner = self.inner.select_raw(expr);
        self
    }

    /// Add an equality `WHERE` condition.
    pub fn filter(mut self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.inner = self.inner.filter(col, val);
        self
    }

    /// Add a raw `WHERE` fragment (no bound params).
    pub fn where_raw(mut self, sql: &str) -> Self {
        self.inner = self.inner.where_raw(sql);
        self
    }

    /// Add a `GROUP BY` clause.
    pub fn group_by(mut self, cols: &[&str]) -> Self {
        self.inner = self.inner.group_by(cols);
        self
    }

    /// Add a raw `HAVING` expression.
    pub fn having_raw(mut self, expr: &str) -> Self {
        self.inner = self.inner.having_raw(expr);
        self
    }

    /// Render the subquery SQL.
    ///
    /// For PostgreSQL the params use `$1`, `$2`, … which are rewritten to
    /// the correct outer offsets when stored in a [`Condition::RawExpr`].
    pub fn to_inner_sql(self) -> (String, Vec<SqlValue>) {
        let dialect = self.inner.dialect;
        self.inner.to_sql_with_dialect(dialect)
    }
}

// ── QueryBuilder subquery / CTE methods ──────────────────────────────────────

impl<T> QueryBuilder<T> {
    /// Add `WHERE col IN (subquery)`.
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("users")
    ///     .where_in_subquery("id", |sq| {
    ///         sq.table("orders")
    ///           .select(&["user_id"])
    ///           .group_by(&["user_id"])
    ///           .having_raw("COUNT(*) > 10")
    ///     })
    ///     .to_sql();
    ///
    /// assert!(sql.contains("id IN (SELECT user_id FROM orders"));
    /// ```
    pub fn where_in_subquery(
        self,
        col: &str,
        f: impl FnOnce(SubQueryBuilder) -> SubQueryBuilder,
    ) -> Self {
        let sq = f(SubQueryBuilder::new(self.dialect));
        let (inner_sql, inner_params) = sq.to_inner_sql();
        self.push(
            JoinOp::And,
            Condition::RawExpr(format!("{col} IN ({inner_sql})"), inner_params),
        )
    }

    /// Add `WHERE EXISTS (subquery)`.
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("users")
    ///     .where_exists(|sq| {
    ///         sq.table("orders")
    ///           .select(&["1"])
    ///           .where_raw("orders.user_id = users.id")
    ///     })
    ///     .to_sql();
    ///
    /// assert!(sql.contains("EXISTS (SELECT 1 FROM orders"));
    /// ```
    pub fn where_exists(
        self,
        f: impl FnOnce(SubQueryBuilder) -> SubQueryBuilder,
    ) -> Self {
        let sq = f(SubQueryBuilder::new(self.dialect));
        let (inner_sql, inner_params) = sq.to_inner_sql();
        self.push(
            JoinOp::And,
            Condition::RawExpr(format!("EXISTS ({inner_sql})"), inner_params),
        )
    }

    /// Add `WHERE NOT EXISTS (subquery)`.
    pub fn where_not_exists(
        self,
        f: impl FnOnce(SubQueryBuilder) -> SubQueryBuilder,
    ) -> Self {
        let sq = f(SubQueryBuilder::new(self.dialect));
        let (inner_sql, inner_params) = sq.to_inner_sql();
        self.push(
            JoinOp::And,
            Condition::RawExpr(format!("NOT EXISTS ({inner_sql})"), inner_params),
        )
    }

    /// Prepend a `WITH name AS (subquery)` CTE to the query.
    ///
    /// > **Note:** The CTE closure does not support bound parameters.
    /// > Use `where_raw` with inlined literal values inside the closure.
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("ranked")
    ///     .with_cte("ranked", |cte| {
    ///         cte.table("users")
    ///            .select_raw("*, ROW_NUMBER() OVER (ORDER BY created_at DESC) AS rn")
    ///     })
    ///     .from_cte("ranked")
    ///     .where_raw("rn <= 10")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("WITH ranked AS ("));
    /// assert!(sql.contains("FROM ranked"));
    /// ```
    pub fn with_cte(
        mut self,
        name: &str,
        f: impl FnOnce(SubQueryBuilder) -> SubQueryBuilder,
    ) -> Self {
        let sq = f(SubQueryBuilder::new(self.dialect));
        let (inner_sql, _params) = sq.to_inner_sql();
        self.ctes.push((name.to_string(), inner_sql));
        self
    }

    /// Set the `FROM` clause to the named CTE (use after [`with_cte`](Self::with_cte)).
    pub fn from_cte(mut self, name: &str) -> Self {
        self.from_override = Some(name.to_string());
        self
    }

    /// Set the `FROM` clause to `(subquery) AS alias`.
    ///
    /// Bound params from the subquery are prepended to the output params so
    /// outer `WHERE` params get the correct `$N` offsets automatically.
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, params) = QueryBuilder::<()>::new("active_users")
    ///     .from_subquery("active_users", |sq| {
    ///         sq.table("users").filter("active", true)
    ///     })
    ///     .to_sql();
    ///
    /// assert!(sql.contains("FROM (SELECT * FROM users WHERE active = $1) AS active_users"));
    /// assert_eq!(params.len(), 1);
    /// ```
    pub fn from_subquery(
        mut self,
        alias: &str,
        f: impl FnOnce(SubQueryBuilder) -> SubQueryBuilder,
    ) -> Self {
        let sq = f(SubQueryBuilder::new(self.dialect));
        let (inner_sql, inner_params) = sq.to_inner_sql();
        self.from_override = Some(format!("({inner_sql}) AS {alias}"));
        self.cte_params.extend(inner_params);
        self
    }
}
