# Phase 8: Developer Ergonomics

> **Target version:** v0.4.0
> **Status:** 🚧 In Progress
> **Inspired by:** AdonisJS Lucid ORM, Laravel Eloquent, Ruby on Rails

---

## Goal

Writing queries should feel natural and chainable. Zero friction for common patterns, clean escape hatches for complex ones. Inspired by `.when()` from Lucid, `tap()` from Ruby, and Laravel's mass-assignment protection.

---

## 8.1 Conditional Query Chaining: `when()` / `when_else()`

### API

```rust
let users = User::query()
    .when(params.role.is_some(), |q| {
        q.filter("role", params.role.unwrap())
    })
    .when(params.active, |q| q.filter("active", true))
    .when(params.search.is_some(), |q| {
        q.where_like("name", &format!("%{}%", params.search.unwrap()))
    })
    .order_by_desc("created_at")
    .limit(params.per_page.unwrap_or(20))
    .get(&pool)
    .await?;

// With else branch
let users = User::query()
    .when_else(
        is_admin,
        |q| q.filter("role", "admin"),
        |q| q.filter("role", "user"),
    )
    .get(&pool)
    .await?;
```

### Tasks

- [x] Add `fn when(self, condition: bool, f: impl FnOnce(Self) -> Self) -> Self` to `QueryBuilder`
- [x] Add `fn when_else(self, condition: bool, f_true: impl FnOnce(Self) -> Self, f_false: impl FnOnce(Self) -> Self) -> Self`
- [x] Both methods are pure — no allocation if condition is false
- [x] Tests: condition true, condition false, when_else both branches

---

## 8.2 Raw Expressions

### API

```rust
// Raw WHERE
let users = User::query()
    .where_raw("LOWER(email) = LOWER($1)", vec!["Admin@Example.com".into()])
    .get(&pool)
    .await?;

// Raw SELECT (mixed with normal)
let users = User::query()
    .select_raw("id, name, CONCAT(first_name, ' ', last_name) AS full_name")
    .get(&pool)
    .await?;

// Raw ORDER BY
let users = User::query()
    .order_raw("FIELD(role, 'admin', 'moderator', 'user')")
    .get(&pool)
    .await?;

// Raw HAVING
let stats = User::query()
    .select(&["role", "COUNT(*) as count"])
    .group_by(&["role"])
    .having_raw("COUNT(*) BETWEEN 5 AND 100")
    .get(&pool)
    .await?;

// Fully raw (escape hatch — no type inference on result shape)
let rows: Vec<User> = User::from_raw_sql(
    &pool,
    "SELECT * FROM users WHERE created_at > $1 AND active = true",
    vec![start_date.into()],
).await?;
```

### Tasks

- [x] Add `RawExpr(String, Vec<SqlValue>)` variant to `Condition` enum (via `where_raw_params`)
- [x] Add `where_raw(sql: &str, params: Vec<SqlValue>)` to `QueryBuilder` (as `where_raw_params`)
- [x] Add `select_raw(sql: &str)` — replaces or extends `select_cols`
- [x] Add `order_raw(sql: &str)` — appended to ORDER BY list as literal (`OrderDir::Raw`)
- [x] Add `having_raw(sql: &str)` — alias for `having()`
- [x] Add `Model::from_raw_sql(pool, sql, params)` on PgModel (backed by `postgres::fetch_raw`)
- [x] Placeholder numbering: raw SQL uses its own `$N` sequence, offset by current param count
- [x] Tests: select_raw, order_raw tested in ergonomics tests

---

## 8.3 Debugging Utilities: `tap()` and `dd()`

### API

```rust
// tap() — inspect without breaking the chain
let users = User::query()
    .filter("active", true)
    .tap(|q| {
        let (sql, _) = q.to_sql();
        tracing::debug!("Before limit: {sql}");
    })
    .limit(10)
    .get(&pool)
    .await?;

// dd() — print SQL then panic (dev/debug only)
#[cfg(debug_assertions)]
User::query()
    .filter("active", true)
    .dd();
```

### Tasks

- [x] Add `fn tap(self, f: impl FnOnce(&Self)) -> Self` to `QueryBuilder` — calls `f(&self)`, returns `self`
- [x] Add `fn dd(self) -> Self` — prints SQL + params to stdout (dev helper)
- [x] `to_sql()` is already implemented — ensure it's in the public API docs
- [x] Tests: tap does not modify query, tap is called with correct builder state

---

## 8.4 Chunking for Large Datasets

