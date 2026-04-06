# rok-orm Improvements Plan

## Executive Summary

This document outlines a structured improvement plan for rok-orm, organized by priority and complexity. The goal is to evolve rok-orm into a production-ready, developer-friendly ORM that rivals established solutions like Adonisjs Lucid and Laravel Eloquent.

---

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

---

## Phase 1: Quick Wins (1-2 weeks)

### 1.1 Soft Delete Full Implementation

**Current State:** Trait methods exist but not integrated with queries
**Goal:** Auto-filter deleted records

```rust
// AFTER: Transparent soft delete filtering
let posts = Post::all(&pool).await?;           // Excludes deleted
let all = Post::with_trashed().get(&pool).await?;
let trashed = Post::only_trashed().get(&pool).await?;
Post::restore(&pool, id).await?;
Post::force_delete(&pool, id).await?;
```

**Changes:**
- [ ] Modify `all()` to check `soft_delete_column()` and auto-add WHERE clause
- [ ] Add `with_trashed()`, `only_trashed()` to QueryBuilder
- [ ] Add `restore()`, `force_delete()` to PgModel
- [ ] Add tests

### 1.2 Auto Timestamps

**Current State:** Trait methods exist but not integrated
**Goal:** Automatic `created_at`/`updated_at` management

```rust
// AFTER: Automatic timestamp management
User::create_returning(&pool, &[("name", "Alice".into())]).await?;
// Internally: adds created_at = NOW()

User::update_by_pk(&pool, 1, &[("name", "Bob".into())]).await?;
// Internally: adds updated_at = NOW()
```

**Changes:**
- [ ] Detect `created_at`/`updated_at` columns in Model
- [ ] Inject timestamp values in `create_returning()`
- [ ] Inject timestamp values in `update_by_pk()`
- [ ] Add tests

### 1.3 Query Builder Enhancements

**Current State:** Basic conditions supported
**Goal:** Complete WHERE clause coverage

```rust
// AFTER: More intuitive API
User::query()
    .filter("email", "admin@example.com")           // shorthand
    .filter("active", true)
    .where_in("role", vec!["admin", "moderator"])
    .where_between("age", 18i64, 65i64)
    .order_by("name")
    .paginate(1, 20)
    .get(&pool)
    .await?;
```

**Changes:**
- [ ] Add `paginate(page, per_page)` returning `Page<T>`
- [ ] Add `exists()` method (returns bool)
- [ ] Add `pluck()` for single column retrieval
- [ ] Add `update_all()` for mass updates

### 1.4 Aggregation Methods

**Current State:** Only `count()` exists
**Goal:** Full aggregate support

```rust
// AFTER: Rich aggregations
let total: i64 = Order::count(&pool).await?;
let revenue: f64 = Order::sum("total", &pool).await?;
let avg_age: f64 = User::avg("age", &pool).await?;
let youngest: Option<i32> = User::min("age", &pool).await?;
let oldest: Option<i32> = User::max("age", &pool).await?;
```

**Changes:**
- [ ] Add `sum()`, `avg()`, `min()`, `max()` to PgModel
- [ ] Implement using `SELECT SUM(col) FROM ...` pattern
- [ ] Add tests with various data types

---

## Phase 2: Core Features (2-4 weeks)

### 2.1 Eager Loading (N+1 Prevention)

**Current State:** Not implemented
**Goal:** Prevent N+1 queries with relation loading

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

**Architecture:**

```rust
pub struct QueryContext {
    builder: QueryBuilder<T>,
    eager_loads: Vec<EagerLoad>,
}

pub enum EagerLoad {
    HasMany(String),   // relation name
    HasOne(String),
    BelongsTo(String),
    BelongsToMany(String, String),  // name, pivot
}
```

**Changes:**
- [ ] Create `QueryContext` to track eager loads
- [ ] Implement `with()` method on QueryBuilder
- [ ] After main query, batch-load relations
- [ ] Use `IN` clause for batch loading (efficient)
- [ ] Add `load()` method to load relations on existing model
- [ ] Add tests for all relation types

