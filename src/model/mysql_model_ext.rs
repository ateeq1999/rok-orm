//! [`MyModelExt`] — aggregates, pagination, upsert, and advanced queries (MySQL).
//!
//! Blanket-implemented for every type that implements [`MyModel`].

use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};
use sqlx::mysql::{MyRow, MyPool};

use crate::executor::mysql;
use super::mysql_model::MyModel;

#[allow(async_fn_in_trait)]
pub trait MyModelExt: MyModel {
    async fn paginate(
        pool: &MyPool,
        page: i64,
        per_page: i64,
    ) -> Result<crate::pagination::Page<Self>, sqlx::Error>
    where Self: Sized,
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
    where Self: Sized,
    {
        let total = mysql::count(pool, builder.clone()).await?;
        let data = mysql::fetch_all(pool, builder.paginate(page, per_page)).await?;
        Ok(crate::pagination::Page::new(data, total, per_page, page))
    }

    async fn sum(pool: &MyPool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where Self: Sized,
    {
        mysql::aggregate::<Self>(pool, Self::query(), "SUM", column).await
    }

    async fn avg(pool: &MyPool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where Self: Sized,
    {
        mysql::aggregate::<Self>(pool, Self::query(), "AVG", column).await
    }

    async fn min(pool: &MyPool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where Self: Sized,
    {
        mysql::aggregate::<Self>(pool, Self::query(), "MIN", column).await
    }

    async fn max(pool: &MyPool, column: &str) -> Result<Option<f64>, sqlx::Error>
    where Self: Sized,
    {
        mysql::aggregate::<Self>(pool, Self::query(), "MAX", column).await
    }

    async fn upsert(
        pool: &MyPool,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        mysql::upsert::<Self>(pool, Self::table_name(), data, conflict_column, update_columns)
            .await
    }

    async fn upsert_returning(
        pool: &MyPool,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> Result<Self, sqlx::Error>
    where Self: Sized,
    {
        mysql::upsert_returning::<Self>(
            pool,
            Self::table_name(),
            data,
            conflict_column,
            update_columns,
        )
        .await
    }

    async fn delete_in(
        pool: &MyPool,
        column: &str,
        values: Vec<SqlValue>,
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        mysql::delete_in::<Self>(pool, Self::table_name(), column, values).await
    }

    async fn exists(pool: &MyPool) -> Result<bool, sqlx::Error>
    where Self: Sized,
    {
        mysql::exists(pool, Self::query()).await
    }

    async fn exists_where(pool: &MyPool, builder: QueryBuilder<Self>) -> Result<bool, sqlx::Error>
    where Self: Sized,
    {
        mysql::exists(pool, builder).await
    }

    async fn pluck(pool: &MyPool, column: &str) -> Result<Vec<SqlValue>, sqlx::Error>
    where Self: Sized,
    {
        mysql::pluck(pool, Self::query(), column).await
    }

    async fn pluck_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
        column: &str,
    ) -> Result<Vec<SqlValue>, sqlx::Error>
    where Self: Sized,
    {
        mysql::pluck(pool, builder, column).await
    }

    async fn update_all(pool: &MyPool, data: &[(&str, SqlValue)]) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        mysql::update_all(pool, Self::query(), data).await
    }

    async fn update_all_where(
        pool: &MyPool,
        builder: QueryBuilder<Self>,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        mysql::update_all(pool, builder, data).await
    }

    async fn insert_ignore(pool: &MyPool, data: &[(&str, SqlValue)]) -> Result<u64, sqlx::Error>
    where Self: Sized,
    {
        let (sql, params) = QueryBuilder::<Self>::insert_ignore_sql(
            crate::Dialect::Mysql,
            Self::table_name(),
            data,
        );
        mysql::execute(pool, &sql, params).await
    }
}

impl<T> MyModelExt for T where T: Model + for<'r> sqlx::FromRow<'r, MyRow> + Send + Unpin {}
