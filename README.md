# rok-orm

> Eloquent-inspired async ORM for Rust — designed for ergonomics first, strongly-typed always.

## Overview

`rok-orm` is a Rust ORM library inspired by Laravel's Eloquent. It provides a fluent query builder, async database operations, and macro-driven model definitions with minimal boilerplate.

```rust
use rok_orm::{Model, PgModel};

#[derive(Model, sqlx::FromRow)]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
}

let users = User::query()
    .where_eq("active", true)
    .order_by_desc("created_at")
    .limit(10)
    .get(&pool)
    .await?;

let user = User::find_by_pk(&pool, 1).await?;
```

## Features

| Feature | Description |
|---------|-------------|
| **Fluent Query Builder** | Chain conditions, joins, pagination with a clean API |
| **Async Execution** | Built on sqlx for high-performance async database operations |
| **Multi-Database** | Support for PostgreSQL and SQLite with dialect-aware SQL generation |
| **Derive Macros** | `#[derive(Model)]` auto-generates table names, columns, and metadata |
| **Relationships** | Define `has_many`, `has_one`, `belongs_to`, `belongs_to_many` relationships |
| **Soft Deletes** | Built-in support for soft delete patterns |
| **Auto Timestamps** | Automatic `created_at`/`updated_at` management |
| **Eager Loading** | Prevent N+1 queries with `.with()` |
| **Pagination** | Built-in pagination with `Page<T>` |
| **Aggregations** | `sum()`, `avg()`, `min()`, `max()`, `count()` |
| **Upsert** | `INSERT ... ON CONFLICT` support |
| **Model Hooks** | Lifecycle events (`before_create`, `after_create`, etc.) |
| **Transactions** | First-class transaction support with the `Tx` wrapper |
| **Error Handling** | Structured `OrmError` with variants for common cases |
| **Query Logging** | Built-in logging with `Logger` and slow query detection |
| **Conditional Queries** | `when()` / `when_else()` for dynamic query building |
| **Raw Expressions** | `where_raw`, `select_raw`, `order_raw`, `from_raw_sql` |
| **Pagination** | `Page<T>` and `CursorResult<T>` with cursor pagination |
| **Chunking** | `chunk()` / `chunk_by_id()` for large dataset processing |
| **Model Observers** | Lifecycle callbacks for model events |
| **Global Scopes** | Apply conditions to all model queries |
| **Mass Assignment** | `fillable` / `guarded` protection |
| **withCount / withSum** | Relationship aggregates as query extras |
| **whereHas** | Filter by relationship existence |

## Crates

| Crate | Description |
|-------|-------------|
| `rok-orm` | Main ORM façade with async executors |
| `rok-orm-core` | Core traits, QueryBuilder, and model abstractions |
| `rok-orm-macros` | `#[derive(Model)]` and `query!()` procedural macros |

## Installation

```toml
[dependencies]
rok-orm = { version = "0.3", features = ["postgres"] }
# or for SQLite:
rok-orm = { version = "0.3", features = ["sqlite"] }
```

## Quick Start

### 1. Define a Model

```rust
use rok_orm::Model;

#[derive(Model, sqlx::FromRow)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
}
```

### 2. Run Queries

```rust
use rok_orm::{PgModel, SqlValue};

let users: Vec<User> = User::all(&pool).await?;
let user = User::find_by_pk(&pool, 1).await?;

let admins: Vec<User> = User::find_where(
    &pool,
    User::query()
        .where_eq("active", true)
        .order_by_desc("created_at")
        .limit(10)
).await?;

let count = User::count(&pool).await?;
```

### 3. Create, Update, Delete

```rust
User::create(&pool, &[
    ("name", "Alice".into()),
    ("email", "alice@example.com".into()),
]).await?;

User::update_by_pk(&pool, 1, &[("name", "Bob".into())]).await?;
User::delete_by_pk(&pool, 1).await?;

let user: User = User::create_returning(&pool, &[
    ("name", "Carol".into()),
    ("email", "carol@example.com".into()),
]).await?;
```

### 4. Soft Deletes

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "posts", soft_delete)]
pub struct Post { ... }

// Excludes deleted records by default
let posts = Post::all(&pool).await?;

// Include deleted
let all = Post::with_trashed().get(&pool).await?;

// Only deleted
let trashed = Post::only_trashed().get(&pool).await?;

// Restore
Post::restore(&pool, id).await?;

// Force delete (permanent)
Post::force_delete(&pool, id).await?;
```

### 5. Auto Timestamps

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "articles", timestamps)]
pub struct Article { ... }

// create_returning() auto-adds created_at and updated_at
// update_by_pk() auto-updates updated_at
```

### 6. Relationships

