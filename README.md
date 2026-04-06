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

## Model Attributes

### Struct-level

```rust
#[derive(Model)]
#[model(table = "articles")]           // Custom table name
#[model(primary_key = "article_id")]   // Custom primary key column
#[model(soft_delete)]                   // Enable soft deletes
#[model(timestamps)]                    // Auto timestamps
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

### Aggregation

| Method | Description |
|--------|-------------|
| `.sum_sql(col)` | Returns `(String, Vec<SqlValue>)` for SUM |
| `.avg_sql(col)` | Returns `(String, Vec<SqlValue>)` for AVG |
| `.min_sql(col)` | Returns `(String, Vec<SqlValue>)` for MIN |
| `.max_sql(col)` | Returns `(String, Vec<SqlValue>)` for MAX |

## Roadmap

See the full [rok ecosystem roadmap](https://github.com/rok-rs/rok) for upcoming features including:

- MySQL support
- MSSQL support
- CLI tooling (rok-cli)
- Redis caching integration
- Framework integrations

## Contributing

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
