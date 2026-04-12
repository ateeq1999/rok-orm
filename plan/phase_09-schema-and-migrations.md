# Phase 9: Schema Builder & Migrations

> **Target version:** v0.5.0
> **Status:** ✅ Complete
> **Dependency:** Phase 9.1 must be complete before 9.2 and 9.3

---

## Goal

Code-first schema management and schema-first model generation. The Blueprint API should feel as natural as writing a migration in Laravel or Rails.

---

## 9.1 Schema Builder (Blueprint API)

### API

```rust
use rok_orm::schema::{Schema, Blueprint};

// Create table
Schema::create("users", |t: &mut Blueprint| {
    t.id();                              // BIGSERIAL PRIMARY KEY (PG) / INTEGER PRIMARY KEY AUTOINCREMENT (SQLite)
    t.string("name", 255);
    t.string("email", 255).unique();
    t.string("password", 255);
    t.boolean("active").default(true);
    t.string("role", 50).default("user");
    t.timestamps();
}).execute(&pool).await?;

// All column types
t.increments("id");                     // SERIAL / INTEGER AUTOINCREMENT
t.big_increments("id");                 // BIGSERIAL / INTEGER AUTOINCREMENT
t.uuid("id").primary();                 // UUID / TEXT
t.string("name", 255);                  // VARCHAR(255) / TEXT
t.text("body");                         // TEXT
t.integer("age");                       // INTEGER
t.big_integer("views");                 // BIGINT
t.small_integer("priority");            // SMALLINT
t.float("score");                       // REAL
t.double("price");                      // DOUBLE PRECISION / REAL
t.decimal("amount", 10, 2);            // DECIMAL(10,2) / NUMERIC
t.boolean("active");                    // BOOLEAN / INTEGER (SQLite)
t.date("birthday");                     // DATE / TEXT
t.datetime("published_at");             // TIMESTAMPTZ / TEXT
t.json("metadata");                     // JSONB (PG) / JSON (MySQL) / TEXT (SQLite)
t.binary("data");                       // BYTEA / BLOB
t.enum_col("status", &["draft", "published", "archived"]);

// Convenience shortcuts
t.id();                                 // big_increments("id")
t.soft_deletes();                       // datetime("deleted_at").nullable()
t.timestamps();                         // datetime("created_at") + datetime("updated_at")
t.timestamp("processed_at").nullable();

// Column modifiers (fluent)
t.string("bio", 500).nullable();
t.integer("views").default(0);
t.string("slug", 255).unique();
t.string("code", 10).not_null();

// Foreign keys
t.foreign("user_id")
    .references("users", "id")
    .on_delete(ForeignAction::Cascade)
    .on_update(ForeignAction::Restrict);

// Indexes
t.index(&["email"]);
t.unique_index(&["email", "tenant_id"]);
t.primary_key(&["user_id", "role_id"]);

// Alter table
Schema::alter("users", |t| {
    t.add_column("avatar_url", |c| c.string(500).nullable());
    t.drop_column("old_field");
    t.rename_column("bio", "biography");
    t.change_column("name", |c| c.string(500));
    t.add_index(&["avatar_url"]);
    t.drop_index("users_email_index");
}).execute(&pool).await?;

// Drop
Schema::drop_if_exists("users").execute(&pool).await?;
Schema::drop("users").execute(&pool).await?;
Schema::rename("old_name", "new_name").execute(&pool).await?;

// Inspect
let exists = Schema::has_table(&pool, "users").await?;
let has_col = Schema::has_column(&pool, "users", "email").await?;
```

### New Files

```
src/schema/
├── mod.rs          Schema::create / alter / drop / rename / has_table / has_column
├── blueprint.rs    Blueprint struct, all column builder methods
├── column.rs       ColumnDef, modifiers, ForeignKey, Index
└── inspector.rs    DB schema inspection (used by 9.3)
```

### Tasks

- [x] Create `src/schema/` module tree
- [x] Define `Schema` struct with static methods (`create`, `alter`, `drop`, `drop_if_exists`, `rename`, `has_table`, `has_column`)
- [x] Define `Blueprint` struct; all column methods return `&mut ColumnDef` for fluent modifiers
- [x] Define `ColumnDef` with: `nullable()`, `default(val)`, `unique()`, `not_null()`, `primary()`
- [x] Define `ForeignKey` builder: `references(table, col)`, `on_delete(action)`, `on_update(action)`
- [x] Define `ForeignAction` enum: `Cascade`, `Restrict`, `SetNull`, `SetDefault`, `NoAction`
- [x] Generate dialect-specific DDL SQL per column type:
  - PostgreSQL: `BIGSERIAL`, `BOOLEAN`, `TIMESTAMPTZ`, `JSONB`, `UUID`
  - SQLite: `INTEGER`, `INTEGER` (bool), `TEXT` (date/json/uuid)
  - MySQL: `BIGINT AUTO_INCREMENT`, `TINYINT(1)`, `DATETIME`, `JSON`, `CHAR(36)`
- [x] Add `Schema::alter()` — generates `ALTER TABLE` with add/drop/rename/change
- [x] Add `Schema::has_table()` and `Schema::has_column()` via dialect-specific queries
- [x] Tests: DDL generation for each type per dialect, modify column, foreign key SQL

---

## 9.2 Migration System (First-Class)

### API