```rust
use rok_orm::relations::{HasMany, BelongsTo};

#[derive(Relations)]
pub struct PostRelations {
    #[has_many(target = "Comment")]
    pub comments: HasMany<Post, Comment>,
    
    #[belongs_to(target = "User")]
    pub user: BelongsTo<Post, User>,
}

// Eager loading (prevents N+1)
let posts = Post::query()
    .with("user")
    .with("comments")
    .limit(10)
    .get(&pool)
    .await?;

// Lazy loading
use rok_orm::relations::lazy;
let comments = lazy::load_has_many(&pool, &post.comments, &[1, 2, 3]).await?;
```

### 7. Pagination

```rust
let page: Page<Post> = Post::paginate(&pool, 1, 20).await?;

println!("Total: {} pages", page.last_page);
println!("Current: {}", page.current_page);
println!("Has next: {}", page.has_next());
println!("Has prev: {}", page.has_prev());
```

### 8. Aggregations

```rust
let total: i64 = User::count(&pool).await?;
let revenue: f64 = Order::sum("total", &pool).await?;
let avg_age: f64 = User::avg("age", &pool).await?;
let oldest = User::max("age", &pool).await?;
```

### 9. Upsert

```rust
User::upsert(&pool, &[
    ("email", "admin@example.com".into()),
    ("name", "Admin".into()),
]).await?;

User::upsert(&pool, &[
    ("email", "admin@example.com".into()),
    ("name", "Admin Updated".into()),
], "email", &["name"]).await?;
```

### 10. Query Scopes

```rust
impl User {
    pub fn active() -> QueryBuilder<User> {
        User::query().filter("active", true)
    }
    
    pub fn admins() -> QueryBuilder<User> {
        User::query().filter("role", "admin")
    }
}

let users = User::active().admins().get(&pool).await?;
```

### 11. Error Handling

```rust
use rok_orm::errors::{OrmError, OrmResult};

let user = User::find_by_pk(&pool, id).await?;

match user {
    Err(OrmError::NotFound { model, pk, id }) => {
        println!("{} not found", model);
    }
    Err(e) => return Err(e),
    Ok(user) => user,
}
```

### 12. Query Logging

```rust
use rok_orm::logging::{Logger, LogLevel, QueryTimer};

let logger = Logger::new()
    .with_slow_query_threshold(100)
    .with_log_level(LogLevel::Debug);

let timer = QueryTimer::new();
// ... execute query ...
let elapsed = timer.elapsed_ms();
if logger.is_slow_query(elapsed) {
    // log slow query
}
```

### 13. Transactions

```rust
use rok_orm::Tx;

let mut tx = Tx::begin(&pool).await?;
tx.insert::<User>("users", &[("name", "Alice".into())]).await?;
tx.insert::<Post>("posts", &[("user_id", 1i64.into())]).await?;
tx.commit().await?;
```

### 14. Shorthand with `query!`

```rust
use rok_orm_macros::query;

let q = query!(User,
    where_eq "active" true,
    order_by_desc "created_at",
    limit 10
);

let users = User::find_where(&pool, q).await?;
```

### 15. Conditional Query Building

```rust
let users = User::query()
    .when(params.role.is_some(), |q| {
        q.filter("role", params.role.unwrap())
    })
    .when(params.active, |q| q.filter("active", true))
    .when(params.search.is_some(), |q| {
        q.where_like("name", &format!("%{}%", params.search.unwrap()))
    })
    .order_by_desc("created_at")
    .limit(20)
    .get(&pool)
    .await?;
```

### 16. Raw Expressions

```rust
let users = User::query()
    .where_raw("LOWER(email) = LOWER($1)", vec!["admin@example.com".into()])
    .get(&pool)
    .await?;

let stats = User::query()
    .select(&["role", "COUNT(*) as count"])
    .group_by(&["role"])
    .having_raw("COUNT(*) BETWEEN 5 AND 100")
    .get(&pool)
    .await?;
```

### 17. Cursor Pagination

```rust
let result = Post::query()
    .order_by_desc("created_at")
    .cursor_paginate(&pool, CursorPage { after: None, limit: 20 })
    .await?;

println!("Next cursor: {:?}", result.next_cursor);
println!("Has more: {}", result.has_more);
```

### 18. Chunking Large Datasets

```rust
User::query()
    .filter("active", true)
    .chunk(&pool, 500, |batch| async move {
        for user in batch {
            process_user(&user).await;
        }
        Ok(())
    })
    .await?;

User::query()
    .chunk_by_id(&pool, 500, |batch| async move {
        process(batch).await
    })
    .await?;
```

### 19. Mass Assignment Protection

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "users", fillable = ["name", "email", "bio"])]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
    pub role: String,     // not fillable - will be ignored
    pub is_admin: bool,   // not fillable - will be ignored
}

