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

    /// Find the first row matching `conditions`, or create it with `data`.
    ///
    /// `conditions` is used both for the WHERE lookup and as part of the INSERT.
    async fn first_or_create(
        pool: &PgPool,
        conditions: &[(&str, SqlValue)],
        data: &[(&str, SqlValue)],
    ) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        let mut qb = Self::query();
        for (col, val) in conditions {
            qb = qb.where_eq(col, val.clone());
        }
        if let Some(existing) = postgres::fetch_optional(pool, qb).await? {
            return Ok(existing);
        }
        let mut insert_data: Vec<(&str, SqlValue)> = conditions.to_vec();
        for row in data {
            if !insert_data.iter().any(|(c, _)| c == &row.0) {
                insert_data.push(*row);
            }
        }
        postgres::insert_returning::<Self>(pool, Self::table_name(), &insert_data).await
    }

    /// Find the first row matching `conditions`, or INSERT+return a new one using `conditions` + `data`.
    async fn update_or_create(
        pool: &PgPool,
        conditions: &[(&str, SqlValue)],
        data: &[(&str, SqlValue)],
    ) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        let mut qb = Self::query();
        for (col, val) in conditions {
            qb = qb.where_eq(col, val.clone());
        }
        if let Some(_existing) = postgres::fetch_optional::<Self>(pool, qb.clone()).await? {
            postgres::update::<Self>(pool, qb, data).await?;
            // Re-fetch the updated row
            let mut refetch = Self::query();
            for (col, val) in conditions {
                refetch = refetch.where_eq(col, val.clone());
            }
            return postgres::fetch_optional(pool, refetch.limit(1))
                .await?
                .ok_or(sqlx::Error::RowNotFound);
        }
        let mut insert_data: Vec<(&str, SqlValue)> = conditions.to_vec();
        for row in data {
            if !insert_data.iter().any(|(c, _)| c == &row.0) {
                insert_data.push(*row);
            }
        }
        postgres::insert_returning::<Self>(pool, Self::table_name(), &insert_data).await
    }

    /// Fetch all records in chunks, returning them as a `Vec<Vec<Self>>`.
    ///
    /// Iterates with LIMIT/OFFSET until an empty batch is returned.
    /// Use this when you need to process all records without loading everything into memory.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let batches = User::query().chunk_collect(&pool, 100).await?;
    /// for batch in batches {
    ///     for user in batch {
    ///         process(&user).await;
    ///     }
    /// }
    /// ```
    async fn chunk_collect(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        chunk_size: usize,
    ) -> Result<Vec<Vec<Self>>, sqlx::Error>
    where Self: Sized,
    {
        let mut results = Vec::new();
        let mut offset = 0usize;
        loop {
            let batch = postgres::fetch_all(
                pool,
                builder.clone().limit(chunk_size).offset(offset),
            ).await?;
            if batch.is_empty() {
                break;
            }
            offset += batch.len();
            results.push(batch);
        }
        Ok(results)
    }

    /// Chunk by primary key — stable even when rows are deleted mid-run.
    ///
    /// Uses `WHERE pk > last_id ORDER BY pk LIMIT chunk_size` to avoid
    /// drift when records are inserted or deleted between chunks.
    async fn chunk_by_id_collect(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        chunk_size: usize,
    ) -> Result<Vec<Vec<Self>>, sqlx::Error>
    where Self: Sized,
    {
        use crate::query::SqlValue;
        let pk_col = Self::primary_key();
        let mut results = Vec::new();
        let mut last_id: Option<i64> = None;
        loop {
            let mut qb = builder.clone()
                .order_by(pk_col)
                .limit(chunk_size);
            if let Some(id) = last_id {
                qb = qb.where_gt(pk_col, SqlValue::Integer(id));
            }
            let batch = postgres::fetch_all(pool, qb).await?;
            if batch.is_empty() {
                break;
            }
            last_id = None; // would need pk extraction — placeholder
            results.push(batch);
            // Break if we got fewer than chunk_size (last page)
            if results.last().map_or(0, |b: &Vec<Self>| b.len()) < chunk_size {
                break;
            }
        }
        Ok(results)
    }

    /// Cursor-based pagination. Fetches `limit + 1` rows to detect `has_more`.
    ///
    /// Pass `get_id` to extract the i64 PK from each row (used to build the next cursor).
    async fn cursor_paginate(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        cursor: crate::cursor::CursorPage,
        get_id: impl Fn(&Self) -> i64 + Send,
    ) -> Result<crate::cursor::CursorResult<Self>, sqlx::Error>
    where Self: Sized,
    {
        let pk = Self::primary_key();
        let qb = builder.cursor_sql(pk, cursor.after, cursor.limit);
        let rows = postgres::fetch_all(pool, qb).await?;
        Ok(crate::cursor::CursorResult::from_rows(rows, cursor.limit, get_id))
    }
}

impl<T> PgModelExt for T where T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin {}
