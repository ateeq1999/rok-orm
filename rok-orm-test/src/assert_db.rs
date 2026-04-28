//! Database assertion helpers for tests (Phase 12.3).
//!
//! All functions are `async` and panic with a descriptive message on failure.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm_test::assert_db;
//!
//! // Model-level
//! assert_db::model_exists::<User>(&pool, 1).await;
//! assert_db::model_count::<User>(&pool, 5).await;
//!
//! // Raw table
//! assert_db::database_has("users", &[("email", "alice@example.com".into())], &pool).await;
//! assert_db::table_empty("sessions", &pool).await;
//! ```

use rok_orm::SqlValue;

// ── Helpers shared between dialects ──────────────────────────────────────────

/// Build a `WHERE col = ? AND ...` clause + bind list from a conditions slice.
fn build_where(conditions: &[(&str, SqlValue)]) -> (String, Vec<SqlValue>) {
    let clause: Vec<String> = conditions
        .iter()
        .enumerate()
        .map(|(i, (col, _))| format!("{} = ${}", col, i + 1))
        .collect();
    let binds: Vec<SqlValue> = conditions.iter().map(|(_, v)| v.clone()).collect();
    (clause.join(" AND "), binds)
}

// ── SQLite assertions ─────────────────────────────────────────────────────────

#[cfg(feature = "sqlite")]
pub mod sqlite {
    use super::*;
    use rok_orm::SqliteModel;
    use sqlx::SqlitePool;

    // ── Internal helpers ──────────────────────────────────────────────────

    async fn raw_count(table: &str, conditions: &[(&str, SqlValue)], pool: &SqlitePool) -> i64 {
        let (where_clause, binds) = build_where(conditions);
        let sql = if conditions.is_empty() {
            format!("SELECT COUNT(*) FROM {}", table)
        } else {
            format!("SELECT COUNT(*) FROM {} WHERE {}", table, where_clause)
        };

        let mut q = sqlx::query_scalar::<_, i64>(&sql);
        for bind in &binds {
            q = match bind {
                SqlValue::Text(s) => q.bind(s.as_str()),
                SqlValue::Integer(n) => q.bind(*n),
                SqlValue::Float(f) => q.bind(*f),
                SqlValue::Bool(b) => q.bind(*b),
                SqlValue::Null => q.bind(Option::<i64>::None),
                _ => q.bind(Option::<i64>::None),
            };
        }
        q.fetch_one(pool).await.expect("assert_db: COUNT query failed")
    }

    // ── Model-level assertions ────────────────────────────────────────────

    /// Assert that a model with the given PK exists.
    ///
    /// # Panics
    ///
    /// Panics with a descriptive message if the record is not found.
    pub async fn model_exists<M>(pool: &SqlitePool, id: impl Into<SqlValue>)
    where
        M: SqliteModel,
    {
        let id_val = id.into();
        let count = raw_count(
            M::table_name(),
            &[(M::primary_key(), id_val.clone())],
            pool,
        )
        .await;
        assert!(
            count > 0,
            "assert_db::model_exists failed — no {} with {} = {:?}",
            M::table_name(),
            M::primary_key(),
            id_val
        );
    }

    /// Assert that no model with the given PK exists.
    ///
    /// # Panics
    ///
    /// Panics with a descriptive message if the record IS found.
    pub async fn model_missing<M>(pool: &SqlitePool, id: impl Into<SqlValue>)
    where
        M: SqliteModel,
    {
        let id_val = id.into();
        let count = raw_count(
            M::table_name(),
            &[(M::primary_key(), id_val.clone())],
            pool,
        )
        .await;
        assert!(
            count == 0,
            "assert_db::model_missing failed — found {} with {} = {:?} (expected none)",
            M::table_name(),
            M::primary_key(),
            id_val
        );
    }

    /// Assert that the total row count for a model equals `expected`.
    ///
    /// # Panics
    ///
    /// Panics with table name, expected count, and actual count.
    pub async fn model_count<M>(pool: &SqlitePool, expected: i64)
    where
        M: SqliteModel,
    {
        let actual = raw_count(M::table_name(), &[], pool).await;
        assert_eq!(
            actual, expected,
            "assert_db::model_count on `{}` — expected {}, got {}",
            M::table_name(), expected, actual
        );
    }

