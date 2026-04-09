---
id: bug-006
type: bug
severity: high
affects: examples/14a-core/src/pagination.rs
---

# Pagination API Used Incorrectly

## Description

`pagination.rs` mixes up two different pagination APIs:

1. **Line 49** — passes `pool` as an argument to `QueryBuilder::paginate()`, which
   is a **builder method** (no pool, no async, returns `Self`).
2. **Line 50** — awaits the result of `paginate()`, but a `QueryBuilder` is not a
   `Future`.
3. **Lines 31 / 42** — `Post::paginate(pool, page, per_page)` is correct but
   requires `PgModelExt` to be in scope (see bug-001).

## Current (broken) Code

```rust
// ✅ This form is correct (needs PgModelExt in scope):
let page: Page<Post> = Post::paginate(pool, 1, 5).await?;

// ❌ This is wrong — paginate() on QueryBuilder takes no pool:
let custom_page = Post::query()
    .order_by_desc("id")
    .paginate(pool, 1, 10)   // ❌ pool is not a parameter here
    .await?;                  // ❌ QueryBuilder is not a Future
```

## Compiler Errors

```
error[E0061]: this method takes 2 arguments but 3 arguments were supplied
   --> src/pagination.rs:49:10
    |
49 |         .paginate(pool, 1, 10)
    |          ^^^^^^^^ ---- unexpected argument #1 of type `&Pool<Postgres>`

error[E0277]: `rok_orm::QueryBuilder<pagination::Post>` is not a future
  --> src/pagination.rs:50:10
```

## Fix

Use `PgModelExt::paginate_where(pool, builder, page, per_page)` for custom-query
pagination, which wraps the builder in a `Page<T>`:

```rust
use rok_orm::{PgModel, PgModelExt};

// Simple pagination (all rows):
let page: Page<Post> = Post::paginate(pool, 1, 5).await?;

// Custom query with pagination:
let custom_page = Post::paginate_where(
    pool,
    Post::query().order_by_desc("id"),
    1,   // page
    10,  // per_page
).await?;
println!("Custom query page: {}/{}", custom_page.current_page, custom_page.last_page);
```
