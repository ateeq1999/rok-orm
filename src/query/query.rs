//! [`QueryBuilder`] ΓÇö fluent SQL builder.
//!
//! Use [`QueryBuilder::to_sql`] for PostgreSQL (`$N` placeholders) and
//! [`QueryBuilder::to_sql_with_dialect`] when targeting SQLite (`?` placeholders).

use std::marker::PhantomData;

use super::condition::{Condition, JoinOp, OrderDir, SqlValue};

// ΓöÇΓöÇ Dialect ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

/// SQL placeholder dialect.
///
/// - [`Dialect::Postgres`] ΓÇö numbered placeholders (`$1`, `$2`, ΓÇª)
/// - [`Dialect::Sqlite | Dialect::Mysql`]  ΓÇö anonymous placeholders (`?`, `?`, ΓÇª)
/// - [`Dialect::Mysql`]   ΓÇö anonymous placeholders (`?`, `?`, ΓÇª)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dialect {
    #[default]
    Postgres,
    Sqlite,
    Mysql,
}

// ΓöÇΓöÇ Join ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

/// A fluent builder that produces parameterized SQL statements.
///
/// Conditions added with `where_*` methods are joined with `AND`.
/// Use `or_where_*` variants to join with `OR`.
///
/// # Example
///
/// ```rust
/// use rok_orm_core::{QueryBuilder, SqlValue};
///
/// let (sql, params) = QueryBuilder::<()>::new("users")
///     .where_eq("active", true)
///     .or_where_eq("role", "admin")
///     .order_by_desc("created_at")
///     .limit(20)
///     .offset(40)
///     .to_sql();
///
/// assert!(sql.contains("WHERE"));
/// assert!(sql.contains("ORDER BY created_at DESC"));
/// assert!(sql.contains("LIMIT 20"));
/// assert!(sql.contains("OFFSET 40"));
/// assert_eq!(params.len(), 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftDeleteMode {
    Exclude,
    Include,
    Only,
}

#[derive(Debug)]
pub struct QueryBuilder<T> {
    table: String,
    select_cols: Option<Vec<String>>,
    distinct: bool,
    joins: Vec<Join>,
    conditions: Vec<(JoinOp, Condition)>,
    group_by: Vec<String>,
    having: Option<String>,
    order: Vec<(String, OrderDir)>,
    limit_val: Option<usize>,
    offset_val: Option<usize>,
    soft_delete_mode: SoftDeleteMode,
    soft_delete_column: Option<String>,
    update_columns: Vec<(String, SqlValue)>,
    eager_loads: Vec<String>,
    _marker: PhantomData<T>,
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

