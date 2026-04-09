---
id: bug-002
type: bug
severity: critical
affects: examples/14a-core/src/relationships.rs
---

# Wrong `#[derive(Relations)]` Syntax

## Description

`relationships.rs` uses an invalid attribute syntax for declaring relations. It
creates separate "relation structs" (`UserRelations`, `PostRelations`) and tries to
attach relation attributes at the field level using `#[has_many(...)]` and
`#[belongs_to(...)]`. Neither `#[has_many]` nor `#[belongs_to]` are valid proc-macro
attributes; they do not exist.

## Current (broken) Code

```rust
#[derive(rok_orm::Relations)]
pub struct UserRelations {
    #[has_many(target = "Post")]      // ❌ attribute does not exist
    pub posts: HasMany<User, Post>,
}

#[derive(rok_orm::Relations)]
pub struct PostRelations {
    #[belongs_to(target = "User")]   // ❌ attribute does not exist
    pub user: BelongsTo<Post, User>,
}
```

## Compiler Error

```
error: cannot find attribute `belongs_to` in this scope
  --> src/relationships.rs:36:7
   |
36 |     #[belongs_to(target = "User")]
   |       ^^^^^^^^^^

error[E0277]: the trait bound `UserRelations: Model` is not satisfied
```

## Fix

Relations must be declared on the **main model struct** using `#[model(has_many = Type)]`
/ `#[model(belongs_to = Type)]` inside `#[model(...)]`, and the struct must also
`#[derive(Relations)]`. No separate "relation struct" is needed.

```rust
use rok_orm::{Model, Relations};

#[derive(Model, Relations, sqlx::FromRow, Debug, Clone)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
    #[model(has_many = Post)]
    posts: (),          // phantom field — only the attribute matters
}

#[derive(Model, Relations, sqlx::FromRow, Debug, Clone)]
#[model(table = "posts")]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub user_id: i64,
    #[model(belongs_to = User)]
    user: (),
}
```

Then use:

```rust
let user_rel = user_instance.posts(); // returns HasMany<User, Post>
```
