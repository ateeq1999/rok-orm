//! DB schema inspector — introspects live table metadata.
//!
//! Used by the auto-model generator (Phase 9.3) and `Schema::has_table` / `Schema::has_column`.

/// Metadata for a single column returned by the inspector.
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub db_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default: Option<String>,
}

/// Inspector for PostgreSQL — queries `information_schema.columns`.
#[cfg(feature = "postgres")]
pub mod postgres {
    use super::ColumnInfo;
    use sqlx::PgPool;

    /// Returns `true` if `table` exists in the `public` schema.
    pub async fn has_table(pool: &PgPool, table: &str) -> Result<bool, sqlx::Error> {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'public' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }

    /// Returns `true` if `column` exists in `table`.
    pub async fn has_column(pool: &PgPool, table: &str, column: &str) -> Result<bool, sqlx::Error> {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_schema = 'public' AND table_name = $1 AND column_name = $2
            )",
        )
        .bind(table)
        .bind(column)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }

    /// Inspect all columns of a table, ordered by position.
    pub async fn inspect_table(pool: &PgPool, table: &str) -> Result<Vec<ColumnInfo>, sqlx::Error> {
        let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT column_name, data_type, is_nullable, column_default
             FROM information_schema.columns
             WHERE table_schema = 'public' AND table_name = $1
             ORDER BY ordinal_position",
        )
        .bind(table)
        .fetch_all(pool)
        .await?;

        // Fetch primary key columns
        let pk_rows: Vec<(String,)> = sqlx::query_as(
            "SELECT kcu.column_name
             FROM information_schema.table_constraints tc
             JOIN information_schema.key_column_usage kcu
               ON tc.constraint_name = kcu.constraint_name
              AND tc.table_schema = kcu.table_schema
             WHERE tc.constraint_type = 'PRIMARY KEY'
               AND tc.table_schema = 'public'
               AND tc.table_name = $1",
        )
        .bind(table)
        .fetch_all(pool)
        .await?;
        let pk_cols: std::collections::HashSet<String> =
            pk_rows.into_iter().map(|(n,)| n).collect();

        Ok(rows
            .into_iter()
            .map(|(name, db_type, is_nullable, default)| ColumnInfo {
                is_primary_key: pk_cols.contains(&name),
                is_nullable: is_nullable.eq_ignore_ascii_case("YES"),
                name,
                db_type,
                default,
            })
            .collect())
    }
}

/// Inspector for SQLite — uses `PRAGMA table_info`.
#[cfg(feature = "sqlite")]
pub mod sqlite {
    use super::ColumnInfo;
    use sqlx::SqlitePool;

    /// Returns `true` if `table` exists.
    pub async fn has_table(pool: &SqlitePool, table: &str) -> Result<bool, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
        )
        .bind(table)
        .fetch_one(pool)
        .await?;
        Ok(row.0 > 0)
    }

    /// Returns `true` if `column` exists in `table`.
    pub async fn has_column(pool: &SqlitePool, table: &str, column: &str) -> Result<bool, sqlx::Error> {
        let infos = inspect_table(pool, table).await?;
        Ok(infos.iter().any(|c| c.name == column))
    }

    /// Inspect all columns of a table via `PRAGMA table_info`.
    pub async fn inspect_table(pool: &SqlitePool, table: &str) -> Result<Vec<ColumnInfo>, sqlx::Error> {
        // PRAGMA table_info returns: cid, name, type, notnull, dflt_value, pk
        let rows: Vec<(i64, String, String, i64, Option<String>, i64)> =
            sqlx::query_as(&format!("PRAGMA table_info('{table}')"))
                .fetch_all(pool)
                .await?;
        Ok(rows
            .into_iter()
            .map(|(_cid, name, db_type, notnull, default, pk)| ColumnInfo {
                is_primary_key: pk > 0,
                is_nullable: notnull == 0,
                name,
                db_type,
                default,
            })
            .collect())
    }
}
