---
id: bug-003
type: bug
severity: critical
affects: crud_operations.rs, soft_deletes.rs, timestamps.rs, aggregations.rs,
         transactions.rs, query_scopes.rs, query_logging.rs, relationships.rs
---

# QueryBuilder Missing Fluent Executor Methods (.get, .count, .first)

## Description

All examples assume that `QueryBuilder<T>` has fluent async executor methods that
accept a pool and run the query directly:

```rust
User::query().filter("active", true).get(pool).await?
User::query().filter("active", true).count(pool).await?
User::query().filter("email", "x").first(pool).await?
```

None of these methods exist on `QueryBuilder`. The builder is **query-construction
only**; execution requires calling `PgModel` trait methods or the `executor::postgres`
free functions separately.

## Affected Patterns

| Example call | Status |
|---|---|
| `.get(pool).await?` | does not exist |
| `.count(pool).await?` | does not exist |
| `.first(pool).await?` | does not exist |
| `.sum(pool).await?` | does not exist |

## Compiler Error

```
error[E0599]: no method named `get` found for struct `QueryBuilder<User>`
error[E0599]: no method named `count` found for struct `QueryBuilder<User>`
error[E0599]: no method named `first` found for struct `QueryBuilder<User>`
```

## Current Correct API

```rust
// Instead of: User::query().filter("active", true).get(pool).await?
let users = User::get_where(pool, User::query().filter("active", true)).await?;

// Instead of: User::query().filter("active", true).count(pool).await?
let n = User::count_where(pool, User::query().filter("active", true)).await?;

// Instead of: User::query().filter("email", "x").first(pool).await?
let u = User::first(pool).await?;   // first row of the table
// or fetch optional with a filter:
let u = postgres::fetch_optional(pool, User::query().filter("email", "x")).await?;

// Or use the executor directly:
use rok_orm::executor::postgres;
let users = postgres::fetch_all(pool, User::query().filter(...)).await?;
```

## Recommended Fix — Option A: Update Examples

Change all fluent executor calls to use `PgModel` / `executor::postgres` APIs.

## Recommended Fix — Option B: Add Fluent Methods to QueryBuilder (API Enhancement)

Add feature-gated methods on `QueryBuilder<T>` that accept a pool and execute:

```rust
// In QueryBuilder<T> impl, gated on feature = "postgres"
pub async fn get(self, pool: &sqlx::PgPool) -> Result<Vec<T>, sqlx::Error>
where T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin
{
    crate::executor::postgres::fetch_all(pool, self).await
}

pub async fn first(self, pool: &sqlx::PgPool) -> Result<Option<T>, sqlx::Error>
where T: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin
{
    crate::executor::postgres::fetch_optional(pool, self.limit(1)).await
}

pub async fn count(self, pool: &sqlx::PgPool) -> Result<i64, sqlx::Error>
{
    crate::executor::postgres::count(pool, self).await
}
```

Option B is the ergonomically correct fix and would make the examples work as written.
