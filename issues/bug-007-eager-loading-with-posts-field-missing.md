---
id: bug-007
type: bug
severity: high
affects: examples/14a-core/src/relationships.rs:62-70
---

# Eager Loading `.with("posts")` Does Not Populate Struct Field

## Description

`relationships.rs` calls `.with("posts")` on the query builder and then attempts to
access `u.posts` as a `Vec`-like field on the returned `User` structs. Two problems:

1. `User` has no `posts` field — accessing `u.posts` does not compile.
2. `.with("posts")` records the eager-load intent on the builder, but the current
   implementation does **not** perform an automatic N+1-preventing batch load and
   attach the results to the parent struct. Full eager-load materialization is not
   yet implemented.

## Current (broken) Code

```rust
let users = User::query()
    .with("posts")       // stores intent, does not batch-load
    .limit(5)
    .get(pool)           // also broken — see bug-003
    .await?;

for u in &users {
    println!("   {} has {} posts", u.name, u.posts.len()); // ❌ no `posts` field
}
```

## Compiler Error

```
error[E0609]: no field `posts` on type `&relationships::User`
```

## Fix

Until full eager-loading materialization is implemented, load relations manually
using the `HasMany` relation object or `executor::postgres::fetch_all`:

```rust
use rok_orm::{PgModel, executor::postgres};

let users = User::get_where(pool, User::query().limit(5)).await?;

for u in &users {
    // Manually load related posts
    let posts = Post::get_where(
        pool,
        Post::query().where_eq("user_id", u.id),
    ).await?;
    println!("   {} has {} posts", u.name, posts.len());
}
```

Or use a `HasMany` relation built via `#[derive(Relations)]` (see bug-002):

```rust
let rel = u.posts();   // returns HasMany<User, Post>
let posts = postgres::fetch_all(pool, rel.query_for(SqlValue::Integer(u.id))).await?;
```

## Note

Adding full eager-loading materialization (batch load + attach to parent) is a
planned feature for a future phase.
