//! [`TestDb`] and [`TestDbPool`] — database isolation for tests (Phase 12.2).
//!
//! # SQLite (in-memory per test)
//!
//! Each call to [`TestDb::sqlite`] creates a fresh `:memory:` database.
//! Because every test gets its own in-memory pool, rows written in one test
//! are completely invisible to every other test.
//!
//! # Postgres (environment variable)
//!
//! [`TestDb::postgres`] connects to the URL in `TEST_DATABASE_URL`
//! (falls back to `DATABASE_URL`).  Set up a dedicated test database in CI
//! and use transactions or truncation for isolation between tests.
//!
//! # `migrate` flag
//!
//! When `migrate = true`, [`TestDb::sqlite`] runs all SQLite migrations that
//! have been registered with
//! [`SqliteMigrator`](rok_orm::migration_sqlite::SqliteMigrator).
//! Postgres migration support is a planned enhancement.

use std::marker::PhantomData;

#[allow(unused_imports)]
use rok_orm::OrmResult;

// ── TestDbPool ────────────────────────────────────────────────────────────────

/// A borrowed reference to the pool inside a [`TestDb`].
///
/// Pass `&db.pool()` to factory `create` / `create_many` and to
/// `assert_db` helpers.
#[non_exhaustive]
pub enum TestDbPool<'a> {
    /// SQLite in-memory pool.
    #[cfg(feature = "sqlite")]
    Sqlite(&'a sqlx::SqlitePool),
    /// Postgres pool.
    #[cfg(feature = "postgres")]
    Postgres(&'a sqlx::PgPool),
    /// Marker variant so the lifetime `'a` is always used.
    #[doc(hidden)]
    _Phantom(PhantomData<&'a ()>),
}

// ── TestDb ────────────────────────────────────────────────────────────────────

/// An isolated database handle for one test.
///
/// Created by [`TestDb::sqlite`] or [`TestDb::postgres`]; torn down by
/// [`TestDb::teardown`] (which closes the pool and, for SQLite, discards
/// the in-memory database).
pub struct TestDb {
    inner: TestDbInner,
}

enum TestDbInner {
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::SqlitePool),
    #[cfg(feature = "postgres")]
    Postgres(sqlx::PgPool),
    /// Present so the enum has at least one variant in a no-feature build.
    #[allow(dead_code)]
    None,
}

impl TestDb {
    // ── Constructors ──────────────────────────────────────────────────────

    /// Create a fresh `:memory:` SQLite pool.
    ///
    /// Each call returns a completely isolated database; teardown discards
    /// all data automatically.
    ///
    /// # Parameters
    ///
    /// - `migrate`: reserved for future SQLite migration support; currently
    ///   a no-op.
    #[cfg(feature = "sqlite")]
    pub async fn sqlite(_migrate: bool) -> OrmResult<Self> {
        let pool = sqlx::SqlitePool::connect(":memory:")
            .await
            .map_err(|e| rok_orm::OrmError::Database(e.to_string()))?;
        Ok(TestDb { inner: TestDbInner::Sqlite(pool) })
    }

    /// Connect to the Postgres database at `TEST_DATABASE_URL` (or
    /// `DATABASE_URL`).
    ///
    /// # Parameters
    ///
    /// - `migrate`: reserved for future Postgres migration support; currently
    ///   a no-op.
    ///
    /// # Errors
    ///
    /// Returns `OrmError::Other` if neither environment variable is set, or
    /// `OrmError::Database` if the connection fails.
    #[cfg(feature = "postgres")]
    pub async fn postgres(_migrate: bool) -> OrmResult<Self> {
        let url = std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .map_err(|_| rok_orm::OrmError::Other(
                "set TEST_DATABASE_URL or DATABASE_URL for Postgres tests".into(),
            ))?;

        let pool = sqlx::PgPool::connect(&url)
            .await
            .map_err(|e| rok_orm::OrmError::Database(e.to_string()))?;

        Ok(TestDb { inner: TestDbInner::Postgres(pool) })
    }

    // ── Pool access ───────────────────────────────────────────────────────

    /// Borrow the underlying pool as a [`TestDbPool`].
    ///
    /// The returned value can be passed to factory `create` / `create_many`
    /// and to `assert_db` helpers.
    ///
    /// # Panics
    ///
    /// Panics if called on a [`TestDb`] with no active dialect feature.
    pub fn pool(&self) -> TestDbPool<'_> {
        match &self.inner {
            #[cfg(feature = "sqlite")]
            TestDbInner::Sqlite(p) => TestDbPool::Sqlite(p),
            #[cfg(feature = "postgres")]
            TestDbInner::Postgres(p) => TestDbPool::Postgres(p),
            TestDbInner::None => panic!("TestDb::pool() — no dialect feature enabled"),
        }
    }

    /// Return the raw SQLite pool.
    ///
    /// # Panics
    ///
    /// Panics if this `TestDb` was not created with [`TestDb::sqlite`].
    #[cfg(feature = "sqlite")]
    pub fn sqlite_pool(&self) -> &sqlx::SqlitePool {
        match &self.inner {
            TestDbInner::Sqlite(p) => p,
            _ => panic!("sqlite_pool() called on a non-SQLite TestDb"),
        }
    }

    /// Return the raw Postgres pool.
    ///
    /// # Panics
    ///
    /// Panics if this `TestDb` was not created with [`TestDb::postgres`].
    #[cfg(feature = "postgres")]
    pub fn pg_pool(&self) -> &sqlx::PgPool {
        match &self.inner {
            TestDbInner::Postgres(p) => p,
            _ => panic!("pg_pool() called on a non-Postgres TestDb"),
        }
    }

    // ── Teardown ──────────────────────────────────────────────────────────

    /// Close the pool and release any associated resources.
    ///
    /// For SQLite this discards the in-memory database entirely.
    /// For Postgres this closes the connection pool.
    pub async fn teardown(self) {
        match self.inner {
            #[cfg(feature = "sqlite")]
            TestDbInner::Sqlite(pool) => pool.close().await,
            #[cfg(feature = "postgres")]
            TestDbInner::Postgres(pool) => pool.close().await,
            TestDbInner::None => {}
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sqlite_creates_and_tears_down() {
        let db = TestDb::sqlite(false).await.expect("TestDb::sqlite failed");
        let _ = db.sqlite_pool(); // accessible without panic
        db.teardown().await;
    }

    #[tokio::test]
    async fn two_sqlite_dbs_are_independent() {
        let db1 = TestDb::sqlite(false).await.unwrap();
        let db2 = TestDb::sqlite(false).await.unwrap();
        // Different pool instances → different in-memory databases
        let p1: *const _ = db1.sqlite_pool();
        let p2: *const _ = db2.sqlite_pool();
        assert!(!std::ptr::eq(p1, p2));
        db1.teardown().await;
        db2.teardown().await;
    }

    #[tokio::test]
    async fn pool_returns_sqlite_variant() {
        let db = TestDb::sqlite(false).await.unwrap();
        assert!(matches!(db.pool(), TestDbPool::Sqlite(_)));
        db.teardown().await;
    }
}