### 2.2 BelongsToMany (Many-to-Many)

**Current State:** Not implemented
**Goal:** Full pivot table support

```rust
// AFTER: Automatic pivot table handling
#[derive(Model, Relations)]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    #[model(belongs_to_many(Tag, through = "post_tags", pivot = "post_id,tag_id"))]
    pub tags: Vec<Tag>,
}

// Usage
let tags = user.posts().get(&pool).await?;  // Uses post_tags pivot
```

**Changes:**
- [ ] Define `BelongsToMany<P, C, Pivot>` struct
- [ ] Add `through` and `pivot` to derive macro
- [ ] Generate pivot queries automatically
- [ ] Implement `attach()`, `detach()`, `sync()` for pivot management
- [ ] Add tests

### 2.3 Model Hooks/Lifecycle Events

**Current State:** Not implemented
**Goal:** Intercept and modify model operations

```rust
// AFTER: Lifecycle hooks
#[derive(Model)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
}

impl ModelHooks for User {
    async fn before_create(&mut self) -> Result<(), OrmError> {
        self.password = hash_password(&self.password)?;
        self.email = self.email.to_lowercase();
        Ok(())
    }

    async fn after_create(&self) {
        tracing::info!("User {} created", self.id);
        event::emit(UserCreated { user_id: self.id });
    }

    async fn before_update(&mut self) -> Result<(), OrmError> {
        if let Some(ref pwd) = self.password {
            self.password = Some(hash_password(pwd)?);
        }
        Ok(())
    }
}
```

**Changes:**
- [ ] Define `ModelHooks` trait with all lifecycle methods
- [ ] Add `HookManager` for registering global hooks
- [ ] Integrate hooks into `create_returning()`, `update_by_pk()`, `delete_by_pk()`
- [ ] Support async hooks
- [ ] Add error handling (hooks can fail operations)

### 2.4 Transactions Enhancement

**Current State:** Basic Tx wrapper exists
**Goal:** More ergonomic transaction API

```rust
// AFTER: Scoped transactions with automatic commit/rollback
let user = sqlx::transaction(&pool, |tx| async {
    let user = User::create_returning(&tx, &data).await?;
    let profile = Profile::create_returning(&tx, &profile_data).await?;
    Ok((user, profile))
}).await?;

// Or with savepoints
let mut tx = Tx::begin(&pool).await?;
tx.savepoint("before_tag_update")?;
tx.update(...)?;
// tx.rollback_to("before_tag_update")?;
// tx.release_savepoint("before_tag_update")?;
tx.commit().await?;
```

**Changes:**
- [ ] Add `sqlx::transaction()` helper
- [ ] Implement savepoint support
- [ ] Add `Transaction` trait for generic transaction handling
- [ ] Add `acquired_connection()` for manual query building

---

## Phase 3: Developer Experience (2-3 weeks)

### 3.1 CLI Tool (rok-cli)

**Current State:** Not implemented
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

**Changes:**
- [ ] Create `rok-cli` workspace member
- [ ] Implement `new` command with template system
- [ ] Implement `make:*` generators with templates
- [ ] Implement `db:*` commands
- [ ] Add interactive prompts using `dialoguer`
- [ ] Add shell completions (bash, zsh, fish, powershell)

### 3.2 Migration System

**Current State:** Not implemented
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

**Changes:**
- [ ] Define `Migration` struct with builder API
- [ ] Create `#[migration]` procedural macro
- [ ] Implement migration runner
- [ ] Track applied migrations in `schema_migrations` table
- [ ] Support rollback SQL
- [ ] Add `migration_order` for explicit ordering
- [ ] Add database-specific migrations (PostgreSQL vs SQLite)

### 3.3 Seeder System

**Current State:** Not implemented
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

**Changes:**
- [ ] Define `Seeder` trait
- [ ] Create `#[Seeder]` procedural macro
- [ ] Implement seeder runner
- [ ] Track seeded state
- [ ] Support partial seeding

---

## Phase 4: Advanced Features (3-4 weeks)

### 4.1 Pagination

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

