//! [`Blueprint`] — fluent table definition builder used inside [`Schema::create`] and [`Schema::alter`].

use super::column::{ColumnDef, ColumnType, ForeignKey, IndexDef, SchemaDialect};

/// Describes the columns, indexes, and foreign keys for one table.
///
/// Passed as `&mut Blueprint` inside the closure given to [`Schema::create`] or [`Schema::alter`].
pub struct Blueprint {
    pub(crate) table: String,
    pub(crate) columns: Vec<ColumnDef>,
    pub(crate) indexes: Vec<IndexDef>,
    pub(crate) foreign_keys: Vec<ForeignKey>,
    pub(crate) primary_columns: Vec<String>,
    /// Columns to drop (used by `alter` only).
    pub(crate) drop_columns: Vec<String>,
    /// Column renames: (old, new).
    pub(crate) rename_columns: Vec<(String, String)>,
    pub(crate) drop_indexes: Vec<String>,
    pub(crate) dialect: SchemaDialect,
}

impl Blueprint {
    pub(crate) fn new(table: impl Into<String>, dialect: SchemaDialect) -> Self {
        Self {
            table: table.into(),
            columns: Vec::new(),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            primary_columns: Vec::new(),
            drop_columns: Vec::new(),
            rename_columns: Vec::new(),
            drop_indexes: Vec::new(),
            dialect,
        }
    }

    // ── Convenience shortcuts ───────────────────────────────────────────────

