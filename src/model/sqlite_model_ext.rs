//! [`SqliteModelExt`] — aggregates, pagination, upsert, and advanced queries (SQLite).
//!
//! Blanket-implemented for every type that implements [`SqliteModel`].

use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};
use sqlx::{sqlite::SqliteRow, SqlitePool};

use crate::executor::sqlite;
use super::sqlite_model::SqliteModel;

#[allow(async_fn_in_trait)]
pub trait SqliteModelExt: SqliteModel {
    async fn paginate(
        pool: &SqlitePool,
        page: i64,
        per_page: i64,
    ) -> Result<crate::pagination::Page<Self>, sqlx::Error>
    where Self: Sized,
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
    where Self: Sized,
    {
        let total = sqlite::count(pool, builder.clone()).await?;
        let data = sqlite::fetch_all(pool, builder.paginate(page, per_page)).await?;
        Ok(crate::pagination::Page::new(data, total, per_page, page))
    }

    async fn sum(pool: &SqlitePool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where Self: Sized,
    {
        sqlite::aggregate::<Self>(pool, Self::query(), "SUM", column).await
    }

    async fn avg(pool: &SqlitePool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where Self: Sized,
    {
        sqlite::aggregate::<Self>(pool, Self::query(), "AVG", column).await
    }

    async fn min(pool: &SqlitePool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where Self: Sized,
    {
        sqlite::aggregate::<Self>(pool, Self::query(), "MIN", column).await
    }

    async fn max(pool: &SqlitePool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where Self: Sized,
    {
        sqlite::aggregate::<Self>(pool, Self::query(), "MAX", column).await
    }

    async fn upsert(
        pool: &SqlitePool,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        sqlite::upsert::<Self>(pool, Self::table_name(), data, conflict_column, update_columns)
            .await
    }

    async fn delete_in(
        pool: &SqlitePool,
        column: &str,
        values: Vec<SqlValue>,
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        sqlite::delete_in::<Self>(pool, column, values).await
    }

    async fn exists(pool: &SqlitePool) -> Result<bool, sqlx::Error>
    where Self: Sized,
    {
        sqlite::exists(pool, Self::query()).await
    }

    async fn exists_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
    ) -> Result<bool, sqlx::Error>
    where Self: Sized,
    {
        sqlite::exists(pool, builder).await
    }

    async fn pluck(pool: &SqlitePool, column: &str) -> Result<Vec<SqlValue>, sqlx::Error>
    where Self: Sized,
    {
        sqlite::pluck(pool, Self::query(), column).await
    }

    async fn pluck_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
        column: &str,
    ) -> Result<Vec<SqlValue>, sqlx::Error>
    where Self: Sized,
    {
        sqlite::pluck(pool, builder, column).await
    }

    async fn update_all(pool: &SqlitePool, data: &[(&str, SqlValue)]) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        sqlite::update_all(pool, Self::query(), data).await
    }

    async fn update_all_where(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        sqlite::update_all(pool, builder, data).await
    }
}

impl<T> SqliteModelExt for T
where
    T: Model + for<'r> sqlx::FromRow<'r, SqliteRow> + Send + Unpin,
{}