    // ── Raw table assertions ──────────────────────────────────────────────

    /// Assert that at least one row in `table` matches all `conditions`.
    ///
    /// # Panics
    ///
    /// Panics with table name and conditions on failure.
    pub async fn database_has(
        table: &str,
        conditions: &[(&str, SqlValue)],
        pool: &SqlitePool,
    ) {
        let count = raw_count(table, conditions, pool).await;
        assert!(
            count > 0,
            "assert_db::database_has failed — no row in `{}` matching {:?}",
            table,
            conditions.iter().map(|(c, v)| format!("{} = {:?}", c, v)).collect::<Vec<_>>()
        );
    }

    /// Assert that no row in `table` matches the given `conditions`.
    ///
    /// # Panics
    ///
    /// Panics with table name and conditions if a match IS found.
    pub async fn database_missing(
        table: &str,
        conditions: &[(&str, SqlValue)],
        pool: &SqlitePool,
    ) {
        let count = raw_count(table, conditions, pool).await;
        assert!(
            count == 0,
            "assert_db::database_missing failed — found {} row(s) in `{}` matching {:?}",
            count,
            table,
            conditions.iter().map(|(c, v)| format!("{} = {:?}", c, v)).collect::<Vec<_>>()
        );
    }

    /// Assert that `table` has exactly `expected` rows.
    ///
    /// # Panics
    ///
    /// Panics with table name, expected, and actual counts.
    pub async fn table_count(table: &str, expected: i64, pool: &SqlitePool) {
        let actual = raw_count(table, &[], pool).await;
        assert_eq!(
            actual, expected,
            "assert_db::table_count on `{}` — expected {}, got {}",
            table, expected, actual
        );
    }

    /// Assert that `table` has zero rows.
    ///
    /// # Panics
    ///
    /// Panics with the actual row count on failure.
    pub async fn table_empty(table: &str, pool: &SqlitePool) {
        let actual = raw_count(table, &[], pool).await;
        assert_eq!(
            actual, 0,
            "assert_db::table_empty on `{}` — expected 0 rows, got {}",
            table, actual
        );
    }
}

// ── Postgres assertions ───────────────────────────────────────────────────────

#[cfg(feature = "postgres")]
pub mod postgres {
    use super::*;
    use rok_orm::PgModel;
    use sqlx::PgPool;

    async fn raw_count(table: &str, conditions: &[(&str, SqlValue)], pool: &PgPool) -> i64 {
        // Postgres uses $1, $2, … placeholders.
        let (where_clause, binds) = build_where(conditions);
        let sql = if conditions.is_empty() {
            format!("SELECT COUNT(*) FROM {}", table)
        } else {
            format!("SELECT COUNT(*) FROM {} WHERE {}", table, where_clause)
        };

        let mut q = sqlx::query_scalar::<_, i64>(&sql);
        for bind in &binds {
            q = match bind {
                SqlValue::Text(s) => q.bind(s.as_str()),
                SqlValue::Integer(n) => q.bind(*n),
                SqlValue::Float(f) => q.bind(*f),
                SqlValue::Bool(b) => q.bind(*b),
                SqlValue::Null => q.bind(Option::<i64>::None),
                _ => q.bind(Option::<i64>::None),
            };
        }
        q.fetch_one(pool).await.expect("assert_db: COUNT query failed")
    }

    pub async fn model_exists<M>(pool: &PgPool, id: impl Into<SqlValue>)
    where M: PgModel,
    {
        let id_val = id.into();
        let count = raw_count(M::table_name(), &[(M::primary_key(), id_val.clone())], pool).await;
        assert!(count > 0,
            "assert_db::model_exists — no {} with {} = {:?}",
            M::table_name(), M::primary_key(), id_val
        );
    }

    pub async fn model_missing<M>(pool: &PgPool, id: impl Into<SqlValue>)
    where M: PgModel,
    {
        let id_val = id.into();
        let count = raw_count(M::table_name(), &[(M::primary_key(), id_val.clone())], pool).await;
        assert!(count == 0,
            "assert_db::model_missing — found {} with {} = {:?}",
            M::table_name(), M::primary_key(), id_val
        );
    }