    /// Shorthand for `big_increments("id")` — the most common primary key pattern.
    pub fn id(&mut self) -> ColumnRef<'_> {
        self.big_increments("id").primary()
    }

    /// `SERIAL PRIMARY KEY` (PG) / `INTEGER PRIMARY KEY AUTOINCREMENT` (SQLite).
    pub fn increments(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Increments)
    }

    /// `BIGSERIAL PRIMARY KEY` (PG) / `INTEGER PRIMARY KEY AUTOINCREMENT` (SQLite).
    pub fn big_increments(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::BigIncrements)
    }

    /// UUID column — `UUID` (PG) / `TEXT` (SQLite) / `CHAR(36)` (MySQL).
    pub fn uuid(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Uuid)
    }

    /// `VARCHAR(len)`.
    pub fn string(&mut self, name: &str, len: u32) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::String(len))
    }

    /// `TEXT`.
    pub fn text(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Text)
    }

    /// `INTEGER`.
    pub fn integer(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Integer)
    }

    /// `BIGINT` (PG/MySQL) / `INTEGER` (SQLite).
    pub fn big_integer(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::BigInteger)
    }

    /// `SMALLINT` (PG/MySQL) / `INTEGER` (SQLite).
    pub fn small_integer(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::SmallInteger)
    }

    /// `REAL` (PG/SQLite) / `FLOAT` (MySQL).
    pub fn float(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Float)
    }

    /// `DOUBLE PRECISION` (PG) / `REAL` (SQLite) / `DOUBLE` (MySQL).
    pub fn double(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Double)
    }

    /// `DECIMAL(precision, scale)` / `NUMERIC` (SQLite).
    pub fn decimal(&mut self, name: &str, precision: u32, scale: u32) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Decimal(precision, scale))
    }

    /// `BOOLEAN` (PG) / `INTEGER` (SQLite) / `TINYINT(1)` (MySQL).
    pub fn boolean(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Boolean)
    }

    /// `DATE` (PG/MySQL) / `TEXT` (SQLite).
    pub fn date(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Date)
    }

    /// `TIMESTAMPTZ` (PG) / `DATETIME` (MySQL) / `TEXT` (SQLite).
    pub fn datetime(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::DateTime)
    }

    /// `TIMESTAMPTZ` (PG) / `TIMESTAMP` (MySQL) / `TEXT` (SQLite).
    pub fn timestamp(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Timestamp)
    }

    /// `JSONB` (PG) / `JSON` (MySQL) / `TEXT` (SQLite).
    pub fn json(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Json)
    }

    /// `BYTEA` (PG) / `BLOB` (MySQL/SQLite).
    pub fn binary(&mut self, name: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Binary)
    }

    /// Enum column — uses `CHECK` constraint on PG/MySQL; raw `TEXT` on SQLite.
    pub fn enum_col(&mut self, name: &str, values: &[&str]) -> ColumnRef<'_> {
        let owned: Vec<String> = values.iter().map(|v| v.to_string()).collect();
        self.add_column(name, ColumnType::Enum(owned))
    }

    /// Raw type string — passes through verbatim.
    pub fn raw_type(&mut self, name: &str, sql_type: &str) -> ColumnRef<'_> {
        self.add_column(name, ColumnType::Raw(sql_type.to_string()))
    }

    /// Add `created_at` and `updated_at` nullable timestamp columns.
    pub fn timestamps(&mut self) {
        self.add_column("created_at", ColumnType::Timestamp).nullable();
        self.add_column("updated_at", ColumnType::Timestamp).nullable();
    }

    /// Add `deleted_at` nullable timestamp column for soft deletes.
    pub fn soft_deletes(&mut self) {
        self.add_column("deleted_at", ColumnType::Timestamp).nullable();
    }

    // ── Foreign keys ────────────────────────────────────────────────────────

    /// Start a foreign key definition on `column`.
    ///
    /// ```rust,ignore
    /// t.foreign("user_id")
    ///     .references("users", "id")
    ///     .on_delete(ForeignAction::Cascade);
    /// ```
    pub fn foreign(&mut self, column: &str) -> ForeignKeyBuilder<'_> {
        ForeignKeyBuilder { blueprint: self, key: ForeignKey::new(column) }
    }

    // ── Indexes ─────────────────────────────────────────────────────────────

    /// Add a non-unique index on the specified columns.
    pub fn index(&mut self, columns: &[&str]) {
        let cols: Vec<String> = columns.iter().map(|c| c.to_string()).collect();
        self.indexes.push(IndexDef::new(cols, false));
    }

    /// Add a unique index on the specified columns.
    pub fn unique_index(&mut self, columns: &[&str]) {
        let cols: Vec<String> = columns.iter().map(|c| c.to_string()).collect();
        self.indexes.push(IndexDef::new(cols, true));
    }

    /// Declare a multi-column primary key.
    pub fn primary_key(&mut self, columns: &[&str]) {
        self.primary_columns = columns.iter().map(|c| c.to_string()).collect();
    }

    // ── Alter helpers ───────────────────────────────────────────────────────

    /// Mark a column for removal (only used in `Schema::alter`).
    pub fn drop_column(&mut self, name: &str) {
        self.drop_columns.push(name.to_string());
    }

    /// Rename a column from `old` to `new` (only used in `Schema::alter`).
    pub fn rename_column(&mut self, old: &str, new: &str) {
        self.rename_columns.push((old.to_string(), new.to_string()));
    }

    /// Drop an index by name (only used in `Schema::alter`).
    pub fn drop_index(&mut self, name: &str) {
        self.drop_indexes.push(name.to_string());
    }

    /// Add a column inside an `alter` call.
    pub fn add_column(&mut self, name: &str, col_type: ColumnType) -> ColumnRef<'_> {
        self.columns.push(ColumnDef::new(name, col_type));
        let idx = self.columns.len() - 1;
        ColumnRef { blueprint: self, idx }
    }

    // ── SQL generation ───────────────────────────────────────────────────────

    /// Generate `CREATE TABLE` SQL.
    pub(crate) fn to_create_sql(&self) -> String {
        let mut defs: Vec<String> = self
            .columns
            .iter()
            .map(|c| format!("    {}", c.to_sql(self.dialect)))
            .collect();

        if !self.primary_columns.is_empty() {
            defs.push(format!("    PRIMARY KEY ({})", self.primary_columns.join(", ")));
        }
        for fk in &self.foreign_keys {
            defs.push(format!("    {}", fk.to_sql()));
        }

        let mut sql = format!("CREATE TABLE {} (\n{}\n)", self.table, defs.join(",\n"));

        for idx in &self.indexes {
            let idx_name = idx.index_name(&self.table);
            let kind = if idx.unique { "UNIQUE INDEX" } else { "INDEX" };
            sql.push_str(&format!(
                ";\nCREATE {kind} {idx_name} ON {} ({})",
                self.table,
                idx.columns.join(", ")
            ));
        }

        sql
    }

    /// Generate `ALTER TABLE` SQL (multiple statements joined by `;\n`).
    pub(crate) fn to_alter_sql(&self) -> String {
        let mut stmts: Vec<String> = Vec::new();
        let t = &self.table;

        for col in &self.columns {
            stmts.push(format!(
                "ALTER TABLE {t} ADD COLUMN {}",
                col.to_sql(self.dialect)
            ));
        }
        for col in &self.drop_columns {
            match self.dialect {
                SchemaDialect::Sqlite => {
                    // SQLite < 3.35 doesn't support DROP COLUMN; emit a comment
                    stmts.push(format!("-- SQLite: recreate table to drop column {col}"));
                }
                _ => stmts.push(format!("ALTER TABLE {t} DROP COLUMN {col}")),
            }
        }
        for (old, new) in &self.rename_columns {
            stmts.push(format!("ALTER TABLE {t} RENAME COLUMN {old} TO {new}"));
        }
        for idx in &self.indexes {
            let idx_name = idx.index_name(t);
            let kind = if idx.unique { "UNIQUE INDEX" } else { "INDEX" };
            stmts.push(format!(
                "CREATE {kind} {idx_name} ON {t} ({})",
                idx.columns.join(", ")
            ));
        }
        for idx_name in &self.drop_indexes {
            stmts.push(format!("DROP INDEX IF EXISTS {idx_name}"));
        }

        stmts.join(";\n")
    }
}