```rust
// Define a migration
pub struct CreateUsersTable;

#[async_trait]
impl Migration for CreateUsersTable {
    fn name(&self) -> &'static str { "001_create_users_table" }

    async fn up(&self, pool: &AnyPool) -> OrmResult<()> {
        Schema::create("users", |t| {
            t.id();
            t.string("name", 255);
            t.string("email", 255).unique();
            t.timestamps();
        }).execute(pool).await
    }

    async fn down(&self, pool: &AnyPool) -> OrmResult<()> {
        Schema::drop_if_exists("users").execute(pool).await
    }
}

// Register and run
let migrator = Migrator::new(&pool)
    .add(CreateUsersTable)
    .add(CreatePostsTable)
    .add(AddAvatarToUsers);

migrator.run().await?;          // Run all pending
migrator.rollback(1).await?;    // Reverse last N batches
migrator.reset().await?;        // Reverse all (run down() in reverse)
migrator.fresh().await?;        // Drop all tables, run all up()

// Status output:
// [✅] 001_create_users_table        batch 1
// [✅] 002_create_posts_table        batch 1
// [🔜] 003_add_avatar_to_users       pending
let statuses = migrator.status().await?;
```

### Schema: `migrations` table

```sql
CREATE TABLE migrations (
    id         BIGSERIAL PRIMARY KEY,
    name       VARCHAR(255) NOT NULL UNIQUE,
    batch      INTEGER NOT NULL,
    run_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Tasks

- [x] Define `Migration` trait: `name() -> &'static str`, `up(pool) -> OrmResult<()>`, `down(pool) -> OrmResult<()>`
- [x] Define `MigrationStatus` struct: `name`, `batch`, `run_at`, `is_pending`
- [x] Create `Migrator` struct with `migrations: Vec<Box<dyn Migration>>`
- [x] Add `add(migration)` builder method
- [x] Implement `run()`:
  1. Ensure `migrations` table exists (create if not)
  2. Fetch list of already-run migration names
  3. Filter pending, sort by name
  4. Run each `up()` in sequence, record in `migrations` with incremented batch
- [x] Implement `rollback(n)`:
  1. Find the last N distinct batches
  2. For each migration in reverse, call `down()`
  3. Delete from `migrations`
- [x] Implement `reset()` — rollback all batches
- [x] Implement `fresh()` — reset + run
- [x] Implement `status()` — return all migrations with applied/pending state
- [x] Create `migrations` table DDL for each dialect
- [x] Tests: run, rollback, reset, fresh, status, idempotent run

---

## 9.3 Auto-Generated Models from Database

### API

```rust
// Programmatic
let generator = ModelGenerator::from_pool(&pool)
    .tables(&["users", "posts", "comments"])  // or .all_tables()
    .output_dir("src/models/")
    .with_derives(&["Debug", "Clone", "Serialize", "Deserialize"])
    .detect_timestamps(true)
    .detect_soft_delete(true);

generator.generate().await?;
// writes: src/models/user.rs, src/models/post.rs, src/models/comment.rs
```

**Generated `src/models/user.rs`:**

```rust
// Auto-generated by rok-orm ModelGenerator — do not edit manually
// Re-generate: rok make:models-from-db --tables=users
use rok_orm::Model;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Model, sqlx::FromRow, Serialize, Deserialize)]
#[model(table = "users", timestamps, soft_delete)]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
    pub active: bool,
    pub role: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}
```

### Type Mapping

| DB type | Rust type |
|---------|-----------|
| `BIGINT`, `BIGSERIAL` | `i64` |
| `INT`, `SERIAL`, `INTEGER` | `i32` |
| `SMALLINT` | `i16` |
| `REAL`, `FLOAT4` | `f32` |
| `DOUBLE PRECISION`, `FLOAT8` | `f64` |
| `DECIMAL`, `NUMERIC` | `rust_decimal::Decimal` |
| `BOOLEAN`, `TINYINT(1)` | `bool` |
| `VARCHAR`, `TEXT`, `CHAR` | `String` |
| `UUID` | `String` (or `uuid::Uuid` with flag) |
| `TIMESTAMPTZ`, `DATETIME` | `Option<DateTime<Utc>>` |
| `DATE` | `Option<NaiveDate>` |
| `JSONB`, `JSON` | `serde_json::Value` |
| `BYTEA`, `BLOB` | `Vec<u8>` |
| nullable column | wraps in `Option<T>` |

### Inspector Queries per Dialect

**PostgreSQL:**
```sql
SELECT column_name, data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_schema = 'public' AND table_name = $1
ORDER BY ordinal_position;
```

**SQLite:**
```sql
PRAGMA table_info('users');
```

**MySQL:**
```sql
SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE, COLUMN_DEFAULT, COLUMN_KEY
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?;
```

### Tasks

- [x] Add `src/schema/inspector.rs`: `inspect_table(pool, table_name) -> Vec<ColumnInfo>`
- [x] Define `ColumnInfo` struct: `name`, `db_type`, `is_nullable`, `is_primary_key`, `default`
- [x] Implement inspector for PG, SQLite, MySQL using dialect detection
- [x] Map `ColumnInfo` → Rust type string using type mapping table above
- [x] Detect `created_at` / `updated_at` → add `timestamps` attribute
- [x] Detect `deleted_at` → add `soft_delete` attribute
- [x] Detect PK column → add `#[model(primary_key)]`
- [x] Generate struct source as a `String` using template
- [x] Write files to `output_dir/{singular_snake_case}.rs`
- [x] Create `ModelGenerator` builder struct
- [x] Add CLI hook: `rok make:models-from-db` (see `orm-cli.md`)
- [x] Tests: generate from mock schema, verify output for all types, nullable handling

---

## Acceptance Criteria for Phase 9

- [x] Schema Builder generates valid SQL for all 3 dialects
- [x] Migration system: run, rollback, reset, fresh all work correctly
- [x] Auto-generated models compile with `cargo check`
- [x] Tests: Blueprint DDL, migration lifecycle, model generation for each dialect
- [x] `cargo clippy -- -D warnings` clean
- [x] Phase file tasks all checked off