    // ΓöÇΓöÇ column selection ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    pub fn select(mut self, cols: &[&str]) -> Self {
        self.select_cols = Some(cols.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Emit `SELECT DISTINCT ΓÇª`.
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    // ΓöÇΓöÇ joins ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    /// Add an `INNER JOIN table ON condition`.
    ///
    /// ```rust
    /// use rok_orm_core::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("orders")
    ///     .inner_join("users", "users.id = orders.user_id")
    ///     .select(&["orders.id", "users.name"])
    ///     .to_sql();
    ///
    /// assert!(sql.contains("INNER JOIN users ON users.id = orders.user_id"));
    /// ```
    pub fn inner_join(mut self, table: &str, on: &str) -> Self {
        self.joins
            .push(Join::Inner(table.to_string(), on.to_string()));
        self
    }

    /// Add a `LEFT JOIN table ON condition`.
    pub fn left_join(mut self, table: &str, on: &str) -> Self {
        self.joins
            .push(Join::Left(table.to_string(), on.to_string()));
        self
    }

    /// Add a `RIGHT JOIN table ON condition`.
    pub fn right_join(mut self, table: &str, on: &str) -> Self {
        self.joins
            .push(Join::Right(table.to_string(), on.to_string()));
        self
    }

    // ΓöÇΓöÇ GROUP BY / HAVING ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    /// Add a `GROUP BY` clause.
    ///
    /// ```rust
    /// use rok_orm_core::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("orders")
    ///     .select(&["user_id", "COUNT(*) as total"])
    ///     .group_by(&["user_id"])
    ///     .having("COUNT(*) > 5")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("GROUP BY user_id"));
    /// assert!(sql.contains("HAVING COUNT(*) > 5"));
    /// ```
    pub fn group_by(mut self, cols: &[&str]) -> Self {
        self.group_by = cols.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a `HAVING` clause (requires [`group_by`]).
    pub fn having(mut self, expr: &str) -> Self {
        self.having = Some(expr.to_string());
        self
    }

    // ΓöÇΓöÇ AND conditions ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    pub fn filter(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.where_eq(col, val)
    }

    pub fn eq(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.where_eq(col, val)
    }

    pub fn where_eq(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.push(JoinOp::And, Condition::Eq(col.into(), val.into()))
    }

    pub fn where_ne(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.push(JoinOp::And, Condition::Ne(col.into(), val.into()))
    }

    pub fn where_gt(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.push(JoinOp::And, Condition::Gt(col.into(), val.into()))
    }

    pub fn where_gte(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.push(JoinOp::And, Condition::Gte(col.into(), val.into()))
    }

    pub fn where_lt(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.push(JoinOp::And, Condition::Lt(col.into(), val.into()))
    }

    pub fn where_lte(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.push(JoinOp::And, Condition::Lte(col.into(), val.into()))
    }

    pub fn where_like(self, col: &str, pattern: &str) -> Self {
        self.push(JoinOp::And, Condition::Like(col.into(), pattern.into()))
    }

    pub fn where_not_like(self, col: &str, pattern: &str) -> Self {
        self.push(JoinOp::And, Condition::NotLike(col.into(), pattern.into()))
    }

    pub fn where_null(self, col: &str) -> Self {
        self.push(JoinOp::And, Condition::IsNull(col.into()))
    }

    pub fn where_not_null(self, col: &str) -> Self {
        self.push(JoinOp::And, Condition::IsNotNull(col.into()))
    }

    pub fn where_in(self, col: &str, vals: Vec<impl Into<SqlValue>>) -> Self {
        self.push(
            JoinOp::And,
            Condition::In(col.into(), vals.into_iter().map(Into::into).collect()),
        )
    }

    pub fn where_not_in(self, col: &str, vals: Vec<impl Into<SqlValue>>) -> Self {
        self.push(
            JoinOp::And,
            Condition::NotIn(col.into(), vals.into_iter().map(Into::into).collect()),
        )
    }

    pub fn where_between(
        self,
        col: &str,
        lo: impl Into<SqlValue>,
        hi: impl Into<SqlValue>,
    ) -> Self {
        self.push(
            JoinOp::And,
            Condition::Between(col.into(), lo.into(), hi.into()),
        )
    }

    pub fn where_not_between(
        self,
        col: &str,
        lo: impl Into<SqlValue>,
        hi: impl Into<SqlValue>,
    ) -> Self {
        self.push(
            JoinOp::And,
            Condition::NotBetween(col.into(), lo.into(), hi.into()),
        )
    }

    pub fn where_raw(self, sql: &str) -> Self {
        self.push(JoinOp::And, Condition::Raw(sql.into()))
    }

    // ΓöÇΓöÇ OR conditions ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    pub fn or_where_eq(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.push(JoinOp::Or, Condition::Eq(col.into(), val.into()))
    }

    pub fn or_where_ne(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.push(JoinOp::Or, Condition::Ne(col.into(), val.into()))
    }

    pub fn or_where_gt(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.push(JoinOp::Or, Condition::Gt(col.into(), val.into()))
    }

    pub fn or_where_gte(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.push(JoinOp::Or, Condition::Gte(col.into(), val.into()))
    }

    pub fn or_where_lt(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.push(JoinOp::Or, Condition::Lt(col.into(), val.into()))
    }

    pub fn or_where_lte(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.push(JoinOp::Or, Condition::Lte(col.into(), val.into()))
    }

    pub fn or_where_like(self, col: &str, pattern: &str) -> Self {
        self.push(JoinOp::Or, Condition::Like(col.into(), pattern.into()))
    }

    pub fn or_where_null(self, col: &str) -> Self {
        self.push(JoinOp::Or, Condition::IsNull(col.into()))
    }

    pub fn or_where_not_null(self, col: &str) -> Self {
        self.push(JoinOp::Or, Condition::IsNotNull(col.into()))
    }

    pub fn or_where_in(self, col: &str, vals: Vec<impl Into<SqlValue>>) -> Self {
        self.push(
            JoinOp::Or,
            Condition::In(col.into(), vals.into_iter().map(Into::into).collect()),
        )
    }

