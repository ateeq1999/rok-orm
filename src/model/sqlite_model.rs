//! [`SqliteModel`] ΓÇö ergonomic async CRUD methods for any [`Model`] + [`sqlx::FromRow`] type,
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

use chrono::Utc;
use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};
use sqlx::{sqlite::SqliteRow, SqlitePool};

use crate::executor::sqlite;

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
        sqlite::fetch_all(pool, Self::query())
    }

    fn find_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite::fetch_all(pool, builder)
    }

    fn find_by_pk(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<Option<Self>, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite::fetch_optional(pool, Self::find(id))
    }

    async fn find_or_404(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::fetch_optional(pool, Self::find(id))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn first(pool: &SqlitePool) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::fetch_optional(pool, Self::query().limit(1))
    }

    async fn first_or_404(pool: &SqlitePool) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::fetch_optional(pool, Self::query().limit(1))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn get(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::fetch_all(pool, Self::query()).await
    }

    async fn get_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::fetch_all(pool, builder).await
    }

    async fn count(pool: &SqlitePool) -> Result<i64, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::count(pool, Self::query()).await
    }

    async fn count_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> Result<i64, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::count(pool, builder).await
    }

    fn create(
        pool: &SqlitePool,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite::insert::<Self>(pool, Self::table_name(), data)
    }

    async fn update_by_pk(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        let mut data_with_timestamps = data.to_vec();
        if Self::timestamps_enabled() {
            if let Some(col) = Self::updated_at_column() {
                data_with_timestamps.push((col, SqlValue::Text(Utc::now().to_rfc3339())));
            }
        }
        let builder = Self::find(id);
        sqlite::update::<Self>(pool, builder, &data_with_timestamps).await
    }

    fn delete_by_pk(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite::delete(pool, Self::find(id))
    }

    fn delete_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite::delete(pool, builder)
    }

    fn update_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite::update::<Self>(pool, builder, data)
    }

    async fn create_returning(
        pool: &SqlitePool,
        data: &[(&str, SqlValue)],
    ) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        let mut data_with_timestamps = data.to_vec();
        if Self::timestamps_enabled() {
            if let Some(col) = Self::created_at_column() {
                data_with_timestamps.push((col, SqlValue::Text(Utc::now().to_rfc3339())));
            }
            if let Some(col) = Self::updated_at_column() {
                data_with_timestamps.push((col, SqlValue::Text(Utc::now().to_rfc3339())));
            }
        }
        sqlite::insert_returning::<Self>(pool, Self::table_name(), &data_with_timestamps).await
    }

    fn restore(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite::restore::<Self>(pool, Self::find(id))
    }

    fn restore_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite::restore::<Self>(pool, builder)
    }

    fn force_delete(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite::force_delete(pool, Self::find(id))
    }

    fn force_delete_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        sqlite::force_delete(pool, builder)
    }

    async fn paginate(
        pool: &SqlitePool,
        page: i64,
        per_page: i64,
    ) -> Result<crate::pagination::Page<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        let total = sqlite::count(pool, Self::query()).await?;
        let data = sqlite::fetch_all(pool, Self::query().paginate(page, per_page)).await?;
        Ok(crate::pagination::Page::new(data, total, per_page, page))
    }

    async fn paginate_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
        page: i64,
        per_page: i64,
    ) -> Result<crate::pagination::Page<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        let total = sqlite::count(pool, builder.clone()).await?;
        let data = sqlite::fetch_all(pool, builder.paginate(page, per_page)).await?;
        Ok(crate::pagination::Page::new(data, total, per_page, page))
    }

    async fn sum(
        pool: &SqlitePool,
        column: &str,
    ) -> Result<Option<f64>, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::aggregate::<Self>(pool, Self::query(), "SUM", column).await
    }

    async fn avg(
        pool: &SqlitePool,
        column: &str,
    ) -> Result<Option<f64>, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::aggregate::<Self>(pool, Self::query(), "AVG", column).await
    }

    async fn min(
        pool: &SqlitePool,
        column: &str,
    ) -> Result<Option<f64>, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::aggregate::<Self>(pool, Self::query(), "MIN", column).await
    }

    async fn max(
        pool: &SqlitePool,
        column: &str,
    ) -> Result<Option<f64>, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::aggregate::<Self>(pool, Self::query(), "MAX", column).await
    }

    async fn upsert(
        pool: &SqlitePool,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::upsert::<Self>(pool, Self::table_name(), data, conflict_column, update_columns).await
    }

    async fn delete_in(
        pool: &SqlitePool,
        column: &str,
        values: Vec<SqlValue>,
    ) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::delete_in::<Self>(pool, column, values).await
    }

    async fn exists(pool: &SqlitePool) -> Result<bool, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::exists(pool, Self::query()).await
    }

    async fn exists_where(pool: &SqlitePool, builder: QueryBuilder<Self>) -> Result<bool, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::exists(pool, builder).await
    }

    async fn pluck(
        pool: &SqlitePool,
        column: &str,
    ) -> Result<Vec<SqlValue>, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::pluck(pool, Self::query(), column).await
    }

    async fn pluck_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
        column: &str,
    ) -> Result<Vec<SqlValue>, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::pluck(pool, builder, column).await
    }

    async fn update_all(
        pool: &SqlitePool,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::update_all(pool, Self::query(), data).await
    }

    async fn update_all_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        sqlite::update_all(pool, builder, data).await
    }
}

impl<T> SqliteModel for T where T: Model + for<'r> sqlx::FromRow<'r, SqliteRow> + Send + Unpin {}
