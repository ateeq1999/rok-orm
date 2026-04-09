//! Async PostgreSQL executor ΓÇö runs [`QueryBuilder`] output against a live pool.
//!
//! Requires the `postgres` feature:
//!
//! ```toml
//! rok-orm = { version = "0.4", features = ["postgres"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::{Model, executor};
//!
//! #[derive(Model, sqlx::FromRow)]
//! pub struct User { pub id: i64, pub name: String }
//!
//! let pool = sqlx::PgPool::connect(&database_url).await?;
//!
//! let users: Vec<User> = executor::fetch_all(&pool, User::query().where_eq("active", true)).await?;
//! let count: i64       = executor::count(&pool, &User::query()).await?;
//! executor::update(&pool, User::query().where_eq("id", 1i64), &[("name", "Bob".into())]).await?;
//! executor::delete(&pool, User::find(42i64)).await?;
//! ```

#[cfg(feature = "postgres")]
use crate::model::Model;

#[cfg(feature = "postgres")]
use crate::query::{QueryBuilder, SqlValue};

#[cfg(feature = "postgres")]
use crate::executor::sqlx_pg;

#[cfg(feature = "postgres")]
use sqlx::postgres::PgRow;

#[cfg(feature = "postgres")]
use sqlx::PgPool;

#[cfg(feature = "postgres")]
/// Fetch all rows matching the query.
pub async fn fetch_all<T>(pool: &PgPool, builder: QueryBuilder<T>) -> Result<Vec<T>, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
{
    let (sql, params) = builder.to_sql();
    sqlx_pg::fetch_all_as::<T>(pool, &sql, params).await
}

/// Fetch at most one row matching the query.  Returns `None` if no rows match.
pub async fn fetch_optional<T>(
    pool: &PgPool,
    builder: QueryBuilder<T>,
) -> Result<Option<T>, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
{
    let (sql, params) = builder.to_sql();
    sqlx_pg::fetch_optional_as::<T>(pool, &sql, params).await
}

/// Return the row count matching the query's WHERE clause.
pub async fn count<T>(pool: &PgPool, builder: QueryBuilder<T>) -> Result<i64, sqlx::Error> {
    let (sql, params) = builder.to_count_sql();
    let row = sqlx_pg::build_query(&sql, params).fetch_one(pool).await?;
    use sqlx::Row;
    row.try_get::<i64, _>(0)
}

/// Fetch rows using a raw SQL string with positional parameters.
pub async fn fetch_raw<T>(
    pool: &PgPool,
    sql: &str,
    params: Vec<SqlValue>,
) -> Result<Vec<T>, sqlx::Error>
where
    T: for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
{
    sqlx_pg::fetch_all_as::<T>(pool, sql, params).await
}

/// Execute a raw SQL string with positional parameters and return rows affected.
pub async fn execute_raw(
    pool: &PgPool,
    sql: &str,
    params: Vec<SqlValue>,
) -> Result<u64, sqlx::Error> {
    sqlx_pg::execute(pool, sql, params).await
}

/// Insert a row using the column-value pairs and return rows affected.
pub async fn insert<T>(
    pool: &PgPool,
    table: &str,
    data: &[(&str, SqlValue)],
) -> Result<u64, sqlx::Error> {
    let (sql, params) = QueryBuilder::<T>::insert_sql(table, data);
    execute_raw(pool, &sql, params).await
}

/// Update rows matching the builder's conditions and return rows affected.
pub async fn update<T>(
    pool: &PgPool,
    builder: QueryBuilder<T>,
    data: &[(&str, SqlValue)],
) -> Result<u64, sqlx::Error> {
    let (sql, params) = builder.to_update_sql(data);
    execute_raw(pool, &sql, params).await
}

/// Delete rows matching the builder's conditions and return rows affected.
pub async fn delete<T>(pool: &PgPool, builder: QueryBuilder<T>) -> Result<u64, sqlx::Error> {
    let (sql, params) = builder.to_delete_sql();
    execute_raw(pool, &sql, params).await
}

/// Insert multiple rows in a single `INSERT INTO ΓÇª VALUES ΓÇª, ΓÇª` statement.
///
/// All rows must have the same columns in the same order as the first row.
/// Returns the total number of rows inserted.
///
/// ```rust,ignore
/// use rok_orm::executor;
///
/// executor::bulk_insert::<User>(
///     &pool,
///     "users",
///     &[
///         vec![("name", "Alice".into()), ("email", "a@a.com".into())],
///         vec![("name", "Bob".into()),   ("email", "b@b.com".into())],
///     ],
/// ).await?;
/// ```
pub async fn bulk_insert<T>(
    pool: &PgPool,
    table: &str,
    rows: &[Vec<(&str, SqlValue)>],
) -> Result<u64, sqlx::Error> {
    if rows.is_empty() {
        return Ok(0);
    }
    let (sql, params) = QueryBuilder::<T>::bulk_insert_sql(table, rows);
    execute_raw(pool, &sql, params).await
}

/// Insert a single row and return it using `RETURNING *`.
///
/// Useful when you need the generated primary key or server-side defaults.
///
/// ```rust,ignore
/// use rok_orm::executor;
///
/// let user: User = executor::insert_returning::<User>(
///     &pool,
///     "users",
///     &[("name", "Alice".into()), ("email", "a@a.com".into())],
/// ).await?;
/// println!("new id = {}", user.id);
/// ```
pub async fn insert_returning<T>(
    pool: &PgPool,
    table: &str,
    data: &[(&str, SqlValue)],
) -> Result<T, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
{
    let (base_sql, params) = QueryBuilder::<T>::insert_sql(table, data);
    let sql = format!("{base_sql} RETURNING *");
    sqlx_pg::fetch_all_as::<T>(pool, &sql, params)
        .await?
        .into_iter()
        .next()
        .ok_or(sqlx::Error::RowNotFound)
}

/// Restore soft-deleted rows by setting the soft_delete_column to NULL.
///
/// ```rust,ignore
/// use rok_orm::PgModel;
///
/// let restored = User::restore(&pool, 42i64).await?;
/// ```
pub async fn restore<T>(pool: &PgPool, builder: QueryBuilder<T>) -> Result<u64, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
{
    if let Some(col) = T::soft_delete_column() {
        let updated_builder = builder.with_trashed().push_update_column(col, SqlValue::Null);
        let (sql, params) = updated_builder.to_restore_sql();
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
/// use rok_orm::PgModel;
///
/// let deleted = User::force_delete(&pool, 42i64).await?;
/// ```
pub async fn force_delete<T>(
    pool: &PgPool,
    builder: QueryBuilder<T>,
) -> Result<u64, sqlx::Error> {
    let (sql, params) = builder.to_force_delete_sql();
    execute_raw(pool, &sql, params).await
}

// ── Advanced helpers (aggregates, upsert, pluck, extras) ─────────────────────
// Defined in postgres_advanced.rs to stay under 300 lines per file.
pub use super::postgres_advanced::*;