let user = User::create_returning(&pool, &[
    ("name", "Alice".into()),
    ("email", "alice@example.com".into()),
    ("role", "admin".into()),   // silently dropped
    ("is_admin", true.into()),   // silently dropped
]).await?;
```

### 20. Model Observers

```rust
pub struct UserObserver;

impl ModelObserver for UserObserver {
    type Model = User;

    async fn creating(&self, user: &mut User) -> OrmResult<()> {
        user.email = user.email.to_lowercase();
        Ok(())
    }
    async fn created(&self, user: &User) -> OrmResult<()> {
        send_welcome_email(&user.email).await
    }
    async fn deleted(&self, user: &User) -> OrmResult<()> {
        invalidate_cache("user", user.id).await
    }
}

User::observe(UserObserver);
```

### 21. Global Query Scopes

```rust
pub struct ActiveScope;

impl GlobalScope<User> for ActiveScope {
    fn apply(&self, query: QueryBuilder<User>) -> QueryBuilder<User> {
        query.filter("active", true)
    }
}

User::add_global_scope(ActiveScope);

let users = User::all(&pool).await?;  // automatically filtered by active=true

let all = User::query()
    .without_global_scope::<ActiveScope>()
    .get(&pool)
    .await?;
```

### 22. Relationship Aggregates (withCount / withSum)

```rust
let posts = Post::query()
    .with_count("comments")
    .with_count_as("published_comments", "comments", |q| q.filter("published", true))
    .get(&pool)
    .await?;

for post in &posts {
    println!("Total comments: {:?}", post.extras.get("comments_count"));
    println!("Published: {:?}", post.extras.get("published_comments_count"));
}

let users = User::query()
    .with_sum("orders", "total")
    .with_avg("orders", "total")
    .get(&pool)
    .await?;
```

### 23. whereHas / whereDoesntHave

```rust
let posts = Post::query()
    .where_has("comments", |q| q.filter("published", true))
    .get(&pool)
    .await?;

let posts = Post::query()
    .where_has_count("comments", 5, CountOp::GreaterThan)
    .get(&pool)
    .await?;

let users = User::query()
    .where_doesnt_have("posts")
    .get(&pool)
    .await?;
```

### 24. firstOrCreate / updateOrCreate

```rust
let user = User::first_or_create(&pool,
    &[("email", "alice@example.com".into())],
    &[("name", "Alice".into()), ("role", "user".into())],
).await?;

let user = User::update_or_create(&pool,
    &[("email", "alice@example.com".into())],
    &[("name", "Alice Updated".into())],
).await?;
```

### 25. Model Replication

```rust
let original = Post::find_or_404(&pool, 1).await?;
let mut copy = original.replicate();
copy.title = format!("Copy of {}", original.title);
let saved = Post::create_returning(&pool, &copy.to_fields()).await?;
```

### 26. withoutTimestamps / Event Muting

```rust
User::without_timestamps(|| async {
    User::update_by_pk(&pool, 1, &[("views", 1000.into())]).await
}).await?;