// Simple cursor pagination for large datasets
let cursor = CursorPaginate::new(&pool);
let posts = cursor.for_table("posts")
    .order_by("created_at", DESC)
    .limit(20)
    .after(cursor_token)
    .await?;
```

**Changes:**
- [ ] Define `Page<T>` struct
- [ ] Add `paginate(page, per_page)` to PgModel
- [ ] Implement `CursorPaginate` for large tables
- [ ] Add `links()` method for pagination URLs
- [ ] Add tests with edge cases

### 4.2 Query Scopes

**Current State:** Document as pattern
**Goal:** First-class scope support

```rust
// AFTER: Scoped queries
#[derive(Model)]
pub struct User { ... }

impl User {
    // Class method scopes
    pub fn active() -> QueryBuilder<User> {
        User::query().filter("active", true)
    }

    pub fn admins() -> QueryBuilder<User> {
        User::query().filter("role", "admin")
    }

    pub fn older_than(age: i32) -> QueryBuilder<User> {
        User::query().filter("age", Operator::Gt, age)
    }

    pub fn search(query: &str) -> QueryBuilder<User> {
        User::query()
            .filter("name", Operator::Like, format!("%{}%", query))
            .or_where("email", Operator::Like, format!("%{}%", query))
    }
}

// Chained scopes
let users = User::active()
    .admins()
    .order_by("name")
    .get(&pool)
    .await?;
```

**Changes:**
- [ ] Document best practices
- [ ] Add `Scope` trait for first-class support
- [ ] Add `scoped_query()` wrapper
- [ ] Add scope composition examples

### 4.3 Upsert (ON CONFLICT)

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
])
.on_conflict("email")
.do_update(&[("name", "Admin Updated".into())])
.await?;

// With partial index conflict
User::upsert(&pool, &[
    ("email", "admin@example.com".into()),
    ("name", "Admin".into()),
])
.on_conflict("idx_active_email")
.where("active", true)
.do_nothing()
.await?;
```

**Changes:**
- [ ] Add `upsert()` with builder pattern
- [ ] Support `ON CONFLICT DO UPDATE`
- [ ] Support `ON CONFLICT DO NOTHING`
- [ ] Add tests

### 4.4 Batch Operations

**Current State:** Manual loops
**Goal:** Efficient bulk updates

```rust
// AFTER: Batch operations
let updates = vec![
    (1i64, "Alice"),
    (2i64, "Bob"),
    (3i64, "Carol"),
];
User::update_batch(&pool, "name", updates).await?;

// Bulk delete
User::delete_in(&pool, vec![1, 2, 3]).await?;

// Bulk insert (already exists, can enhance)
User::insert_ignore(&pool, &rows).await?;  // INSERT IGNORE (MySQL)
User::insert_on_conflict(&pool, &rows, "email").await?;  // PostgreSQL
```

**Changes:**
- [ ] Add `update_batch()`
- [ ] Add `delete_in()`
- [ ] Add `insert_ignore()` (MySQL style)
- [ ] Optimize for large batches (chunking)

---

## Phase 5: Production Readiness (2-3 weeks)

### 5.1 Performance Optimizations

**Issues:**
- No prepared statement caching
- No connection pool tuning
- Potential N+1 queries

**Solutions:**

```rust
// Prepared statement cache
pub struct CachedStatement {
    cache: RwLock<HashMap<String, PreparedStatement>>,
    pool: PgPool,
}

impl PgModel {
    async fn cached_query(sql: &str) -> PreparedStatement {
        // Check cache, prepare if missing
    }
}

// Connection pool tuning via config
#[derive(Config)]
pub struct DatabaseConfig {
    #[env("DATABASE_URL")]
    pub url: String,

    #[env("DB_MIN_CONNECTIONS", default = 5)]
    pub min_connections: u32,

    #[env("DB_MAX_CONNECTIONS", default = 20)]
    pub max_connections: u32,

    #[env("DB_IDLE_TIMEOUT", default = 300)]
    pub idle_timeout_secs: u64,
}
```

