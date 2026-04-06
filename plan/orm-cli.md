# rok-cli — Command-Line Tool

> **Crate:** `rok-cli` (separate workspace member)
> **Binary:** `rok`
> **Status:** 🔜 Planned
> **Version:** targets rok-orm v0.5.0+

---

## Overview

`rok` is an Artisan-style CLI for rok-orm projects. Every command accepts input in two equivalent forms:

1. **JSON payload** — structured, scriptable, IDE-completable
2. **Flags** — ergonomic for interactive shell use

All commands support `--dry-run` to preview what would happen without writing any files or executing any SQL.

---

## Command Input Conventions

### JSON Payload Style

Pass a JSON object as the first positional argument. Every key maps to the equivalent flag.

```bash
rok <command> '<json-payload>' [global-flags]
```

### Flag Style

```bash
rok <command> --key=value --bool-flag [global-flags]
```

### Global Flags

| Flag | Description |
|------|-------------|
| `--dry-run` | Preview output without writing files or running SQL |
| `--json` | Output results as JSON (machine-readable) |
| `--quiet` | Suppress informational output |
| `--verbose` | Extra output (SQL statements, file paths) |
| `--env=<path>` | Path to `.env` file (default: `./.env`) |
| `--db=<url>` | Database URL override (overrides `DATABASE_URL`) |

---

## Command Reference

---

### `rok new` — Create a New Project

**JSON:**
```bash
rok new '{"name": "my-api", "template": "api", "db": "postgres"}'
```

**Flags:**
```bash
rok new my-api --template=api --db=postgres
```

**Dry run:**
```bash
rok new '{"name": "my-api", "template": "api"}' --dry-run
```

**Dry-run output:**
```
[dry-run] Would create project: my-api/
[dry-run]   my-api/Cargo.toml
[dry-run]   my-api/src/main.rs
[dry-run]   my-api/src/models/mod.rs
[dry-run]   my-api/.env.example
[dry-run]   my-api/.gitignore
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | required | Project name (directory name) |
| `template` | string | `"minimal"` | `"minimal"`, `"api"`, `"cli"`, `"full"` |
| `db` | string | `"postgres"` | `"postgres"`, `"sqlite"`, `"mysql"` |
| `git` | bool | `true` | Initialize git repo |

---

### `rok make:model` — Generate a Model

**JSON:**
```bash
rok make:model '{
  "name": "Post",
  "table": "posts",
  "attributes": [
    {"name": "title",   "type": "string", "length": 255},
    {"name": "body",    "type": "text"},
    {"name": "user_id", "type": "foreign", "references": "users"},
    {"name": "published_at", "type": "datetime", "nullable": true}
  ],
  "timestamps": true,
  "soft_delete": true,
  "fillable": ["title", "body", "published_at"]
}'
```

**Flags:**
```bash
rok make:model Post \
  --table=posts \
  --attributes="title:string:255,body:text,user_id:foreign:users,published_at:datetime:nullable" \
  --timestamps \
  --soft-delete \
  --fillable="title,body,published_at"
```

**Dry run:**
```bash
rok make:model '{"name": "Post", "attributes": [{"name": "title", "type": "string"}]}' --dry-run
```

**Dry-run output:**
```
[dry-run] Would write: src/models/post.rs

--- src/models/post.rs ---
use rok_orm::Model;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Model, sqlx::FromRow)]
#[model(table = "posts", timestamps, soft_delete, fillable = ["title", "body", "published_at"])]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub body: String,
    pub user_id: i64,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}