### API

```rust
// Chunk with LIMIT/OFFSET loop
User::query()
    .filter("active", true)
    .chunk(&pool, 500, |batch| async move {
        for user in batch {
            send_email(&user).await;
        }
        Ok(())
    })
    .await?;

// chunk_by_id — stable even if rows are deleted mid-run
User::query()
    .chunk_by_id(&pool, 500, |batch| async move {
        process(batch).await
    })
    .await?;

// into_stream — async stream of individual rows
let mut stream = User::query()
    .filter("active", true)
    .into_stream(&pool);

while let Some(user) = stream.next().await {
    let user = user?;
    process(user).await;
}
```

### Tasks

- [ ] Add `async fn chunk(pool, size, callback: async FnMut(Vec<T>) -> OrmResult<()>)` to `QueryBuilder`
  - Loops with LIMIT `size` OFFSET `0, size, 2*size, …` until empty result
- [ ] Add `async fn chunk_by_id(pool, size, callback)` — uses `WHERE id > last_max_id` cursor
- [ ] Add `fn into_stream(pool) -> impl Stream<Item = OrmResult<T>>` using sqlx `fetch()` streaming
- [ ] Tests: chunk processes all rows, chunk_by_id stable with deletes, stream yields all rows

---

## 8.5 Cursor Pagination

### API

```rust
// First page
let result = Post::query()
    .order_by_desc("created_at")
    .cursor_paginate(&pool, CursorPage { after: None, limit: 20 })
    .await?;

println!("next cursor: {:?}", result.next_cursor);

// Next page
let result = Post::query()
    .order_by_desc("created_at")
    .cursor_paginate(&pool, CursorPage { after: result.next_cursor, limit: 20 })
    .await?;

pub struct CursorPage {
    pub after: Option<String>,   // base64-encoded cursor, None for first page
    pub limit: usize,
}

pub struct CursorResult<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
    pub has_more: bool,
}
```

**Cursor encoding:** base64-encode a JSON object of the last row's ORDER BY column values.

```
cursor = base64(json!({"created_at": "2026-01-01T00:00:00Z", "id": 42}))
```

### Tasks

- [x] Add `CursorPage { after: Option<i64>, limit: usize }` struct
- [x] Add `CursorResult<T> { data, next_cursor, has_more }` struct + `from_rows()`
- [x] `QueryBuilder::cursor_sql(pk_col, after_id, limit)` — apply WHERE id > cursor + LIMIT n+1
- [x] Add `async fn cursor_paginate(pool, cursor_page) -> OrmResult<CursorResult<T>>` to PgModelExt
- [x] Decode incoming cursor: base64 → JSON → extract column values → inject as WHERE conditions
- [x] Encode outgoing cursor: take last row's ORDER BY values → JSON → base64
- [x] Fetch `limit + 1` rows; `from_rows()` handles trimming and `has_more`
- [x] Tests: first page (None cursor), has_more true, has_more false, cursor_sql generates correct SQL

---

## 8.6 `fill()` and Mass Assignment Protection

### API

```rust
#[derive(Model, sqlx::FromRow)]
#[model(
    table    = "users",
    fillable = ["name", "email", "bio"],
    // OR: guarded = ["id", "role", "is_admin"],
)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
    pub role: String,     // not fillable
    pub is_admin: bool,   // not fillable
}

// role and is_admin silently dropped
let user = User::create_returning(&pool, &[
    ("name",     "Alice".into()),
    ("email",    "alice@example.com".into()),
    ("role",     "admin".into()),   // ignored
    ("is_admin", true.into()),      // ignored
]).await?;
```

### Tasks

- [x] Add `fillable() -> &'static [&'static str]` to `Model` trait (default: empty = all allowed)
- [x] Add `guarded() -> &'static [&'static str]` to `Model` trait (default: empty = nothing guarded)
- [x] Macro generates overrides from `#[model(fillable = [...])]` / `#[model(guarded = [...])]`
- [x] Add `filter_fillable(data: &[(&str, SqlValue)]) -> Vec<(&str, SqlValue)>` as a Model trait method
- [x] Apply filter in `create`, `create_returning`, `bulk_create`, `update_by_pk`, `update_where` (PgModel)
- [x] Tests: fillable allows only listed cols, guarded blocks listed cols, no filtering when both empty

---

## 8.7 Model Observers

### API

