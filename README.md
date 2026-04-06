# rok-orm

> Eloquent-inspired async ORM for Rust — designed for ergonomics first, strongly-typed always.

## Overview

`rok-orm` is a Rust ORM library inspired by Laravel's Eloquent. It provides a fluent query builder, async database operations, and macro-driven model definitions with minimal boilerplate.

```rust
use rok_orm::{Model, PgModel};

#[derive(Model, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

// Fluent query building
let users = User::query()
    .where_eq("active", true)
    .order_by_desc("created_at")
    .limit(10)
    .get(&pool)
    .await?;

// Find by primary key
let user = User::find_by_pk(&pool, 1).await?;
```

## Features

- **Fluent Query Builder** — Chain conditions, joins, pagination with a clean API
- **Async Execution** — Built on sqlx for high-performance async database operations
- **Multi-Database** — Support for PostgreSQL and SQLite with dialect-aware SQL generation
- **Derive Macros** — `#[derive(Model)]` auto-generates table names, columns, and metadata
- **Relationships** — Define `has_many`, `has_one`, `belongs_to`, `belongs_to_many` relationships
- **Soft Deletes** — Built-in support for soft delete patterns
- **Auto Timestamps** — Automatic `created_at`/`updated_at` management
- **Model Hooks** — Lifecycle events (`before_create`, `after_create`, etc.)
- **Transactions** — First-class transaction support with the `Tx` wrapper

## Crates

| Crate | Description |
|-------|-------------|
| `rok-orm` | Main ORM façade with async executors |
| `rok-orm-core` | Core traits, QueryBuilder, and model abstractions |
| `rok-orm-macros` | `#[derive(Model)]` and `query!()` procedural macros |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rok-orm = { version = "0.2", features = ["postgres"] }
# or for SQLite:
rok-orm = { version = "0.2", features = ["sqlite"] }
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
    pub active: bool,
}
```

### 2. Run Queries

```rust
use rok_orm::{PgModel, SqlValue};

// All records
let users: Vec<User> = User::all(&pool).await?;

// Find by ID
let user = User::find_by_pk(&pool, 1).await?;

// Custom query
let admins: Vec<User> = User::find_where(
    &pool,
    User::query()
        .where_eq("active", true)
        .order_by_desc("created_at")
        .limit(10)
).await?;

// Count
let count = User::count(&pool).await?;
```

### 3. Create, Update, Delete

```rust
// Create
User::create(&pool, &[
    ("name", "Alice".into()),
    ("email", "alice@example.com".into()),
]).await?;

// Update
User::update_by_pk(&pool, 1, &[
    ("name", "Bob".into()),
]).await?;

// Delete
User::delete_by_pk(&pool, 1).await?;

// Insert and get back the row
let user: User = User::create_returning(&pool, &[
    ("name", "Carol".into()),
    ("email", "carol@example.com".into()),
]).await?;
```

### 4. Use Transactions

```rust
use rok_orm::Tx;

let mut tx = Tx::begin(&pool).await?;

tx.insert::<User>("users", &[("name", "Alice".into())]).await?;
tx.insert::<Post>("posts", &[("user_id", 1i64.into())]).await?;

tx.commit().await?;
```

### 5. Shorthand with `query!`

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
#[model(table = "articles")]          // Custom table name
#[model(primary_key = "article_id")]  // Custom primary key column
#[model(soft_delete)]                  // Enable soft deletes
#[model(timestamps)]                   // Auto timestamps
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
    
    #[model(skip)]                      // Exclude from columns
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

## Roadmap

See the full [rok ecosystem roadmap](https://github.com/rok-rs/rok) for upcoming features including:

- Relationships (has_many, belongs_to, etc.)
- Eager loading with `.with()`
- Soft deletes
- Auto timestamps
- Model hooks/events
- CLI tooling

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
