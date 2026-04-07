//! Core relation traits: [`Relations`], [`Relation`], [`RelationQuery`].

use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};

/// Marker trait for models that declare relationships via `#[derive(Relations)]`.
///
/// The derive macro generates named relation accessor methods (e.g. `fn posts(&self) -> HasMany<...>`)
/// for each field annotated with `#[model(has_many(T))]`, `#[model(has_one(T))]`, etc.
pub trait Relations: Model {}

/// Type-erased interface to build a child query from a parent ID.
pub trait Relation<P: Model, C: Model> {
    fn query(&self, parent_id: SqlValue) -> QueryBuilder<C>;
    fn foreign_key_value(&self, parent: &P) -> SqlValue;
}

/// Fluent query API available on relation builders.
pub trait RelationQuery<C: Model> {
    fn filter(self, col: &str, val: impl Into<SqlValue>) -> Self;
    fn order_by(self, col: &str) -> Self;
    fn order_by_desc(self, col: &str) -> Self;
    fn limit(self, n: usize) -> Self;
    fn offset(self, n: usize) -> Self;
    fn where_eq(self, col: &str, val: impl Into<SqlValue>) -> Self;
    fn where_in(self, col: &str, vals: Vec<impl Into<SqlValue>>) -> Self;
    fn where_between(self, col: &str, lo: impl Into<SqlValue>, hi: impl Into<SqlValue>) -> Self;
    fn where_null(self, col: &str) -> Self;
    fn where_not_null(self, col: &str) -> Self;
    fn where_like(self, col: &str, pattern: &str) -> Self;
    fn builder(&self) -> &QueryBuilder<C>;
    fn builder_mut(&mut self) -> &mut QueryBuilder<C>;
}

impl<C: Model> RelationQuery<C> for QueryBuilder<C> {
    fn filter(self, col: &str, val: impl Into<SqlValue>) -> Self {
        self.where_eq(col, val)
    }
    fn order_by(self, col: &str) -> Self {
        QueryBuilder::order_by(self, col)
    }
    fn order_by_desc(self, col: &str) -> Self {
        QueryBuilder::order_by_desc(self, col)
    }
    fn limit(self, n: usize) -> Self {
        QueryBuilder::limit(self, n)
    }
    fn offset(self, n: usize) -> Self {
        QueryBuilder::offset(self, n)
    }
    fn where_eq(self, col: &str, val: impl Into<SqlValue>) -> Self {
        QueryBuilder::where_eq(self, col, val)
    }
    fn where_in(self, col: &str, vals: Vec<impl Into<SqlValue>>) -> Self {
        QueryBuilder::where_in(self, col, vals)
    }
    fn where_between(
        self,
        col: &str,
        lo: impl Into<SqlValue>,
        hi: impl Into<SqlValue>,
    ) -> Self {
        QueryBuilder::where_between(self, col, lo, hi)
    }
    fn where_null(self, col: &str) -> Self {
        QueryBuilder::where_null(self, col)
    }
    fn where_not_null(self, col: &str) -> Self {
        QueryBuilder::where_not_null(self, col)
    }
    fn where_like(self, col: &str, pattern: &str) -> Self {
        QueryBuilder::where_like(self, col, pattern)
    }
    fn builder(&self) -> &QueryBuilder<C> {
        self
    }
    fn builder_mut(&mut self) -> &mut QueryBuilder<C> {
        self
    }
}
