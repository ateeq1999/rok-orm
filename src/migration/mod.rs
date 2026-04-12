//! Migration system — code-first, versioned schema migrations.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use rok_orm::migration::{Migration, Migrator};
//! use rok_orm::schema::Schema;
//!
//! pub struct CreateUsersTable;
//!
//! #[async_trait::async_trait]
//! impl Migration for CreateUsersTable {
//!     fn name(&self) -> &'static str { "001_create_users_table" }
//!
//!     async fn up(&self, pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
//!         Schema::create("users", |t| {
//!             t.id();
//!             t.string("name", 255);
//!             t.string("email", 255).unique();
//!             t.timestamps();
//!         }).execute(pool).await.map_err(rok_orm::OrmError::from)
//!     }
//!
//!     async fn down(&self, pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
//!         Schema::drop_if_exists("users").execute(pool).await.map_err(rok_orm::OrmError::from)
//!     }
//! }
//!
//! let migrator = Migrator::new(&pool)
//!     .add(CreateUsersTable);
//!
//! migrator.run().await?;          // run all pending
//! migrator.rollback(1).await?;    // reverse last batch
//! migrator.status().await?;       // show applied/pending
//! ```
//!
//! The `migrations` table is created automatically the first time `run()` is called.

#[cfg(test)]
mod tests;

use crate::errors::{OrmError, OrmResult};

// ── Migration trait ───────────────────────────────────────────────────────────

/// Implement this trait for every migration file.
#[async_trait::async_trait]
#[cfg(feature = "postgres")]
pub trait Migration: Send + Sync {
    /// Unique, stable identifier — typically `"NNN_description"`.
    fn name(&self) -> &'static str;

    /// Apply this migration (DDL forward).
    async fn up(&self, pool: &sqlx::PgPool) -> OrmResult<()>;

    /// Reverse this migration (DDL rollback).
    async fn down(&self, pool: &sqlx::PgPool) -> OrmResult<()>;
}

// ── MigrationStatus ───────────────────────────────────────────────────────────

/// Status record for one migration.
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub name: String,
    pub batch: Option<i32>,
    pub run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_pending: bool,
}

// ── Migrator ─────────────────────────────────────────────────────────────────

/// Collects migrations and applies them to the database.
#[cfg(feature = "postgres")]
pub struct Migrator<'pool> {
    pool: &'pool sqlx::PgPool,
    migrations: Vec<Box<dyn Migration>>,
}

#[cfg(feature = "postgres")]
impl<'pool> Migrator<'pool> {
    /// Create a new migrator bound to `pool`.
    pub fn new(pool: &'pool sqlx::PgPool) -> Self {
        Self { pool, migrations: Vec::new() }
    }

    /// Register a migration.
    pub fn add(mut self, migration: impl Migration + 'static) -> Self {
        self.migrations.push(Box::new(migration));
        self
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    /// Ensure the `migrations` tracking table exists.
    async fn ensure_migrations_table(&self) -> OrmResult<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS migrations (
                id      BIGSERIAL PRIMARY KEY,
                name    VARCHAR(255) NOT NULL UNIQUE,
                batch   INTEGER NOT NULL,
                run_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(self.pool)
        .await
        .map_err(OrmError::from)?;
        Ok(())
    }

    /// Fetch the set of migration names already applied.
    async fn applied_names(&self) -> OrmResult<std::collections::HashSet<String>> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT name FROM migrations")
            .fetch_all(self.pool)
            .await
            .map_err(OrmError::from)?;
        Ok(rows.into_iter().map(|(n,)| n).collect())
    }

