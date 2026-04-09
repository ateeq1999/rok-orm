---
id: bug-011
type: bug
severity: high
affects: examples/14a-core/src/soft_deletes.rs:42-47
---

# Soft Delete Methods Called as Static Methods Instead of Query Builder Methods

## Description

`soft_deletes.rs` calls `with_soft_delete()` and `only_trashed()` as if they were
static methods on `Post` (like `Post::with_soft_delete()`). These methods do not
exist as static calls — they are **builder methods** on `QueryBuilder<T>` and must be
called on `Post::query()`.

## Current (broken) Code

```rust
let all = Post::with_soft_delete().get(pool).await?;   // ❌ no static method
let trashed = Post::only_trashed().get(pool).await?;   // ❌ no static method
```

## Compiler Error

```
error[E0599]: no function or associated item named `with_soft_delete` found
              for struct `soft_deletes::Post` in the current scope
```

## Fix

Call these methods on `Post::query()` (a `QueryBuilder`), then execute via the
correct API:

```rust
use rok_orm::{PgModel, executor::postgres};

// Include soft-deleted rows:
let all = postgres::fetch_all(pool, Post::query().with_trashed()).await?;
println!("   Total posts (with trashed): {}", all.len());

// Only soft-deleted rows:
let trashed = postgres::fetch_all(pool, Post::query().only_trashed()).await?;
println!("   Trashed posts: {}", trashed.len());
```

Note: `with_soft_delete(column)` requires a column name argument; for a model with
`#[model(soft_delete)]` use the parameterless `with_trashed()` / `only_trashed()`
builder methods instead.

Also, `Post::all(pool)`, `Post::delete_by_pk(pool, id)`, and `Post::restore(pool, id)`
all require `use rok_orm::PgModel;` in scope (see bug-001).