    pub fn or_where_between(
        self,
        col: &str,
        lo: impl Into<SqlValue>,
        hi: impl Into<SqlValue>,
    ) -> Self {
        self.push(
            JoinOp::Or,
            Condition::Between(col.into(), lo.into(), hi.into()),
        )
    }

    pub fn or_where_raw(self, sql: &str) -> Self {
        self.push(JoinOp::Or, Condition::Raw(sql.into()))
    }

    // ΓöÇΓöÇ ordering ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    pub fn order_by(mut self, col: &str) -> Self {
        self.order.push((col.into(), OrderDir::Asc));
        self
    }

    pub fn order_by_desc(mut self, col: &str) -> Self {
        self.order.push((col.into(), OrderDir::Desc));
        self
    }

    // ΓöÇΓöÇ pagination ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

    // ΓöÇΓöÇ aggregation ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

    // ΓöÇΓöÇ SQL generation ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    /// Build a parameterized `SELECT` statement (PostgreSQL `$N` placeholders).
    ///
    /// Returns `(sql, params)` ΓÇö params are ordered to match `$1`, `$2`, ΓÇª
    ///
    /// For SQLite use [`to_sql_with_dialect(Dialect::Sqlite | Dialect::Mysql)`](Self::to_sql_with_dialect).
    pub fn to_sql(&self) -> (String, Vec<SqlValue>) {
        self.to_sql_with_dialect(Dialect::Postgres)
    }

    /// Build a parameterized `SELECT` statement for the given [`Dialect`].
    ///
    /// - [`Dialect::Postgres`] emits `$1, $2, ΓÇª`
    /// - [`Dialect::Sqlite | Dialect::Mysql`]   emits `?, ?, ΓÇª`
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

    /// Build a `SELECT COUNT(*) FROM ΓÇª` statement (PostgreSQL dialect).
    pub fn to_count_sql(&self) -> (String, Vec<SqlValue>) {
        self.to_count_sql_with_dialect(Dialect::Postgres)
    }

    /// Build a `SELECT COUNT(*) FROM ΓÇª` statement for the given dialect.
    pub fn to_count_sql_with_dialect(&self, dialect: Dialect) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let joins = self.build_joins();
        let where_clause = self.build_where_with_soft_delete(dialect, &mut params);
        (
            format!(
                "SELECT COUNT(*) FROM {}{}{}",
                self.table, joins, where_clause
            ),
            params,
        )
    }

    /// Build a `DELETE FROM ΓÇª WHERE ΓÇª` statement (PostgreSQL dialect).
    pub fn to_delete_sql(&self) -> (String, Vec<SqlValue>) {
        self.to_delete_sql_with_dialect(Dialect::Postgres)
    }

    /// Build a `DELETE FROM ΓÇª WHERE ΓÇª` statement for the given dialect.
    pub fn to_delete_sql_with_dialect(&self, dialect: Dialect) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let where_clause = self.build_where_dialect(dialect, &mut params);
        (
            format!("DELETE FROM {}{}", self.table, where_clause),
            params,
        )
    }

    /// Build an `UPDATE ΓÇª SET ΓÇª WHERE ΓÇª` statement (PostgreSQL dialect).
    pub fn to_update_sql(&self, data: &[(&str, SqlValue)]) -> (String, Vec<SqlValue>) {
        self.to_update_sql_with_dialect(Dialect::Postgres, data)
    }

    /// Build an `UPDATE ΓÇª SET ΓÇª WHERE ΓÇª` statement for the given dialect.
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

    // ΓöÇΓöÇ static helpers ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    /// Build an `INSERT INTO` statement (PostgreSQL `$N` placeholders).
    pub fn insert_sql(table: &str, data: &[(&str, SqlValue)]) -> (String, Vec<SqlValue>) {
        Self::insert_sql_with_dialect(Dialect::Postgres, table, data)
    }

    /// Build an `INSERT INTO` statement for the given dialect.
    pub fn insert_sql_with_dialect(
        dialect: Dialect,
        table: &str,
        data: &[(&str, SqlValue)],
    ) -> (String, Vec<SqlValue>) {
        let cols: Vec<&str> = data.iter().map(|(c, _)| *c).collect();
        let placeholders: Vec<String> = match dialect {
            Dialect::Postgres => (1..=data.len()).map(|i| format!("${i}")).collect(),
            Dialect::Sqlite | Dialect::Mysql => (0..data.len()).map(|_| "?".to_string()).collect(),
        };
        let params: Vec<SqlValue> = data.iter().map(|(_, v)| v.clone()).collect();
        (
            format!(
                "INSERT INTO {table} ({}) VALUES ({})",
                cols.join(", "),
                placeholders.join(", ")
            ),
            params,
        )
    }

    /// Build an `INSERT INTO ΓÇª VALUES ΓÇª, ΓÇª` statement for multiple rows.
    ///
    /// All rows must have the same columns in the same order as the first row.
    ///
    /// ```rust
    /// use rok_orm_core::{QueryBuilder, SqlValue};
    ///
    /// let rows: Vec<Vec<(&str, SqlValue)>> = vec![
    ///     vec![("name", "Alice".into()), ("email", "a@a.com".into())],
    ///     vec![("name", "Bob".into()),   ("email", "b@b.com".into())],
    /// ];
    /// let (sql, params) = QueryBuilder::<()>::bulk_insert_sql("users", &rows);
    /// assert!(sql.contains("($1, $2), ($3, $4)"));
    /// assert_eq!(params.len(), 4);
    /// ```
    pub fn bulk_insert_sql(table: &str, rows: &[Vec<(&str, SqlValue)>]) -> (String, Vec<SqlValue>) {
        assert!(
            !rows.is_empty(),
            "bulk_insert_sql requires at least one row"
        );
        let cols: Vec<&str> = rows[0].iter().map(|(c, _)| *c).collect();
        let mut params: Vec<SqlValue> = Vec::new();
        let mut value_groups: Vec<String> = Vec::new();
        let mut offset = 1usize;

        for row in rows {
            let placeholders: Vec<String> = (offset..offset + row.len())
                .map(|i| format!("${i}"))
                .collect();
            value_groups.push(format!("({})", placeholders.join(", ")));
            for (_, v) in row.iter() {
                params.push(v.clone());
            }
            offset += row.len();
        }

        (
            format!(
                "INSERT INTO {table} ({}) VALUES {}",
                cols.join(", "),
                value_groups.join(", ")
            ),
            params,
        )
    }

    /// Build an `UPDATE ΓÇª SET ΓÇª WHERE ΓÇª` statement from explicit conditions.
    ///
    /// Prefer [`to_update_sql`] when you already have a `QueryBuilder`.
    pub fn update_sql(
        table: &str,
        data: &[(&str, SqlValue)],
        conditions: &[(JoinOp, Condition)],
    ) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let set_clauses: Vec<String> = data
            .iter()
            .enumerate()
            .map(|(i, (col, val))| {
                params.push(val.clone());
                format!("{col} = ${}", i + 1)
            })
            .collect();

        let mut sql = format!("UPDATE {table} SET {}", set_clauses.join(", "));

        if !conditions.is_empty() {
            let where_frag = build_where_from(conditions, &mut params);
            sql.push_str(&where_frag);
        }

        (sql, params)
    }

    pub fn upsert_sql(
        table: &str,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> (String, Vec<SqlValue>) {
        let cols: Vec<&str> = data.iter().map(|(c, _)| *c).collect();
        let placeholders: Vec<String> = (1..=data.len()).map(|i| format!("${i}")).collect();
        let mut params: Vec<SqlValue> = data.iter().map(|(_, v)| v.clone()).collect();

        let update_clauses: Vec<String> = update_columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let _param_idx = params.len() + i + 1;
                format!("{col} = EXCLUDED.{col}")
            })
            .collect();

        let sql = format!(
            "INSERT INTO {table} ({}) VALUES ({}) ON CONFLICT ({conflict_column}) DO UPDATE SET {}",
            cols.join(", "),
            placeholders.join(", "),
            update_clauses.join(", ")
        );

        (sql, params)
    }

    pub fn upsert_sql_with_dialect(
        dialect: Dialect,
        table: &str,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> (String, Vec<SqlValue>) {
        let cols: Vec<&str> = data.iter().map(|(c, _)| *c).collect();
        let placeholders: Vec<String> = match dialect {
            Dialect::Postgres => (1..=data.len()).map(|i| format!("${i}")).collect(),
            Dialect::Sqlite | Dialect::Mysql => (0..data.len()).map(|_| "?".to_string()).collect(),
        };
        let params: Vec<SqlValue> = data.iter().map(|(_, v)| v.clone()).collect();

        let update_clauses: Vec<String> = update_columns
            .iter()
            .map(|col| format!("{col} = VALUES({col})"))
            .collect();

        let sql = match dialect {
            Dialect::Postgres => format!(
                "INSERT INTO {table} ({}) VALUES ({}) ON CONFLICT ({conflict_column}) DO UPDATE SET {}",
                cols.join(", "),
                placeholders.join(", "),
                update_clauses.join(", ")
            ),
            Dialect::Sqlite => format!(
                "INSERT INTO {table} ({}) VALUES ({}) ON CONFLICT ({conflict_column}) DO UPDATE SET {}",
                cols.join(", "),
                placeholders.join(", "),
                update_clauses.join(", ")
            ),
            Dialect::Mysql => format!(
                "INSERT INTO {table} ({}) VALUES ({}) ON DUPLICATE KEY UPDATE {}",
                cols.join(", "),
                placeholders.join(", "),
                update_clauses.join(", ")
            ),
        };

        (sql, params)
    }

    pub fn upsert_do_nothing_sql(
        dialect: Dialect,
        table: &str,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
    ) -> (String, Vec<SqlValue>) {
        let cols: Vec<&str> = data.iter().map(|(c, _)| *c).collect();
        let placeholders: Vec<String> = match dialect {
            Dialect::Postgres => (1..=data.len()).map(|i| format!("${i}")).collect(),
            Dialect::Sqlite | Dialect::Mysql => (0..data.len()).map(|_| "?".to_string()).collect(),
        };
        let params: Vec<SqlValue> = data.iter().map(|(_, v)| v.clone()).collect();

        let sql = match dialect {
            Dialect::Mysql => format!(
                "INSERT IGNORE INTO {table} ({}) VALUES ({})",
                cols.join(", "),
                placeholders.join(", ")
            ),
            _ => format!(
                "INSERT INTO {table} ({}) VALUES ({}) ON CONFLICT ({conflict_column}) DO NOTHING",
                cols.join(", "),
                placeholders.join(", ")
            ),
        };

        (sql, params)
    }

    pub fn insert_ignore_sql(
        dialect: Dialect,
        table: &str,
        data: &[(&str, SqlValue)],
    ) -> (String, Vec<SqlValue>) {
        let cols: Vec<&str> = data.iter().map(|(c, _)| *c).collect();
        let placeholders: Vec<String> = match dialect {
            Dialect::Postgres => (1..=data.len()).map(|i| format!("${i}")).collect(),
            Dialect::Sqlite | Dialect::Mysql => (0..data.len()).map(|_| "?".to_string()).collect(),
        };
        let params: Vec<SqlValue> = data.iter().map(|(_, v)| v.clone()).collect();

        let sql = match dialect {
            Dialect::Mysql => format!(
                "INSERT IGNORE INTO {table} ({}) VALUES ({})",
                cols.join(", "),
                placeholders.join(", ")
            ),
            _ => format!(
                "INSERT INTO {table} ({}) VALUES ({})",
                cols.join(", "),
                placeholders.join(", ")
            ),
        };

        (sql, params)
    }

    pub fn delete_in_sql(&self, column: &str, values: &[SqlValue]) -> (String, Vec<SqlValue>) {
        self.to_delete_in_sql_with_dialect(Dialect::Postgres, column, values)
    }

    pub fn to_delete_in_sql_with_dialect(
        &self,
        dialect: Dialect,
        column: &str,
        values: &[SqlValue],
    ) -> (String, Vec<SqlValue>) {
        let params = values.to_vec();
        let placeholders: Vec<String> = match dialect {
            Dialect::Postgres => (1..=values.len()).map(|i| format!("${}", i)).collect(),
            Dialect::Sqlite | Dialect::Mysql => {
                (0..values.len()).map(|_| "?".to_string()).collect()
            }
        };

        let sql = format!(
            "DELETE FROM {} WHERE {} IN ({})",
            self.table,
            column,
            placeholders.join(", ")
        );

        (sql, params)
    }

    pub fn update_batch_sql(
        table: &str,
        id_column: &str,
        updates: &[(i64, &str, SqlValue)],
    ) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let mut set_clauses: Vec<String> = Vec::new();
        let mut where_clauses: Vec<String> = Vec::new();
        let mut param_offset = 0;

        for (id, _column, value) in updates {
            params.push(value.clone());
            param_offset += 1;
            set_clauses.push(format!("WHEN ${} THEN ${}", param_offset, param_offset));
            where_clauses.push(format!("${}", updates.len() + param_offset));
            params.push(SqlValue::Integer(*id));
        }

        let columns: Vec<&str> = updates.iter().map(|(_, col, _)| *col).collect();
        let case_sql = if !set_clauses.is_empty() {
            let cases: Vec<String> = columns
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    let cases: Vec<String> = updates
                        .iter()
                        .enumerate()
                        .map(|(j, (_, _, _))| {
                            let param_idx = j + 1;
                            let val_idx = updates.len() + j * 2 + 1 + i;
                            format!("WHEN ${} THEN ${}", param_idx, val_idx)
                        })
                        .collect();
                    format!("{} = CASE {} END", col, cases.join(" "))
                })
                .collect();
            cases.join(", ")
        } else {
            String::new()
        };

        let sql = if !case_sql.is_empty() {
            format!(
                "UPDATE {table} SET {case_sql} WHERE {id_column} IN ({})",
                where_clauses.join(", ")
            )
        } else {
            format!("DELETE FROM {table} WHERE 1=0")
        };

        (sql, params)
    }

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

    // ΓöÇΓöÇ internals ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    fn push(mut self, op: JoinOp, cond: Condition) -> Self {
        self.conditions.push((op, cond));
        self
    }

    fn build_joins(&self) -> String {
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

    fn build_where_dialect(&self, dialect: Dialect, params: &mut Vec<SqlValue>) -> String {
        build_where_from_dialect(dialect, &self.conditions, params)
    }

    fn build_where_with_soft_delete(&self, dialect: Dialect, params: &mut Vec<SqlValue>) -> String {
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

        build_where_from_dialect(dialect, &conditions, params)
    }

    fn build_group_by(&self) -> String {
        let mut out = String::new();
        if !self.group_by.is_empty() {
            out.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }
        if let Some(ref h) = self.having {
            out.push_str(&format!(" HAVING {h}"));
        }
        out
    }

    fn build_order(&self) -> String {
        if self.order.is_empty() {
            return String::new();
        }
        let parts: Vec<String> = self
            .order
            .iter()
            .map(|(col, dir)| format!("{col} {dir}"))
            .collect();
        format!(" ORDER BY {}", parts.join(", "))
    }

    /// Expose the raw conditions (useful for callers that need to inspect them).
    pub fn conditions(&self) -> &[(JoinOp, Condition)] {
        &self.conditions
    }
}

