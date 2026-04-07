//! [`PgModel`] — core CRUD methods for any [`Model`] + [`sqlx::FromRow`] (PostgreSQL).
//!
//! For aggregates, pagination, upsert, and advanced queries see [`PgModelExt`].

use chrono::Utc;
use crate::model::{Model, model::{timestamps_muted, events_muted}};
use crate::query::{QueryBuilder, SqlValue};
use sqlx::{postgres::PgRow, PgPool};

use crate::executor::postgres;

// ── PgModel ──────────────────────────────────────────────────────────────────

#[allow(async_fn_in_trait)]
pub trait PgModel: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin {
    fn all(
        pool: &PgPool,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where Self: Sized + Send + 'static,
    {
        postgres::fetch_all(pool, Self::scoped_query())
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
    where Self: Sized + Send + 'static,
    {
        postgres::fetch_optional(pool, Self::scoped_query().limit(1)).await
    }

    async fn first_or_404(pool: &PgPool) -> Result<Self, sqlx::Error>
    where Self: Sized + Send + 'static,
    {
        postgres::fetch_optional(pool, Self::scoped_query().limit(1))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn get(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error>
    where Self: Sized + Send + 'static,
    {
        postgres::fetch_all(pool, Self::scoped_query()).await
    }

    async fn get_where(pool: &PgPool, builder: QueryBuilder<Self>) -> Result<Vec<Self>, sqlx::Error>
    where Self: Sized,
    {
        postgres::fetch_all(pool, builder).await
    }

    async fn count(pool: &PgPool) -> Result<i64, sqlx::Error>
    where Self: Sized + Send + 'static,
    {
        postgres::count(pool, Self::scoped_query()).await
    }

    async fn count_where(pool: &PgPool, builder: QueryBuilder<Self>) -> Result<i64, sqlx::Error>
    where Self: Sized,
    {
        postgres::count(pool, builder).await
    }

    async fn create(
        pool: &PgPool,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        let mut d = Self::filter_fillable(data);
        if let Some(pk_val) = Self::new_unique_id() {
            d.insert(0, (Self::primary_key(), pk_val));
        }
        postgres::insert::<Self>(pool, Self::table_name(), &d).await
    }

    async fn update_by_pk(
        pool: &PgPool,
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

    async fn update_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        let d = Self::filter_fillable(data);
        postgres::update::<Self>(pool, builder, &d).await
    }

    async fn bulk_create(
        pool: &PgPool,
        rows: &[Vec<(&str, SqlValue)>],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        let filtered: Vec<Vec<(&str, SqlValue)>> = rows
            .iter()
            .map(|row| Self::filter_fillable(row))
            .collect();
        postgres::bulk_insert::<Self>(pool, Self::table_name(), &filtered).await
    }

    async fn create_returning(pool: &PgPool, data: &[(&str, SqlValue)]) -> Result<Self, sqlx::Error>
    where Self: Sized + 'static,
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
        let row = postgres::insert_returning::<Self>(pool, Self::table_name(), &d).await?;
        if !events_muted() {
            use crate::observer::{ObserverRegistry, ObserverEvent};
            ObserverRegistry::dispatch::<Self>(&row, ObserverEvent::Created);
            ObserverRegistry::dispatch::<Self>(&row, ObserverEvent::Saved);
        }
        Ok(row)
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

    /// Atomically increment a column by `delta` for a given PK.
    async fn increment(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
        column: &str,
        delta: i64,
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        let table = Self::table_name();
        let pk = Self::primary_key();
        let sql = format!("UPDATE {table} SET {column} = {column} + $1 WHERE {pk} = $2");
        postgres::execute_raw(pool, &sql, vec![SqlValue::Integer(delta), id.into()]).await
    }

    /// Atomically decrement a column by `delta` for a given PK.
    async fn decrement(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
        column: &str,
        delta: i64,
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        let table = Self::table_name();
        let pk = Self::primary_key();
        let sql = format!("UPDATE {table} SET {column} = {column} - $1 WHERE {pk} = $2");
        postgres::execute_raw(pool, &sql, vec![SqlValue::Integer(delta), id.into()]).await
    }

    /// Increment a column without touching `updated_at`.
    async fn increment_without_timestamps(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
        column: &str,
        delta: i64,
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        Self::without_timestamps_async(|| Self::increment(pool, id, column, delta)).await
    }

    /// Fetch rows using a raw SQL string and positional parameters (`$1`, `$2`, …).
    fn from_raw_sql(
        pool: &PgPool,
        sql: &str,
        params: Vec<SqlValue>,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where Self: Sized,
    {
        postgres::fetch_raw::<Self>(pool, sql, params)
    }

    /// Update this record with events muted (no observer hooks fired).
    async fn save_quietly(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        Self::without_events_async(|| Self::update_by_pk(pool, id, data)).await
    }
}

impl<T> PgModel for T where T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin {}
