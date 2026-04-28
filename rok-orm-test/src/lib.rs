//! # rok-orm-test
//!
//! Testing utilities for [`rok-orm`](https://docs.rs/rok-orm):
//!
//! - **12.1 Factories** — fluent [`Factory`] / [`FactoryBuilder`] API for
//!   generating realistic test data
//! - **12.2 Transaction isolation** — [`TestDb`] provides a fresh, isolated
//!   database handle per test; the `#[rok_orm_test::test]` attribute macro
//!   wires up setup + teardown automatically
//! - **12.3 Assertion helpers** — expressive [`assert_db`] functions that
//!   panic with descriptive messages on failure
//!
//! ## Feature flags
//!
//! | Feature    | Effect                                         |
//! |------------|------------------------------------------------|
//! | `sqlite`   | Enable SQLite pool support and assertions      |
//! | `postgres` | Enable Postgres pool support and assertions    |
//! | `fake`     | Re-export the `fake` crate for faker values   |
//!
//! ## Quick example
//!
//! ```rust,ignore
//! use rok_orm_test::{Factory, assert_db};
//! use rok_orm::SqlValue;
//!
//! pub struct UserFactory;
//!
//! impl Factory for UserFactory {
//!     type Model = User;
//!     fn definition() -> Vec<(&'static str, SqlValue)> {
//!         vec![("name", "Alice".into()), ("email", "alice@example.com".into())]
//!     }
//! }
//!
//! #[rok_orm_test::test(sqlite)]
//! async fn test_create_user(db: &TestDb) -> rok_orm::OrmResult<()> {
//!     let user = UserFactory::new().create(&db.pool()).await?;
//!     assert_db::model_exists::<User>(db.sqlite_pool(), user.id).await;
//!     Ok(())
//! }
//! ```

pub mod assert_db;
pub mod factory;
pub mod test_db;

pub use factory::{Factory, FactoryBuilder};
pub use test_db::{TestDb, TestDbPool};

// Re-export the proc-macro attribute so users can write
// `#[rok_orm_test::test(sqlite)]` without a separate import.
pub use rok_orm_test_macros::test;

// Optionally re-export `fake` when the feature is enabled.
#[cfg(feature = "fake")]
pub use fake;
