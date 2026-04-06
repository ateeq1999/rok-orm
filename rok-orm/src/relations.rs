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
//!
//! // Chainable relation queries
//! let posts = user.posts()
//!     .filter("published", true)
//!     .order_by_desc("created_at")
//!     .limit(10)
//!     .get(&pool)
//!     .await?;
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
    fn filter(mut self, col: &str, val: impl Into<SqlValue>) -> Self {
        self = self.where_eq(col, val);
        self
    }
    fn order_by(mut self, col: &str) -> Self {
        self = self.order_by(col);
        self
    }
    fn order_by_desc(mut self, col: &str) -> Self {
        self = self.order_by_desc(col);
        self
    }
    fn limit(mut self, n: usize) -> Self {
        self = self.limit(n);
        self
    }
    fn offset(mut self, n: usize) -> Self {
        self = self.offset(n);
        self
    }
    fn where_eq(mut self, col: &str, val: impl Into<SqlValue>) -> Self {
        self = self.where_eq(col, val);
        self
    }
    fn where_in(mut self, col: &str, vals: Vec<impl Into<SqlValue>>) -> Self {
        self = self.where_in(col, vals);
        self
    }
    fn where_between(
        mut self,
        col: &str,
        lo: impl Into<SqlValue>,
        hi: impl Into<SqlValue>,
    ) -> Self {
        self = self.where_between(col, lo, hi);
        self
    }
    fn where_null(mut self, col: &str) -> Self {
        self = self.where_null(col);
        self
    }
    fn where_not_null(mut self, col: &str) -> Self {
        self = self.where_not_null(col);
        self
    }
    fn where_like(mut self, col: &str, pattern: &str) -> Self {
        self = self.where_like(col, pattern);
        self
    }
    fn builder(&self) -> &QueryBuilder<C> {
        self
    }
    fn builder_mut(&mut self) -> &mut QueryBuilder<C> {
        self
    }
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

    pub fn foreign_key(&self) -> &str {
        &self.foreign_key
    }

    pub fn child_table(&self) -> &str {
        self.child_table
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

    fn foreign_key_value(&self, parent: &P) -> SqlValue {
        SqlValue::Null
    }
}

#[cfg(feature = "postgres")]
pub mod lazy {
    use sqlx::PgPool;

    use crate::{Model, executor};

    pub async fn load_has_many<P, C>(
        pool: &PgPool,
        relation: &super::HasMany<P, C>,
        parent_ids: &[i64],
    ) -> Result<Vec<C>, sqlx::Error>
    where
        P: Model,
        C: Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let builder = relation.query_for(crate::SqlValue::Null);
        let mut combined_builder = builder;
        
        for id in parent_ids {
            combined_builder = combined_builder.or_where_eq(relation.foreign_key(), *id);
        }
        
        executor::fetch_all(pool, combined_builder).await
    }

    pub async fn load_has_one<P, C>(
        pool: &PgPool,
        relation: &super::HasOne<P, C>,
        parent_id: i64,
    ) -> Result<Option<C>, sqlx::Error>
    where
        P: Model,
        C: Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let builder = relation.query_for(crate::SqlValue::Integer(parent_id));
        executor::fetch_optional(pool, builder).await
    }

    pub async fn load_belongs_to<P, C>(
        pool: &PgPool,
        relation: &super::BelongsTo<P, C>,
        parent: &P,
    ) -> Result<Option<C>, sqlx::Error>
    where
        P: Model,
        C: Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let fk_value = relation.foreign_key_value(parent);
        let builder = relation.query_for(fk_value);
        executor::fetch_optional(pool, builder).await
    }
}