User::without_events(|| async {
    User::create(&pool, &[("name", "Seeded".into())]).await
}).await?;
```

## Model Attributes

### Struct-level

```rust
#[derive(Model)]
#[model(table = "articles")]           // Custom table name
#[model(primary_key = "article_id")]   // Custom primary key column
#[model(soft_delete)]                   // Enable soft deletes
#[model(timestamps)]                    // Auto timestamps
#[model(timestamps, created_at_col = "creation_date", updated_at_col = "modified_date")] // Custom timestamp columns
#[model(touches = ["user"])]            // Update parent timestamp on write
#[model(connection = "audit_db")]       // Per-model database connection
#[model(uuid)]                          // UUID primary key
#[model(ulid)]                          // ULID primary key
#[model(fillable = ["name", "email"])]  // Allow mass assignment
#[model(guarded = ["role", "is_admin"])] // Block mass assignment
#[model(prunable)]                      // Enable model pruning
pub struct Article { ... }
```

### Field-level

```rust
#[derive(Model)]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    
    #[model(column = "post_title")]    // Map to different column name
    pub title: String,
    
    #[model(skip)]                     // Exclude from columns
    pub cache: String,
}
```

## Query Builder Methods

### Conditions

| Method | Description |
|--------|-------------|
| `.where_eq(col, val)` | WHERE col = val |
| `.where_ne(col, val)` | WHERE col != val |
| `.where_gt(col, val)` | WHERE col > val |
| `.where_gte(col, val)` | WHERE col >= val |
| `.where_lt(col, val)` | WHERE col < val |
| `.where_lte(col, val)` | WHERE col <= val |
| `.where_like(col, pattern)` | WHERE col LIKE pattern |
| `.where_null(col)` | WHERE col IS NULL |
| `.where_not_null(col)` | WHERE col IS NOT NULL |
| `.where_in(col, vec)` | WHERE col IN (...) |
| `.where_between(col, lo, hi)` | WHERE col BETWEEN lo AND hi |
| `.or_where_eq(col, val)` | OR col = val |
| `.filter(col, val)` | Alias for `.where_eq()` |
| `.eq(col, val)` | Short alias for `.where_eq()` |

### Eager Loading

| Method | Description |
|--------|-------------|
| `.with(relation)` | Eager load a relation |
| `.with_many(relation)` | Eager load a has_many relation |

### Soft Deletes

| Method | Description |
|--------|-------------|
| `.with_soft_delete()` | Include soft-deleted records |
| `.with_trashed()` | Alias for `.with_soft_delete()` |
| `.only_trashed()` | Only soft-deleted records |

### Other

| Method | Description |
|--------|-------------|
| `.select(&[cols])` | SELECT specific columns |
| `.distinct()` | SELECT DISTINCT |
| `.order_by(col)` | ORDER BY col ASC |
| `.order_by_desc(col)` | ORDER BY col DESC |
| `.limit(n)` | LIMIT n |
| `.offset(n)` | OFFSET n |
| `.inner_join(t, on)` | INNER JOIN |
| `.left_join(t, on)` | LEFT JOIN |
| `.group_by(&[cols])` | GROUP BY |
| `.having(expr)` | HAVING clause |
| `.paginate(page, per_page)` | Add pagination |

### Conditional & Raw

| Method | Description |
|--------|-------------|
| `.when(condition, fn)` | Apply query builder closure when condition is true |
| `.when_else(condition, fn_true, fn_false)` | Conditional branching |
| `.where_raw(sql, params)` | Raw WHERE clause |
| `.select_raw(sql)` | Raw SELECT clause |
| `.order_raw(sql)` | Raw ORDER BY clause |
| `.having_raw(sql)` | Raw HAVING clause |
| `.from_raw_sql(pool, sql, params)` | Execute raw SQL query |
| `.tap(fn)` | Debug tap without modifying query |
| `.dd()` | Debug: print SQL then panic (dev only) |

### Relationship Filtering

| Method | Description |
|--------|-------------|
| `.where_has(rel, closure)` | WHERE EXISTS (subquery) |
| `.where_doesnt_have(rel, closure?)` | WHERE NOT EXISTS (subquery) |
| `.where_has_count(rel, n, op)` | Filter by relationship count |
| `.with_count(rel)` | Add count of related records as extra |
| `.with_sum(rel, col)` | Add sum of related column as extra |
| `.with_avg(rel, col)` | Add average of related column as extra |
| `.with_max(rel, col)` | Add max of related column as extra |
| `.with_min(rel, col)` | Add min of related column as extra |

### Aggregation

| Method | Description |
|--------|-------------|
| `.sum_sql(col)` | Returns `(String, Vec<SqlValue>)` for SUM |
| `.avg_sql(col)` | Returns `(String, Vec<SqlValue>)` for AVG |
| `.min_sql(col)` | Returns `(String, Vec<SqlValue>)` for MIN |
| `.max_sql(col)` | Returns `(String, Vec<SqlValue>)` for MAX |

## Roadmap

See the full [rok ecosystem roadmap](https://github.com/rok-rs/rok) for upcoming features including:

### v0.5.0 (Q4 2026)

- Schema Builder with Blueprint API for code-first migrations
- Migration system with run, rollback, reset, fresh commands
- Auto-generate models from database schema
- JSON column queries (where_json_contains, where_json_path)
- Full-text search (PostgreSQL tsvector, MySQL MATCH, SQLite FTS5)
- Sub-queries and Common Table Expressions (CTEs)
- Window functions support

### v0.6.0 (Q1 2027)

- Attribute casting (json, datetime, bool, csv, encrypted)
- Serialization control (hidden, visible, appends)
- Accessors and mutators
- Model factories with faker integration
- Database transactions per test
- Assertion helpers

### v1.0.0 (Q2 2027)

- MySQL support (already in progress)
- MSSQL / SQL Server support
- Redis cache integration
- Axum / Actix-web integration
- CLI tooling (rok-cli)

### Already Available

- PostgreSQL support
- SQLite support
- Fluent query builder
- Relationships (has_many, has_one, belongs_to, belongs_to_many)
- Eager loading
- Soft deletes
- Auto timestamps
- Pagination
- Aggregations
- Upsert operations
- Model hooks
- Transactions
- Query logging

## Contributing

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
