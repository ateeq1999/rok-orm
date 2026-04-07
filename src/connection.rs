//! [`ConnectionRegistry`] — named pool registration for per-model connections.
//!
//! Register named pools at startup; models with `#[model(connection = "name")]`
//! can resolve their pool from the registry instead of accepting one as a parameter.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::connection::ConnectionRegistry;
//!
//! // Register pools at startup
//! ConnectionRegistry::register_pg("audit_db", audit_pool);
//! ConnectionRegistry::register_pg("default", main_pool);
//!
//! // Retrieve later
//! let pool = ConnectionRegistry::get_pg("audit_db").unwrap();
//! AuditLog::all(pool).await?;
//! ```

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

// ── PostgreSQL pool registry ────────────────────────────────────────────────

#[cfg(feature = "postgres")]
static PG_REGISTRY: OnceLock<RwLock<HashMap<String, sqlx::PgPool>>> = OnceLock::new();

#[cfg(feature = "postgres")]
fn pg_registry() -> &'static RwLock<HashMap<String, sqlx::PgPool>> {
    PG_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

// ── SQLite pool registry ────────────────────────────────────────────────────

#[cfg(feature = "sqlite")]
static SQLITE_REGISTRY: OnceLock<RwLock<HashMap<String, sqlx::SqlitePool>>> = OnceLock::new();

#[cfg(feature = "sqlite")]
fn sqlite_registry() -> &'static RwLock<HashMap<String, sqlx::SqlitePool>> {
    SQLITE_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

// ── MySQL pool registry ─────────────────────────────────────────────────────

#[cfg(feature = "mysql")]
static MYSQL_REGISTRY: OnceLock<RwLock<HashMap<String, sqlx::MySqlPool>>> = OnceLock::new();

#[cfg(feature = "mysql")]
fn mysql_registry() -> &'static RwLock<HashMap<String, sqlx::MySqlPool>> {
    MYSQL_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Global named-connection registry.
pub struct ConnectionRegistry;

impl ConnectionRegistry {
    /// Register a PostgreSQL pool under `name`.
    #[cfg(feature = "postgres")]
    pub fn register_pg(name: impl Into<String>, pool: sqlx::PgPool) {
        pg_registry().write().unwrap().insert(name.into(), pool);
    }

    /// Retrieve a registered PostgreSQL pool by name.
    #[cfg(feature = "postgres")]
    pub fn get_pg(name: &str) -> Option<sqlx::PgPool> {
        pg_registry().read().unwrap().get(name).cloned()
    }

    /// Register a SQLite pool under `name`.
    #[cfg(feature = "sqlite")]
    pub fn register_sqlite(name: impl Into<String>, pool: sqlx::SqlitePool) {
        sqlite_registry().write().unwrap().insert(name.into(), pool);
    }

    /// Retrieve a registered SQLite pool by name.
    #[cfg(feature = "sqlite")]
    pub fn get_sqlite(name: &str) -> Option<sqlx::SqlitePool> {
        sqlite_registry().read().unwrap().get(name).cloned()
    }

    /// Register a MySQL pool under `name`.
    #[cfg(feature = "mysql")]
    pub fn register_mysql(name: impl Into<String>, pool: sqlx::MySqlPool) {
        mysql_registry().write().unwrap().insert(name.into(), pool);
    }

    /// Retrieve a registered MySQL pool by name.
    #[cfg(feature = "mysql")]
    pub fn get_mysql(name: &str) -> Option<sqlx::MySqlPool> {
        mysql_registry().read().unwrap().get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_registry_struct_exists() {
        // Registry compiles and is accessible
        let _ = std::mem::size_of::<ConnectionRegistry>();
    }
}
