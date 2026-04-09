---
id: bug-004
type: bug
severity: high
affects: examples/14a-core/src/crud_operations.rs:41-42
         examples/14a-core/src/timestamps.rs:44-46
---

# `find_by_pk` Returns `Option<T>` but Is Used as `T`

## Description

`PgModel::find_by_pk` returns `Result<Option<T>, sqlx::Error>`. After `?`-unwrapping
the `Result`, the value is still `Option<T>`. Both example files use the result
directly as a `T`, attempting to access `.name`, `.id`, `.updated_at` fields on an
`Option`, which will not compile.

## Affected Code

**crud_operations.rs:41-42**
```rust
let user = User::find_by_pk(pool, bob.id).await?;
// user is Option<User>, not User
println!("   ✅ Found: {} (id={})", user.name, user.id);  // ❌ field on Option
```

**timestamps.rs:44-46**
```rust
let updated = Article::find_by_pk(pool, id).await?;
// updated is Option<Article>
println!("   ✅ updated_at changed to: {}",
    updated.updated_at.unwrap()...);  // ❌ field on Option
```

## Fix

Use `find_or_404` if the record is expected to always exist (returns `T` or
`sqlx::Error::RowNotFound`), or explicitly handle the `Option`:

```rust
// Option A — use find_or_404 (returns T directly or errors)
let user = User::find_or_404(pool, bob.id).await?;
println!("Found: {} (id={})", user.name, user.id);

// Option B — handle Option explicitly
let user = User::find_by_pk(pool, bob.id).await?
    .ok_or(sqlx::Error::RowNotFound)?;
println!("Found: {} (id={})", user.name, user.id);
```
