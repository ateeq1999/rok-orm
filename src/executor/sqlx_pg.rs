//! PostgreSQL binding helpers for [`SqlValue`] and [`QueryBuilder`].
//!
//! Enable with `features = ["postgres"]`.
//!
//! ```toml
//! rok-orm = { version = "0.4", features = ["postgres"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::{Model, executor::sqlx_pg};
//! use sqlx::PgPool;
//!
//! let pool = PgPool::connect(&url).await?;
//! let (sql, params) = User::query().where_eq("active", true).to_sql();
//! let rows: Vec<User> = sqlx_pg::fetch_all_as(&pool, &sql, params).await?;
//! ```

#[cfg(feature = "postgres")]
use sqlx::postgres::PgArguments;

#[cfg(feature = "postgres")]
use sqlx::{query::Query, query::QueryAs, Postgres};

#[cfg(feature = "postgres")]
use crate::query::SqlValue;

#[cfg(feature = "postgres")]
// ── low-level binders ─────────────────────────────────────────────────────────

#[cfg(feature = "postgres")]
/// Bind a single [`SqlValue`] to an in-progress sqlx Postgres query.
pub fn bind_value<'q>(
    q: Query<'q, Postgres, PgArguments>,
    v: SqlValue,
) -> Query<'q, Postgres, PgArguments> {
    match v {
        SqlValue::Text(s) => q.bind(s),
        SqlValue::Integer(n) => q.bind(n),
        SqlValue::Float(f) => q.bind(f),
        SqlValue::Bool(b) => q.bind(b),
        SqlValue::Null => q.bind(Option::<String>::None),
    }
}

#[cfg(feature = "postgres")]
/// Bind a single [`SqlValue`] to an in-progress sqlx `query_as` Postgres query.
pub fn bind_value_as<'q, T>(
    q: QueryAs<'q, Postgres, T, PgArguments>,
    v: SqlValue,
) -> QueryAs<'q, Postgres, T, PgArguments>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>,
{
    match v {
        SqlValue::Text(s) => q.bind(s),
        SqlValue::Integer(n) => q.bind(n),
        SqlValue::Float(f) => q.bind(f),
        SqlValue::Bool(b) => q.bind(b),
        SqlValue::Null => q.bind(Option::<String>::None),
    }
}

#[cfg(feature = "postgres")]
// ── convenience builders ────────────────────────────────────────────────────────

#[cfg(feature = "postgres")]
/// Build a sqlx Postgres query from a SQL string + parameter list.
pub fn build_query(sql: &str, params: Vec<SqlValue>) -> Query<'_, Postgres, PgArguments> {
    params
        .into_iter()
        .fold(sqlx::query(sql), |q, v| bind_value(q, v))
}

#[cfg(feature = "postgres")]
/// Build a typed sqlx `query_as` from a SQL string + parameter list.
pub fn build_query_as<T>(sql: &str, params: Vec<SqlValue>) -> QueryAs<'_, Postgres, T, PgArguments>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>,
{
    params
        .into_iter()
        .fold(sqlx::query_as::<_, T>(sql), |q, v| bind_value_as(q, v))
}

#[cfg(feature = "postgres")]
// ── high-level executors ─────────────────────────────────────────────────────

#[cfg(feature = "postgres")]
/// Fetch all rows matching the given SQL and parameters.
pub async fn fetch_all_as<T>(
    pool: &sqlx::PgPool,
    sql: &str,
    params: Vec<SqlValue>,
) -> Result<Vec<T>, sqlx::Error>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    build_query_as::<T>(sql, params).fetch_all(pool).await
}

#[cfg(feature = "postgres")]
/// Fetch at most one row.  Returns `None` if no rows matched.
pub async fn fetch_optional_as<T>(
    pool: &sqlx::PgPool,
    sql: &str,
    params: Vec<SqlValue>,
) -> Result<Option<T>, sqlx::Error>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    build_query_as::<T>(sql, params).fetch_optional(pool).await
}

#[cfg(feature = "postgres")]
/// Execute a SQL statement and return the number of rows affected.
pub async fn execute(
    pool: &sqlx::PgPool,
    sql: &str,
    params: Vec<SqlValue>,
) -> Result<u64, sqlx::Error> {
    let result = build_query(sql, params).execute(pool).await?;
    Ok(result.rows_affected())
}
