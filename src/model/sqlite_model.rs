//! [`SqliteModel`] — core CRUD methods for any [`Model`] + [`sqlx::FromRow`] (SQLite).
//!
//! For aggregates, pagination, upsert, and advanced queries see [`SqliteModelExt`].

use std::fmt;
use std::fmt::Display;

use chrono::Utc;
use crate::model::{Model, model::timestamps_muted};
use crate::query::{QueryBuilder, SqlValue};
use sqlx::{sqlite::SqliteRow, SqlitePool};

use crate::executor::sqlite;

// ── error type ──────────────────────────────────────────────────────────────

#[allow(dead_code)]
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

// ── SqliteModel ──────────────────────────────────────────────────────────────

#[allow(async_fn_in_trait)]
pub trait SqliteModel: Model + for<'r> sqlx::FromRow<'r, SqliteRow> + Send + Unpin {
    fn all(
        pool: &SqlitePool,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where Self: Sized + Send + 'static,
    {
        sqlite::fetch_all(pool, Self::scoped_query())
    }

    fn find_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where Self: Sized,
    {
        sqlite::fetch_all(pool, builder)
    }

    fn find_by_pk(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<Option<Self>, sqlx::Error>> + Send
    where Self: Sized,
    {
        sqlite::fetch_optional(pool, Self::find(id))
    }

    async fn find_or_404(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        sqlite::fetch_optional(pool, Self::find(id))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn first(pool: &SqlitePool) -> Result<Option<Self>, sqlx::Error>
    where Self: Sized + Send + 'static,
    {
        sqlite::fetch_optional(pool, Self::scoped_query().limit(1)).await
    }

    async fn first_or_404(pool: &SqlitePool) -> Result<Self, sqlx::Error>
    where Self: Sized + Send + 'static,
    {
        sqlite::fetch_optional(pool, Self::scoped_query().limit(1))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn get(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error>
    where Self: Sized + Send + 'static,
    {
        sqlite::fetch_all(pool, Self::scoped_query()).await
    }

    async fn get_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> Result<Vec<Self>, sqlx::Error>
    where Self: Sized,
    {
        sqlite::fetch_all(pool, builder).await
    }

    async fn count(pool: &SqlitePool) -> Result<i64, sqlx::Error>
    where Self: Sized + Send + 'static,
    {
        sqlite::count(pool, Self::scoped_query()).await
    }

    async fn count_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> Result<i64, sqlx::Error>
    where Self: Sized,
    {
        sqlite::count(pool, builder).await
    }

    async fn create(
        pool: &SqlitePool,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        let mut d = Self::filter_fillable(data);
        if let Some(pk_val) = Self::new_unique_id() {
            d.insert(0, (Self::primary_key(), pk_val));
        }
        sqlite::insert::<Self>(pool, Self::table_name(), &d).await
    }

    async fn update_by_pk(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        let mut d = Self::filter_fillable(data);
        if Self::timestamps_enabled() && !timestamps_muted() {
            if let Some(col) = Self::updated_at_column() {
                d.push((col, SqlValue::Text(Utc::now().to_rfc3339())));
            }
        }
        sqlite::update::<Self>(pool, Self::find(id), &d).await
    }

    fn delete_by_pk(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        sqlite::delete(pool, Self::find(id))
    }

    fn delete_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        sqlite::delete(pool, builder)
    }

    fn update_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        sqlite::update::<Self>(pool, builder, data)
    }

    async fn create_returning(
        pool: &SqlitePool,
        data: &[(&str, SqlValue)],
    ) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        let mut d = Self::filter_fillable(data);
        if let Some(pk_val) = Self::new_unique_id() {
            d.insert(0, (Self::primary_key(), pk_val));
        }
        if Self::timestamps_enabled() && !timestamps_muted() {
            if let Some(col) = Self::created_at_column() {
                d.push((col, SqlValue::Text(Utc::now().to_rfc3339())));
            }
            if let Some(col) = Self::updated_at_column() {
                d.push((col, SqlValue::Text(Utc::now().to_rfc3339())));
            }
        }
        sqlite::insert_returning::<Self>(pool, Self::table_name(), &d).await
    }

    fn restore(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        sqlite::restore::<Self>(pool, Self::find(id))
    }

    fn restore_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        sqlite::restore::<Self>(pool, builder)
    }

    fn force_delete(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        sqlite::force_delete(pool, Self::find(id))
    }

    fn force_delete_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        sqlite::force_delete(pool, builder)
    }

    /// Fetch rows using a raw SQL string with `?` placeholders.
    fn from_raw_sql(
        pool: &SqlitePool,
        sql: &str,
        params: Vec<SqlValue>,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where Self: Sized,
    {
        sqlite::fetch_raw::<Self>(pool, sql, params)
    }

    /// Update this record with events muted (no observer hooks fired).
    async fn save_quietly(
        pool: &SqlitePool,
        id: impl Into<SqlValue> + Send,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        Self::without_events(|| async move {
            Self::update_by_pk(pool, id, data).await
        })
        .await
    }
}

impl<T> SqliteModel for T where T: Model + for<'r> sqlx::FromRow<'r, SqliteRow> + Send + Unpin {}
