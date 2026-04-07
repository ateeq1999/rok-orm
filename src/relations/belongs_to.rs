//! [`BelongsTo`] — inverse of HasMany / HasOne; child owns the foreign key.

use std::marker::PhantomData;

use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};

use super::traits::Relation;

/// Represents the inverse side of a one-to-many association.
///
/// The child model owns the foreign key column that points at the parent.
#[derive(Debug, Clone)]
pub struct BelongsTo<P, C>
where
    P: Model,
    C: Model,
{
    #[allow(dead_code)]
    parent_table: &'static str,
    pub(crate) foreign_key: String,
    pub(crate) related_table: &'static str,
    pub(crate) related_pk: &'static str,
    _phantom: PhantomData<(P, C)>,
}

impl<P, C> BelongsTo<P, C>
where
    P: Model,
    C: Model,
{
    pub fn new(
        parent_table: &'static str,
        foreign_key: String,
        related_table: &'static str,
        related_pk: &'static str,
    ) -> Self {
        Self {
            parent_table,
            foreign_key,
            related_table,
            related_pk,
            _phantom: PhantomData,
        }
    }

    pub fn query_for(&self, fk_value: SqlValue) -> QueryBuilder<C> {
        QueryBuilder::<C>::new(self.related_table).where_eq(self.related_pk, fk_value)
    }

    pub fn foreign_key(&self) -> &str {
        &self.foreign_key
    }

    pub fn related_table(&self) -> &str {
        self.related_table
    }

    pub fn related_pk(&self) -> &str {
        self.related_pk
    }
}

impl<P, C> Relation<P, C> for BelongsTo<P, C>
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