// ── ColumnRef ── (RAII handle for fluent column modifiers) ──────────────────

/// A temporary handle returned by column methods that allows chaining modifiers.
pub struct ColumnRef<'a> {
    blueprint: &'a mut Blueprint,
    idx: usize,
}

impl<'a> ColumnRef<'a> {
    fn col(&mut self) -> &mut ColumnDef {
        &mut self.blueprint.columns[self.idx]
    }

    /// Allow this column to be `NULL`.
    pub fn nullable(mut self) -> Self {
        self.col().nullable = true;
        self
    }

    /// Require `NOT NULL` (the default, but explicit is clearer).
    pub fn not_null(mut self) -> Self {
        self.col().nullable = false;
        self
    }

    /// Set a default value. Pass raw SQL literal: `"0"`, `"true"`, `"CURRENT_TIMESTAMP"`.
    pub fn default(mut self, val: &str) -> Self {
        self.col().default = Some(val.to_string());
        self
    }

    /// Add a `UNIQUE` constraint.
    pub fn unique(mut self) -> Self {
        self.col().unique = true;
        self
    }

    /// Mark as `PRIMARY KEY`.
    pub fn primary(mut self) -> Self {
        self.col().primary = true;
        self
    }
}

// ── ForeignKeyBuilder ───────────────────────────────────────────────────────

/// Fluent builder for a foreign key; commits to `Blueprint` on drop.
pub struct ForeignKeyBuilder<'a> {
    blueprint: &'a mut Blueprint,
    key: ForeignKey,
}

impl<'a> ForeignKeyBuilder<'a> {
    /// Set the referenced table and column.
    pub fn references(mut self, table: &str, column: &str) -> Self {
        self.key = self.key.clone().references(table, column);
        self
    }

    /// Set the `ON DELETE` action.
    pub fn on_delete(mut self, action: super::column::ForeignAction) -> Self {
        self.key = self.key.clone().on_delete(action);
        self
    }

    /// Set the `ON UPDATE` action.
    pub fn on_update(mut self, action: super::column::ForeignAction) -> Self {
        self.key = self.key.clone().on_update(action);
        self
    }
}

impl<'a> Drop for ForeignKeyBuilder<'a> {
    fn drop(&mut self) {
        // Commit the key to the blueprint when the builder is dropped.
        self.blueprint.foreign_keys.push(self.key.clone());
    }
}
