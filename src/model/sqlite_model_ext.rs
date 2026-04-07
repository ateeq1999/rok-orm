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

    async fn upsert_returning(
        pool: &SqlitePool,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        sqlite::upsert_returning::<Self>(
            pool, Self::table_name(), data, conflict_column, update_columns,
        ).await
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

    async fn first_or_create(
        pool: &SqlitePool,
        conditions: &[(&str, SqlValue)],
        data: &[(&str, SqlValue)],
    ) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        let mut qb = Self::query();
        for (col, val) in conditions { qb = qb.where_eq(col, val.clone()); }
        if let Some(existing) = sqlite::fetch_optional(pool, qb).await? {
            return Ok(existing);
        }
        let mut insert_data: Vec<(&str, SqlValue)> = conditions.to_vec();
        for row in data {
            if !insert_data.iter().any(|(c, _)| c == &row.0) { insert_data.push(*row); }
        }
        sqlite::insert_returning::<Self>(pool, Self::table_name(), &insert_data).await
    }

    async fn update_or_create(
        pool: &SqlitePool,
        conditions: &[(&str, SqlValue)],
        data: &[(&str, SqlValue)],
    ) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        let mut qb = Self::query();
        for (col, val) in conditions { qb = qb.where_eq(col, val.clone()); }
        if let Some(_existing) = sqlite::fetch_optional::<Self>(pool, qb.clone()).await? {
            sqlite::update_all(pool, qb, data).await?;
            let mut qb2 = Self::query();
            for (col, val) in conditions { qb2 = qb2.where_eq(col, val.clone()); }
            return sqlite::fetch_optional(pool, qb2).await?
                .ok_or_else(|| sqlx::Error::RowNotFound);
        }
        let mut insert_data: Vec<(&str, SqlValue)> = conditions.to_vec();
        for row in data {
            if !insert_data.iter().any(|(c, _)| c == &row.0) { insert_data.push(*row); }
        }
        sqlite::insert_returning::<Self>(pool, Self::table_name(), &insert_data).await
    }

    /// Fetch all records in chunks via LIMIT/OFFSET.
    async fn chunk_collect(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
        chunk_size: usize,
    ) -> Result<Vec<Vec<Self>>, sqlx::Error>
    where Self: Sized,
    {
        let mut results = Vec::new();
        let mut offset = 0usize;
        loop {
            let batch = sqlite::fetch_all(
                pool,
                builder.clone().limit(chunk_size).offset(offset),
            ).await?;
            if batch.is_empty() { break; }
            offset += batch.len();
            results.push(batch);
        }
        Ok(results)
    }

    /// Cursor-based pagination for SQLite. Fetches `limit + 1` rows to detect `has_more`.
    async fn cursor_paginate(
        pool: &SqlitePool,
        builder: QueryBuilder<Self>,
        cursor: crate::cursor::CursorPage,
        get_id: impl Fn(&Self) -> i64 + Send,
    ) -> Result<crate::cursor::CursorResult<Self>, sqlx::Error>
    where Self: Sized,
    {
        let pk = Self::primary_key();
        let qb = builder.cursor_sql(pk, cursor.after, cursor.limit);
        let rows = sqlite::fetch_all(pool, qb).await?;
        Ok(crate::cursor::CursorResult::from_rows(rows, cursor.limit, get_id))
    }
}

impl<T> SqliteModelExt for T
where
    T: Model + for<'r> sqlx::FromRow<'r, SqliteRow> + Send + Unpin,
{}