    /// Return the highest batch number used so far (0 if none).
    async fn max_batch(&self) -> OrmResult<i32> {
        let row: (Option<i64>,) =
            sqlx::query_as("SELECT MAX(batch) FROM migrations")
                .fetch_one(self.pool)
                .await
                .map_err(OrmError::from)?;
        Ok(row.0.unwrap_or(0) as i32)
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /// Run all pending migrations in registration order.
    ///
    /// Already-applied migrations are skipped. All new migrations run in a
    /// single batch whose number is `MAX(batch) + 1`.
    pub async fn run(&self) -> OrmResult<()> {
        self.ensure_migrations_table().await?;
        let applied = self.applied_names().await?;
        let next_batch = self.max_batch().await? + 1;

        let pending: Vec<&dyn Migration> = self
            .migrations
            .iter()
            .filter(|m| !applied.contains(m.name()))
            .map(|m| m.as_ref())
            .collect();

        for migration in pending {
            migration.up(self.pool).await?;
            sqlx::query("INSERT INTO migrations (name, batch) VALUES ($1, $2)")
                .bind(migration.name())
                .bind(next_batch)
                .execute(self.pool)
                .await
                .map_err(OrmError::from)?;
        }
        Ok(())
    }

    /// Reverse the last `n` batches (calling `down()` in reverse registration order).
    pub async fn rollback(&self, n: u32) -> OrmResult<()> {
        self.ensure_migrations_table().await?;
        let max_batch = self.max_batch().await?;

        for batch_num in (max_batch - n as i32 + 1..=max_batch).rev() {
            if batch_num < 1 {
                break;
            }
            // Fetch names in this batch
            let rows: Vec<(String,)> =
                sqlx::query_as("SELECT name FROM migrations WHERE batch = $1 ORDER BY id DESC")
                    .bind(batch_num)
                    .fetch_all(self.pool)
                    .await
                    .map_err(OrmError::from)?;

            for (name,) in rows {
                // Find the matching migration in our registry
                if let Some(m) = self.migrations.iter().find(|m| m.name() == name) {
                    m.down(self.pool).await?;
                }
                sqlx::query("DELETE FROM migrations WHERE name = $1")
                    .bind(&name)
                    .execute(self.pool)
                    .await
                    .map_err(OrmError::from)?;
            }
        }
        Ok(())
    }

    /// Reverse ALL migrations in reverse order.
    pub async fn reset(&self) -> OrmResult<()> {
        self.ensure_migrations_table().await?;
        let max = self.max_batch().await?;
        if max > 0 {
            self.rollback(max as u32).await?;
        }
        Ok(())
    }

    /// Drop all tables, then re-run all migrations from scratch.
    pub async fn fresh(&self) -> OrmResult<()> {
        self.reset().await?;
        // Drop the tracking table itself so it is recreated cleanly
        sqlx::query("DROP TABLE IF EXISTS migrations")
            .execute(self.pool)
            .await
            .map_err(OrmError::from)?;
        self.run().await
    }

    /// Return combined applied + pending status for all registered migrations.
    pub async fn status(&self) -> OrmResult<Vec<MigrationStatus>> {
        self.ensure_migrations_table().await?;

        // Fetch applied rows
        let applied: Vec<(String, i32, chrono::DateTime<chrono::Utc>)> =
            sqlx::query_as("SELECT name, batch, run_at FROM migrations ORDER BY id")
                .fetch_all(self.pool)
                .await
                .map_err(OrmError::from)?;

        let applied_map: std::collections::HashMap<String, (i32, chrono::DateTime<chrono::Utc>)> =
            applied.into_iter().map(|(n, b, t)| (n, (b, t))).collect();

        let statuses = self
            .migrations
            .iter()
            .map(|m| {
                if let Some((batch, run_at)) = applied_map.get(m.name()) {
                    MigrationStatus {
                        name: m.name().to_string(),
                        batch: Some(*batch),
                        run_at: Some(*run_at),
                        is_pending: false,
                    }
                } else {
                    MigrationStatus {
                        name: m.name().to_string(),
                        batch: None,
                        run_at: None,
                        is_pending: true,
                    }
                }
            })
            .collect();

        Ok(statuses)
    }
}
