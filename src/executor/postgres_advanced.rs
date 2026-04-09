//! Advanced PostgreSQL executor helpers: aggregates, upsert, pluck, fetch_with_extras.

use super::sqlx_pg;
use super::postgres::execute_raw;
use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};
use sqlx::{postgres::PgRow, PgPool};

pub async fn aggregate<T>(
    pool: &PgPool,
    builder: QueryBuilder<T>,
    func: &str,
    column: &str,
) -> Result<Option<f64>, sqlx::Error> {
    let (sql, params) = builder.aggregate_sql(func, column);
    let row = sqlx_pg::build_query(&sql, params).fetch_optional(pool).await?;
    match row {
        Some(r) => {
            use sqlx::Row;
            Ok(r.try_get::<Option<f64>, _>(0)?)
        }
        None => Ok(None),
    }
}

pub async fn upsert<T>(
    pool: &PgPool,
    table: &str,
    data: &[(&str, SqlValue)],
    conflict_column: &str,
    update_columns: &[&str],
) -> Result<u64, sqlx::Error> {
    let (sql, params) = QueryBuilder::<T>::upsert_sql(table, data, conflict_column, update_columns);
    execute_raw(pool, &sql, params).await
}

pub async fn upsert_returning<T>(
    pool: &PgPool,
    table: &str,
    data: &[(&str, SqlValue)],
    conflict_column: &str,
    update_columns: &[&str],
) -> Result<T, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    let (sql, params) = QueryBuilder::<T>::upsert_sql(table, data, conflict_column, update_columns);
    let full_sql = format!("{sql} RETURNING *");
    sqlx_pg::fetch_all_as::<T>(pool, &full_sql, params)
        .await?
        .into_iter()
        .next()
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn delete_in<T: Model>(
    pool: &PgPool,
    column: &str,
    values: Vec<SqlValue>,
) -> Result<u64, sqlx::Error> {
    if values.is_empty() {
        return Ok(0);
    }
    let (sql, params) = QueryBuilder::<T>::new(T::table_name()).to_delete_in_sql_with_dialect(
        crate::Dialect::Postgres,
        column,
        &values,
    );
    execute_raw(pool, &sql, params).await
}

pub async fn exists<T>(
    pool: &PgPool,
    builder: QueryBuilder<T>,
) -> Result<bool, sqlx::Error> {
    let (sql, params) = builder.exists_sql();
    let row = sqlx_pg::build_query(&sql, params).fetch_one(pool).await?;
    use sqlx::Row;
    row.try_get::<bool, _>(0)
}

pub async fn update_all<T>(
    pool: &PgPool,
    builder: QueryBuilder<T>,
    data: &[(&str, SqlValue)],
) -> Result<u64, sqlx::Error> {
    let (sql, params) = builder.to_update_sql(data);
    execute_raw(pool, &sql, params).await
}

/// Fetch a single column's values from all matching rows.
pub async fn pluck<T>(
    pool: &PgPool,
    builder: QueryBuilder<T>,
    column: &str,
) -> Result<Vec<SqlValue>, sqlx::Error> {
    let (sql, params) = builder.pluck_sql(column);
    let rows = sqlx_pg::build_query(&sql, params).fetch_all(pool).await?;
    use sqlx::Row;
    let mut values = Vec::with_capacity(rows.len());
    for row in rows {
        let val = if let Ok(n) = row.try_get::<i64, _>(0) {
            SqlValue::Integer(n)
        } else if let Ok(f) = row.try_get::<f64, _>(0) {
            SqlValue::Float(f)
        } else if let Ok(s) = row.try_get::<String, _>(0) {
            SqlValue::Text(s)
        } else if let Ok(b) = row.try_get::<bool, _>(0) {
            SqlValue::Bool(b)
        } else {
            SqlValue::Null
        };
        values.push(val);
    }
    Ok(values)
}

/// Stream rows one-by-one — avoids loading all rows into memory.
///
/// Returns a pinned, boxed `Stream<Item = Result<T, sqlx::Error>>` backed by sqlx's
/// `fetch()` cursor.  Useful for large result sets where `fetch_all` would OOM.
///
/// ```rust,ignore
/// use futures::StreamExt;
/// let mut stream = executor::fetch_stream::<User>(&pool, User::query());
/// while let Some(row) = stream.next().await {
///     let user = row?;
///     process(user).await;
/// }
/// ```
pub fn fetch_stream<'a, T>(
    pool: &'a PgPool,
    builder: QueryBuilder<T>,
) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = Result<T, sqlx::Error>> + Send + 'a>>
where
    T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin + 'static,
{
    let (sql, params) = builder.to_sql();
    // sqlx query_as borrows the SQL string; we need it to outlive the returned stream.
    // Leaking is intentional: parameterized ORM SQL templates are small and bounded in number.
    let sql_static: &'static str = Box::leak(sql.into_boxed_str());
    let query = sqlx_pg::build_query_as::<T>(sql_static, params);
    Box::pin(query.fetch(pool))
}

/// Fetch rows and collect extra aggregate/subquery columns into [`WithExtras<T>`].
///
/// Use when a query includes `with_count_col`, `with_sum_col`, etc.
/// Pass the alias names (e.g. `"comments_count"`) in `extra_cols`.
pub async fn fetch_with_extras<T>(
    pool: &PgPool,
    builder: QueryBuilder<T>,
    extra_cols: &[&str],
) -> Result<Vec<crate::extras::WithExtras<T>>, sqlx::Error>
where
    T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
{
    use sqlx::Row;
    let (sql, params) = builder.to_sql();
    let rows = sqlx_pg::build_query(&sql, params).fetch_all(pool).await?;
    let mut results = Vec::with_capacity(rows.len());
    for row in &rows {
        let model = T::from_row(row)?;
        let mut we = crate::extras::WithExtras::new(model);
        for col in extra_cols {
            let val = if let Ok(n) = row.try_get::<i64, _>(*col) {
                SqlValue::Integer(n)
            } else if let Ok(f) = row.try_get::<f64, _>(*col) {
                SqlValue::Float(f)
            } else if let Ok(s) = row.try_get::<String, _>(*col) {
                SqlValue::Text(s)
            } else if let Ok(b) = row.try_get::<bool, _>(*col) {
                SqlValue::Bool(b)
            } else {
                SqlValue::Null
            };
            we = we.with_extra(*col, val);
        }
        results.push(we);
    }
    Ok(results)
}
