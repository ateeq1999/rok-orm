# rok-orm Plan

> Unified development and improvement plan for rok-orm.

---

## Project Status

**Current Version:** 0.2.0 (pre-release)

**Phase 1 (Foundation):** ✅ Complete
**Phase 2 (Eloquent API):** ✅ Complete
**Phase 3 (Relationships):** ✅ Complete
**Phase 4 (Model Hooks):** ✅ Complete

---

## Architecture Overview

```
rok-orm/
├── Cargo.toml              # Workspace root
├── README.md               # Documentation
├── PLAN.md                 # This file
├── LICENSE-*               # MIT/Apache-2.0
├── .gitignore
├── .github/workflows/      # CI/CD
│
├── rok-orm-core/           # Core crate
│   ├── src/
│   │   ├── lib.rs
│   │   ├── model.rs        # Model trait
│   │   ├── query.rs        # QueryBuilder
│   │   ├── condition.rs    # SqlValue, Condition
│   │   ├── sqlx_pg.rs      # PostgreSQL bindings
│   │   └── sqlx_sqlite.rs  # SQLite bindings
│   └── Cargo.toml
│
├── rok-orm/                # Main ORM crate
│   ├── src/
│   │   ├── lib.rs
│   │   ├── pg_model.rs     # PgModel trait (PostgreSQL)
│   │   ├── sqlite_model.rs # SqliteModel trait (SQLite)
│   │   ├── executor.rs     # PostgreSQL executor
│   │   ├── sqlite_executor.rs
│   │   ├── transaction.rs  # Tx wrapper
│   │   ├── relations.rs    # Relationship types
│   │   ├── belongs_to_many.rs
│   │   └── hooks.rs        # Model lifecycle hooks
│   ├── tests/
│   └── Cargo.toml
│
└── rok-orm-macros/         # Procedural macros
    ├── src/lib.rs          # #[derive(Model)], #[derive(Relations)], query!()
    └── Cargo.toml
```

---

# Section 1: Development Plan

## Implemented Features

### ✅ Core Features (v0.2.0)

| Feature | Status | Notes |
|---------|--------|-------|
| Fluent QueryBuilder | ✅ | All WHERE conditions, joins, pagination |
| `#[derive(Model)]` | ✅ | Auto-generates table_name, columns, primary_key |
| `#[model(...)]` | ✅ | table, primary_key, soft_delete, timestamps, skip, column |
| `#[derive(Relations)]` | ✅ | HasMany, HasOne, BelongsTo definitions |
| `#[model(belongs_to_many)]` | ✅ | Many-to-many relationship support |
| `#[derive(ModelHooks)]` | ✅ | Lifecycle hooks (before/after create, update, delete) |
| PostgreSQL Support | ✅ | Full async executor with PgModel trait |
| SQLite Support | ✅ | Full async executor with SqliteModel trait |
| Transactions | ✅ | Tx wrapper for PostgreSQL |
| `query!()` macro | ✅ | Shorthand query building |
| `.filter()` shorthand | ✅ | Alias for `.where_eq()` |
| `.eq()` shorthand | ✅ | Short alias for `.where_eq()` |
| `find_or_404()` | ✅ | Returns RowNotFound error |
| `first()`, `first_or_404()` | ✅ | Eloquent-style fetch methods |
| `#[model(soft_delete)]` | ✅ | Adds soft_delete_column() to model |
| `#[model(timestamps)]` | ✅ | Adds timestamps_enabled() to model |
| Dialect Support | ✅ | PostgreSQL ($1) and SQLite (?) placeholders |
| Multi-word table names | ✅ | snake_case conversion |

---

## Phase 1: Soft Deletes & Auto Timestamps (v0.3.0)

### 1.1 Soft Delete Full Implementation ✅

**Status:** Complete - All features implemented and tested

```rust
// AFTER: Transparent soft delete filtering
let posts = Post::all(&pool).await?;           // Excludes deleted
let all = Post::with_trashed().get(&pool).await?;
let trashed = Post::only_trashed().get(&pool).await?;
Post::restore(&pool, id).await?;
Post::force_delete(&pool, id).await?;
```

