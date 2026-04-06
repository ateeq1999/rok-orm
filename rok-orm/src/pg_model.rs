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

use chrono::Utc;
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

    async fn update_by_pk(
        pool: &PgPool,
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
        executor::update::<Self>(pool, builder, &data_with_timestamps).await
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

    async fn create_returning(
        pool: &PgPool,
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
        executor::insert_returning::<Self>(pool, Self::table_name(), &data_with_timestamps).await
    }

    fn restore(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::restore::<Self>(pool, Self::find(id))
    }

    fn restore_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::restore::<Self>(pool, builder)
    }

    fn force_delete(
        pool: &PgPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::force_delete(pool, Self::find(id))
    }

    fn force_delete_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        executor::force_delete(pool, builder)
    }

    async fn paginate(
        pool: &PgPool,
        page: i64,
        per_page: i64,
    ) -> Result<crate::pagination::Page<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        let total = executor::count(pool, Self::query()).await?;
        let data = executor::fetch_all(pool, Self::query().paginate(page, per_page)).await?;
        Ok(crate::pagination::Page::new(data, total, per_page, page))
    }

    async fn paginate_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        page: i64,
        per_page: i64,
    ) -> Result<crate::pagination::Page<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        let total = executor::count(pool, builder.clone()).await?;
        let data = executor::fetch_all(pool, builder.paginate(page, per_page)).await?;
        Ok(crate::pagination::Page::new(data, total, per_page, page))
    }

    async fn sum(
        pool: &PgPool,
        column: &str,
    ) -> Result<Option<f64>, sqlx::Error>
    where
        Self: Sized,
    {
        executor::aggregate::<Self>(pool, Self::query(), "SUM", column).await
    }

    async fn avg(
        pool: &PgPool,
        column: &str,
    ) -> Result<Option<f64>, sqlx::Error>
    where
        Self: Sized,
    {
        executor::aggregate::<Self>(pool, Self::query(), "AVG", column).await
    }

    async fn min(
        pool: &PgPool,
        column: &str,
    ) -> Result<Option<f64>, sqlx::Error>
    where
        Self: Sized,
    {
        executor::aggregate::<Self>(pool, Self::query(), "MIN", column).await
    }

    async fn max(
        pool: &PgPool,
        column: &str,
    ) -> Result<Option<f64>, sqlx::Error>
    where
        Self: Sized,
    {
        executor::aggregate::<Self>(pool, Self::query(), "MAX", column).await
    }

    async fn upsert(
        pool: &PgPool,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        executor::upsert::<Self>(pool, Self::table_name(), data, conflict_column, update_columns).await
    }

    async fn upsert_returning(
        pool: &PgPool,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        executor::upsert_returning::<Self>(pool, Self::table_name(), data, conflict_column, update_columns).await
    }

    async fn delete_in(
        pool: &PgPool,
        column: &str,
        values: Vec<SqlValue>,
    ) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        executor::delete_in::<Self>(pool, column, values).await
    }

    async fn exists(pool: &PgPool) -> Result<bool, sqlx::Error>
    where
        Self: Sized,
    {
        executor::exists(pool, Self::query()).await
    }

    async fn exists_where(pool: &PgPool, builder: QueryBuilder<Self>) -> Result<bool, sqlx::Error>
    where
        Self: Sized,
    {
        executor::exists(pool, builder).await
    }

    async fn pluck(
        pool: &PgPool,
        column: &str,
    ) -> Result<Vec<SqlValue>, sqlx::Error>
    where
        Self: Sized,
    {
        executor::pluck(pool, Self::query(), column).await
    }

    async fn pluck_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        column: &str,
    ) -> Result<Vec<SqlValue>, sqlx::Error>
    where
        Self: Sized,
    {
        executor::pluck(pool, builder, column).await
    }

    async fn update_all(
        pool: &PgPool,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        executor::update_all(pool, Self::query(), data).await
    }

    async fn update_all_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        executor::update_all(pool, builder, data).await
    }
}

impl<T> PgModel for T where T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin {}
