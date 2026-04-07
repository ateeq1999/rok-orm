//! [`QueryBuilder`] struct definition and fluent builder API.

use std::marker::PhantomData;

use super::condition::{Condition, JoinOp, OrderDir, SqlValue};

// ── Dialect ────────────────────────────────────────────────────────────────

/// SQL placeholder dialect.
///
/// - [`Dialect::Postgres`] — numbered placeholders (`$1`, `$2`, …)
/// - [`Dialect::Sqlite`]   — anonymous placeholders (`?`, `?`, …)
/// - [`Dialect::Mysql`]    — anonymous placeholders (`?`, `?`, …)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dialect {
    #[default]
    Postgres,
    Sqlite,
    Mysql,
}

// ── Join ───────────────────────────────────────────────────────────────────

/// A SQL JOIN clause.
#[derive(Debug, Clone)]
pub enum Join {
    /// `INNER JOIN table ON condition`
    Inner(String, String),
    /// `LEFT JOIN table ON condition`
    Left(String, String),
    /// `RIGHT JOIN table ON condition`
    Right(String, String),
}

// ── SoftDeleteMode ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftDeleteMode {
    Exclude,
    Include,
    Only,
}

// ── QueryBuilder ───────────────────────────────────────────────────────────

/// A fluent builder that produces parameterized SQL statements.
///
/// Conditions added with `where_*` methods are joined with `AND`.
/// Use `or_where_*` variants to join with `OR`.
#[derive(Debug)]
pub struct QueryBuilder<T> {
    pub(super) table: String,
    pub(super) select_cols: Option<Vec<String>>,
    pub(super) distinct: bool,
    pub(super) joins: Vec<Join>,
    pub(super) conditions: Vec<(JoinOp, Condition)>,
    pub(super) group_by: Vec<String>,
    pub(super) having: Option<String>,
    pub(super) order: Vec<(String, OrderDir)>,
    pub(super) limit_val: Option<usize>,
    pub(super) offset_val: Option<usize>,
    pub(super) soft_delete_mode: SoftDeleteMode,
    pub(super) soft_delete_column: Option<String>,
    pub(super) update_columns: Vec<(String, SqlValue)>,
    pub(super) eager_loads: Vec<String>,
    pub(super) _marker: PhantomData<T>,
}

impl<T> Clone for QueryBuilder<T> {
    fn clone(&self) -> Self {
        QueryBuilder {
            table: self.table.clone(),
            select_cols: self.select_cols.clone(),
            distinct: self.distinct,
            joins: self.joins.clone(),
            conditions: self.conditions.clone(),
            group_by: self.group_by.clone(),
            having: self.having.clone(),
            order: self.order.clone(),
            limit_val: self.limit_val,
            offset_val: self.offset_val,
            soft_delete_mode: self.soft_delete_mode,
            soft_delete_column: self.soft_delete_column.clone(),
            update_columns: self.update_columns.clone(),
            eager_loads: self.eager_loads.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> QueryBuilder<T> {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            select_cols: None,
            distinct: false,
            joins: Vec::new(),
            conditions: Vec::new(),
            group_by: Vec::new(),
            having: None,
            order: Vec::new(),
            limit_val: None,
            offset_val: None,
            soft_delete_mode: SoftDeleteMode::Exclude,
            soft_delete_column: None,
            update_columns: Vec::new(),
            eager_loads: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn with_soft_delete(mut self, column: impl Into<String>) -> Self {
        self.soft_delete_column = Some(column.into());
        self
    }

    pub fn with(mut self, relation: impl Into<String>) -> Self {
        self.eager_loads.push(relation.into());
        self
    }

    pub fn with_many(mut self, relations: Vec<String>) -> Self {
        self.eager_loads.extend(relations);
        self
    }

    pub fn eager_loads(&self) -> &[String] {
        &self.eager_loads
    }

    pub fn with_trashed(mut self) -> Self {
        self.soft_delete_mode = SoftDeleteMode::Include;
        self
    }

    pub fn only_trashed(mut self) -> Self {
        self.soft_delete_mode = SoftDeleteMode::Only;
        self
    }

    // ── column selection ────────────────────────────────────────────────────

    pub fn select(mut self, cols: &[&str]) -> Self {
        self.select_cols = Some(cols.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Emit `SELECT DISTINCT …`.
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    // ── joins ───────────────────────────────────────────────────────────────

    /// Add an `INNER JOIN table ON condition`.
    pub fn inner_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(Join::Inner(table.to_string(), on.to_string()));
        self
    }

    /// Add a `LEFT JOIN table ON condition`.
    pub fn left_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(Join::Left(table.to_string(), on.to_string()));
        self
    }

    /// Add a `RIGHT JOIN table ON condition`.
    pub fn right_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(Join::Right(table.to_string(), on.to_string()));
        self
    }

    // ── GROUP BY / HAVING ───────────────────────────────────────────────────

    /// Add a `GROUP BY` clause.
    pub fn group_by(mut self, cols: &[&str]) -> Self {
        self.group_by = cols.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a `HAVING` clause (requires [`group_by`]).
    pub fn having(mut self, expr: &str) -> Self {
        self.having = Some(expr.to_string());
        self
    }

    // ── ordering ────────────────────────────────────────────────────────────

    pub fn order_by(mut self, col: &str) -> Self {
        self.order.push((col.into(), OrderDir::Asc));
        self
    }

    pub fn order_by_desc(mut self, col: &str) -> Self {
        self.order.push((col.into(), OrderDir::Desc));
        self
    }

    // ── pagination ──────────────────────────────────────────────────────────

    pub fn limit(mut self, n: usize) -> Self {
        self.limit_val = Some(n);
        self
    }

    pub fn offset(mut self, n: usize) -> Self {
        self.offset_val = Some(n);
        self
    }

    pub fn paginate(mut self, page: i64, per_page: i64) -> Self {
        let per_page = per_page.max(1).min(100);
        let offset = ((page.max(1) - 1) * per_page) as usize;
        self.limit_val = Some(per_page as usize);
        self.offset_val = Some(offset);
        self
    }

    // ── accessors ───────────────────────────────────────────────────────────

    /// Expose the raw conditions (useful for callers that need to inspect them).
    pub fn conditions(&self) -> &[(JoinOp, Condition)] {
        &self.conditions
    }

    // ── internals ───────────────────────────────────────────────────────────

    pub(super) fn push(mut self, op: JoinOp, cond: Condition) -> Self {
        self.conditions.push((op, cond));
        self
    }

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