**Changes:**
- [ ] Add prepared statement cache
- [ ] Add connection pool configuration
- [ ] Add query timeout support
- [ ] Add retry logic for transient failures
- [ ] Add metrics (query time, connection usage)

### 5.2 Error Handling Enhancement

**Current State:** Basic sqlx::Error
**Goal:** Structured ORM errors

```rust
// AFTER: Rich error types
#[derive(Error, Debug)]
pub enum OrmError {
    #[error("Record not found: {model}::{pk}={id}")]
    NotFound { model: String, pk: String, id: String },

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Constraint violation: {0}")]
    Constraint(#[from] ConstraintError),

    #[error("Transaction failed: {0}")]
    Transaction(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Hook failed: {0}")]
    Hook(String),
}

impl From<sqlx::Error> for OrmError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => OrmError::NotFound { ... },
            _ => OrmError::Database(err),
        }
    }
}

// Usage with anyhow-like ergonomics
let user = User::find_or_404(&pool, id)
    .await
    .map_err(|e| match e {
        OrmError::NotFound { .. } => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
```

**Changes:**
- [ ] Define `OrmError` enum with all variants
- [ ] Add `OrmResult<T>` type alias
- [ ] Implement `From<sqlx::Error>` conversions
- [ ] Add context to errors (model, column, etc.)

### 5.3 Observability

**Goal:** Debugging and monitoring support

```rust
// AFTER: Query logging
#[derive(Config)]
pub struct LoggingConfig {
    #[env("LOG_SQL", default = false)]
    pub log_sql: bool,

    #[env("LOG_SLOW_QUERIES", default = true)]
    pub log_slow_queries: bool,

    #[env("SLOW_QUERY_THRESHOLD_MS", default = 1000)]
    pub slow_query_threshold_ms: u64,
}

// Usage
let pool = PgPool::connect(&url).await?;
pool.attach_callback(|event| {
    match event {
        QueryEvent::Executed { sql, duration } => {
            tracing::debug!(sql, duration_ms = duration.as_millis());
        }
        QueryEvent::SlowQuery { sql, duration } => {
            tracing::warn!(sql, duration_ms = duration.as_millis(), "Slow query");
        }
    }
});
```

**Changes:**
- [ ] Add query logging with feature flag
- [ ] Add slow query detection
- [ ] Add OpenTelemetry tracing
- [ ] Add metrics export

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

## Phase 6: Ecosystem (Ongoing)

### 6.1 MySQL Support

```rust
// crates/rok-orm-mysql/
// - MyModel trait
// - MyExecutor
// - MySqlDialect
```

### 6.2 Redis Caching Integration

```rust
// After: Transparent caching
#[derive(Model)]
#[model(cache = "users", ttl = 300)]
pub struct User { ... }

let user = User::find_cached(&pool, id).await?;  // Redis backed
User::invalidate_cache(&pool, id).await?;
```

### 6.3 Rocket Integration

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

## Implementation Order

### Week 1-2: Quick Wins
1. Soft delete full implementation
2. Auto timestamps
3. Aggregation methods

### Week 3-4: Core Features
1. Eager loading
2. BelongsToMany
3. Pagination

### Week 5-6: Developer Experience
1. CLI tool (rok-cli)
2. Migration system
3. Seeder system

### Week 7-8: Advanced Features
1. Model hooks
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

## Success Metrics

- [ ] 100% test coverage for public API
- [ ] < 100ms average query time for standard operations
- [ ] < 1MB memory overhead for typical application
- [ ] Zero breaking changes in minor versions
- [ ] Complete docs.rs documentation
- [ ] 10+ example projects in examples/
- [ ] Active community Discord/forum
- [ ] 500+ GitHub stars
- [ ] Production users (to be tracked)

---

## Appendix: Feature Priority Voting

Based on user feedback, prioritize:

1. **[Vote: Eager Loading]** - Critical for N+1 prevention
2. **[Vote: Migrations]** - Essential for schema management
3. **[Vote: Pagination]** - Common web app requirement
4. **[Vote: BelongsToMany]** - Many apps need many-to-many
5. **[Vote: CLI Tool]** - Developer productivity
