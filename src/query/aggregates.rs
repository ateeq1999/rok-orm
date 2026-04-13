//! Aggregate-SQL shortcuts for [`QueryBuilder`]: count, sum, avg, min, max, exists, pluck.
//!
//! These were split out of `sql_gen.rs` to keep that file under 300 lines.

use super::builder::{Dialect, QueryBuilder};
use super::condition::SqlValue;

impl<T> QueryBuilder<T> {
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
}