---
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | required | Struct name in PascalCase |
| `table` | string | auto (snake_case plural) | Database table name |
| `attributes` | array | `[]` | Column definitions |
| `attributes[].name` | string | required | Column name |
| `attributes[].type` | string | required | `string`, `text`, `integer`, `bigint`, `float`, `double`, `decimal`, `boolean`, `date`, `datetime`, `json`, `uuid`, `foreign`, `binary` |
| `attributes[].length` | int | type default | For `string` type |
| `attributes[].nullable` | bool | `false` | Allow NULL |
| `attributes[].unique` | bool | `false` | UNIQUE constraint |
| `attributes[].default` | any | none | Default value |
| `attributes[].references` | string | — | For `foreign`: target table name |
| `timestamps` | bool | `true` | Add `created_at` / `updated_at` |
| `soft_delete` | bool | `false` | Add `deleted_at` |
| `uuid` | bool | `false` | UUID primary key |
| `ulid` | bool | `false` | ULID primary key |
| `fillable` | array | `[]` | `#[model(fillable = [...])]` |
| `guarded` | array | `[]` | `#[model(guarded = [...])]` |
| `output_dir` | string | `"src/models"` | Output directory |

---

### `rok make:migration` — Generate a Migration

**JSON:**
```bash
rok make:migration '{
  "name": "create_posts_table",
  "table": "posts",
  "action": "create",
  "columns": [
    {"name": "title",   "type": "string",  "length": 255},
    {"name": "body",    "type": "text"},
    {"name": "user_id", "type": "foreign", "references": "users", "on_delete": "cascade"},
    {"name": "published_at", "type": "datetime", "nullable": true}
  ],
  "timestamps": true,
  "soft_deletes": true
}'
```

**Flags:**
```bash
rok make:migration create_posts_table \
  --table=posts \
  --action=create \
  --columns="title:string:255,body:text,user_id:foreign:users:cascade,published_at:datetime:nullable" \
  --timestamps \
  --soft-deletes
```

**Dry run:**
```bash
rok make:migration '{"name": "create_posts_table", "table": "posts", "action": "create"}' --dry-run
```

**Dry-run output:**
```
[dry-run] Would write: migrations/20260407120000_create_posts_table.rs

--- migrations/20260407120000_create_posts_table.rs ---
pub struct CreatePostsTable;

#[async_trait]
impl Migration for CreatePostsTable {
    fn name(&self) -> &'static str { "20260407120000_create_posts_table" }

    async fn up(&self, pool: &AnyPool) -> OrmResult<()> {
        Schema::create("posts", |t| {
            t.id();
            t.string("title", 255);
            t.text("body");
            t.foreign("user_id").references("users", "id").on_delete(ForeignAction::Cascade);
            t.datetime("published_at").nullable();
            t.timestamps();
            t.soft_deletes();
        }).execute(pool).await
    }

    async fn down(&self, pool: &AnyPool) -> OrmResult<()> {
        Schema::drop_if_exists("posts").execute(pool).await
    }
}
---
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | required | Migration name (snake_case) |
| `table` | string | derived from name | Target table |
| `action` | string | `"create"` | `"create"`, `"alter"`, `"drop"`, `"rename"` |
| `columns` | array | `[]` | Column definitions (same schema as make:model attributes) |
| `timestamps` | bool | `true` | Add `t.timestamps()` |
| `soft_deletes` | bool | `false` | Add `t.soft_deletes()` |
| `output_dir` | string | `"migrations"` | Output directory |

---

### `rok make:seeder` — Generate a Seeder

**JSON:**
```bash
rok make:seeder '{
  "name": "UserSeeder",
  "model": "User",
  "count": 10,
  "factory": "UserFactory"
}'
```

**Flags:**
```bash
rok make:seeder UserSeeder --model=User --count=10 --factory=UserFactory
```

**Dry run:**
```bash
rok make:seeder '{"name": "UserSeeder", "model": "User"}' --dry-run
```

**Dry-run output:**
```
[dry-run] Would write: database/seeders/user_seeder.rs

--- database/seeders/user_seeder.rs ---
pub struct UserSeeder;

