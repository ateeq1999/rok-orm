//! [`MyModel`] — core CRUD methods for any [`Model`] + MySQL.
//!
//! For aggregates, pagination, upsert, and advanced queries see [`MyModelExt`].

use chrono::Utc;
use crate::model::{Model, model::timestamps_muted};
use crate::query::{QueryBuilder, SqlValue};
use sqlx::mysql::{MyRow, MyPool};

use crate::executor::mysql;

#[allow(async_fn_in_trait)]
pub trait MyModel: Model + for<'r> sqlx::FromRow<'r, MyRow> + Send + Unpin {
    fn all(
        pool: &MyPool,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where Self: Sized + Send + 'static,
    {
        mysql::fetch_all(pool, Self::scoped_query())
    }

    fn find_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where Self: Sized,
    {
        mysql::fetch_all(pool, builder)
    }

    fn find_by_pk(
        pool: &MyPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<Option<Self>, sqlx::Error>> + Send
    where Self: Sized,
    {
        mysql::fetch_optional(pool, Self::find(id))
    }

    async fn find_or_404(pool: &MyPool, id: impl Into<SqlValue> + Send) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        mysql::fetch_optional(pool, Self::find(id))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn first(pool: &MyPool) -> Result<Option<Self>, sqlx::Error>
    where Self: Sized + Send + 'static,
    {
        mysql::fetch_optional(pool, Self::scoped_query().limit(1)).await
    }

    async fn first_or_404(pool: &MyPool) -> Result<Self, sqlx::Error>
    where Self: Sized + Send + 'static,
    {
        mysql::fetch_optional(pool, Self::scoped_query().limit(1))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn get(pool: &MyPool) -> Result<Vec<Self>, sqlx::Error>
    where Self: Sized + Send + 'static,
    {
        mysql::fetch_all(pool, Self::scoped_query()).await
    }

    async fn get_where(pool: &MyPool, builder: QueryBuilder<Self>) -> Result<Vec<Self>, sqlx::Error>
    where Self: Sized,
    {
        mysql::fetch_all(pool, builder).await
    }

    async fn count(pool: &MyPool) -> Result<i64, sqlx::Error>
    where Self: Sized + Send + 'static,
    {
        mysql::count(pool, Self::scoped_query()).await
    }

    async fn count_where(pool: &MyPool, builder: QueryBuilder<Self>) -> Result<i64, sqlx::Error>
    where Self: Sized,
    {
        mysql::count(pool, builder).await
    }

    async fn create(
        pool: &MyPool,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        let mut d = Self::filter_fillable(data);
        if let Some(pk_val) = Self::new_unique_id() {
            d.insert(0, (Self::primary_key(), pk_val));
        }
        mysql::insert::<Self>(pool, Self::table_name(), &d).await
    }

    async fn update_by_pk(
        pool: &MyPool,
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
        mysql::update::<Self>(pool, Self::find(id), &d).await
    }

    fn delete_by_pk(
        pool: &MyPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        mysql::delete(pool, Self::find(id))
    }

    fn delete_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        mysql::delete(pool, builder)
    }

    fn update_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        mysql::update::<Self>(pool, builder, data)
    }

    fn bulk_create(
        pool: &MyPool,
        rows: &[Vec<(&str, SqlValue)>],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        mysql::bulk_insert::<Self>(pool, Self::table_name(), rows)
    }

    async fn create_returning(pool: &MyPool, data: &[(&str, SqlValue)]) -> Result<Self, sqlx::Error>
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
        mysql::insert_returning::<Self>(pool, Self::table_name(), &d).await
    }

    fn restore(
        pool: &MyPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        mysql::restore::<Self>(pool, Self::find(id))
    }

    fn restore_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        mysql::restore::<Self>(pool, builder)
    }

    fn force_delete(
        pool: &MyPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        mysql::force_delete(pool, Self::find(id))
    }

    fn force_delete_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where Self: Sized,
    {
        mysql::force_delete(pool, builder)
    }

    /// Atomically increment a column by `delta` for a given PK.
    async fn increment(
        pool: &MyPool,
        id: impl Into<SqlValue> + Send,
        column: &str,
        delta: i64,
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        let table = Self::table_name();
        let pk = Self::primary_key();
        let sql = format!("UPDATE {table} SET {column} = {column} + ? WHERE {pk} = ?");
        mysql::execute(pool, &sql, vec![SqlValue::Integer(delta), id.into()]).await
    }

    /// Atomically decrement a column by `delta` for a given PK.
    async fn decrement(
        pool: &MyPool,
        id: impl Into<SqlValue> + Send,
        column: &str,
        delta: i64,
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        let table = Self::table_name();
        let pk = Self::primary_key();
        let sql = format!("UPDATE {table} SET {column} = {column} - ? WHERE {pk} = ?");
        mysql::execute(pool, &sql, vec![SqlValue::Integer(delta), id.into()]).await
    }

    /// Fetch rows using a raw SQL string with `?` placeholders.
    fn from_raw_sql(
        pool: &MyPool,
        sql: &str,
        params: Vec<SqlValue>,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where Self: Sized,
    {
        mysql::fetch_raw::<Self>(pool, sql, params)
    }

    /// Update this record with events muted (no observer hooks fired).
    async fn save_quietly(
        pool: &MyPool,
        id: impl Into<SqlValue> + Send,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        Self::without_events_async(|| Self::update_by_pk(pool, id, data)).await
    }
}

impl<T> MyModel for T where T: Model + for<'r> sqlx::FromRow<'r, MyRow> + Send + Unpin {}