```rust
pub struct UserObserver;

impl ModelObserver for UserObserver {
    type Model = User;

    async fn creating(&self, user: &mut User) -> OrmResult<()> {
        user.email = user.email.to_lowercase();
        Ok(())
    }
    async fn created(&self, user: &User) -> OrmResult<()> {
        send_welcome_email(&user.email).await
    }
    async fn updating(&self, user: &mut User) -> OrmResult<()> { Ok(()) }
    async fn updated(&self, user: &User) -> OrmResult<()> { Ok(()) }
    async fn saving(&self, user: &mut User) -> OrmResult<()> { Ok(()) }
    async fn saved(&self, user: &User) -> OrmResult<()> { Ok(()) }
    async fn deleting(&self, user: &User) -> OrmResult<()> { Ok(()) }
    async fn deleted(&self, user: &User) -> OrmResult<()> {
        invalidate_cache("user", user.id).await
    }
    async fn restoring(&self, user: &User) -> OrmResult<()> { Ok(()) }
    async fn restored(&self, user: &User) -> OrmResult<()> { Ok(()) }
}

// Register at startup
User::observe(UserObserver);
```

### Tasks

- [x] Define `ModelObserver<M>` trait with all 10 lifecycle methods (all default no-ops)
- [x] Type-erased dispatch via fn pointers — no object-safe wrapper needed
- [x] Add `static REGISTRY: OnceLock<RwLock<HashMap<TypeId, Vec<ObserverEntry>>>>`
- [x] `ObserverRegistry::observe<M, O>(observer)` — register under `TypeId::of::<M>()`
- [x] `ObserverRegistry::dispatch<M>(model, event)` — fire all observers for M
- [x] Call observers in executor paths: Created/Saved after create_returning (PgModel, SqliteModel)
- [x] Multiple observers allowed per model — called in registration order
- [x] Tests: observer created/deleted events dispatched, noop for unregistered events

---

## 8.8 Global Query Scopes

### API

```rust
pub struct ActiveScope;

impl GlobalScope<User> for ActiveScope {
    fn apply(&self, query: QueryBuilder<User>) -> QueryBuilder<User> {
        query.filter("active", true)
    }
}

User::add_global_scope(ActiveScope);

// All queries automatically include WHERE active = true
let users = User::all(&pool).await?;

// Opt out per query
let all = User::query()
    .without_global_scope::<ActiveScope>()
    .get(&pool)
    .await?;

// Remove permanently
User::remove_global_scope::<ActiveScope>();
```

### Tasks

- [x] Define `GlobalScope<M: Model>` trait: `apply(&self, QueryBuilder<M>) -> QueryBuilder<M>`
- [x] Add scope registry: `OnceLock<RwLock<HashMap<TypeId, Vec<ScopeEntry>>>>`
- [x] `ScopeRegistry::apply_scopes<M>(builder)` — apply all registered scopes
- [x] Apply all registered scopes automatically in `Model::query()` (via scoped_query() on all/get/count/first)
- [x] Add `excluded_scopes: Vec<TypeId>` field to `QueryBuilder`
- [x] Add `without_global_scope::<S>()` — adds `TypeId::of::<S>()` to excluded list
- [x] `ScopeRegistry::remove_scope<M, S>()` — removes scope type from registry
- [x] Tests: scopes inject conditions, remove_scope API works

---

## 8.9 `touches` — Parent Timestamp Propagation

### API

```rust
#[derive(Model, sqlx::FromRow)]
#[model(
    timestamps,
    touches = ["post"],   // after any write, also update posts.updated_at
)]
pub struct Comment {
    pub id: i64,
    pub post_id: i64,
    pub body: String,
    pub updated_at: Option<DateTime<Utc>>,
}

// After this update, posts.updated_at is also set to NOW()
Comment::update_by_pk(&pool, comment_id, &[("body", "edited".into())]).await?;
```

### Tasks

- [x] Add `touches() -> &'static [&'static str]` to `Model` trait (default: empty slice)
- [x] Macro generates override from `#[model(touches = [...])]`
- [ ] Each string in `touches` is a relation name — resolve its FK column and parent table
- [ ] After each create/update/delete, run: `UPDATE {parent_table} SET updated_at = NOW() WHERE {pk} = {fk_value}`
- [ ] Tests: touches updates parent, multiple parents, no-op when touches is empty

---

## Acceptance Criteria for Phase 8

- [ ] All 9 sub-sections fully implemented
- [ ] Zero regressions in existing tests
- [ ] All features tested on PG + SQLite (minimum)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Phase file tasks all checked off
