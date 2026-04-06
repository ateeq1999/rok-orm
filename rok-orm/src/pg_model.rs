//! [`PgModel`] — ergonomic async CRUD methods for any [`Model`] + [`sqlx::FromRow`] type.
//!
//! All methods are provided as defaults; no manual implementation is required.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::{Model, PgModel, SqlValue};
//!
//! #[derive(Model, sqlx::FromRow)]
//! pub struct User {
//!     pub id: i64,
//!     pub name: String,
//! }
//!
//! let pool = sqlx::PgPool::connect(&url).await?;
//!
//! let all: Vec<User>    = User::all(&pool).await?;
//! let one: Option<User> = User::find_by_pk(&pool, 1i64).await?;
//! let n: i64            = User::count(&pool).await?;
//! User::create(&pool, &[("name", "Alice".into())]).await?;
//! User::update_by_pk(&pool, 1i64, &[("name", "Bob".into())]).await?;
//! User::delete_by_pk(&pool, 1i64).await?;
//! ```

use std::fmt;
use std::fmt::Display;

use rok_orm_core::{Model, QueryBuilder, SqlValue};
use sqlx::{postgres::PgRow, PgPool};

use crate::executor;

#[derive(Debug)]
pub struct NotFoundError {
    pub model: &'static str,
    pub id: String,
}

impl Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} with id '{}' not found", self.model, self.id)
    }
}

impl std::error::Error for NotFoundError {}

pub trait PgModel: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin {
    fn all(
        pool: &PgPool,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::fetch_all(pool, Self::query())
    }

    fn find_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::fetch_all(pool, builder)
    }

    fn find_by_pk(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<Option<Self>, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::fetch_optional(pool, Self::find(id))
    }

    async fn find_or_404(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
    ) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        executor::fetch_optional(pool, Self::find(id))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn first(pool: &PgPool) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        executor::fetch_optional(pool, Self::query().limit(1))
    }

    async fn first_or_404(pool: &PgPool) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        executor::fetch_optional(pool, Self::query().limit(1))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn get(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        executor::fetch_all(pool, Self::query()).await
    }

    async fn get_where(pool: &PgPool, builder: QueryBuilder<Self>) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        executor::fetch_all(pool, builder).await
    }

    async fn count(pool: &PgPool) -> Result<i64, sqlx::Error>
    where
        Self: Sized,
    {
        executor::count(pool, Self::query()).await
    }

    async fn count_where(pool: &PgPool, builder: QueryBuilder<Self>) -> Result<i64, sqlx::Error>
    where
        Self: Sized,
    {
        executor::count(pool, builder).await
    }

    fn create(
        pool: &PgPool,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::insert::<Self>(pool, Self::table_name(), data)
    }

    fn update_by_pk(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        let builder = Self::find(id);
        executor::update::<Self>(pool, builder, data)
    }

    fn delete_by_pk(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::delete(pool, Self::find(id))
    }

    fn delete_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::delete(pool, builder)
    }

    fn update_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::update::<Self>(pool, builder, data)
    }

    fn bulk_create(
        pool: &PgPool,
        rows: &[Vec<(&str, SqlValue)>],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::bulk_insert::<Self>(pool, Self::table_name(), rows)
    }

    fn create_returning(
        pool: &PgPool,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<Self, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::insert_returning::<Self>(pool, Self::table_name(), data)
    }
}

impl<T> PgModel for T where T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin {}