#[async_trait]
impl Seeder for UserSeeder {
    async fn run(&self, pool: &AnyPool) -> OrmResult<()> {
        UserFactory::new().count(10).create_many(pool).await?;
        Ok(())
    }
}
---
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | required | Seeder struct name |
| `model` | string | derived from name | Model to seed |
| `factory` | string | `"{Model}Factory"` | Factory to use |
| `count` | int | `10` | Records to create |
| `output_dir` | string | `"database/seeders"` | Output directory |

---

### `rok make:factory` — Generate a Factory

**JSON:**
```bash
rok make:factory '{
  "name": "UserFactory",
  "model": "User",
  "fields": [
    {"name": "name",   "fake": "Name()"},
    {"name": "email",  "fake": "SafeEmail()"},
    {"name": "active", "value": true},
    {"name": "role",   "value": "user"}
  ]
}'
```

**Flags:**
```bash
rok make:factory UserFactory --model=User \
  --fields="name:Name(),email:SafeEmail(),active:true,role:user"
```

**Dry run:**
```bash
rok make:factory '{"name": "UserFactory", "model": "User"}' --dry-run
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | required | Factory struct name |
| `model` | string | derived | Target model |
| `fields` | array | `[]` | Field definitions |
| `fields[].name` | string | required | Column name |
| `fields[].fake` | string | — | `fake` crate expression, e.g. `"Name()"` |
| `fields[].value` | any | — | Literal value (if no fake) |
| `output_dir` | string | `"database/factories"` | Output directory |

---

### `rok make:observer` — Generate a Model Observer

**JSON:**
```bash
rok make:observer '{
  "name": "UserObserver",
  "model": "User",
  "events": ["creating", "created", "updating", "updated", "deleting", "deleted"]
}'
```

**Flags:**
```bash
rok make:observer UserObserver --model=User \
  --events=creating,created,updating,updated,deleting,deleted
```

**Dry run:**
```bash
rok make:observer '{"name": "UserObserver", "model": "User"}' --dry-run
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | string | required | Observer struct name |
| `model` | string | derived | Target model |
| `events` | array | all 10 events | Lifecycle events to implement |
| `output_dir` | string | `"src/observers"` | Output directory |

---

### `rok make:models-from-db` — Generate Models from Live Database

**JSON:**
```bash
rok make:models-from-db '{
  "tables": ["users", "posts", "comments"],
  "output_dir": "src/models",
  "derives": ["Debug", "Clone", "Serialize", "Deserialize"],
  "detect_timestamps": true,
  "detect_soft_delete": true,
  "uuid_type": "String"
}'
```

**Flags:**
```bash
rok make:models-from-db \
  --tables=users,posts,comments \
  --output-dir=src/models \
  --derives="Debug,Clone,Serialize,Deserialize" \
  --detect-timestamps \
  --detect-soft-delete
```

**All tables:**
```bash
rok make:models-from-db '{"all": true}' --dry-run
```

**Dry-run output:**
```
[dry-run] Connecting to: postgres://...@localhost/mydb
[dry-run] Inspecting 3 table(s): users, posts, comments

[dry-run] Would write: src/models/user.rs      (7 columns)
[dry-run] Would write: src/models/post.rs      (8 columns, timestamps, soft_delete)
[dry-run] Would write: src/models/comment.rs   (5 columns, timestamps)
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `tables` | array | — | Specific table names to inspect |
| `all` | bool | `false` | Inspect all tables in the schema |
| `output_dir` | string | `"src/models"` | Output directory |
| `derives` | array | `["Debug", "Clone", "Model", "sqlx::FromRow"]` | Additional derive macros |
| `detect_timestamps` | bool | `true` | Auto-detect `created_at`/`updated_at` |
| `detect_soft_delete` | bool | `true` | Auto-detect `deleted_at` |
| `uuid_type` | string | `"String"` | `"String"` or `"Uuid"` (requires `uuid` crate) |
| `overwrite` | bool | `false` | Overwrite existing files |

---

### `rok db:migrate` — Run Pending Migrations

**JSON:**
```bash
rok db:migrate '{"step": 1}'
```

**Flags:**
```bash
rok db:migrate --step=1
```

**Dry run:**
```bash
rok db:migrate --dry-run
```

**Dry-run output:**
```
[dry-run] Would run 2 migration(s):

  [🔜] 002_create_posts_table
       SQL: CREATE TABLE posts (...)

  [🔜] 003_add_avatar_to_users
       SQL: ALTER TABLE users ADD COLUMN avatar_url VARCHAR(500)
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `step` | int | all | Number of migrations to run |

