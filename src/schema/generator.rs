//! Auto-model generator — introspects a live DB and writes Rust struct source files.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::schema::ModelGenerator;
//!
//! let gen = ModelGenerator::new()
//!     .tables(&["users", "posts"])
//!     .output_dir("src/models")
//!     .with_derives(&["Debug", "Clone", "Serialize", "Deserialize"])
//!     .detect_timestamps(true)
//!     .detect_soft_delete(true);
//!
//! gen.generate_postgres(&pool).await?;
//! // writes: src/models/user.rs, src/models/post.rs
//! ```

use crate::schema::inspector::ColumnInfo;

// ── Type mapping ──────────────────────────────────────────────────────────────

/// Map a raw DB type string to a Rust type string.
pub fn db_type_to_rust(db_type: &str, is_nullable: bool) -> String {
    let base = match db_type.to_lowercase().as_str() {
        "bigint" | "bigserial" | "int8" => "i64",
        "integer" | "int" | "int4" | "serial" => "i32",
        "smallint" | "int2" | "smallserial" => "i16",
        "real" | "float4" => "f32",
        "double precision" | "float8" => "f64",
        "decimal" | "numeric" => "f64",
        "boolean" | "bool" => "bool",
        "text" | "citext" => "String",
        t if t.starts_with("varchar") || t.starts_with("character varying") || t.starts_with("char") => "String",
        "uuid" => "String",
        "timestamptz" | "timestamp with time zone" => "chrono::DateTime<chrono::Utc>",
        "timestamp" | "timestamp without time zone" | "datetime" => "chrono::NaiveDateTime",
        "date" => "chrono::NaiveDate",
        "jsonb" | "json" => "serde_json::Value",
        "bytea" | "blob" => "Vec<u8>",
        _ => "String", // fallback
    };

    if is_nullable {
        format!("Option<{base}>")
    } else {
        base.to_string()
    }
}

/// Singularize a table name to a struct name (simple `s`/`es` strip).
pub fn table_to_struct_name(table: &str) -> String {
    use heck::ToUpperCamelCase;
    // Strip trailing 's' / 'es' heuristic
    let singular = if table.ends_with("ies") {
        format!("{}y", &table[..table.len() - 3])
    } else if table.ends_with("ses") || table.ends_with("xes") || table.ends_with("ches") {
        table[..table.len() - 2].to_string()
    } else if table.ends_with('s') {
        table[..table.len() - 1].to_string()
    } else {
        table.to_string()
    };
    singular.to_upper_camel_case()
}

// ── ModelGenerator ────────────────────────────────────────────────────────────

/// Builder for the auto-model generator.
pub struct ModelGenerator {
    tables: Option<Vec<String>>,
    output_dir: String,
    derives: Vec<String>,
    detect_timestamps: bool,
    detect_soft_delete: bool,
}

impl Default for ModelGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelGenerator {
    pub fn new() -> Self {
        Self {
            tables: None,
            output_dir: "src/models".to_string(),
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            detect_timestamps: true,
            detect_soft_delete: true,
        }
    }

    /// Restrict generation to specific tables. If not called, all tables are used.
    pub fn tables(mut self, tables: &[&str]) -> Self {
        self.tables = Some(tables.iter().map(|t| t.to_string()).collect());
        self
    }

    /// Directory to write generated `.rs` files into.
    pub fn output_dir(mut self, dir: &str) -> Self {
        self.output_dir = dir.to_string();
        self
    }

    /// Extra derive macros to add to every generated struct.
    pub fn with_derives(mut self, derives: &[&str]) -> Self {
        self.derives = derives.iter().map(|d| d.to_string()).collect();
        self
    }

    /// Whether to add `#[model(timestamps)]` when `created_at`/`updated_at` columns exist.
    pub fn detect_timestamps(mut self, detect: bool) -> Self {
        self.detect_timestamps = detect;
        self
    }

    /// Whether to add `#[model(soft_delete)]` when a `deleted_at` column exists.
    pub fn detect_soft_delete(mut self, detect: bool) -> Self {
        self.detect_soft_delete = detect;
        self
    }

    // ── Source generation ─────────────────────────────────────────────────────

