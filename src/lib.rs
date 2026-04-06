//! rok-orm — Eloquent-inspired ORM for the rok ecosystem.
//!
//! # Quick start
//!
//! ```rust
//! use rok_orm::Model;
//!
//! #[derive(Model)]
//! pub struct User {
//!     pub id: i64,
//!     pub name: String,
//!     pub email: String,
//! }
//!
//! // Generated table name and columns
//! assert_eq!(User::table_name(), "users");
//! assert_eq!(User::columns(), &["id", "name", "email"]);
//!
//! // Build a query
//! let (sql, params) = User::query()
//!     .filter("active", true)
//!     .order_by_desc("created_at")
//!     .limit(10)
//!     .to_sql();
//!
//! assert!(sql.contains("FROM users"));
//! assert!(sql.contains("LIMIT 10"));
//! ```
//!
//! # Crate Features
//!
//! - `postgres` — PostgreSQL support
//! - `sqlite` — SQLite support  
//! - `mysql` — MySQL support

// ── Core ─────────────────────────────────────────────────────────────────────

pub mod query;
pub use query::{Condition, Dialect, Join, JoinOp, OrderDir, QueryBuilder, SqlValue};

pub mod model;
pub use model::Model;
#[cfg(feature = "postgres")]
pub use model::PgModel;
#[cfg(feature = "sqlite")]
pub use model::SqliteModel;
#[cfg(feature = "mysql")]
pub use model::MyModel;

// ── Executor ─────────────────────────────────────────────────────────────────

#[cfg(feature = "postgres")]
pub mod executor;

// ── Relations ────────────────────────────────────────────────────────────────

pub mod relations;
pub use relations::{BelongsTo, HasMany, HasOne, Relation, Relations, BelongsToMany};

#[cfg(feature = "postgres")]
pub use relations::eager::{BelongsToEager, HasManyEager, HasOneEager};

// ── Additional Modules ──────────────────────────────────────────────────────

pub mod pagination;
pub use pagination::{Page, PaginationOptions};

pub mod scopes;
pub mod errors;
pub use errors::{OrmError, OrmResult, IntoOrmResult};

pub mod logging;
pub use logging::{Logger, LogLevel, LogEntry, QueryTimer};

pub mod hooks;
pub use hooks::{HookError, HookType, ModelHooks, HookExecutor};

#[cfg(feature = "postgres")]
pub mod transaction;

#[cfg(feature = "postgres")]
pub use transaction::Tx;

// ── Macros ───────────────────────────────────────────────────────────────────
// proc-macro crate handles: #[derive(Model)], #[derive(Relations)], #[derive(ModelHooks)], query!