---

### `rok db:rollback` — Rollback Migrations

**JSON:**
```bash
rok db:rollback '{"step": 1}'
```

**Flags:**
```bash
rok db:rollback --step=1
```

**Dry run:**
```bash
rok db:rollback '{"step": 2}' --dry-run
```

**Dry-run output:**
```
[dry-run] Would rollback 2 batch(es):

  [↩] 003_add_avatar_to_users
      SQL: ALTER TABLE users DROP COLUMN avatar_url

  [↩] 002_create_posts_table
      SQL: DROP TABLE IF EXISTS posts
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `step` | int | `1` | Number of batches to rollback |

---

### `rok db:status` — Show Migration Status

**JSON:**
```bash
rok db:status '{}'
```

**Flags:**
```bash
rok db:status
```

**Output:**
```
Migration Status
────────────────────────────────────────────────────
  ✅  001_create_users_table         batch 1   2026-04-01 10:00
  ✅  002_create_posts_table         batch 1   2026-04-01 10:00
  ✅  003_add_avatar_to_users        batch 2   2026-04-07 15:30
  🔜  004_create_comments_table      pending
────────────────────────────────────────────────────
  3 applied, 1 pending
```

**JSON output (`--json`):**
```json
[
  {"name": "001_create_users_table",  "batch": 1, "run_at": "2026-04-01T10:00:00Z", "status": "applied"},
  {"name": "004_create_comments_table", "batch": null, "run_at": null, "status": "pending"}
]
```

---

### `rok db:reset` — Rollback All and Re-migrate

**JSON:**
```bash
rok db:reset '{}'
```

**Flags:**
```bash
rok db:reset
```

**Dry run:**
```bash
rok db:reset --dry-run
```

**Dry-run output:**
```
[dry-run] Would rollback all 3 batch(es):
  [↩] 003_add_avatar_to_users   → ALTER TABLE users DROP COLUMN avatar_url
  [↩] 002_create_posts_table    → DROP TABLE IF EXISTS posts
  [↩] 001_create_users_table    → DROP TABLE IF EXISTS users

[dry-run] Would then run all 3 migration(s):
  [🔜] 001_create_users_table
  [🔜] 002_create_posts_table
  [🔜] 003_add_avatar_to_users
```

---

### `rok db:fresh` — Drop All Tables and Re-migrate

**JSON:**
```bash
rok db:fresh '{"seed": true}'
```

**Flags:**
```bash
rok db:fresh --seed
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `seed` | bool | `false` | Run seeders after migrating |

---

### `rok db:seed` — Run Seeders

**JSON:**
```bash
rok db:seed '{
  "seeder": "UserSeeder",
  "count": 50
}'
```

**Flags:**
```bash
rok db:seed --seeder=UserSeeder --count=50
```

**All seeders:**
```bash
rok db:seed '{}' --dry-run
```

