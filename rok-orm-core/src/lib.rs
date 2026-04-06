//! rok-orm-core — traits and query builder for the rok ORM.

pub mod condition;
pub mod model;
pub mod query;

/// PostgreSQL binding helpers.  Enable with `features = ["sqlx-postgres"]`.
#[cfg(feature = "sqlx-postgres")]
pub mod sqlx_pg;

/// SQLite binding helpers.  Enable with `features = ["sqlx-sqlite"]`.
#[cfg(feature = "sqlx-sqlite")]
pub mod sqlx_sqlite;

pub use condition::{Condition, JoinOp, OrderDir, SqlValue};
pub use model::Model;
pub use query::{Dialect, Join, QueryBuilder};