**Completed:**
- [x] Auto-filter deleted records in `Model::query()`
- [x] Add `with_soft_delete()`, `with_trashed()`, `only_trashed()` to QueryBuilder
- [x] Add `restore()`, `force_delete()` to PgModel and SqliteModel
- [x] Add unit tests for soft delete functionality

### 1.2 Auto Timestamps ✅

**Status:** Complete - Timestamps auto-injected in create/update

```rust
// AFTER: Automatic timestamp management
User::create_returning(&pool, &[("name", "Alice".into())]).await?;
// Internally: adds created_at = NOW() and updated_at = NOW()

User::update_by_pk(&pool, 1, &[("name", "Bob".into())]).await?;
// Internally: adds updated_at = NOW()
```

**Completed:**
- [x] Add `created_at_column()` and `updated_at_column()` to Model trait
- [x] Update macro to generate timestamp column methods
- [x] Inject `created_at`/`updated_at` in `create_returning()`
- [x] Inject `updated_at` in `update_by_pk()`
- [x] Add unit tests for timestamp column methods

---

## Phase 2: Eager Loading & Relations (v0.3.0)

### 2.1 Eager Loading (N+1 Prevention) ✅

**Status:** Complete - `.with()` method implemented with batch loading support

```rust
// AFTER: Single query with joins or batched queries
let posts = Post::query()
    .with("user")
    .with("tags")
    .limit(10)
    .get(&pool)
    .await?;

// posts[0].user is pre-loaded, no additional query
```

**Completed:**
- [x] Add `with()` and `with_many()` methods on QueryBuilder
- [x] Add `eager_loads()` accessor
- [x] Create `eager.rs` module with `HasManyEager`, `HasOneEager`, `BelongsToEager`
- [x] Implement `build_query()` for batch loading with `IN` clause
- [x] Add tests for eager loading

### 2.2 Relation Instance Methods ✅

**Status:** Complete - Chainable relation queries supported

```rust
// AFTER: Chainable relation queries
let posts = user.posts()
    .filter("published", true)
    .order_by_desc("created_at")
    .limit(10)
    .get(&pool)
    .await?;
```

**Completed:**
- [x] Add `RelationQuery` trait with chainable methods
- [x] Add helper accessors (`foreign_key()`, `child_table()`, etc.)
- [x] QueryBuilder implements `RelationQuery` for chaining

### 2.3 Lazy Relation Loading ✅

**Status:** Complete - On-demand relation loading supported

```rust
// AFTER: Lazy loading
use rok_orm::relations::lazy;

// Load has_many relation
let posts = lazy::load_has_many(&pool, &user.posts, &[1, 2, 3]).await?;

// Load has_one relation
let profile = lazy::load_has_one(&pool, &user.profile, user_id).await?;

// Load belongs_to relation
let user = lazy::load_belongs_to(&pool, &post.user, &post).await?;
```

**Completed:**
- [x] Add `lazy::load_has_many()` for batch loading children
- [x] Add `lazy::load_has_one()` for single related record
- [x] Add `lazy::load_belongs_to()` for parent record
- [x] Gated behind `postgres` feature flag

---

## Phase 3: Pagination & Aggregates (v0.4.0)

### 3.1 Pagination

**Current State:** Manual limit/offset
**Goal:** Built-in pagination with metadata

```rust
// AFTER: Automatic pagination
let page: Page<Post> = Post::paginate(&pool, 1, 20).await?;

println!("Total: {} pages", page.total_pages);
println!("Current: {}", page.current_page);
println!("Per page: {}", page.per_page);
println!("Has next: {}", page.has_next);
println!("Has prev: {}", page.has_prev);

// Template-friendly
json!({
    "data": page.data,
    "meta": {
        "total": page.total,
        "per_page": page.per_page,
        "current_page": page.current_page,
        "last_page": page.total_pages,
    }
})
```

**Completed:**
- [x] Define `Page<T>` struct with total, per_page, current_page, last_page
- [x] Add `has_next()` and `has_prev()` methods
- [x] Add `paginate(page, per_page)` to QueryBuilder
- [x] Add `paginate()` and `paginate_where()` to PgModel/SqliteModel
- [x] Add tests

### 3.2 Aggregation Methods ✅

**Status:** Complete - Full aggregate support

