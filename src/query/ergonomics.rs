//! Developer ergonomics: `when`, `tap`, `dd`, raw clause variants for [`QueryBuilder`].

use crate::query::{QueryBuilder, SqlValue};

impl<T> QueryBuilder<T> {
    // ── conditional builder ──────────────────────────────────────────────────

    /// Apply `f` to `self` only when `condition` is `true`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let active_only = true;
    /// let (sql, _) = QueryBuilder::<()>::new("users")
    ///     .when(active_only, |q| q.where_eq("active", true))
    ///     .to_sql();
    ///
    /// assert!(sql.contains("WHERE active"));
    /// ```
    pub fn when(self, condition: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if condition { f(self) } else { self }
    }

    /// Apply `then_fn` when `condition` is `true`, otherwise apply `else_fn`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let is_admin = false;
    /// let (sql, _) = QueryBuilder::<()>::new("users")
    ///     .when_else(
    ///         is_admin,
    ///         |q| q,                               // admin: no filter
    ///         |q| q.where_eq("active", true),      // regular: active only
    ///     )
    ///     .to_sql();
    ///
    /// assert!(sql.contains("WHERE active"));
    /// ```
    pub fn when_else(
        self,
        condition: bool,
        then_fn: impl FnOnce(Self) -> Self,
        else_fn: impl FnOnce(Self) -> Self,
    ) -> Self {
        if condition { then_fn(self) } else { else_fn(self) }
    }

    // ── tap / debug ──────────────────────────────────────────────────────────

    /// Inspect the builder without modifying it (useful for debugging).
    ///
    /// # Example
    ///
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("users")
    ///     .where_eq("id", 1i64)
    ///     .tap(|q| println!("SQL so far: {:?}", q.to_sql().0))
    ///     .limit(1)
    ///     .to_sql();
    /// ```
    pub fn tap(self, f: impl FnOnce(&Self)) -> Self {
        f(&self);
        self
    }

    /// Print the generated SQL to stdout and return `self` (debug helper).
    ///
    /// The name `dd` is inspired by Laravel's "dump and die" — here it only dumps.
    pub fn dd(self) -> Self {
        let (sql, params) = self.to_sql();
        println!("[rok-orm dd] SQL: {sql}");
        println!("[rok-orm dd] params ({} total): {:?}", params.len(), params);
        self
    }

    // ── raw clause shortcuts ─────────────────────────────────────────────────

    /// Add a raw expression to the SELECT list.
    ///
    /// Appends to any existing `select()` columns, or replaces `*` with the raw expr.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("orders")
    ///     .select_raw("id, SUM(amount) as total")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("SELECT id, SUM(amount) as total"));
    /// ```
    pub fn select_raw(mut self, expr: &str) -> Self {
        self.select_cols = Some(vec![expr.to_string()]);
        self
    }

    /// Add a raw `ORDER BY` expression.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("posts")
    ///     .order_raw("RANDOM()")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("ORDER BY RANDOM()"));
    /// ```
    pub fn order_raw(mut self, expr: &str) -> Self {
        use super::condition::OrderDir;
        self.order.push((String::new(), OrderDir::Raw(expr.to_string())));
        self
    }

    /// Add a raw `HAVING` expression (alias that mirrors `having()`).
    pub fn having_raw(self, expr: &str) -> Self {
        self.having(expr)
    }

    // ── cursor pagination ────────────────────────────────────────────────────

    /// Apply cursor-based pagination constraints.
    ///
    /// If `after_id` is `Some(n)`, adds `WHERE id > n`. Sets `LIMIT limit + 1`
    /// so callers can detect whether more records exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("posts")
    ///     .order_by_desc("id")
    ///     .cursor_sql("id", Some(42i64), 20)
    ///     .to_sql();
    ///
    /// assert!(sql.contains("id > "));
    /// assert!(sql.contains("LIMIT 21"));
    /// ```
    pub fn cursor_sql(self, pk_col: &str, after_id: Option<i64>, limit: usize) -> Self {
        let q = if let Some(id) = after_id {
            self.where_gt(pk_col, id)
        } else {
            self
        };
        q.limit(limit + 1)
    }
}

#[cfg(test)]
mod tests {
    use crate::query::QueryBuilder;

    #[test]
    fn when_applies_when_true() {
        let (sql, _) = QueryBuilder::<()>::new("users")
            .when(true, |q| q.where_eq("active", true))
            .to_sql();
        assert!(sql.contains("WHERE active"));
    }

    #[test]
    fn when_skips_when_false() {
        let (sql, _) = QueryBuilder::<()>::new("users")
            .when(false, |q| q.where_eq("active", true))
            .to_sql();
        assert!(!sql.contains("WHERE"));
    }

    #[test]
    fn when_else_branches() {
        let (sql_admin, _) = QueryBuilder::<()>::new("u")
            .when_else(true, |q| q, |q| q.where_eq("active", true))
            .to_sql();
        assert!(!sql_admin.contains("WHERE"));

        let (sql_user, _) = QueryBuilder::<()>::new("u")
            .when_else(false, |q| q, |q| q.where_eq("active", true))
            .to_sql();
        assert!(sql_user.contains("WHERE active"));
    }

    #[test]
    fn select_raw_overrides_star() {
        let (sql, _) = QueryBuilder::<()>::new("orders")
            .select_raw("id, SUM(amount) as total")
            .to_sql();
        assert!(sql.contains("SELECT id, SUM(amount) as total"));
    }

    #[test]
    fn tap_does_not_modify() {
        let mut captured = String::new();
        let (sql, _) = QueryBuilder::<()>::new("users")
            .where_eq("id", 1i64)
            .tap(|q| captured = q.to_sql().0.clone())
            .limit(5)
            .to_sql();
        assert!(captured.contains("WHERE id"));
        assert!(sql.contains("LIMIT 5"));
    }
}