    pub async fn model_count<M>(pool: &PgPool, expected: i64)
    where M: PgModel,
    {
        let actual = raw_count(M::table_name(), &[], pool).await;
        assert_eq!(actual, expected,
            "assert_db::model_count on `{}` — expected {}, got {}",
            M::table_name(), expected, actual
        );
    }

    pub async fn database_has(table: &str, conditions: &[(&str, SqlValue)], pool: &PgPool) {
        let count = raw_count(table, conditions, pool).await;
        assert!(count > 0,
            "assert_db::database_has — no row in `{}` matching {:?}",
            table,
            conditions.iter().map(|(c, v)| format!("{} = {:?}", c, v)).collect::<Vec<_>>()
        );
    }

    pub async fn database_missing(table: &str, conditions: &[(&str, SqlValue)], pool: &PgPool) {
        let count = raw_count(table, conditions, pool).await;
        assert!(count == 0,
            "assert_db::database_missing — found {} row(s) in `{}` matching {:?}",
            count, table,
            conditions.iter().map(|(c, v)| format!("{} = {:?}", c, v)).collect::<Vec<_>>()
        );
    }

    pub async fn table_count(table: &str, expected: i64, pool: &PgPool) {
        let actual = raw_count(table, &[], pool).await;
        assert_eq!(actual, expected,
            "assert_db::table_count on `{}` — expected {}, got {}",
            table, expected, actual
        );
    }

    pub async fn table_empty(table: &str, pool: &PgPool) {
        let actual = raw_count(table, &[], pool).await;
        assert_eq!(actual, 0,
            "assert_db::table_empty on `{}` — expected 0, got {}", table, actual
        );
    }
}

// ── Unified assert_db via TestDbPool ─────────────────────────────────────────
//
// These accept `&TestDbPool` and dispatch to the right dialect module.
// They mirror the functions in the `sqlite` / `postgres` sub-modules.

/// Assert that at least one row in `table` matches all `conditions`.
pub async fn database_has<'p>(
    table: &str,
    conditions: &[(&str, SqlValue)],
    pool: &crate::TestDbPool<'p>,
) {
    match pool {
        #[cfg(feature = "sqlite")]
        crate::TestDbPool::Sqlite(p) => sqlite::database_has(table, conditions, p).await,
        #[cfg(feature = "postgres")]
        crate::TestDbPool::Postgres(p) => postgres::database_has(table, conditions, p).await,
        _ => panic!("assert_db::database_has — no dialect feature enabled"),
    }
}

/// Assert that no row in `table` matches the given `conditions`.
pub async fn database_missing<'p>(
    table: &str,
    conditions: &[(&str, SqlValue)],
    pool: &crate::TestDbPool<'p>,
) {
    match pool {
        #[cfg(feature = "sqlite")]
        crate::TestDbPool::Sqlite(p) => sqlite::database_missing(table, conditions, p).await,
        #[cfg(feature = "postgres")]
        crate::TestDbPool::Postgres(p) => postgres::database_missing(table, conditions, p).await,
        _ => panic!("assert_db::database_missing — no dialect feature enabled"),
    }
}

/// Assert that `table` has exactly `expected` rows.
pub async fn table_count<'p>(table: &str, expected: i64, pool: &crate::TestDbPool<'p>) {
    match pool {
        #[cfg(feature = "sqlite")]
        crate::TestDbPool::Sqlite(p) => sqlite::table_count(table, expected, p).await,
        #[cfg(feature = "postgres")]
        crate::TestDbPool::Postgres(p) => postgres::table_count(table, expected, p).await,
        _ => panic!("assert_db::table_count — no dialect feature enabled"),
    }
}

/// Assert that `table` is empty.
pub async fn table_empty<'p>(table: &str, pool: &crate::TestDbPool<'p>) {
    match pool {
        #[cfg(feature = "sqlite")]
        crate::TestDbPool::Sqlite(p) => sqlite::table_empty(table, p).await,
        #[cfg(feature = "postgres")]
        crate::TestDbPool::Postgres(p) => postgres::table_empty(table, p).await,
        _ => panic!("assert_db::table_empty — no dialect feature enabled"),
    }
}