**Dry-run output:**
```
[dry-run] Would run 2 seeder(s):
  [🌱] UserSeeder   → 10 User records
  [🌱] PostSeeder   → 30 Post records
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `seeder` | string | all | Specific seeder to run |
| `count` | int | factory default | Override count for this run |

---

### `rok db:prune` — Prune Stale Records

**JSON:**
```bash
rok db:prune '{
  "models": ["ActivityLog", "PasswordReset"],
  "force": false
}'
```

**Flags:**
```bash
rok db:prune --models=ActivityLog,PasswordReset
```

**Dry run:**
```bash
rok db:prune '{"models": ["ActivityLog"]}' --dry-run
```

**Dry-run output:**
```
[dry-run] Would prune ActivityLog:
  Query: WHERE created_at < '2026-03-08 12:00:00'
  Estimated rows: 1,847
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `models` | array | all Prunable models | Which models to prune |
| `force` | bool | `false` | Skip confirmation prompt |

---

### `rok routes:list` — List All Routes (Web Projects)

**JSON:**
```bash
rok routes:list '{"format": "table", "filter": "users"}'
```

**Flags:**
```bash
rok routes:list --format=table --filter=users
```

**Output:**
```
Method   URI              Handler
──────   ───              ───────
GET      /users           list_users
POST     /users           create_user
GET      /users/:id       get_user
PUT      /users/:id       update_user
DELETE   /users/:id       delete_user
```

**Payload schema:**

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `format` | string | `"table"` | `"table"`, `"json"`, `"compact"` |
| `filter` | string | none | Filter by URI substring |

---

### `rok about` — Project Information

```bash
rok about
```

**Output:**
```
rok-orm    v0.5.0
rok-cli    v0.5.0
Rust       1.78.0
OS         Windows 11 / Linux / macOS
Database   postgres://...@localhost/mydb  (connected ✅)

Feature Flags: postgres, test-utils
```

---

## Implementation Plan (rok-cli crate)

### Crate Structure

```
rok-cli/
├── Cargo.toml
└── src/
    ├── main.rs             CLI entry point (clap-based)
    ├── commands/
    │   ├── mod.rs
    │   ├── new.rs          rok new
    │   ├── make/
    │   │   ├── model.rs    rok make:model
    │   │   ├── migration.rs
    │   │   ├── seeder.rs
    │   │   ├── factory.rs
    │   │   ├── observer.rs
    │   │   └── from_db.rs  rok make:models-from-db
    │   └── db/
    │       ├── migrate.rs  rok db:migrate
    │       ├── rollback.rs
    │       ├── status.rs
    │       ├── reset.rs
    │       ├── fresh.rs
    │       ├── seed.rs
    │       └── prune.rs
    ├── payload.rs          JSON payload → typed struct deserialization
    ├── dry_run.rs          DryRunOutput writer
    ├── templates/          Handlebars or tera templates for code generation
    │   ├── model.hbs
    │   ├── migration.hbs
    │   ├── seeder.hbs
    │   ├── factory.hbs
    │   └── observer.hbs
    └── config.rs           .env loading, DATABASE_URL resolution
```

### Key Dependencies

```toml
[dependencies]
clap         = { version = "4", features = ["derive"] }
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
tokio        = { version = "1", features = ["rt-macro", "macros"] }
rok-orm      = { path = "../", features = ["postgres", "sqlite", "mysql"] }
tera         = "1"        # templating
dotenvy      = "0.15"    # .env loading
colored      = "2"        # terminal colour
```

### Tasks

- [ ] Set up `rok-cli` as workspace member
- [ ] Implement dual-input parser: JSON first argument OR `--flag=value` flags
- [ ] Implement `--dry-run` flag: collect operations → print without executing
- [ ] Implement `--json` flag: structured JSON output for all commands
- [ ] Implement all `make:*` generators using Tera templates
- [ ] Implement all `db:*` commands calling rok-orm `Migrator` / `Seeder` API
- [ ] Add `rok db:prune` calling `Prunable::prune_all(pool)`
- [ ] Add `rok routes:list` (Axum router introspection via inventory/linkme)
- [ ] Add `rok about` system info command
- [ ] Add shell completion generation (bash, zsh, fish, PowerShell) via `clap_complete`
- [ ] Tests: command parsing (JSON and flags), dry-run output, generated file content
