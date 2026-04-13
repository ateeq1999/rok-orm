//! 10.1 JSON column support for [`QueryBuilder`].
//!
//! Generates dialect-appropriate JSON extraction SQL at method-call time
//! using the builder's stored [`Dialect`] (set via [`QueryBuilder::with_dialect`]).
//!
//! | Method | PostgreSQL | SQLite | MySQL |
//! |--------|-----------|--------|-------|
//! | `where_json_contains` | `col->>'key' = $N` | `json_extract(col,'$.key') = ?` | `JSON_VALUE(col,'$.key') = ?` |
//! | `where_json_path`     | `col #>> '{path}' = $N` | `json_extract(col,'$.path') = ?` | `JSON_VALUE(col,'$.path') = ?` |
//! | `where_json_array_contains` | `col @> $N::jsonb` | `EXISTS (SELECT 1 FROM json_each(col) WHERE value = ?)` | `JSON_CONTAINS(col,?)` |
//! | `select_json_field`   | `col->>'key' AS alias` | `json_extract(col,'$.key') AS alias` | `JSON_VALUE(col,'$.key') AS alias` |

use super::builder::{Dialect, QueryBuilder};
use super::condition::{Condition, JoinOp, SqlValue};

impl<T> QueryBuilder<T> {
    /// Filter rows where `col->>'key' = val` (dialect-aware).
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("users")
    ///     .where_json_contains("metadata", "role", "admin")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("metadata->>'role'"));
    /// ```
    pub fn where_json_contains(
        self,
        col: &str,
        key: &str,
        val: impl Into<SqlValue>,
    ) -> Self {
        let val = val.into();
        let sql = match self.dialect {
            Dialect::Postgres => format!("{col}->>'{}' = $1", key),
            Dialect::Sqlite => format!("json_extract({col},'$.{}') = ?", key),
            Dialect::Mysql => format!("JSON_VALUE({col},'$.{}') = ?", key),
        };
        self.push(JoinOp::And, Condition::RawExpr(sql, vec![val]))
    }

    /// Filter rows using a JSON path expression (dialect-aware).
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("users")
    ///     .where_json_path("settings", "$.theme", "dark")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("settings"));
    /// assert!(sql.contains("theme"));
    /// ```
    pub fn where_json_path(
        self,
        col: &str,
        path: &str,
        val: impl Into<SqlValue>,
    ) -> Self {
        let val = val.into();
        // Strip leading '$.' from path for PG arrow syntax
        let pg_key = path.trim_start_matches("$.");
        let sql = match self.dialect {
            Dialect::Postgres => format!("{col} #>> '{{{}}}' = $1", pg_key),
            Dialect::Sqlite => format!("json_extract({col},'{path}') = ?"),
            Dialect::Mysql => format!("JSON_VALUE({col},'{path}') = ?"),
        };
        self.push(JoinOp::And, Condition::RawExpr(sql, vec![val]))
    }

    /// Filter rows where a JSON array column contains a value (dialect-aware).
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("users")
    ///     .where_json_array_contains("permissions", "posts:write")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("permissions"));
    /// ```
    pub fn where_json_array_contains(
        self,
        col: &str,
        val: impl Into<SqlValue>,
    ) -> Self {
        let val = val.into();
        let sql = match self.dialect {
            Dialect::Postgres => format!("{col} @> $1::jsonb"),
            Dialect::Sqlite => {
                format!("EXISTS (SELECT 1 FROM json_each({col}) WHERE value = ?)")
            }
            Dialect::Mysql => format!("JSON_CONTAINS({col}, ?)"),
        };
        self.push(JoinOp::And, Condition::RawExpr(sql, vec![val]))
    }

    /// Add a JSON field extraction to the SELECT list (dialect-aware).
    ///
    /// If no columns are selected yet, `*` is prepended automatically.
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("users")
    ///     .select_json_field("metadata", "role", "user_role")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("user_role"));
    /// ```
    pub fn select_json_field(
        mut self,
        col: &str,
        key: &str,
        alias: &str,
    ) -> Self {
        let expr = match self.dialect {
            Dialect::Postgres => format!("{col}->>'{}' AS {alias}", key),
            Dialect::Sqlite => format!("json_extract({col},'$.{}') AS {alias}", key),
            Dialect::Mysql => format!("JSON_VALUE({col},'$.{}') AS {alias}", key),
        };
        let cols = self
            .select_cols
            .get_or_insert_with(|| vec!["*".to_string()]);
        cols.push(expr);
        self
    }
}
