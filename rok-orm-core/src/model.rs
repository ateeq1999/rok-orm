//! [`Model`] trait — implemented automatically by `#[derive(Model)]`.

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

    fn query() -> QueryBuilder<Self> {
        QueryBuilder::new(Self::table_name())
    }

    fn find(id: impl Into<crate::condition::SqlValue>) -> QueryBuilder<Self> {
        Self::query().where_eq(Self::primary_key(), id)
    }

    fn find_where(builder: QueryBuilder<Self>) -> QueryBuilder<Self> {
        builder
    }
}
