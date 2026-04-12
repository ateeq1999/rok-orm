//! Schema builder — create, alter, drop tables and inspect DB structure.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use rok_orm::schema::{Schema, Blueprint};
//!
//! // Create a table
//! Schema::create("users", |t| {
//!     t.id();
//!     t.string("name", 255);
//!     t.string("email", 255).unique();
//!     t.boolean("active").default("true");
//!     t.timestamps();
//! }).execute(&pool).await?;
//!
//! // Alter a table
//! Schema::alter("users", |t| {
//!     t.add_column("bio", ColumnType::Text).nullable();
//!     t.drop_column("old_field");
//!     t.rename_column("bio", "biography");
//! }).execute(&pool).await?;
//!
//! // Drop
//! Schema::drop_if_exists("users").execute(&pool).await?;
//!
//! // Inspect
//! let exists = Schema::has_table(&pool, "users").await?;
//! ```

pub mod blueprint;
pub mod column;
pub mod inspector;
#[cfg(test)]
mod tests;

pub use blueprint::Blueprint;
pub use column::{ColumnDef, ColumnType, ForeignAction, ForeignKey, SchemaDialect};

use column::IndexDef;

// ── SchemaOp ────────────────────────────────────────────────────────────────

/// A pending schema operation that can be `.execute()`d against a database.
pub struct SchemaOp {
    kind: OpKind,
}

enum OpKind {
    Create(Blueprint),
    Alter(Blueprint),
    Drop { table: String, if_exists: bool },
    DropIfExists(String),
    Rename { from: String, to: String },
    RawSql(String),
}

impl SchemaOp {
    /// Generate the SQL string for this operation (dialect-specific).
    pub fn to_sql(&self) -> String {
        match &self.kind {
            OpKind::Create(bp) => bp.to_create_sql(),
            OpKind::Alter(bp) => bp.to_alter_sql(),
            OpKind::Drop { table, if_exists } => {
                if *if_exists {
                    format!("DROP TABLE IF EXISTS {table}")
                } else {
                    format!("DROP TABLE {table}")
                }
            }
            OpKind::DropIfExists(table) => format!("DROP TABLE IF EXISTS {table}"),
            OpKind::Rename { from, to } => format!("ALTER TABLE {from} RENAME TO {to}"),
            OpKind::RawSql(sql) => sql.clone(),
        }
    }

    /// Execute the operation against a PostgreSQL connection pool.
    #[cfg(feature = "postgres")]
    pub async fn execute(self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        let sql = self.to_sql();
        // Multiple statements may be separated by ";\n" (from index creation)
        for stmt in sql.split(";\n") {
            let trimmed = stmt.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(pool).await?;
            }
        }
        Ok(())
    }

    /// Execute the operation against a SQLite connection pool.
    #[cfg(feature = "sqlite")]
    pub async fn execute_sqlite(self, pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        let sql = self.to_sql();
        for stmt in sql.split(";\n") {
            let trimmed = stmt.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("--") {
                sqlx::query(trimmed).execute(pool).await?;
            }
        }
        Ok(())
    }
}

// ── Schema ───────────────────────────────────────────────────────────────────

/// Entry point for schema operations.
///
/// All methods are static and return a [`SchemaOp`] that must be executed.
pub struct Schema;

impl Schema {
    /// Build a `CREATE TABLE` statement.
    ///
    /// ```rust,ignore
    /// Schema::create("users", |t| {
    ///     t.id();
    ///     t.string("name", 255);
    ///     t.timestamps();
    /// }).execute(&pool).await?;
    /// ```
    pub fn create(table: &str, f: impl FnOnce(&mut Blueprint)) -> SchemaOp {
        Self::create_with_dialect(table, SchemaDialect::Postgres, f)
    }

    /// Same as [`create`] but with an explicit dialect.
    pub fn create_with_dialect(
        table: &str,
        dialect: SchemaDialect,
        f: impl FnOnce(&mut Blueprint),
    ) -> SchemaOp {
        let mut bp = Blueprint::new(table, dialect);
        f(&mut bp);
        SchemaOp { kind: OpKind::Create(bp) }
    }

    /// Build an `ALTER TABLE` statement.
    pub fn alter(table: &str, f: impl FnOnce(&mut Blueprint)) -> SchemaOp {
        Self::alter_with_dialect(table, SchemaDialect::Postgres, f)
    }

    /// Same as [`alter`] but with an explicit dialect.
    pub fn alter_with_dialect(
        table: &str,
        dialect: SchemaDialect,
        f: impl FnOnce(&mut Blueprint),
    ) -> SchemaOp {
        let mut bp = Blueprint::new(table, dialect);
        f(&mut bp);
        SchemaOp { kind: OpKind::Alter(bp) }
    }

    /// Build a `DROP TABLE` statement.
    pub fn drop(table: &str) -> SchemaOp {
        SchemaOp { kind: OpKind::Drop { table: table.to_string(), if_exists: false } }
    }

    /// Build a `DROP TABLE IF EXISTS` statement.
    pub fn drop_if_exists(table: &str) -> SchemaOp {
        SchemaOp { kind: OpKind::DropIfExists(table.to_string()) }
    }

    /// Build an `ALTER TABLE … RENAME TO` statement.
    pub fn rename(from: &str, to: &str) -> SchemaOp {
        SchemaOp { kind: OpKind::Rename { from: from.to_string(), to: to.to_string() } }
    }

    /// Check whether a table exists (PostgreSQL).
    #[cfg(feature = "postgres")]
    pub async fn has_table(pool: &sqlx::PgPool, table: &str) -> Result<bool, sqlx::Error> {
        inspector::postgres::has_table(pool, table).await
    }

    /// Check whether a column exists in a table (PostgreSQL).
    #[cfg(feature = "postgres")]
    pub async fn has_column(
        pool: &sqlx::PgPool,
        table: &str,
        column: &str,
    ) -> Result<bool, sqlx::Error> {
        inspector::postgres::has_column(pool, table, column).await
    }

    /// Check whether a table exists (SQLite).
    #[cfg(feature = "sqlite")]
    pub async fn has_table_sqlite(
        pool: &sqlx::SqlitePool,
        table: &str,
    ) -> Result<bool, sqlx::Error> {
        inspector::sqlite::has_table(pool, table).await
    }

    /// Check whether a column exists in a table (SQLite).
    #[cfg(feature = "sqlite")]
    pub async fn has_column_sqlite(
        pool: &sqlx::SqlitePool,
        table: &str,
        column: &str,
    ) -> Result<bool, sqlx::Error> {
        inspector::sqlite::has_column(pool, table, column).await
    }

    /// Create an index outside of a `create`/`alter` call.
    pub fn create_index(table: &str, columns: &[&str], unique: bool) -> SchemaOp {
        let idx = IndexDef::new(columns.iter().map(|c| c.to_string()).collect(), unique);
        let name = idx.index_name(table);
        let kind = if unique { "UNIQUE INDEX" } else { "INDEX" };
        let cols = columns.join(", ");
        SchemaOp {
            kind: OpKind::RawSql(format!("CREATE {kind} {name} ON {table} ({cols})")),
        }
    }

    /// Drop an index by name.
    pub fn drop_index(name: &str) -> SchemaOp {
        SchemaOp {
            kind: OpKind::RawSql(format!("DROP INDEX IF EXISTS {name}")),
        }
    }
}
