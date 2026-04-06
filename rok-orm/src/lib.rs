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

pub use rok_orm_core::{Condition, Dialect, Join, JoinOp, Model, OrderDir, QueryBuilder, SqlValue};
pub use rok_orm_macros::{Model, Relations};
pub use rok_orm_macros::query;

pub mod relations;
pub use relations::{BelongsTo, HasMany, HasOne, Relation, Relations};

pub mod belongs_to_many;
pub use belongs_to_many::BelongsToMany;

pub mod hooks;
pub use hooks::{HookError, HookType, ModelHooks, HookExecutor};

pub mod eager;
pub mod pagination;
pub use pagination::{Page, PaginationOptions};

#[cfg(feature = "postgres")]
pub mod executor;

#[cfg(feature = "postgres")]
pub mod pg_model;

#[cfg(feature = "postgres")]
pub mod transaction;

#[cfg(feature = "postgres")]
pub use pg_model::PgModel;

#[cfg(feature = "postgres")]
pub use transaction::Tx;

#[cfg(feature = "sqlite")]
pub mod sqlite_executor;

#[cfg(feature = "sqlite")]
pub mod sqlite_model;

#[cfg(feature = "sqlite")]
pub use sqlite_model::SqliteModel;
