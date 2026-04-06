//! [`Model`] trait ΓÇö implemented automatically by `#[derive(Model)]`.

use crate::query::QueryBuilder;

pub trait Model: Sized {
    fn table_name() -> &'static str;

    fn primary_key() -> &'static str {
        "id"
    }

    fn columns() -> &'static [&'static str];

    fn soft_delete_column() -> Option<&'static str> {
        None
    }

    fn timestamps_enabled() -> bool {
        false
    }

    fn created_at_column() -> Option<&'static str> {
        None
    }

    fn updated_at_column() -> Option<&'static str> {
        None
    }

    fn query() -> QueryBuilder<Self> {
        let builder = QueryBuilder::new(Self::table_name());
        if let Some(col) = Self::soft_delete_column() {
            builder.with_soft_delete(col)
        } else {
            builder
        }
    }

    fn find(id: impl Into<crate::query::SqlValue>) -> QueryBuilder<Self> {
        Self::query().where_eq(Self::primary_key(), id)
    }

    fn find_where(builder: QueryBuilder<Self>) -> QueryBuilder<Self> {
        builder
    }
}
