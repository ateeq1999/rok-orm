//! [`HasOne`] — one-to-one relationship (parent owns foreign key on child).

use std::marker::PhantomData;

use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};

use super::traits::Relation;

/// Represents a one-to-one association (`parent` → one optional `child` row).
#[derive(Debug, Clone)]
pub struct HasOne<P, C>
where
    P: Model,
    C: Model,
{
    #[allow(dead_code)]
    parent_table: &'static str,
    #[allow(dead_code)]
    parent_pk: &'static str,
    pub(crate) child_table: &'static str,
    pub(crate) foreign_key: String,
    _phantom: PhantomData<(P, C)>,
}

impl<P, C> HasOne<P, C>
where
    P: Model,
    C: Model,
{
    pub fn new(
        parent_table: &'static str,
        parent_pk: &'static str,
        child_table: &'static str,
        foreign_key: String,
    ) -> Self {
        Self {
            parent_table,
            parent_pk,
            child_table,
            foreign_key,
            _phantom: PhantomData,
        }
    }

    pub fn query_for(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        QueryBuilder::new(self.child_table).where_eq(&self.foreign_key, parent_id)
    }

    pub fn foreign_key(&self) -> &str {
        &self.foreign_key
    }

    pub fn child_table(&self) -> &str {
        self.child_table
    }
}

impl<P, C> Relation<P, C> for HasOne<P, C>
where
    P: Model,
    C: Model,
{
    fn query(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        self.query_for(parent_id)
    }

    fn foreign_key_value(&self, _parent: &P) -> SqlValue {
        SqlValue::Null
    }
}
