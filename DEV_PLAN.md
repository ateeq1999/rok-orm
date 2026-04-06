# rok-orm Development Plan

## Project Status

**Current Version:** 0.2.0 (pre-release)

**Phase 1 (Foundation):** ✅ Complete
Phase 2 (Eloquent API): ✅ Complete
Phase 3 (Relationships): ✅ Foundation Complete

---

## Architecture Overview

```
rok-orm/
├── Cargo.toml              # Workspace root
├── README.md               # Documentation
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
│   │   └── relations.rs    # Relationship types
│   ├── tests/
│   └── Cargo.toml
│
└── rok-orm-macros/         # Procedural macros
    ├── src/lib.rs          # #[derive(Model)], #[derive(Relations)], query!()
    └── Cargo.toml
```

---

## Implemented Features

### ✅ Core Features (v0.2.0)

| Feature | Status | Notes |
|---------|--------|-------|
| Fluent QueryBuilder | ✅ | All WHERE conditions, joins, pagination |
| `#[derive(Model)]` | ✅ | Auto-generates table_name, columns, primary_key |
| `#[model(...)]` | ✅ | table, primary_key, soft_delete, timestamps, skip, column |
| `#[derive(Relations)]` | ✅ | HasMany, HasOne, BelongsTo definitions |
| `#[derive(BelongsToMany)]` | ✅ | Many-to-many relationship support |
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

## Development Roadmap

### v0.2.x — Stabilization

#### TODO: Eager Loading with `.with()`

```rust
// Target API
let post = Post::find(1).with("user").with("tags").first(&pool).await?;
let posts = Post::all().with("user").get(&pool).await?;
```

**Implementation:**
- [ ] Add `with()` method to QueryBuilder
- [ ] Track loaded relations in a context struct
- [ ] Implement N+1 query prevention
- [ ] Add `lazy` vs `eager` loading options

#### TODO: Named Scopes

```rust
// Target API
impl User {
    fn active() -> QueryBuilder<User> {
        User::query().filter("active", true)
    }
    
    fn admins() -> QueryBuilder<User> {
        User::query().filter("role", "admin")
    }
}

let admins = User::admins().get(&pool).await?;
```

**Implementation:**
- [ ] Document the pattern
- [ ] Add examples to README

#### TODO: Model Hooks/Events

```rust
// Target API
impl ModelHooks for User {
    async fn before_create(&mut self) -> Result<(), OrmError> {
        self.email = self.email.to_lowercase();
        Ok(())
    }
    
    async fn after_create(&self) {
        events::emit(UserCreated { id: self.id });
    }
}
```

**Implementation:**
- [ ] Define `ModelHooks` trait
- [ ] Add `before_create`, `after_create`, `before_update`, `after_update`, `before_delete`, `after_delete`
- [ ] Integrate with PgModel/SqliteModel methods

#### TODO: Soft Delete Auto-Filtering

```rust
// Target API
Post::all(&pool).await?;  // Automatically excludes soft-deleted
Post::with_trashed().get(&pool).await?;
Post::restore(id).await?;
Post::force_delete(id).await?;
```

**Implementation:**
- [ ] Modify `all()` and `get()` to auto-add `WHERE deleted_at IS NULL`
- [ ] Add `with_trashed()`, `only_trashed()` methods
- [ ] Add `restore()` and `force_delete()` methods

#### TODO: Auto Timestamps

```rust
// Target API
User::create_returning(&pool, &[("name", "Alice".into())]).await?;
// Automatically sets created_at
```

**Implementation:**
- [ ] Hook into `create_returning()` to inject `created_at`
- [ ] Hook into `update_by_pk()` to inject `updated_at`

---

### v0.3.0 — Relationships & Relations

#### TODO: BelongsToMany (Many-to-Many)

