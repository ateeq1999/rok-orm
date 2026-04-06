//! Relationship definitions for rok-orm.
//!
//! Use `#[derive(Relations)]` to implement relationships on your models.
//!
//! ```rust,ignore
//! use rok_orm::{Model, Relations};
//!
//! #[derive(Model, Relations, sqlx::FromRow)]
//! pub struct User {
//!     pub id: i64,
//!     pub name: String,
//!     #[model(has_many(Post))]
//!     pub posts: Post,
//! }
//!
//! #[derive(Model, sqlx::FromRow)]
//! pub struct Post {
//!     pub id: i64,
//!     pub user_id: i64,
//!     pub title: String,
//!     #[model(belongs_to(User))]
//!     pub user: User,
//! }
//! ```

use std::marker::PhantomData;

use crate::{Model, QueryBuilder, SqlValue};

pub trait Relations: Model {
    fn has_many<T: Model>(&self) -> HasMany<Self, T>
    where
        Self: Sized;

    fn has_one<T: Model>(&self) -> HasOne<Self, T>
    where
        Self: Sized;

    fn belongs_to<T: Model>(&self) -> BelongsTo<Self, T>
    where
        Self: Sized;
}

pub trait Relation<P: Model, C: Model> {
    fn query(&self, parent_id: SqlValue) -> QueryBuilder<C>;

    fn foreign_key_value(&self, parent: &P) -> SqlValue;
}

#[derive(Debug, Clone)]
pub struct HasMany<P, C>
where
    P: Model,
    C: Model,
{
    parent_table: &'static str,
    parent_pk: &'static str,
    child_table: &'static str,
    child_pk: &'static str,
    foreign_key: String,
    _phantom: PhantomData<(P, C)>,
}

impl<P, C> HasMany<P, C>
where
    P: Model,
    C: Model,
{
    pub fn new(
        parent_table: &'static str,
        parent_pk: &'static str,
        child_table: &'static str,
        child_pk: &'static str,
        foreign_key: String,
    ) -> Self {
        Self {
            parent_table,
            parent_pk,
            child_table,
            child_pk,
            foreign_key,
            _phantom: PhantomData,
        }
    }

    pub fn query_for(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        QueryBuilder::new(self.child_table).where_eq(&self.foreign_key, parent_id)
    }
}

impl<P, C> Relation<P, C> for HasMany<P, C>
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

#[derive(Debug, Clone)]
pub struct HasOne<P, C>
where
    P: Model,
    C: Model,
{
    parent_table: &'static str,
    parent_pk: &'static str,
    child_table: &'static str,
    foreign_key: String,
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

#[derive(Debug, Clone)]
pub struct BelongsTo<P, C>
where
    P: Model,
    C: Model,
{
    parent_table: &'static str,
    foreign_key: String,
    related_table: &'static str,
    related_pk: &'static str,
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
}

impl<P, C> Relation<P, C> for BelongsTo<P, C>
where
    P: Model,
    C: Model,
{
    fn query(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        self.query_for(parent_id)
    }

    fn foreign_key_value(&self, parent: &P) -> SqlValue {
        SqlValue::Null
    }
}
