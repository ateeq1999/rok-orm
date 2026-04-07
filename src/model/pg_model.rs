//! [`PgModel`] — core CRUD methods for any [`Model`] + [`sqlx::FromRow`] (PostgreSQL).
//!
//! For aggregates, pagination, upsert, and advanced queries see [`PgModelExt`].

use std::fmt;
use std::fmt::Display;

use chrono::Utc;
use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};
use sqlx::{postgres::PgRow, PgPool};

use crate::executor::postgres;

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

// ── PgModel ──────────────────────────────────────────────────────────────────

#[allow(async_fn_in_trait)]
pub trait PgModel: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin {
    fn all(
        pool: &PgPool,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::fetch_all(pool, Self::query())
    }

    fn find_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::fetch_all(pool, builder)
    }

    fn find_by_pk(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<Option<Self>, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::fetch_optional(pool, Self::find(id))
    }

    async fn find_or_404(pool: &PgPool, id: impl Into<SqlValue> + Send) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        postgres::fetch_optional(pool, Self::find(id))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn first(pool: &PgPool) -> Result<Option<Self>, sqlx::Error>
    where Self: Sized,
    {
        postgres::fetch_optional(pool, Self::query().limit(1)).await
    }

    async fn first_or_404(pool: &PgPool) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        postgres::fetch_optional(pool, Self::query().limit(1))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn get(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error>
    where Self: Sized,
    {
        postgres::fetch_all(pool, Self::query()).await
    }

    async fn get_where(pool: &PgPool, builder: QueryBuilder<Self>) -> Result<Vec<Self>, sqlx::Error>
    where Self: Sized,
    {
        postgres::fetch_all(pool, builder).await
    }

    async fn count(pool: &PgPool) -> Result<i64, sqlx::Error>
    where Self: Sized,
    {
        postgres::count(pool, Self::query()).await
    }

    async fn count_where(pool: &PgPool, builder: QueryBuilder<Self>) -> Result<i64, sqlx::Error>
    where Self: Sized,
    {
        postgres::count(pool, builder).await
    }

    fn create(
        pool: &PgPool,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::insert::<Self>(pool, Self::table_name(), data)
    }

    async fn update_by_pk(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        let mut d = data.to_vec();
        if Self::timestamps_enabled() {
            if let Some(col) = Self::updated_at_column() {
                d.push((col, SqlValue::Text(Utc::now().to_rfc3339())));
            }
        }
        postgres::update::<Self>(pool, Self::find(id), &d).await
    }

    fn delete_by_pk(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::delete(pool, Self::find(id))
    }

    fn delete_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::delete(pool, builder)
    }

    fn update_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::update::<Self>(pool, builder, data)
    }

    fn bulk_create(
        pool: &PgPool,
        rows: &[Vec<(&str, SqlValue)>],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::bulk_insert::<Self>(pool, Self::table_name(), rows)
    }

    async fn create_returning(pool: &PgPool, data: &[(&str, SqlValue)]) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        let mut d = data.to_vec();
        if Self::timestamps_enabled() {
            if let Some(col) = Self::created_at_column() {
                d.push((col, SqlValue::Text(Utc::now().to_rfc3339())));
            }
            if let Some(col) = Self::updated_at_column() {
                d.push((col, SqlValue::Text(Utc::now().to_rfc3339())));
            }
        }
        postgres::insert_returning::<Self>(pool, Self::table_name(), &d).await
    }

    fn restore(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::restore::<Self>(pool, Self::find(id))
    }

    fn restore_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::restore::<Self>(pool, builder)
    }

    fn force_delete(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::force_delete(pool, Self::find(id))
    }

    fn force_delete_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::force_delete(pool, builder)
    }
}

impl<T> PgModel for T where T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin {}
