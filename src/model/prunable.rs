//! [`Prunable`] trait — delete old/expired model records on a schedule.
//!
//! Implement `prunable_query()` to define which records should be deleted.
//! Call `prune(pool)` to execute the deletion.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::{Model, Prunable, QueryBuilder};
//!
//! #[derive(Model, sqlx::FromRow)]
//! #[model(table = "activity_logs")]
//! pub struct ActivityLog {
//!     pub id: i64,
//!     pub created_at: String,
//! }
//!
//! impl Prunable for ActivityLog {
//!     fn prunable_query() -> QueryBuilder<Self> {
//!         // Delete logs older than 30 days
//!         ActivityLog::query().where_raw("created_at < NOW() - INTERVAL '30 days'")
//!     }
//! }
//!
//! let deleted = ActivityLog::prune(&pool).await?;
//! ```

use std::sync::OnceLock;

use crate::model::Model;
use crate::query::QueryBuilder;

/// A model that can prune (delete) its own expired or old records.
///
/// Implement `prunable_query()` to define the set of records to delete.
/// The default `prune()` implementation uses this query to issue a DELETE.
#[allow(async_fn_in_trait)]
pub trait Prunable: Model + Sized {
    /// Return a `QueryBuilder` scoped to the records that should be deleted.
    fn prunable_query() -> QueryBuilder<Self>;

    /// Delete all records matching [`prunable_query()`] and return the count.
    #[cfg(feature = "postgres")]
    async fn prune(pool: &sqlx::PgPool) -> Result<u64, sqlx::Error> {
        use crate::executor::postgres;
        postgres::delete(pool, Self::prunable_query()).await
    }

    /// Delete all records matching [`prunable_query()`] and return the count (SQLite).
    #[cfg(feature = "sqlite")]
    async fn prune_sqlite(pool: &sqlx::SqlitePool) -> Result<u64, sqlx::Error> {
        use crate::executor::sqlite;
        sqlite::delete(pool, Self::prunable_query()).await
    }

    /// Delete all records matching [`prunable_query()`] and return the count (MySQL).
    #[cfg(feature = "mysql")]
    async fn prune_mysql(pool: &sqlx::MySqlPool) -> Result<u64, sqlx::Error> {
        use crate::executor::mysql;
        mysql::delete(pool, Self::prunable_query()).await
    }
}

// ── PrunableRegistry ─────────────────────────────────────────────────────────

/// A type-erased prune function: returns `(DELETE sql, params)` on call.
type PruneFn = Box<dyn Fn() -> (String, Vec<crate::query::SqlValue>) + Send + Sync>;

static PRUNE_REGISTRY: OnceLock<std::sync::RwLock<Vec<PruneFn>>> = OnceLock::new();

fn registry() -> &'static std::sync::RwLock<Vec<PruneFn>> {
    PRUNE_REGISTRY.get_or_init(|| std::sync::RwLock::new(Vec::new()))
}

/// Batch pruner — register models once at startup, then call `prune_all(pool)`.
pub struct PrunableRegistry;

impl PrunableRegistry {
    /// Register a `Prunable` model so that `prune_all` will include it.
    pub fn register<M: Prunable + 'static>() {
        let f: PruneFn = Box::new(|| {
            let (sql, _) = M::prunable_query().to_delete_sql();
            (sql, vec![])
        });
        registry().write().unwrap().push(f);
    }

    /// Run every registered model's prunable delete and return total rows affected.
    #[cfg(feature = "postgres")]
    pub async fn prune_all(pool: &sqlx::PgPool) -> Result<u64, sqlx::Error> {
        use crate::executor::postgres;
        let fns: Vec<(String, Vec<crate::query::SqlValue>)> = {
            registry().read().unwrap().iter().map(|f| f()).collect()
        };
        let mut total = 0u64;
        for (sql, params) in fns {
            total += postgres::execute_raw(pool, &sql, params).await?;
        }
        Ok(total)
    }

    /// Run every registered model's prunable delete and return total rows affected (SQLite).
    #[cfg(feature = "sqlite")]
    pub async fn prune_all_sqlite(pool: &sqlx::SqlitePool) -> Result<u64, sqlx::Error> {
        use crate::executor::sqlite;
        let fns: Vec<(String, Vec<crate::query::SqlValue>)> = {
            registry().read().unwrap().iter().map(|f| f()).collect()
        };
        let mut total = 0u64;
        for (sql, params) in fns {
            total += sqlite::execute_raw(pool, &sql, params).await?;
        }
        Ok(total)
    }

    /// Run every registered model's prunable delete and return total rows affected (MySQL).
    #[cfg(feature = "mysql")]
    pub async fn prune_all_mysql(pool: &sqlx::MySqlPool) -> Result<u64, sqlx::Error> {
        use crate::executor::mysql;
        let fns: Vec<(String, Vec<crate::query::SqlValue>)> = {
            registry().read().unwrap().iter().map(|f| f()).collect()
        };
        let mut total = 0u64;
        for (sql, params) in fns {
            total += mysql::execute(pool, &sql, params).await?;
        }
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::QueryBuilder;

    struct LogModel;
    impl Model for LogModel {
        fn table_name() -> &'static str { "logs" }
        fn columns() -> &'static [&'static str] { &["id", "created_at"] }
    }

    impl Prunable for LogModel {
        fn prunable_query() -> QueryBuilder<Self> {
            LogModel::query().where_raw("created_at < '2020-01-01'")
        }
    }

    #[test]
    fn prunable_query_generates_delete_sql() {
        let (sql, _) = LogModel::prunable_query().to_delete_sql();
        assert!(sql.contains("DELETE FROM logs"));
        assert!(sql.contains("WHERE created_at < '2020-01-01'"));
    }
}
