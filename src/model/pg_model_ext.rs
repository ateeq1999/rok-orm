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
                insert_data.push(row.clone());
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
                insert_data.push(row.clone());
            }
        }
        postgres::insert_returning::<Self>(pool, Self::table_name(), &insert_data).await
    }

    /// Process all matching rows in chunks using a callback.
    ///
    /// Iterates with LIMIT/OFFSET until an empty batch is returned.
    /// The callback receives each `Vec<Self>` batch and can do async work.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// User::chunk(&pool, User::query().filter("active", true), 500, |batch| async move {
    ///     for user in batch {
    ///         send_email(&user).await;
    ///     }
    ///     Ok(())
    /// }).await?;
    /// ```
    async fn chunk<F, Fut>(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        chunk_size: usize,
        mut callback: F,
    ) -> crate::errors::OrmResult<()>
    where
        Self: Sized,
        F: FnMut(Vec<Self>) -> Fut,
        Fut: std::future::Future<Output = crate::errors::OrmResult<()>>,
    {
        let mut offset = 0usize;
        loop {
            let batch = postgres::fetch_all(
                pool,
                builder.clone().limit(chunk_size).offset(offset),
            ).await.map_err(crate::errors::OrmError::from)?;
            if batch.is_empty() {
                break;
            }
            offset += batch.len();
            callback(batch).await?;
        }
        Ok(())
    }

    /// Chunk by primary key — stable even when rows are deleted mid-run.
    ///
    /// Uses `WHERE pk > last_id ORDER BY pk ASC LIMIT chunk_size` to avoid
    /// drift from concurrent deletes/inserts.
    ///
    /// `get_id` extracts the i64 PK from each row for cursor tracking.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// User::chunk_by_id(&pool, User::query(), 500, |u| u.id, |batch| async move {
    ///     process(batch).await?;
    ///     Ok(())
    /// }).await?;
    /// ```
    async fn chunk_by_id<F, Fut>(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        chunk_size: usize,
        get_id: impl Fn(&Self) -> i64,
        mut callback: F,
    ) -> crate::errors::OrmResult<()>
    where
        Self: Sized,
        F: FnMut(Vec<Self>) -> Fut,
        Fut: std::future::Future<Output = crate::errors::OrmResult<()>>,
    {
        let pk_col = Self::primary_key();
        let mut last_id: Option<i64> = None;
        loop {
            let mut qb = builder.clone().order_by(pk_col).limit(chunk_size);
            if let Some(id) = last_id {
                qb = qb.where_gt(pk_col, SqlValue::Integer(id));
            }
            let batch = postgres::fetch_all(pool, qb)
                .await.map_err(crate::errors::OrmError::from)?;
            if batch.is_empty() {
                break;
            }
            last_id = batch.last().map(&get_id);
            let is_last = batch.len() < chunk_size;
            callback(batch).await?;
            if is_last {
                break;
            }
        }
        Ok(())
    }

    /// Fetch rows including extra aggregate columns (e.g. from `with_count_col`).
    ///
    /// `extra_cols` must match the aliases used in `with_count_col` / `with_sum_col` etc.
    /// Extra values are accessible via [`WithExtras::extra_i64`], [`WithExtras::extra_f64`], etc.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let posts = Post::get_with_extras(
    ///     &pool,
    ///     Post::query().with_count_col("comments", "post_id", "id", "comments_count"),
    ///     &["comments_count"],
    /// ).await?;
    /// assert_eq!(posts[0].extra_i64("comments_count"), Some(3));
    /// ```
    async fn get_with_extras(
        pool: &PgPool,
        builder: QueryBuilder<Self>,
        extra_cols: &[&str],
    ) -> Result<Vec<crate::extras::WithExtras<Self>>, sqlx::Error>
    where Self: Sized,
    {
        postgres::fetch_with_extras(pool, builder, extra_cols).await
    }

    /// Stream rows one-by-one from the database — avoids loading all rows into memory.
    ///
    /// Backed by sqlx's `fetch()` cursor.  Use when the result set may be too large for
    /// `fetch_all`.  Requires `StreamExt::next` from a futures library to consume items.
    ///
    /// ```rust,ignore
    /// use futures::StreamExt;
    /// let mut s = User::into_stream(&pool, User::query().where_eq("active", true));
    /// while let Some(row) = s.next().await {
    ///     let user = row?;
    ///     process(user).await;
    /// }
    /// ```
    fn into_stream<'a>(
        pool: &'a PgPool,
        builder: QueryBuilder<Self>,
    ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = Result<Self, sqlx::Error>> + Send + 'a>>
    where Self: Sized + 'static,
    {
        postgres::fetch_stream(pool, builder)
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
