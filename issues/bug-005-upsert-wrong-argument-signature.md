---
id: bug-005
type: bug
severity: high
affects: examples/14a-core/src/crud_operations.rs:61-70
---

# `upsert()` Called with Wrong / Incomplete Arguments

## Description

`crud_operations.rs` calls `User::upsert()` twice with incorrect signatures.

`PgModelExt::upsert` requires four arguments:
```rust
async fn upsert(pool, data, conflict_column, update_columns) -> Result<u64>
```

The first call passes only `pool` and `data`, omitting the required
`conflict_column` and `update_columns` parameters entirely.

## Current (broken) Code

```rust
// First call — missing conflict_column and update_columns
User::upsert(pool, &[
    ("email", "admin@example.com".into()),
    ("name", "Admin".into()),
]).await?;                                     // ❌ wrong arg count

// Second call — correct arg count but structured weirdly
User::upsert(pool, &[
    ("email", "admin@example.com".into()),
    ("name", "Admin Updated".into()),
], "email", &["name"]).await?;
```

## Fix

Both calls need the full four-argument form:

```rust
// INSERT ... ON CONFLICT (email) DO UPDATE SET name = excluded.name
User::upsert(
    pool,
    &[("email", "admin@example.com".into()), ("name", "Admin".into())],
    "email",       // conflict column
    &["name"],     // columns to update on conflict
).await?;

User::upsert(
    pool,
    &[("email", "admin@example.com".into()), ("name", "Admin Updated".into())],
    "email",
    &["name"],
).await?;
```