```rust
// AFTER: Rich aggregations
let total: i64 = Order::count(&pool).await?;
let revenue: f64 = Order::sum("total", &pool).await?;
let avg_age: f64 = User::avg("age", &pool).await?;
let youngest: Option<i32> = User::min("age", &pool).await?;
let oldest: Option<i32> = User::max("age", &pool).await?;
```

**Completed:**
- [x] Add `sum()`, `avg()`, `min()`, `max()` to QueryBuilder (SQL generation)
- [x] Add aggregate methods to PgModel and SqliteModel
- [x] Add tests

### 3.3 Query Builder Enhancements ✅

**Status:** Complete - Additional query methods added

```rust
// Check if records exist
let exists = User::exists(&pool).await?;

// Pluck single column values
let emails: Vec<SqlValue> = User::pluck(&pool, "email").await?;

// Mass update
let updated = User::update_all(&pool, &[("status", "inactive".into())]).await?;
```

**Completed:**
- [x] Add `exists()` and `exists_where()` methods
- [x] Add `pluck()` and `pluck_where()` for single column retrieval
- [x] Add `update_all()` and `update_all_where()` for mass updates
- [x] Add tests

---

## Phase 4: Advanced Operations (v0.4.0)

### 4.1 Upsert (ON CONFLICT)

**Current State:** Manual SQL required
**Goal:** Built-in upsert support

```rust
// AFTER: Simple upsert
User::upsert(&pool, &[
    ("email", "admin@example.com".into()),
    ("name", "Admin".into()),
]).await?;

// With conflict handling
User::upsert(&pool, &[
    ("email", "admin@example.com".into()),
    ("name", "Admin Updated".into()),
], "email", &["name"]).await?;
```

**Completed:**
- [x] Add `upsert_sql()` with `ON CONFLICT DO UPDATE`
- [x] Add `upsert_do_nothing_sql()` with `ON CONFLICT DO NOTHING`
- [x] Add `upsert()` and `upsert_returning()` to PgModel/SqliteModel
- [x] Add tests

### 4.2 Batch Operations ✅

**Status:** Complete - Efficient bulk operations

```rust
// Bulk delete
User::delete_in(&pool, "id", vec![1i64, 2i64, 3i64]).await?;
```

**Completed:**
- [x] Add `delete_in()` to QueryBuilder and PgModel/SqliteModel
- [x] Support both PostgreSQL and SQLite dialects
- [x] Add tests
User::insert_ignore(&pool, &rows).await?;  // INSERT IGNORE (MySQL)
User::insert_on_conflict(&pool, &rows, "email").await?;  // PostgreSQL
```

**Changes:**
- [ ] Add `update_batch()`
- [ ] Add `delete_in()`
- [ ] Add `insert_ignore()` (MySQL style)
- [ ] Optimize for large batches (chunking)

### 4.3 Query Scopes ✅

**Status:** Complete - Scope pattern documented with traits

```rust
// Scoped queries
impl User {
    pub fn active() -> QueryBuilder<User> {
        User::query().filter("active", true)
    }

    pub fn admins() -> QueryBuilder<User> {
        User::query().filter("role", "admin")
    }
}

// Chained scopes
let users = User::active()
    .admins()
    .order_by("name")
    .get(&pool)
    .await?;
```

**Completed:**
- [x] Add `Scope` and `ScopeMut` traits for reusable scopes
- [x] Add scope composition examples (AndScope, OrScope)
- [x] Document global scopes pattern
- [x] Add scopes module with comprehensive documentation

---

## Phase 5: Production Readiness (v0.5.0)

### 5.1 Error Handling Enhancement ✅

**Status:** Complete - Structured ORM errors

```rust
// AFTER: Rich error types
use rok_orm::errors::{OrmError, OrmResult};

let user = User::find_by_pk(&pool, id)
    .await
    .map_err(OrmError::Database)?;