    /// Generate source for a single table given its column metadata.
    pub fn generate_source(&self, table: &str, columns: &[ColumnInfo]) -> String {
        let struct_name = table_to_struct_name(table);

        let has_timestamps = self.detect_timestamps
            && columns.iter().any(|c| c.name == "created_at")
            && columns.iter().any(|c| c.name == "updated_at");
        let has_soft_delete = self.detect_soft_delete
            && columns.iter().any(|c| c.name == "deleted_at");

        let mut model_attrs = vec![format!("table = \"{table}\"")];
        if has_timestamps {
            model_attrs.push("timestamps".to_string());
        }
        if has_soft_delete {
            model_attrs.push("soft_delete".to_string());
        }

        let derives = self.derives.join(", ");
        let model_attr = model_attrs.join(", ");

        let mut fields = Vec::new();
        for col in columns {
            // Skip timestamp/soft-delete columns — the macro handles them
            if has_timestamps && (col.name == "created_at" || col.name == "updated_at") {
                // Still emit the field so FromRow works
            }
            if has_soft_delete && col.name == "deleted_at" {
                // Still emit — soft_delete needs the field for FromRow
            }

            let rust_type = db_type_to_rust(&col.db_type, col.is_nullable);
            let pk_attr = if col.is_primary_key {
                "    #[model(primary_key)]\n"
            } else {
                ""
            };
            fields.push(format!("{pk_attr}    pub {}: {rust_type},", col.name));
        }

        format!(
            "// Auto-generated by rok-orm ModelGenerator — do not edit manually\n\
             use rok_orm::Model;\n\
             use serde::{{Deserialize, Serialize}};\n\
             \n\
             #[derive({derives}, Model, sqlx::FromRow, Serialize, Deserialize)]\n\
             #[model({model_attr})]\n\
             pub struct {struct_name} {{\n\
             {fields}\n\
             }}\n",
            fields = fields.join("\n")
        )
    }

    /// Write generated files to `output_dir` (creates directory if needed).
    pub fn write_files(&self, sources: &[(String, String)]) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.output_dir)?;
        for (table, src) in sources {
            let file_name = format!("{}/{}.rs", self.output_dir, table);
            std::fs::write(&file_name, src)?;
        }
        Ok(())
    }

    // ── Async generators ──────────────────────────────────────────────────────

    /// Inspect the PostgreSQL database and return `(table_name, source_code)` pairs.
    #[cfg(feature = "postgres")]
    pub async fn generate_postgres(
        &self,
        pool: &sqlx::PgPool,
    ) -> Result<Vec<(String, String)>, sqlx::Error> {
        use crate::schema::inspector::postgres;

        let table_names = match &self.tables {
            Some(t) => t.clone(),
            None => {
                let rows: Vec<(String,)> = sqlx::query_as(
                    "SELECT table_name FROM information_schema.tables
                     WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
                     ORDER BY table_name",
                )
                .fetch_all(pool)
                .await?;
                rows.into_iter().map(|(n,)| n).collect()
            }
        };

        let mut results = Vec::new();
        for table in &table_names {
            let cols = postgres::inspect_table(pool, table).await?;
            let src = self.generate_source(table, &cols);
            results.push((table.clone(), src));
        }
        Ok(results)
    }

    /// Inspect the SQLite database and return `(table_name, source_code)` pairs.
    #[cfg(feature = "sqlite")]
    pub async fn generate_sqlite(
        &self,
        pool: &sqlx::SqlitePool,
    ) -> Result<Vec<(String, String)>, sqlx::Error> {
        use crate::schema::inspector::sqlite;

        let table_names = match &self.tables {
            Some(t) => t.clone(),
            None => {
                let rows: Vec<(String,)> = sqlx::query_as(
                    "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
                )
                .fetch_all(pool)
                .await?;
                rows.into_iter().map(|(n,)| n).collect()
            }
        };

        let mut results = Vec::new();
        for table in &table_names {
            let cols = sqlite::inspect_table(pool, table).await?;
            let src = self.generate_source(table, &cols);
            results.push((table.clone(), src));
        }
        Ok(results)
    }
}
