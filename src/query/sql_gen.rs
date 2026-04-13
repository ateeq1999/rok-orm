//! SELECT / COUNT / DELETE / UPDATE SQL generation for [`QueryBuilder`].
//!
//! Aggregate shortcuts (sum_sql, avg_sql, etc.) live in `aggregates.rs`.

use super::builder::{Dialect, Join, QueryBuilder, SoftDeleteMode};
use super::condition::{Condition, JoinOp, OrderDir, SqlValue};

impl<T> QueryBuilder<T> {
    // ── SELECT ──────────────────────────────────────────────────────────────

    /// Build a parameterized `SELECT` statement (PostgreSQL `$N` placeholders).
    pub fn to_sql(&self) -> (String, Vec<SqlValue>) {
        self.to_sql_with_dialect(Dialect::Postgres)
    }

    /// Build a parameterized `SELECT` statement for the given [`Dialect`].
    pub fn to_sql_with_dialect(&self, dialect: Dialect) -> (String, Vec<SqlValue>) {
        // 10.4: having_rank wraps the inner query in an outer subquery.
        if let Some((alias, n)) = &self.having_rank_n {
            let mut inner = self.clone();
            inner.having_rank_n = None;
            inner.limit_val = None;
            inner.offset_val = None;
            let (inner_sql, inner_params) = inner.to_sql_with_dialect(dialect);
            return (
                format!("SELECT * FROM ({inner_sql}) AS __ranked WHERE {alias} = {n}"),
                inner_params,
            );
        }

        let cols = self
            .select_cols
            .as_ref()
            .map(|c| c.join(", "))
            .unwrap_or_else(|| "*".into());
        let distinct_kw = if self.distinct { "DISTINCT " } else { "" };

        // 10.3: from_override replaces the table name (from_cte / from_subquery).
        let from = self.from_override.as_deref().unwrap_or(&self.table);
        let mut sql = format!("SELECT {distinct_kw}{cols} FROM {from}");

        // 10.3: Pre-populate params with CTE / from_subquery params so outer
        // WHERE params get the correct $N offsets automatically.
        let mut params: Vec<SqlValue> = self.cte_params.clone();

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

        // 10.3: Prepend WITH clause for CTEs.
        if !self.ctes.is_empty() {
            let cte_clauses: Vec<String> = self
                .ctes
                .iter()
                .map(|(name, inner_sql)| format!("{name} AS ({inner_sql})"))
                .collect();
            sql = format!("WITH {} {sql}", cte_clauses.join(", "));
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

    // ── internal build helpers ──────────────────────────────────────────────

    pub(crate) fn build_joins(&self) -> String {
        let mut out = String::new();
        for join in &self.joins {
            match join {
                Join::Inner(t, on) => out.push_str(&format!(" INNER JOIN {t} ON {on}")),
                Join::Left(t, on) => out.push_str(&format!(" LEFT JOIN {t} ON {on}")),
                Join::Right(t, on) => out.push_str(&format!(" RIGHT JOIN {t} ON {on}")),
            }
        }
        out
    }

    pub(crate) fn build_where_dialect(
        &self,
        dialect: Dialect,
        params: &mut Vec<SqlValue>,
    ) -> String {
        super::build_where_from_dialect(dialect, &self.conditions, params)
    }

    pub(crate) fn build_where_with_soft_delete(
        &self,
        dialect: Dialect,
        params: &mut Vec<SqlValue>,
    ) -> String {
        let mut conditions = self.conditions.clone();
        if let Some(ref col) = self.soft_delete_column {
            match self.soft_delete_mode {
                SoftDeleteMode::Exclude => {
                    conditions.push((JoinOp::And, Condition::IsNull(col.clone())));
                }
                SoftDeleteMode::Include => {}
                SoftDeleteMode::Only => {
                    conditions.push((JoinOp::And, Condition::IsNotNull(col.clone())));
                }
            }
        }
        super::build_where_from_dialect(dialect, &conditions, params)
    }

    pub(crate) fn build_group_by(&self) -> String {
        let mut out = String::new();
        if !self.group_by.is_empty() {
            out.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }
        if let Some(ref h) = self.having {
            out.push_str(&format!(" HAVING {h}"));
        }
        out
    }

    pub(crate) fn build_order(&self) -> String {
        if self.order.is_empty() {
            return String::new();
        }
        let parts: Vec<String> = self
            .order
            .iter()
            .map(|(col, dir)| match dir {
                OrderDir::Raw(expr) => expr.clone(),
                _ => format!("{col} {dir}"),
            })
            .collect();
        format!(" ORDER BY {}", parts.join(", "))
    }
}
