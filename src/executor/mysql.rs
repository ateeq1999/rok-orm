//! Async MySQL executor ΓÇö runs [`QueryBuilder`] output against a live MySQL pool.
//!
//! Requires the `mysql` feature:
//!
//! ```toml
//! rok-orm = { version = "0.3", features = ["mysql"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::{Model, mysql_executor as executor};
//!
//! #[derive(Model, sqlx::FromRow)]
//! pub struct User { pub id: i64, pub name: String }
//!
//! let pool = sqlx::MyPool::connect(&database_url).await?;
//!
//! let users: Vec<User> = executor::fetch_all(&pool, User::query().where_eq("active", true)).await?;
//! let count: i64       = executor::count(&pool, &User::query()).await?;
//! executor::update(&pool, User::query().where_eq("id", 1i64), &[("name", "Bob".into())]).await?;
//! executor::delete(&pool, User::find(42i64)).await?;
//! ```

use chrono::Utc;
use crate::model::Model;
use crate::query::{Dialect, QueryBuilder, SqlValue};
use sqlx::mysql::{MyPool, MyRow};

pub async fn fetch_all<T>(pool: &MyPool, builder: QueryBuilder<T>) -> Result<Vec<T>, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, MyRow> + Send + Unpin,
{
    let (sql, params) = builder.to_sql_with_dialect(Dialect::Mysql);
    fetch_all_as::<T>(pool, &sql, params).await
}

pub async fn fetch_all_as<T>(pool: &MyPool, sql: &str, params: Vec<SqlValue>) -> Result<Vec<T>, sqlx::Error>
where
    T: for<'r> sqlx::FromRow<'r, MyRow> + Send + Unpin,
{
    let mut query = sqlx::query(sql);
    for p in params {
        add_param(&mut query, p);
    }
    query.fetch_all(pool).await
}

pub async fn fetch_optional<T>(pool: &MyPool, builder: QueryBuilder<T>) -> Result<Option<T>, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, MyRow> + Send + Unpin,
{
    let (sql, params) = builder.to_sql_with_dialect(Dialect::Mysql);
    fetch_optional_as::<T>(pool, &sql, params).await
}

pub async fn fetch_optional_as<T>(pool: &MyPool, sql: &str, params: Vec<SqlValue>) -> Result<Option<T>, sqlx::Error>
where
    T: for<'r> sqlx::FromRow<'r, MyRow> + Send + Unpin,
{
    let mut query = sqlx::query(sql);
    for p in params {
        add_param(&mut query, p);
    }
    query.fetch_optional(pool).await
}

pub async fn count<T>(pool: &MyPool, builder: QueryBuilder<T>) -> Result<i64, sqlx::Error> {
    let (sql, params) = builder.to_count_sql_with_dialect(Dialect::Mysql);
    let mut query = sqlx::query(&sql);
    for p in params {
        add_param(&mut query, p);
    }
    let row = query.fetch_one(pool).await?;
    row.try_get::<i64, _>(0)
}

pub async fn execute(pool: &MyPool, sql: &str, params: Vec<SqlValue>) -> Result<u64, sqlx::Error> {
    let mut query = sqlx::query(sql);
    for p in params {
        add_param(&mut query, p);
    }
    query.execute(pool).await.map(|r| r.rows_affected())
}

pub async fn insert<T>(
    pool: &MyPool,
    table: &str,
    data: &[(&str, SqlValue)],
) -> Result<u64, sqlx::Error> {
    let (sql, params) = QueryBuilder::<T>::insert_sql_with_dialect(Dialect::Mysql, table, data);
    execute(pool, &sql, params).await
}

pub async fn update<T>(
    pool: &MyPool,
    builder: QueryBuilder<T>,
    data: &[(&str, SqlValue)],
) -> Result<u64, sqlx::Error> {
    let (sql, params) = builder.to_update_sql_with_dialect(Dialect::Mysql, data);
    execute(pool, &sql, params).await
}

pub async fn delete<T>(pool: &MyPool, builder: QueryBuilder<T>) -> Result<u64, sqlx::Error> {
    let (sql, params) = builder.to_delete_sql_with_dialect(Dialect::Mysql);
    execute(pool, &sql, params).await
}

pub async fn bulk_insert<T>(
    pool: &MyPool,
    table: &str,
    rows: &[Vec<(&str, SqlValue)>],
) -> Result<u64, sqlx::Error> {
    if rows.is_empty() {
        return Ok(0);
    }
    let (sql, params) = QueryBuilder::<T>::bulk_insert_sql(table, rows);
    execute(pool, &sql, params).await
}

pub async fn insert_returning<T>(
    pool: &MyPool,
    table: &str,
    data: &[(&str, SqlValue)],
) -> Result<T, sqlx::Error>
where
    T: for<'r> sqlx::FromRow<'r, MyRow> + Send + Unpin,
{
    let (sql, params) = QueryBuilder::<T>::insert_sql_with_dialect(Dialect::Mysql, table, data);
    let full_sql = format!("{sql}; SELECT LAST_INSERT_ID()");
    let mut query = sqlx::query(&full_sql);
    for p in &params {
        add_param(&mut query, p.clone());
    }
    query.fetch_all(pool).await?;
    drop(query);
    let (sql2, params2) = QueryBuilder::<T>::insert_sql_with_dialect(Dialect::Mysql, table, data);
    let select_sql = format!("SELECT * FROM {table} WHERE id = LAST_INSERT_ID()");
    let mut query = sqlx::query(&select_sql);
    for p in params2 {
        add_param(&mut query, p);
    }
    query.fetch_one(pool).await.map_err(|e| e.into())
}

