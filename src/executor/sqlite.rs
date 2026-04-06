//! Async SQLite executor ΓÇö runs [`QueryBuilder`] output against a live pool.
//!
//! Requires the `sqlite` feature:
//!
//! ```toml
//! rok-orm = { version = "0.1", features = ["sqlite"] }
//! ```
//!
//! All SQL is generated with `?` placeholders via [`Dialect::Sqlite`].

use crate::model::Model;
use crate::query::{Dialect, QueryBuilder, SqlValue};
use crate::executor::sqlx_sqlite;
use sqlx::sqlite::SqliteRow;
use sqlx::SqlitePool;

/// Fetch all rows matching the query.
pub async fn fetch_all<T>(
    pool: &SqlitePool,
    builder: QueryBuilder<T>,
) -> Result<Vec<T>, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, SqliteRow> + Send + Unpin,
{
    let (sql, params) = builder.to_sql_with_dialect(Dialect::Sqlite);
    sqlx_sqlite::fetch_all_as::<T>(pool, &sql, params).await
}

/// Fetch at most one row matching the query.  Returns `None` if no rows match.
pub async fn fetch_optional<T>(
    pool: &SqlitePool,
    builder: QueryBuilder<T>,
) -> Result<Option<T>, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, SqliteRow> + Send + Unpin,
{
    let (sql, params) = builder.to_sql_with_dialect(Dialect::Sqlite);
    sqlx_sqlite::fetch_optional_as::<T>(pool, &sql, params).await
}

/// Return the row count matching the query's WHERE clause.
pub async fn count<T>(pool: &SqlitePool, builder: QueryBuilder<T>) -> Result<i64, sqlx::Error> {
    let (sql, params) = builder.to_count_sql_with_dialect(Dialect::Sqlite);
    let row = sqlx_sqlite::build_query(&sql, params).fetch_one(pool).await?;
    use sqlx::Row;
    row.try_get::<i64, _>(0)
}

/// Execute a raw SQL string and return rows affected.
pub async fn execute_raw(
    pool: &SqlitePool,
    sql: &str,
    params: Vec<SqlValue>,
) -> Result<u64, sqlx::Error> {
    sqlx_sqlite::execute(pool, sql, params).await
}

/// Insert a row and return rows affected.
pub async fn insert<T>(
    pool: &SqlitePool,
    table: &str,
    data: &[(&str, SqlValue)],
) -> Result<u64, sqlx::Error> {
    let (sql, params) = QueryBuilder::<T>::insert_sql_with_dialect(Dialect::Sqlite, table, data);
    execute_raw(pool, &sql, params).await
}

/// Update rows matching the builder's conditions and return rows affected.
pub async fn update<T>(
    pool: &SqlitePool,
    builder: QueryBuilder<T>,
    data: &[(&str, SqlValue)],
) -> Result<u64, sqlx::Error> {
    let (sql, params) = builder.to_update_sql_with_dialect(Dialect::Sqlite, data);
    execute_raw(pool, &sql, params).await
}

/// Delete rows matching the builder's conditions and return rows affected.
pub async fn delete<T>(pool: &SqlitePool, builder: QueryBuilder<T>) -> Result<u64, sqlx::Error> {
    let (sql, params) = builder.to_delete_sql_with_dialect(Dialect::Sqlite);
    execute_raw(pool, &sql, params).await
}

/// Insert a single row and return it via `RETURNING *`.
///
/// Requires SQLite 3.35+ (released 2021-03-12).
pub async fn insert_returning<T>(
    pool: &SqlitePool,
    table: &str,
    data: &[(&str, SqlValue)],
) -> Result<T, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, SqliteRow> + Send + Unpin,
{
    let (base_sql, params) =
        QueryBuilder::<T>::insert_sql_with_dialect(Dialect::Sqlite, table, data);
    let sql = format!("{base_sql} RETURNING *");
    sqlx_sqlite::fetch_all_as::<T>(pool, &sql, params)
        .await?
        .into_iter()
        .next()
        .ok_or(sqlx::Error::RowNotFound)
}

