//! BelongsToMany relationship for many-to-many associations.
//!
//! Use with `#[derive(Relations)]` and the `#[model(belongs_to_many(...))]` attribute.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::{Model, Relations, BelongsToMany};
//!
//! #[derive(Model, Relations, sqlx::FromRow)]
//! pub struct Post {
//!     pub id: i64,
//!     pub title: String,
//!
//!     #[model(belongs_to_many(Tag))]
//!     pub tags: Vec<Tag>,
//! }
//!
//! #[derive(Model, sqlx::FromRow)]
//! pub struct Tag {
//!     pub id: i64,
//!     pub name: String,
//! }
//!
//! // In your code:
//! let post = Post::find_or_404(&pool, 1).await?;
//! let tags: Vec<Tag> = post.tags().get(&pool).await?;
//! ```

use std::marker::PhantomData;

use crate::{Model, QueryBuilder, SqlValue};

#[derive(Clone)]
pub struct BelongsToMany<P, C>
where
    P: Model,
    C: Model,
{
    parent_table: &'static str,
    parent_pk: &'static str,
    pivot_table: String,
    left_key: String,
    right_key: String,
    related_table: &'static str,
    related_pk: &'static str,
    _phantom: PhantomData<(P, C)>,
}

impl<P, C> BelongsToMany<P, C>
where
    P: Model,
    C: Model,
{
    pub fn new(
        parent_table: &'static str,
        parent_pk: &'static str,
        pivot_table: String,
        left_key: String,
        right_key: String,
        related_table: &'static str,
        related_pk: &'static str,
    ) -> Self {
        Self {
            parent_table,
            parent_pk,
            pivot_table,
            left_key,
            right_key,
            related_table,
            related_pk,
            _phantom: PhantomData,
        }
    }

    pub fn pivot_table(&self) -> &str {
        &self.pivot_table
    }

    pub fn left_key(&self) -> &str {
        &self.left_key
    }

    pub fn right_key(&self) -> &str {
        &self.right_key
    }

    pub fn query_for(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        QueryBuilder::<C>::new(self.related_table)
            .inner_join(
                &self.pivot_table,
                &format!(
                    "{}.{} = {}.{}",
                    self.related_table, self.related_pk, self.pivot_table, self.right_key
                ),
            )
            .where_eq(&self.pivot_table, parent_id)
    }

    pub fn get_sql_for(&self, parent_id: SqlValue) -> (String, Vec<SqlValue>) {
        self.query_for(parent_id).to_sql()
    }

    pub fn pivot_query(&self) -> QueryBuilder<()> {
        QueryBuilder::new(&self.pivot_table)
    }

    pub fn count_sql_for(&self, parent_id: SqlValue) -> (String, Vec<SqlValue>) {
        QueryBuilder::<()>::new(self.related_table)
            .inner_join(
                &self.pivot_table,
                &format!(
                    "{}.{} = {}.{}",
                    self.related_table, self.related_pk, self.pivot_table, self.right_key
                ),
            )
            .where_eq(&self.left_key, parent_id)
            .to_count_sql()
    }
}

impl<P, C> std::fmt::Debug for BelongsToMany<P, C>
where
    P: Model,
    C: Model,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BelongsToMany")
            .field("pivot_table", &self.pivot_table)
            .field("left_key", &self.left_key)
            .field("right_key", &self.right_key)
            .field("related_table", &self.related_table)
            .finish()
    }
}