```rust
// Target API
#[derive(Model)]
pub struct Post {
    pub id: i64,
    pub title: String,
    #[model(belongs_to_many(Tag, through = "post_tags"))]
    pub tags: Vec<Tag>,
}

let tags = post.tags().get(&pool).await?;
```

**Implementation:**
- [ ] Define `BelongsToMany` struct
- [ ] Add `through` parameter to derive macro
- [ ] Implement pivot table handling

#### TODO: Relation Instance Methods

```rust
// Target API
let posts = user.posts().filter("published", true).get(&pool).await?;
```

**Implementation:**
- [ ] Refine the Relations derive to generate proper methods
- [ ] Return a query builder from relation methods

#### TODO: Lazy Relation Loading

```rust
// Target API
let post = Post::find(1).await?;
let user = post.user(&pool).await?;  // Lazy load
```

**Implementation:**
- [ ] Add lazy loading methods to relation types
- [ ] Require pool reference for lazy loading

---

### v0.4.0 — CLI & Tooling

#### TODO: rok-cli Binary

```bash
rok new my-project
rok make:model User
rok make:migration create_users_table
rok db:migrate
rok db:rollback
rok db:seed
rok routes:list
```

**Implementation:**
- [ ] Create `rok-cli` crate
- [ ] Implement `new`, `make:*` commands
- [ ] Implement `db:migrate`, `db:rollback`
- [ ] Implement `db:seed`
- [ ] Interactive prompts with `dialoguer`

#### TODO: Migration System

```rust
// Target usage
#[migration]
fn m001_create_users() -> String {
    r#"
    CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        email VARCHAR(255) UNIQUE NOT NULL,
        created_at TIMESTAMPTZ DEFAULT NOW()
    );
    "#
}
```

**Implementation:**
- [ ] Define `#[migration]` procedural macro
- [ ] Create migration runner
- [ ] Track migration state in database
- [ ] Support rollback SQL

#### TODO: Seeder System

```rust
#[Seeder]
struct UserSeeder;

impl UserSeeder {
    async fn run(pool: &PgPool) {
        User::create(&pool, &[("name", "Admin".into())]).await?;
    }
}
```

**Implementation:**
- [ ] Define `#[Seeder]` attribute
- [ ] Create seeder runner
- [ ] Track seeded state

---

### v0.5.0 — Enterprise Features

#### TODO: Pagination

```rust
// Target API
let page = Post::paginate(&pool, 1, 20).await?;
assert_eq!(page.total, 100);
assert_eq!(page.data.len(), 20);
```

**Implementation:**
- [ ] Define `Page<T>` struct with `data`, `total`, `per_page`, `current_page`
- [ ] Add `paginate(page, per_page)` method

#### TODO: Aggregates

```rust
// Target API
let avg: f64 = Post::avg("price", &pool).await?;
let sum: i64 = Order::sum("total", &pool).await?;
```

**Implementation:**
- [ ] Add `avg()`, `sum()`, `min()`, `max()` methods

#### TODO: Upsert

```rust
// Target API
User::upsert(&pool, &[("email", email.into())], &["name"]).await?;
```

**Implementation:**
- [ ] Add `upsert()` method for PostgreSQL ON CONFLICT

#### TODO: Batch Operations

```rust
// Target API
User::update_batch(&pool, vec![(1, "Alice"), (2, "Bob")]).await?;
```

**Implementation:**
- [ ] Add batch update/delete methods

---

### v1.0.0 — Production Release

#### TODO: Performance Optimizations

- [ ] Connection pooling tuning
- [ ] Query caching layer
- [ ] Prepared statement caching

#### TODO: Documentation

- [ ] Full API documentation on docs.rs
- [ ] User guide / tutorial
- [ ] Examples repository

#### TODO: Compatibility

- [ ] MySQL support (via sqlx)
- [ ] MSSQL support (via sqlx)
- [ ] Full feature parity with Eloquent

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
| 0.2.0 | 2026 | Added soft_delete, timestamps, relations, find_or_404, Eloquent-style API |
