//! [`MyModel`] ΓÇö ergonomic async CRUD methods for any [`Model`] + MySQL.
//!
//! All methods are provided as defaults; no manual implementation is required.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::{Model, MyModel, SqlValue};
//!
//! #[derive(Model, sqlx::FromRow)]
//! pub struct User {
//!     pub id: i64,
//!     pub name: String,
//! }
//!
//! let pool = sqlx::MyPool::connect(&url).await?;
//!
//! let all: Vec<User>    = User::all(&pool).await?;
//! let one: Option<User> = User::find_by_pk(&pool, 1i64).await?;
//! let n: i64            = User::count(&pool).await?;
//! User::create(&pool, &[("name", "Alice".into())]).await?;
//! User::update_by_pk(&pool, 1i64, &[("name", "Bob".into())]).await?;
//! User::delete_by_pk(&pool, 1i64).await?;
//! ```

use chrono::Utc;
use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};
use sqlx::mysql::{MyPool, MyRow};

use crate::executor::mysql;

pub trait MyModel: Model + for<'r> sqlx::FromRow<'r, MyRow> + Send + Unpin {
    fn all(
        pool: &MyPool,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        mysql::fetch_all(pool, Self::query())
    }

    fn find_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        mysql::fetch_all(pool, builder)
    }

    fn find_by_pk(
        pool: &MyPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<Option<Self>, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        mysql::fetch_optional(pool, Self::find(id))
    }

    async fn find_or_404(pool: &MyPool, id: impl Into<SqlValue> + Send) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::fetch_optional(pool, Self::find(id))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn first(pool: &MyPool) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::fetch_optional(pool, Self::query().limit(1))
    }

    async fn first_or_404(pool: &MyPool) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::fetch_optional(pool, Self::query().limit(1))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn get(pool: &MyPool) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::fetch_all(pool, Self::query()).await
    }

    async fn get_where(pool: &MyPool, builder: QueryBuilder<Self>) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::fetch_all(pool, builder).await
    }

    async fn count(pool: &MyPool) -> Result<i64, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::count(pool, Self::query()).await
    }

    async fn count_where(pool: &MyPool, builder: QueryBuilder<Self>) -> Result<i64, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::count(pool, builder).await
    }

    fn create(
        pool: &MyPool,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        mysql::insert::<Self>(pool, Self::table_name(), data)
    }

    async fn update_by_pk(
        pool: &MyPool,
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
        mysql::update::<Self>(pool, builder, &data_with_timestamps).await
    }

    fn delete_by_pk(
        pool: &MyPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        mysql::delete(pool, Self::find(id))
    }

    fn delete_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        mysql::delete(pool, builder)
    }

    fn update_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        mysql::update::<Self>(pool, builder, data)
    }

    fn bulk_create(
        pool: &MyPool,
        rows: &[Vec<(&str, SqlValue)>],
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        mysql::bulk_insert::<Self>(pool, Self::table_name(), rows)
    }

    async fn create_returning(pool: &MyPool, data: &[(&str, SqlValue)]) -> Result<Self, sqlx::Error>
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
        mysql::insert_returning::<Self>(pool, Self::table_name(), &data_with_timestamps).await
    }

    fn restore(
        pool: &MyPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        mysql::restore::<Self>(pool, Self::find(id))
    }

    fn restore_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        mysql::restore::<Self>(pool, builder)
    }

    fn force_delete(
        pool: &MyPool,
        id: impl Into<SqlValue> + Send,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        mysql::force_delete(pool, Self::find(id))
    }

    fn force_delete_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
    ) -> impl std::future::Future<Output = Result<u64, sqlx::Error>> + Send
    where
        Self: Sized,
    {
        mysql::force_delete(pool, builder)
    }

    async fn paginate(
        pool: &MyPool,
        page: i64,
        per_page: i64,
    ) -> Result<crate::pagination::Page<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        let total = mysql::count(pool, Self::query()).await?;
        let data = mysql::fetch_all(pool, Self::query().paginate(page, per_page)).await?;
        Ok(crate::pagination::Page::new(data, total, per_page, page))
    }

    async fn paginate_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
        page: i64,
        per_page: i64,
    ) -> Result<crate::pagination::Page<Self>, sqlx::Error>
    where
        Self: Sized,
    {
        let total = mysql::count(pool, builder.clone()).await?;
        let data = mysql::fetch_all(pool, builder.paginate(page, per_page)).await?;
        Ok(crate::pagination::Page::new(data, total, per_page, page))
    }

    async fn sum(pool: &MyPool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::aggregate::<Self>(pool, Self::query(), "SUM", column).await
    }

    async fn avg(pool: &MyPool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::aggregate::<Self>(pool, Self::query(), "AVG", column).await
    }

    async fn min(pool: &MyPool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::aggregate::<Self>(pool, Self::query(), "MIN", column).await
    }

    async fn max(pool: &MyPool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::aggregate::<Self>(pool, Self::query(), "MAX", column).await
    }

    async fn upsert(
        pool: &MyPool,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::upsert::<Self>(pool, Self::table_name(), data, conflict_column, update_columns).await
    }

    async fn upsert_returning(
        pool: &MyPool,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> Result<Self, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::upsert_returning::<Self>(pool, Self::table_name(), data, conflict_column, update_columns).await
    }

    async fn delete_in(pool: &MyPool, column: &str, values: Vec<SqlValue>) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::delete_in::<Self>(pool, Self::table_name(), column, values).await
    }

    async fn exists(pool: &MyPool) -> Result<bool, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::exists(pool, Self::query()).await
    }

    async fn exists_where(pool: &MyPool, builder: QueryBuilder<Self>) -> Result<bool, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::exists(pool, builder).await
    }

    async fn pluck(pool: &MyPool, column: &str) -> Result<Vec<SqlValue>, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::pluck(pool, Self::query(), column).await
    }

    async fn pluck_where(pool: &MyPool, builder: QueryBuilder<Self>, column: &str) -> Result<Vec<SqlValue>, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::pluck(pool, builder, column).await
    }

    async fn update_all(pool: &MyPool, data: &[(&str, SqlValue)]) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::update_all(pool, Self::query(), data).await
    }

    async fn update_all_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        mysql::update_all(pool, builder, data).await
    }

    async fn insert_ignore(pool: &MyPool, data: &[(&str, SqlValue)]) -> Result<u64, sqlx::Error>
    where
        Self: Sized,
    {
        let (sql, params) = QueryBuilder::<Self>::insert_ignore_sql(
            crate::Dialect::Mysql,
            Self::table_name(),
            data,
        );
        mysql::execute(pool, &sql, params).await
    }
}

impl<T> MyModel for T where T: Model + for<'r> sqlx::FromRow<'r, MyRow> + Send + Unpin {}
