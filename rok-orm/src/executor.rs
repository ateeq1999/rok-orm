//! Async PostgreSQL executor — runs [`QueryBuilder`] output against a live pool.
//!
//! Requires the `postgres` feature:
//!
//! ```toml
//! rok-orm = { version = "0.1", features = ["postgres"] }
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

use rok_orm_core::{sqlx_pg, Model, QueryBuilder, SqlValue};
use sqlx::postgres::PgRow;
use sqlx::PgPool;

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

/// Insert multiple rows in a single `INSERT INTO … VALUES …, …` statement.
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
