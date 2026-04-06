//! Eager loading support for preventing N+1 queries.
//!
//! Use `.with()` to preload relations in a single query (or batched queries).
//!
//! ```rust,ignore
//! use rok_orm::{Model, PgModel};
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
//! }
//!
//! // Without eager loading (N+1):
//! for user in users {
//!     let posts = user.posts().get(&pool).await; // Extra query per user!
//! }
//!
//! // With eager loading (1 query):
//! let users = User::query()
//!     .with("posts")
//!     .get(&pool)
//!     .await?;
//! ```

use std::marker::PhantomData;

use crate::{Model, QueryBuilder, SqlValue};

#[derive(Debug, Clone)]
pub struct HasManyEager<P> {
    pub child_table: &'static str,
    pub foreign_key: String,
    pub child_pk: &'static str,
    _phantom: PhantomData<P>,
}

#[derive(Debug, Clone)]
pub struct HasOneEager<P> {
    pub child_table: &'static str,
    pub foreign_key: String,
    _phantom: PhantomData<P>,
}

#[derive(Debug, Clone)]
pub struct BelongsToEager<P> {
    pub parent_table: &'static str,
    pub foreign_key: String,
    pub related_table: &'static str,
    pub related_pk: &'static str,
    _phantom: PhantomData<P>,
}

impl<P> HasManyEager<P> {
    pub fn new(child_table: &'static str, foreign_key: String, child_pk: &'static str) -> Self {
        Self {
            child_table,
            foreign_key,
            child_pk,
            _phantom: PhantomData,
        }
    }

    pub fn build_query<C: Model>(&self, parent_ids: &[SqlValue]) -> QueryBuilder<C> {
        if parent_ids.is_empty() {
            return QueryBuilder::new(self.child_table).limit(0);
        }
        QueryBuilder::<C>::new(self.child_table).where_in(&self.foreign_key, parent_ids.to_vec())
    }
}

impl<P> HasOneEager<P> {
    pub fn new(child_table: &'static str, foreign_key: String) -> Self {
        Self {
            child_table,
            foreign_key,
            _phantom: PhantomData,
        }
    }

    pub fn build_query<C: Model>(&self, parent_ids: &[SqlValue]) -> QueryBuilder<C> {
        if parent_ids.is_empty() {
            return QueryBuilder::new(self.child_table).limit(0);
        }
        QueryBuilder::<C>::new(self.child_table).where_in(&self.foreign_key, parent_ids.to_vec())
    }
}

impl<P> BelongsToEager<P> {
    pub fn new(
        _parent_table: &'static str,
        foreign_key: String,
        related_table: &'static str,
        related_pk: &'static str,
    ) -> Self {
        Self {
            parent_table: _parent_table,
            foreign_key,
            related_table,
            related_pk,
            _phantom: PhantomData,
        }
    }

    pub fn foreign_key(&self) -> &str {
        &self.foreign_key
    }

    pub fn build_query<C: Model>(&self, parent_ids: &[SqlValue]) -> QueryBuilder<C> {
        if parent_ids.is_empty() {
            return QueryBuilder::new(self.related_table).limit(0);
        }
        QueryBuilder::<C>::new(self.related_table).where_in(self.related_pk, parent_ids.to_vec())
    }
}

#[derive(Debug, Clone)]
pub enum EagerRelation<P> {
    HasMany(HasManyEager<P>),
    HasOne(HasOneEager<P>),
    BelongsTo(BelongsToEager<P>),
}

impl<P> EagerRelation<P> {
    pub fn relation_name(&self) -> &'static str {
        match self {
            EagerRelation::HasMany(_) => "has_many",
            EagerRelation::HasOne(_) => "has_one",
            EagerRelation::BelongsTo(_) => "belongs_to",
        }
    }

    pub fn build_query<C: Model>(&self, parent_ids: &[SqlValue]) -> QueryBuilder<C> {
        match self {
            EagerRelation::HasMany(e) => e.build_query::<C>(parent_ids),
            EagerRelation::HasOne(e) => e.build_query::<C>(parent_ids),
            EagerRelation::BelongsTo(e) => e.build_query::<C>(parent_ids),
        }
    }
}
