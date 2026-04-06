//! `where_*` and `or_where_*` condition methods for [`QueryBuilder`].

use super::builder::QueryBuilder;
use super::condition::{Condition, JoinOp, SqlValue};

impl<T> QueryBuilder<T> {
    // ── AND conditions ──────────────────────────────────────────────────────

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

    /// Add a raw `WHERE` fragment with bound parameters.
    ///
    /// Use `$1`, `$2`, … placeholders; they are rewritten to the correct
    /// offset automatically when combined with other conditions.
    pub fn where_raw_params(self, sql: &str, params: Vec<impl Into<SqlValue>>) -> Self {
        self.push(
            JoinOp::And,
            Condition::RawExpr(sql.into(), params.into_iter().map(Into::into).collect()),
        )
    }

    // ── OR conditions ───────────────────────────────────────────────────────

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
}