fn build_where_from(conditions: &[(JoinOp, Condition)], params: &mut Vec<SqlValue>) -> String {
    build_where_from_dialect(Dialect::Postgres, conditions, params)
}

fn build_where_from_dialect(
    dialect: Dialect,
    conditions: &[(JoinOp, Condition)],
    params: &mut Vec<SqlValue>,
) -> String {
    if conditions.is_empty() {
        return String::new();
    }
    let mut out = " WHERE ".to_string();
    for (idx, (op, cond)) in conditions.iter().enumerate() {
        let (frag, ps) = match dialect {
            Dialect::Postgres => cond.to_param_sql(params.len() + 1),
            Dialect::Sqlite | Dialect::Mysql => cond.to_param_sql_sqlite(),
        };
        params.extend(ps);
        if idx > 0 {
            out.push(' ');
            out.push_str(&op.to_string());
            out.push(' ');
        }
        out.push_str(&frag);
    }
    out
}

// ΓöÇΓöÇ tests ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_select() {
        let (sql, params) = QueryBuilder::<()>::new("users").to_sql();
        assert_eq!(sql, "SELECT * FROM users");
        assert!(params.is_empty());
    }

    #[test]
    fn distinct_select() {
        let (sql, _) = QueryBuilder::<()>::new("users").distinct().to_sql();
        assert!(sql.starts_with("SELECT DISTINCT * FROM users"));
    }

    #[test]
    fn where_eq_generates_param() {
        let (sql, params) = QueryBuilder::<()>::new("users")
            .where_eq("id", 42i64)
            .to_sql();
        assert!(sql.contains("WHERE id = $1"));
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], SqlValue::Integer(42));
    }

    #[test]
    fn multiple_conditions() {
        let (sql, params) = QueryBuilder::<()>::new("posts")
            .where_eq("active", true)
            .where_like("title", "%rust%")
            .to_sql();
        assert!(sql.contains("WHERE active = $1 AND title LIKE $2"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn or_conditions() {
        let (sql, params) = QueryBuilder::<()>::new("users")
            .where_eq("role", "admin")
            .or_where_eq("role", "moderator")
            .to_sql();
        assert!(sql.contains("WHERE role = $1 OR role = $2"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn where_between() {
        let (sql, params) = QueryBuilder::<()>::new("orders")
            .where_between("amount", 10i64, 100i64)
            .to_sql();
        assert!(sql.contains("amount BETWEEN $1 AND $2"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn where_not_in() {
        let (sql, params) = QueryBuilder::<()>::new("users")
            .where_not_in("status", vec!["banned", "deleted"])
            .to_sql();
        assert!(sql.contains("status NOT IN ($1, $2)"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn where_not_like() {
        let (sql, _) = QueryBuilder::<()>::new("users")
            .where_not_like("email", "%@spam.com")
            .to_sql();
        assert!(sql.contains("email NOT LIKE $1"));
    }

    #[test]
    fn to_update_sql() {
        let (sql, params) = QueryBuilder::<()>::new("users")
            .where_eq("id", 1i64)
            .to_update_sql(&[("name", "Bob".into()), ("active", true.into())]);
        assert!(sql.starts_with("UPDATE users SET name = $1, active = $2"));
        assert!(sql.contains("WHERE id = $3"));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn order_limit_offset() {
        let (sql, _) = QueryBuilder::<()>::new("users")
            .order_by_desc("created_at")
            .order_by("name")
            .limit(10)
            .offset(20)
            .to_sql();
        assert!(sql.contains("ORDER BY created_at DESC, name ASC"));
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 20"));
    }

    #[test]
    fn count_sql() {
        let (sql, _) = QueryBuilder::<()>::new("users")
            .where_eq("active", true)
            .to_count_sql();
        assert!(sql.starts_with("SELECT COUNT(*) FROM users"));
    }

    #[test]
    fn delete_sql() {
        let (sql, params) = QueryBuilder::<()>::new("sessions")
            .where_eq("user_id", 5i64)
            .to_delete_sql();
        assert!(sql.contains("DELETE FROM sessions WHERE user_id = $1"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn insert_sql() {
        let (sql, params) = QueryBuilder::<()>::insert_sql(
            "users",
            &[("name", "Alice".into()), ("email", "a@a.com".into())],
        );
        assert!(sql.contains("INSERT INTO users (name, email) VALUES ($1, $2)"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn where_in() {
        let (sql, params) = QueryBuilder::<()>::new("users")
            .where_in("id", vec![1i64, 2, 3])
            .to_sql();
        assert!(sql.contains("id IN ($1, $2, $3)"));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn select_specific_columns() {
        let (sql, _) = QueryBuilder::<()>::new("users")
            .select(&["id", "email"])
            .to_sql();
        assert!(sql.starts_with("SELECT id, email FROM users"));
    }

    #[test]
    fn option_value_null() {
        let val: SqlValue = Option::<i64>::None.into();
        assert_eq!(val, SqlValue::Null);
    }

    #[test]
    fn option_value_some() {
        let val: SqlValue = Some(42i64).into();
        assert_eq!(val, SqlValue::Integer(42));
    }

    #[test]
    fn inner_join() {
        let (sql, _) = QueryBuilder::<()>::new("orders")
            .inner_join("users", "users.id = orders.user_id")
            .to_sql();
        assert!(sql.contains("INNER JOIN users ON users.id = orders.user_id"));
    }

    #[test]
    fn left_join_with_where() {
        let (sql, params) = QueryBuilder::<()>::new("orders")
            .left_join("users", "users.id = orders.user_id")
            .where_eq("orders.status", "paid")
            .to_sql();
        assert!(sql.contains("LEFT JOIN users ON users.id = orders.user_id"));
        assert!(sql.contains("WHERE orders.status = $1"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn right_join() {
        let (sql, _) = QueryBuilder::<()>::new("orders")
            .right_join("products", "products.id = orders.product_id")
            .to_sql();
        assert!(sql.contains("RIGHT JOIN products ON products.id = orders.product_id"));
    }

    #[test]
    fn group_by_and_having() {
        let (sql, _) = QueryBuilder::<()>::new("orders")
            .select(&["user_id", "COUNT(*) as total"])
            .group_by(&["user_id"])
            .having("COUNT(*) > 5")
            .to_sql();
        assert!(sql.contains("GROUP BY user_id"));
        assert!(sql.contains("HAVING COUNT(*) > 5"));
        // GROUP BY must come before ORDER BY
        let gpos = sql.find("GROUP BY").unwrap();
        let hpos = sql.find("HAVING").unwrap();
        assert!(gpos < hpos);
    }

    #[test]
    fn count_sql_with_join() {
        let (sql, _) = QueryBuilder::<()>::new("orders")
            .inner_join("users", "users.id = orders.user_id")
            .where_eq("users.active", true)
            .to_count_sql();
        assert!(sql.contains("INNER JOIN users ON users.id = orders.user_id"));
        assert!(sql.contains("SELECT COUNT(*) FROM orders"));
    }

    #[test]
    fn bulk_insert_sql_two_rows() {
        let rows: Vec<Vec<(&str, SqlValue)>> = vec![
            vec![("name", "Alice".into()), ("email", "a@a.com".into())],
            vec![("name", "Bob".into()), ("email", "b@b.com".into())],
        ];
        let (sql, params) = QueryBuilder::<()>::bulk_insert_sql("users", &rows);
        assert!(sql.starts_with("INSERT INTO users (name, email) VALUES"));
        assert!(sql.contains("($1, $2), ($3, $4)"));
        assert_eq!(params.len(), 4);
    }

    #[test]
    fn bulk_insert_sql_single_row() {
        let rows = vec![vec![("x", SqlValue::Integer(1))]];
        let (sql, params) = QueryBuilder::<()>::bulk_insert_sql("t", &rows);
        assert!(sql.contains("($1)"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn soft_delete_auto_filter() {
        let (sql, _) = QueryBuilder::<()>::new("posts")
            .with_soft_delete("deleted_at")
            .to_sql();
        assert!(sql.contains("WHERE deleted_at IS NULL"));
    }

    #[test]
    fn with_trashed_includes_deleted() {
        let (sql, _) = QueryBuilder::<()>::new("posts")
            .with_soft_delete("deleted_at")
            .with_trashed()
            .to_sql();
        assert!(!sql.contains("WHERE"));
        assert!(sql.contains("SELECT * FROM posts"));
    }

    #[test]
    fn only_trashed_filters_deleted_only() {
        let (sql, _) = QueryBuilder::<()>::new("posts")
            .with_soft_delete("deleted_at")
            .only_trashed()
            .to_sql();
        assert!(sql.contains("WHERE deleted_at IS NOT NULL"));
    }

    #[test]
    fn soft_delete_with_conditions() {
        let (sql, params) = QueryBuilder::<()>::new("posts")
            .with_soft_delete("deleted_at")
            .where_eq("author_id", 42i64)
            .to_sql();
        assert!(sql.contains("WHERE author_id = $1 AND deleted_at IS NULL"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn restore_sql() {
        let (sql, params) = QueryBuilder::<()>::new("posts")
            .with_soft_delete("deleted_at")
            .where_eq("id", 1i64)
            .push_update_column("deleted_at", SqlValue::Null)
            .to_restore_sql();
        assert!(sql.starts_with("UPDATE posts SET deleted_at = $1"));
        assert!(sql.contains("WHERE id = $2"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn force_delete_sql_bypasses_soft_delete() {
        let (sql, params) = QueryBuilder::<()>::new("posts")
            .with_soft_delete("deleted_at")
            .where_eq("id", 1i64)
            .to_force_delete_sql();
        assert!(sql.starts_with("DELETE FROM posts"));
        assert!(sql.contains("WHERE id = $1"));
        assert!(!sql.contains("deleted_at"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn soft_delete_count_sql() {
        let (sql, _) = QueryBuilder::<()>::new("posts")
            .with_soft_delete("deleted_at")
            .to_count_sql();
        assert!(sql.starts_with("SELECT COUNT(*) FROM posts"));
        assert!(sql.contains("WHERE deleted_at IS NULL"));
    }
}