/// Restore soft-deleted rows by setting the soft_delete_column to NULL.
///
/// ```rust,ignore
/// use rok_orm::SqliteModel;
///
/// let restored = Note::restore(&pool, 42i64).await?;
/// ```
pub async fn restore<T>(
    pool: &SqlitePool,
    builder: QueryBuilder<T>,
) -> Result<u64, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, SqliteRow> + Send + Unpin,
{
    if let Some(col) = T::soft_delete_column() {
        let updated_builder = builder.with_trashed().push_update_column(col, SqlValue::Null);
        let (sql, params) = updated_builder.to_restore_sql_with_dialect(Dialect::Sqlite);
        execute_raw(pool, &sql, params).await
    } else {
        Err(sqlx::Error::Protocol(
            "restore() called on model without soft_delete_column".into(),
        ))
    }
}

/// Permanently delete rows, bypassing soft delete filters.
///
/// ```rust,ignore
/// use rok_orm::SqliteModel;
///
/// let deleted = Note::force_delete(&pool, 42i64).await?;
/// ```
pub async fn force_delete<T>(
    pool: &SqlitePool,
    builder: QueryBuilder<T>,
) -> Result<u64, sqlx::Error> {
    let (sql, params) = builder.to_force_delete_sql_with_dialect(Dialect::Sqlite);
    execute_raw(pool, &sql, params).await
}

pub async fn aggregate<T>(
    pool: &SqlitePool,
    builder: QueryBuilder<T>,
    func: &str,
    column: &str,
) -> Result<Option<f64>, sqlx::Error> {
    let (sql, params) = builder.aggregate_sql_with_dialect(Dialect::Sqlite, func, column);
    let row = sqlx_sqlite::build_query(&sql, params).fetch_optional(pool).await?;
    match row {
        Some(r) => {
            use sqlx::Row;
            Ok(r.try_get::<Option<f64>, _>(0)?)
        }
        None => Ok(None),
    }
}

pub async fn upsert<T>(
    pool: &SqlitePool,
    table: &str,
    data: &[(&str, SqlValue)],
    conflict_column: &str,
    update_columns: &[&str],
) -> Result<u64, sqlx::Error> {
    let (sql, params) = QueryBuilder::<T>::upsert_sql_with_dialect(
        Dialect::Sqlite,
        table,
        data,
        conflict_column,
        update_columns,
    );
    execute_raw(pool, &sql, params).await
}

pub async fn delete_in<T>(
    pool: &SqlitePool,
    column: &str,
    values: Vec<SqlValue>,
) -> Result<u64, sqlx::Error>
where
    T: Model,
{
    if values.is_empty() {
        return Ok(0);
    }
    let (sql, params) = QueryBuilder::<T>::new(T::table_name()).to_delete_in_sql_with_dialect(
        Dialect::Sqlite,
        column,
        &values,
    );
    execute_raw(pool, &sql, params).await
}

pub async fn exists<T>(
    pool: &SqlitePool,
    builder: QueryBuilder<T>,
) -> Result<bool, sqlx::Error> {
    let (sql, params) = builder.exists_sql_with_dialect(Dialect::Sqlite);
    let row = sqlx_sqlite::build_query(&sql, params).fetch_one(pool).await?;
    use sqlx::Row;
    row.try_get::<bool, _>(0)
}

#[allow(dead_code)]
pub async fn pluck<T>(
    _pool: &SqlitePool,
    _builder: QueryBuilder<T>,
    _column: &str,
) -> Result<Vec<SqlValue>, sqlx::Error> {
    unimplemented!("pluck requires concrete types; use a typed query instead")
}

pub async fn update_all<T>(
    pool: &SqlitePool,
    builder: QueryBuilder<T>,
    data: &[(&str, SqlValue)],
) -> Result<u64, sqlx::Error> {
    let (sql, params) = builder.to_update_sql_with_dialect(Dialect::Sqlite, data);
    execute_raw(pool, &sql, params).await
}