match result {
    Err(OrmError::NotFound { model, pk, id }) => {
        println!("{} not found", model);
    }
    Err(e) => return Err(e),
    Ok(user) => user,
}
```

**Completed:**
- [x] Define `OrmError` enum with variants (NotFound, Validation, Constraint, Transaction, Hook, Database, Other)
- [x] Add `OrmResult<T>` type alias
- [x] Add `from_sqlx_error()` conversion from sqlx::Error
- [x] Add helper methods (not_found, validation, constraint, etc.)
- [x] Add `is_not_found()`, `is_validation()`, `is_constraint()` checks
- [x] Add `IntoOrmResult` trait for ergonomic conversions

### 5.2 Performance Optimizations

**Status:** Partial - Logging infrastructure ready

**Changes:**
- [ ] Add prepared statement cache
- [ ] Add connection pool configuration
- [ ] Add query timeout support

### 5.3 Observability ✅

**Status:** Complete - Query logging and monitoring

```rust
use rok_orm::logging::{Logger, LogLevel, QueryTimer};

let logger = Logger::new()
    .with_slow_query_threshold(100)
    .with_log_level(LogLevel::Debug);

let timer = QueryTimer::new();
// ... execute query ...
let entry = LogEntry::new(sql, params, timer.elapsed(), LogLevel::Info);
logger.log(entry.with_slow_flag(100));
```

**Completed:**
- [x] Add `Logger` struct with configurable log level
- [x] Add `LogLevel` enum (Trace, Debug, Info, Warn, Error)
- [x] Add `LogEntry` for query log data
- [x] Add `QueryTimer` for measuring query duration
- [x] Add slow query detection with threshold
- [x] Add tests

### 5.4 Testing Utilities

**Goal:** Easy integration testing

```rust
// AFTER: Test helpers
#[cfg(test)]
mod tests {
    use rok_orm_test::prelude::*;

    async fn app_pool() -> PgPool {
        test_pool().await
    }

    #[tokio::test]
    async fn create_user() {
        let pool = app_pool().await;

        let user: User = User::create_returning(&pool, &[
            ("name", "Test".into()),
            ("email", "test@example.com".into()),
        ]).await.unwrap();

        assert_eq!(user.name, "Test");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn find_or_404() {
        let pool = app_pool().await;
        let user = factory::<User>().create(&pool).await;

        let found = User::find_or_404(&pool, user.id).await.unwrap();
        assert_eq!(found.id, user.id);
    }
}

// Factory pattern
pub struct UserFactory;

impl Factory for UserFactory {
    type Model = User;