pub async fn restore<T>(pool: &MyPool, builder: QueryBuilder<T>) -> Result<u64, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, MyRow> + Send + Unpin,
{
    if let Some(col) = T::soft_delete_column() {
        let mut updated_builder = builder.with_trashed();
        updated_builder.push_update_column(col, SqlValue::Null);
        let (sql, params) = updated_builder.to_restore_sql_with_dialect(Dialect::Mysql);
        execute(pool, &sql, params).await
    } else {
        Err(sqlx::Error::Protocol(
            "restore() called on model without soft_delete_column".into(),
        ))
    }
}

pub async fn force_delete<T>(pool: &MyPool, builder: QueryBuilder<T>) -> Result<u64, sqlx::Error> {
    let (sql, params) = builder.to_force_delete_sql_with_dialect(Dialect::Mysql);
    execute(pool, &sql, params).await
}

pub async fn aggregate<T>(
    pool: &MyPool,
    builder: QueryBuilder<T>,
    func: &str,
    column: &str,
) -> Result<Option<f64>, sqlx::Error> {
    let (sql, params) = builder.aggregate_sql_with_dialect(Dialect::Mysql, func, column);
    let mut query = sqlx::query(&sql);
    for p in params {
        add_param(&mut query, p);
    }
    let row = query.fetch_optional(pool).await?;
    match row {
        Some(r) => r.try_get::<Option<f64>, _>(0),
        None => Ok(None),
    }
}

pub async fn upsert<T>(
    pool: &MyPool,
    table: &str,
    data: &[(&str, SqlValue)],
    conflict_column: &str,
    update_columns: &[&str],
) -> Result<u64, sqlx::Error> {
    let (sql, params) =
        QueryBuilder::<T>::upsert_sql_with_dialect(Dialect::Mysql, table, data, conflict_column, update_columns);
    execute(pool, &sql, params).await
}

pub async fn upsert_returning<T>(
    pool: &MyPool,
    table: &str,
    data: &[(&str, SqlValue)],
    conflict_column: &str,
    update_columns: &[&str],
) -> Result<T, sqlx::Error>
where
    T: for<'r> sqlx::FromRow<'r, MyRow> + Send + Unpin,
{
    let (sql, params) =
        QueryBuilder::<T>::upsert_sql_with_dialect(Dialect::Mysql, table, data, conflict_column, update_columns);
    let full_sql = format!("{sql}; SELECT * FROM {table} WHERE {conflict_column} = VALUES({conflict_column})");
    fetch_all_as::<T>(pool, &full_sql, params)
        .await?
        .into_iter()
        .next()
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn delete_in<T>(pool: &MyPool, table: &str, column: &str, values: Vec<SqlValue>) -> Result<u64, sqlx::Error> {
    if values.is_empty() {
        return Ok(0);
    }
    let (sql, params) =
        QueryBuilder::<T>::new(table).to_delete_in_sql_with_dialect(Dialect::Mysql, column, &values);
    execute(pool, &sql, params).await
}

pub async fn exists<T>(pool: &MyPool, builder: QueryBuilder<T>) -> Result<bool, sqlx::Error> {
    let (sql, params) = builder.exists_sql_with_dialect(Dialect::Mysql);
    let mut query = sqlx::query(&sql);
    for p in params {
        add_param(&mut query, p);
    }
    let row = query.fetch_one(pool).await?;
    row.try_get::<bool, _>(0)
}

#[allow(dead_code)]
pub async fn pluck<T>(
    _pool: &MyPool,
    _builder: QueryBuilder<T>,
    _column: &str,
) -> Result<Vec<SqlValue>, sqlx::Error> {
    unimplemented!("pluck requires concrete types; use a typed query instead")
}
}

pub async fn update_all<T>(
    pool: &MyPool,
    builder: QueryBuilder<T>,
    data: &[(&str, SqlValue)],
) -> Result<u64, sqlx::Error> {
    let (sql, params) = builder.to_update_sql_with_dialect(Dialect::Mysql, data);
    execute(pool, &sql, params).await
}

fn table_name<T: Model>() -> &'static str {
    T::table_name()
}

fn add_param(query: &mut sqlx::query::Query<sqlx::MySql, sqlx::mysql::MySqlArguments>, value: SqlValue) {
    match value {
        SqlValue::Null => query.bind(None::<String>),
        SqlValue::Bool(v) => query.bind(v),
        SqlValue::I32(v) => query.bind(v),
        SqlValue::I64(v) => query.bind(v),
        SqlValue::F64(v) => query.bind(v),
        SqlValue::Text(v) => query.bind(v),
        SqlValue::Binary(v) => query.bind(v),
    };
}
