//! SELECT / COUNT / DELETE / UPDATE SQL generation for [`QueryBuilder`].

use super::builder::{Dialect, QueryBuilder};
use super::condition::SqlValue;

impl<T> QueryBuilder<T> {
    // ── aggregation shortcuts ───────────────────────────────────────────────

    pub fn count_sql(&self) -> (String, Vec<SqlValue>) {
        self.to_count_sql()
    }

    pub fn aggregate_sql(&self, func: &str, column: &str) -> (String, Vec<SqlValue>) {
        self.aggregate_sql_with_dialect(Dialect::Postgres, func, column)
    }

    pub fn aggregate_sql_with_dialect(
        &self,
        dialect: Dialect,
        func: &str,
        column: &str,
    ) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let joins = self.build_joins();
        let where_clause = self.build_where_with_soft_delete(dialect, &mut params);
        let group_by = self.build_group_by();
        let order = self.build_order();
        let sql = format!(
            "SELECT {}({}) FROM {}{}{}{}{}",
            func, column, self.table, joins, where_clause, group_by, order
        );
        (sql, params)
    }

    pub fn sum_sql(&self, column: &str) -> (String, Vec<SqlValue>) {
        self.aggregate_sql("SUM", column)
    }

    pub fn avg_sql(&self, column: &str) -> (String, Vec<SqlValue>) {
        self.aggregate_sql("AVG", column)
    }

    pub fn min_sql(&self, column: &str) -> (String, Vec<SqlValue>) {
        self.aggregate_sql("MIN", column)
    }

    pub fn max_sql(&self, column: &str) -> (String, Vec<SqlValue>) {
        self.aggregate_sql("MAX", column)
    }

    pub fn exists_sql(&self) -> (String, Vec<SqlValue>) {
        self.exists_sql_with_dialect(Dialect::Postgres)
    }

    pub fn exists_sql_with_dialect(&self, dialect: Dialect) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let where_clause = self.build_where_with_soft_delete(dialect, &mut params);
        let joins = self.build_joins();
        let sql = format!(
            "SELECT EXISTS(SELECT 1 FROM {}{}{})",
            self.table, joins, where_clause
        );
        (sql, params)
    }

    pub fn pluck_sql(&self, column: &str) -> (String, Vec<SqlValue>) {
        self.pluck_sql_with_dialect(Dialect::Postgres, column)
    }

    pub fn pluck_sql_with_dialect(
        &self,
        dialect: Dialect,
        column: &str,
    ) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let where_clause = self.build_where_with_soft_delete(dialect, &mut params);
        let joins = self.build_joins();
        let order = self.build_order();
        let limit = self
            .limit_val
            .map(|n| format!(" LIMIT {n}"))
            .unwrap_or_default();
        let sql = format!(
            "SELECT {} FROM {}{}{}{}{}",
            column, self.table, joins, where_clause, order, limit
        );
        (sql, params)
    }

    // ── SELECT ──────────────────────────────────────────────────────────────

    /// Build a parameterized `SELECT` statement (PostgreSQL `$N` placeholders).
    pub fn to_sql(&self) -> (String, Vec<SqlValue>) {
        self.to_sql_with_dialect(Dialect::Postgres)
    }

    /// Build a parameterized `SELECT` statement for the given [`Dialect`].
    pub fn to_sql_with_dialect(&self, dialect: Dialect) -> (String, Vec<SqlValue>) {
        let cols = self
            .select_cols
            .as_ref()
            .map(|c| c.join(", "))
            .unwrap_or_else(|| "*".into());
        let distinct_kw = if self.distinct { "DISTINCT " } else { "" };
        let mut sql = format!("SELECT {distinct_kw}{cols} FROM {}", self.table);
        let mut params: Vec<SqlValue> = Vec::new();

        sql.push_str(&self.build_joins());
        sql.push_str(&self.build_where_with_soft_delete(dialect, &mut params));
        sql.push_str(&self.build_group_by());
        sql.push_str(&self.build_order());

        if let Some(n) = self.limit_val {
            sql.push_str(&format!(" LIMIT {n}"));
        }
        if let Some(n) = self.offset_val {
            sql.push_str(&format!(" OFFSET {n}"));
        }
        (sql, params)
    }

    /// Build a `SELECT COUNT(*) FROM …` statement (PostgreSQL dialect).
    pub fn to_count_sql(&self) -> (String, Vec<SqlValue>) {
        self.to_count_sql_with_dialect(Dialect::Postgres)
    }

    /// Build a `SELECT COUNT(*) FROM …` statement for the given dialect.
    pub fn to_count_sql_with_dialect(&self, dialect: Dialect) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let joins = self.build_joins();
        let where_clause = self.build_where_with_soft_delete(dialect, &mut params);
        (
            format!("SELECT COUNT(*) FROM {}{}{}", self.table, joins, where_clause),
            params,
        )
    }

    // ── DELETE ──────────────────────────────────────────────────────────────

    /// Build a `DELETE FROM … WHERE …` statement (PostgreSQL dialect).
    pub fn to_delete_sql(&self) -> (String, Vec<SqlValue>) {
        self.to_delete_sql_with_dialect(Dialect::Postgres)
    }

    /// Build a `DELETE FROM … WHERE …` statement for the given dialect.
    pub fn to_delete_sql_with_dialect(&self, dialect: Dialect) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let where_clause = self.build_where_dialect(dialect, &mut params);
        (
            format!("DELETE FROM {}{}", self.table, where_clause),
            params,
        )
    }

    // ── UPDATE ──────────────────────────────────────────────────────────────

    /// Build an `UPDATE … SET … WHERE …` statement (PostgreSQL dialect).
    pub fn to_update_sql(&self, data: &[(&str, SqlValue)]) -> (String, Vec<SqlValue>) {
        self.to_update_sql_with_dialect(Dialect::Postgres, data)
    }

    /// Build an `UPDATE … SET … WHERE …` statement for the given dialect.
    pub fn to_update_sql_with_dialect(
        &self,
        dialect: Dialect,
        data: &[(&str, SqlValue)],
    ) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let set_clauses: Vec<String> = data
            .iter()
            .enumerate()
            .map(|(i, (col, val))| {
                params.push(val.clone());
                match dialect {
                    Dialect::Postgres => format!("{col} = ${}", i + 1),
                    Dialect::Sqlite | Dialect::Mysql => format!("{col} = ?"),
                }
            })
            .collect();
        let mut sql = format!("UPDATE {} SET {}", self.table, set_clauses.join(", "));
        sql.push_str(&self.build_where_dialect(dialect, &mut params));
        (sql, params)
    }

    // ── restore / force-delete ──────────────────────────────────────────────

    pub fn push_update_column(mut self, col: impl Into<String>, val: SqlValue) -> Self {
        self.update_columns.push((col.into(), val));
        self
    }

    pub fn to_restore_sql(&self) -> (String, Vec<SqlValue>) {
        self.to_restore_sql_with_dialect(Dialect::Postgres)
    }

    pub fn to_restore_sql_with_dialect(&self, dialect: Dialect) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let set_clauses: Vec<String> = self
            .update_columns
            .iter()
            .enumerate()
            .map(|(i, (col, val))| {
                params.push(val.clone());
                match dialect {
                    Dialect::Postgres => format!("{col} = ${}", i + 1),
                    Dialect::Sqlite | Dialect::Mysql => format!("{col} = ?"),
                }
            })
            .collect();
        let mut sql = format!("UPDATE {} SET {}", self.table, set_clauses.join(", "));
        sql.push_str(&self.build_where_dialect(dialect, &mut params));
        (sql, params)
    }

    pub fn to_force_delete_sql(&self) -> (String, Vec<SqlValue>) {
        self.to_force_delete_sql_with_dialect(Dialect::Postgres)
    }

    pub fn to_force_delete_sql_with_dialect(&self, dialect: Dialect) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let where_clause = self.build_where_dialect(dialect, &mut params);
        (
            format!("DELETE FROM {}{}", self.table, where_clause),
            params,
        )
    }
}
