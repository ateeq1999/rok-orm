//! [`SqliteModel`] — ergonomic async CRUD methods for any [`Model`] + [`sqlx::FromRow`] type,
//! backed by SQLite.
//!
//! All methods are provided as defaults; no manual implementation is required.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::{Model, SqliteModel, SqlValue};
//!
//! #[derive(Model, sqlx::FromRow)]
//! pub struct Note {
//!     pub id:    i64,
//!     pub title: String,
//!     pub body:  String,
//! }
//!
//! let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
//!
//! let all: Vec<Note>     = Note::all(&pool).await?;
//! let one: Option<Note>  = Note::find_by_pk(&pool, 1i64).await?;
//! let n:   i64           = Note::count(&pool).await?;
//! Note::create(&pool, &[("title", "Hello".into()), ("body", "World".into())]).await?;
//! Note::update_by_pk(&pool, 1i64, &[("title", "Updated".into())]).await?;
//! Note::delete_by_pk(&pool, 1i64).await?;
//! ```

use std::fmt;
use std::fmt::Display;

use rok_orm_core::{Model, QueryBuilder, SqlValue};
use sqlx::{sqlite::SqliteRow, SqlitePool};

use crate::sqlite_executor;

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

pub trait SqliteModel: Model + for<'r> sqlx::FromRow<'r, SqliteRow> + Send + Unpin {
    fn all(
        pool: &SqlitePool,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite_executor::fetch_all(pool, Self::query())
    }

    fn find_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite_executor::fetch_all(pool, builder)
    }

    fn find_by_pk(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<Option<Self>, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite_executor::fetch_optional(pool, Self::find(id))
    }

    async fn find_or_404(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite_executor::fetch_optional(pool, Self::find(id))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn first(pool: &SqlitePool) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite_executor::fetch_optional(pool, Self::query().limit(1))
    }

    async fn first_or_404(pool: &SqlitePool) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite_executor::fetch_optional(pool, Self::query().limit(1))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn get(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite_executor::fetch_all(pool, Self::query()).await
    }

    async fn get_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite_executor::fetch_all(pool, builder).await
    }

    async fn count(pool: &SqlitePool) -> Result<i64, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite_executor::count(pool, Self::query()).await
    }

    async fn count_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> Result<i64, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite_executor::count(pool, builder).await
    }

    fn create(
        pool: &SqlitePool,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite_executor::insert::<Self>(pool, Self::table_name(), data)
    }

    fn update_by_pk(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        let builder = Self::find(id);
        sqlite_executor::update::<Self>(pool, builder, data)
    }

    fn delete_by_pk(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite_executor::delete(pool, Self::find(id))
    }

    fn delete_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite_executor::delete(pool, builder)
    }

    fn update_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite_executor::update::<Self>(pool, builder, data)
    }

    fn create_returning(
        pool: &SqlitePool,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<Self, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite_executor::insert_returning::<Self>(pool, Self::table_name(), data)
    }
}

impl<T> SqliteModel for T where T: Model + for<'r> sqlx::FromRow<'r, SqliteRow> + Send + Unpin {}
