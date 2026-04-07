//! [`PgModelExt`] — aggregates, pagination, upsert, and advanced queries (PostgreSQL).
//!
//! Blanket-implemented for every type that implements [`PgModel`].

use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};
use sqlx::{postgres::PgRow, PgPool};

use crate::executor::postgres;
use super::pg_model::PgModel;

#[allow(async_fn_in_trait)]
pub trait PgModelExt: PgModel {
    async fn paginate(
        pool: &PgPool,
        page: i64,
        per_page: i64,
    ) -> Result<crate::pagination::Page<Self>, sqlx::Error>
    where Self: Sized,
    {
        let total = postgres::count(pool, Self::query()).await?;
        let data = postgres::fetch_all(pool, Self::query().paginate(page, per_page)).await?;
        Ok(crate::pagination::Page::new(data, total, per_page, page))
    }

    async fn paginate_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        page: i64,
        per_page: i64,
    ) -> Result<crate::pagination::Page<Self>, sqlx::Error>
    where Self: Sized,
    {
        let total = postgres::count(pool, builder.clone()).await?;
        let data = postgres::fetch_all(pool, builder.paginate(page, per_page)).await?;
        Ok(crate::pagination::Page::new(data, total, per_page, page))
    }

    async fn sum(pool: &PgPool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where Self: Sized,
    {
        postgres::aggregate::<Self>(pool, Self::query(), "SUM", column).await
    }

    async fn avg(pool: &PgPool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where Self: Sized,
    {
        postgres::aggregate::<Self>(pool, Self::query(), "AVG", column).await
    }

    async fn min(pool: &PgPool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where Self: Sized,
    {
        postgres::aggregate::<Self>(pool, Self::query(), "MIN", column).await
    }

    async fn max(pool: &PgPool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where Self: Sized,
    {
        postgres::aggregate::<Self>(pool, Self::query(), "MAX", column).await
    }

    async fn upsert(
        pool: &PgPool,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        postgres::upsert::<Self>(pool, Self::table_name(), data, conflict_column, update_columns)
            .await
    }

    async fn upsert_returning(
        pool: &PgPool,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        postgres::upsert_returning::<Self>(
            pool,
            Self::table_name(),
            data,
            conflict_column,
            update_columns,
        )
        .await
    }

    async fn delete_in(
        pool: &PgPool,
        column: &str,
        values: Vec<SqlValue>,
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        postgres::delete_in::<Self>(pool, column, values).await
    }

    async fn exists(pool: &PgPool) -> Result<bool, sqlx::Error>
    where Self: Sized,
    {
        postgres::exists(pool, Self::query()).await
    }

    async fn exists_where(pool: &PgPool, builder: QueryBuilder<Self>) -> Result<bool, sqlx::Error>
    where Self: Sized,
    {
        postgres::exists(pool, builder).await
    }

    async fn pluck(pool: &PgPool, column: &str) -> Result<Vec<SqlValue>, sqlx::Error>
    where Self: Sized,
    {
        postgres::pluck(pool, Self::query(), column).await
    }

    async fn pluck_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        column: &str,
    ) -> Result<Vec<SqlValue>, sqlx::Error>
    where Self: Sized,
    {
        postgres::pluck(pool, builder, column).await
    }

    async fn update_all(pool: &PgPool, data: &[(&str, SqlValue)]) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        postgres::update_all(pool, Self::query(), data).await
    }

    async fn update_all_where(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        postgres::update_all(pool, builder, data).await
    }
}

impl<T> PgModelExt for T where T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin {}