    fn definition() -> Vec<(&'static str, Faker)> {
        vec![
            ("name", Faker.fake()),
            ("email", Faker.fake()),
        ]
    }
}

let user = UserFactory::new()
    .property("name", "Admin")
    .create(&pool)
    .await?;
```

**Changes:**
- [ ] Create `rok-orm-test` crate
- [ ] Add test pool setup
- [ ] Add `Factory` trait with faker integration
- [ ] Add test assertions
- [ ] Add database transaction rollback per test

---

## Phase 6: v1.0.0 — Production Release

### 6.1 Documentation

- [ ] Full API documentation on docs.rs
- [ ] User guide / tutorial
- [ ] Examples repository
- [ ] Migration guides from other ORMs

### 6.2 Compatibility

- [ ] MySQL support (via sqlx)
- [ ] MSSQL support (via sqlx)
- [ ] Full feature parity with Eloquent

### 6.3 Final Polish

- [ ] 100% test coverage for public API
- [ ] < 100ms average query time for standard operations
- [ ] < 1MB memory overhead for typical application
- [ ] Zero breaking changes in minor versions

---

# Section 2: Improvement Plan

## Priority Matrix

```
Impact
  ^
  |  [3] Relationships  [3] Eager Loading  [3] Migrations
  |  [2] Soft Deletes   [2] Pagination     [2] Hooks
  |  [1] Scopes         [1] Aggregates     [1] CLI
  +-------------------------------------------> Effort
     Low                Medium              High
```

## Implementation Order

### Week 1-2: Quick Wins
1. Soft delete full implementation
2. Auto timestamps
3. Aggregation methods

### Week 3-4: Core Features
1. Eager loading
2. BelongsToMany (enhanced)
3. Pagination

### Week 5-6: Developer Experience
1. CLI tool (rok-cli) — deferred to separate crate
2. Migration system — deferred to separate crate
3. Seeder system — deferred to separate crate

### Week 7-8: Advanced Features
1. Model hooks (integration with executors)
2. Upsert
3. Batch operations

### Week 9-10: Production Readiness
1. Error handling
2. Performance optimization
3. Testing utilities

### Ongoing: Ecosystem
1. MySQL support
2. Redis integration
3. Framework integrations

---

# Section 3: Deferred CLI Features (rok-cli crate)

> All CLI features are deferred to a separate `rok-cli` crate. This section documents the planned features for reference.

## 3.1 CLI Tool (rok-cli)

**Goal:** Artisan-like CLI for common tasks

```bash
# Project scaffolding
rok new my-api
rok new my-api --template=api  # Pre-configured API template
rok new my-api --template=cli  # CLI application

# Code generation
rok make:model User --attributes="name:string,email:string:unique"
rok make:model Post --attributes="title:string,body:text,user_id:foreign:users"
rok make:migration create_users_table
rok make:seeder UserSeeder
rok make:middleware AuthMiddleware
rok make:controller UserController

# Database operations
rok db:setup          # Create database
rok db:migrate        # Run migrations
rok db:rollback       # Rollback last batch
rok db:reset          # Drop and recreate
rok db:seed           # Run seeders
rok db:fresh          # migrate + seed

# Development
rok serve             # Start dev server
rok routes:list       # List all routes
rok routes:list --json > routes.json
rok about             # Show installed packages, versions

# Utilities
rok key:generate      # Generate app key
rok test              # Run tests
rok lint              # Run clippy
```

**Implementation Plan:**

```rust
// crates/rok-cli/src/main.rs
#[tokio::main]
async fn main() {
    Cli::run().await;
}

pub struct Cli {
    commands: Vec<Command>,
}

pub enum Command {
    New { name: String, template: Option<String> },
    Make { resource: String, attributes: Vec<String> },
    Db { action: DbAction },
    Serve,
    Routes,
    KeyGenerate,
    Test,
    Lint,
}
```

**Changes (for rok-cli crate):**
- [ ] Create `rok-cli` workspace member
- [ ] Implement `new` command with template system
- [ ] Implement `make:*` generators with templates
- [ ] Implement `db:*` commands
- [ ] Add interactive prompts using `dialoguer`
- [ ] Add shell completions (bash, zsh, fish, powershell)

## 3.2 Migration System

**Goal:** First-class migration support

```rust
// database/migrations/001_create_users.rs
#[migration]
pub fn m001_create_users() -> Migration {
    Migration::new()
        .create_table("users", |t| {
            t.id("id").primary_key().auto_increment();
            t.string("name", 255).nullable(false);
            t.string("email", 255).nullable(false).unique();
            t.string("password", 255).nullable(false);
            t.boolean("active").default(true);
            t.timestamps();
        })
        .create_index("idx_users_email", &["email"])
}

#[migration]
pub fn m002_create_posts() -> Migration {
    Migration::new()
        .create_table("posts", |t| {
            t.id("id").primary_key().auto_increment();
            t.string("title", 255).nullable(false);
            t.text("body").nullable();
            t.foreign("user_id").references("users", "id").on_delete(Cascade);
            t.timestamps();
            t.soft_delete();
        })
}

// Run: rok db:migrate
// Creates: schema_migrations table tracking applied migrations
```

**Changes (for rok-cli crate):**
- [ ] Define `Migration` struct with builder API
- [ ] Create `#[migration]` procedural macro
- [ ] Implement migration runner
- [ ] Track applied migrations in `schema_migrations` table
- [ ] Support rollback SQL
- [ ] Add `migration_order` for explicit ordering
- [ ] Add database-specific migrations (PostgreSQL vs SQLite)

## 3.3 Seeder System

**Goal:** Database seeding for development/testing

```rust
// database/seeders/user_seeder.rs
#[Seeder]
pub struct UserSeeder;

impl UserSeeder {
    pub async fn run(pool: &PgPool) -> Result<(), sqlx::Error> {
        let users = vec![
            ("Admin", "admin@example.com", "password123"),
            ("User 1", "user1@example.com", "password123"),
            ("User 2", "user2@example.com", "password123"),
        ];

        for (name, email, password) in users {
            User::create(pool, &[
                ("name", name.into()),
                ("email", email.into()),
                ("password", hash(password).into()),
            ]).await?;
        }

        info!("Seeded {} users", users.len());
        Ok(())
    }
}

// database/seeders/mod.rs
pub struct DatabaseSeeder {
    seeders: Vec<Box<dyn Seeder>>,
}

impl DatabaseSeeder {
    pub fn new() -> Self {
        Self {
            seeders: vec![
                Box::new(UserSeeder),
                Box::new(PostSeeder),
                Box::new(TagSeeder),
            ],
        }
    }
}

// Run: rok db:seed
// Or: rok db:seed --seeder=UserSeeder
```

**Changes (for rok-cli crate):**
- [ ] Define `Seeder` trait
- [ ] Create `#[Seeder]` procedural macro
- [ ] Implement seeder runner
- [ ] Track seeded state
- [ ] Support partial seeding

---

# Section 4: Ecosystem (Ongoing)

## 4.1 MySQL Support

```rust
// crates/rok-orm-mysql/
// - MyModel trait
// - MyExecutor
// - MySqlDialect
```

## 4.2 Redis Caching Integration

```rust
// After: Transparent caching
#[derive(Model)]
#[model(cache = "users", ttl = 300)]
pub struct User { ... }

let user = User::find_cached(&pool, id).await?;  // Redis backed
User::invalidate_cache(&pool, id).await?;
```

## 4.3 Rocket Integration

```rust
// After: First-class Rocket support
#[get("/users/<id>")]
async fn get_user(pool: &State<DbPool>, id: i64) -> Json<User> {
    User::find_or_404(&**pool, id).await?.into()
}

#[derive(FromForm)]
struct CreateUser {
    name: String,
    email: String,
}

#[post("/users", data = "<form>")]
async fn create_user(pool: &State<DbPool>, form: Form<CreateUser>) -> Created<Json<User>> {
    let user = User::create_returning(&**pool, &[
        ("name", form.name.into()),
        ("email", form.email.into()),
    ]).await?;
    Created::new("/users").body(Json(user))
}
```

---

## Contributing

### Getting Started

```bash
# Clone the repository
git clone https://github.com/rok-rs/rok-orm.git
cd rok-orm

# Build
cargo build --workspace

# Test
cargo test --workspace

# Format
cargo fmt --all
cargo clippy --workspace
```

### Code Structure

| Crate | Responsibility |
|-------|----------------|
| `rok-orm-core` | Pure logic, no async. QueryBuilder, Model trait, SqlValue |
| `rok-orm` | Async executors, database operations, PgModel/SqliteModel |
| `rok-orm-macros` | `#[derive(Model)]`, `#[derive(Relations)]`, `query!()` |

### PR Guidelines

1. All tests must pass
2. Run `cargo fmt` before committing
3. Add tests for new features
4. Update documentation in README.md
5. Keep commits atomic and well-described

---

## API Cheat Sheet

```rust
use rok_orm::{Model, PgModel, SqlValue};

// Define model
#[derive(Model, sqlx::FromRow)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
}

// Query
let users = User::query()
    .filter("active", true)
    .order_by_desc("created_at")
    .limit(10)
    .get(&pool)
    .await?;

// Find
let user = User::find_or_404(&pool, 1).await?;

// Create
User::create(&pool, &[
    ("name", "Alice".into()),
    ("email", "alice@example.com".into()),
]).await?;

// Update
User::update_by_pk(&pool, 1, &[("name", "Bob".into())]).await?;

// Delete
User::delete_by_pk(&pool, 1).await?;
```

---

## Feature Flags

```toml
# rok-orm/Cargo.toml
[dependencies]
rok-orm = { version = "0.2", features = ["postgres"] }
rok-orm = { version = "0.2", features = ["sqlite"] }

# rok-orm-core/Cargo.toml
rok-orm-core = { features = ["sqlx-postgres"] }
rok-orm-core = { features = ["sqlx-sqlite"] }
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2024 | Initial release with QueryBuilder |
| 0.2.0 | 2026 | Added soft_delete, timestamps, relations, find_or_404, Eloquent-style API, model hooks, belongs_to_many |
| 0.3.0 | 2026 | Full soft delete implementation (auto-filtering, restore, force_delete), auto timestamps (created_at/updated_at), eager loading (.with()), pagination (Page<T>), aggregation methods (sum/avg/min/max), upsert (ON CONFLICT), batch delete_in(), relation chaining, lazy loading, exists/pluck/update_all, query scopes, OrmError, logging/observability |
