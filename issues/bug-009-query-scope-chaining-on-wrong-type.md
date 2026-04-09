---
id: bug-009
type: bug
severity: medium
affects: examples/14a-core/src/query_scopes.rs:71
---

# Query Scope Method Chained on `QueryBuilder` Instead of `User`

## Description

`query_scopes.rs:71` chains `.role("user")` on the result of `User::active()`, which
returns a `QueryBuilder<User>`. The `role()` method is defined on `User` (as an
associated function returning a new `QueryBuilder`), not on `QueryBuilder<User>`.
Calling it on a `QueryBuilder` fails.

## Current (broken) Code

```rust
impl User {
    pub fn active() -> QueryBuilder<User> { ... }
    pub fn role(scope: &str) -> QueryBuilder<User> { ... }  // on User, not QueryBuilder
}

// Usage:
let active_users = User::active().role("user").get(pool).await?;
//                               ^^^^^^^^^^^^ does not exist on QueryBuilder<User>
```

## Fix

Chain conditions using `QueryBuilder` methods (`.filter()`) instead of calling `User`
scope functions on the builder:

```rust
// Option A — compose using QueryBuilder::filter
let active_users = User::active()
    .filter("role", "user")
    .get(pool)              // also needs bug-003 fixed
    .await?;

// Option B — build the intersection manually
let qb = User::query()
    .filter("active", true)
    .filter("role", "user");
let active_users = User::get_where(pool, qb).await?;
```

If scope composition is a desired API, the scope functions should return
`QueryBuilder<User>` and be implemented as extension methods on `QueryBuilder`, or
the builder should expose a `.pipe(|qb| ...)` combinator.
