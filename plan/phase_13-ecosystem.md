# Phase 13: Ecosystem Additions

> **Target version:** v1.0.0
> **Status:** 🔜 Planned
> **Note:** All three sub-sections are independent — can be worked in parallel

---

## Goal

Stable v1.0 integrations: MSSQL dialect, transparent Redis caching, and first-class Axum/Actix-web support.

---

## 13.1 MSSQL / SQL Server Support

### Feature Flag

```toml
rok-orm = { version = "1.0", features = ["mssql"] }
```

### API

```rust
use rok_orm::MssqlModel;
use sqlx::MssqlPool;

#[derive(Model, sqlx::FromRow)]
#[model(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
}

let pool = MssqlPool::connect(&url).await?;
let users = User::all(&pool).await?;
let user  = User::find_or_404(&pool, 1i64).await?;
User::create(&pool, &[("name", "Alice".into())]).await?;
```

### T-SQL Specifics

| Feature | T-SQL syntax |
|---------|-------------|
| LIMIT N | `SELECT TOP N ...` |
| Placeholders | `@P1, @P2, …` |
| OFFSET / pagination | `OFFSET x ROWS FETCH NEXT y ROWS ONLY` |
| RETURNING | Not supported — use `OUTPUT INSERTED.*` |
| Upsert | `MERGE INTO ... USING ...` |
| Auto-increment | `IDENTITY(1,1)` |

### Tasks

- [ ] Add `Dialect::Mssql` variant to `Dialect` enum
- [ ] Implement T-SQL SQL generation in `QueryBuilder::to_sql_with_dialect(Mssql)`:
  - Replace `LIMIT N` → `SELECT TOP N`
  - Replace `$1, $2` → `@P1, @P2`
  - Implement `OFFSET x ROWS FETCH NEXT y ROWS ONLY` for pagination
- [ ] Add `src/executor/mssql.rs` with `fetch_all`, `fetch_optional`, `execute`, `fetch_one`
- [ ] Add `MssqlModel` trait in `src/model/mssql_model.rs` mirroring `PgModel`
- [ ] Add `OUTPUT INSERTED.*` for `create_returning`
- [ ] Add MSSQL `MERGE` upsert implementation
- [ ] Add `sqlx` mssql feature dependency
- [ ] Tests: CRUD, pagination, upsert on MSSQL (requires MSSQL Docker in CI)

---

## 13.2 Redis Cache Integration

### Feature Flag

```toml
rok-orm = { version = "1.0", features = ["postgres", "redis"] }
```

### API

```rust
use rok_orm::cache::RedisCache;

// Model-level cache config
#[derive(Model, sqlx::FromRow)]
#[model(table = "users", cache(ttl = 300, key_prefix = "users"))]
pub struct User {
    pub id: i64,
    pub name: String,
}

// Register cache driver at startup
rok_orm::cache::set_driver(RedisCache::new("redis://127.0.0.1:6379").await?);

// Find — checks Redis first, falls back to DB, writes result to Redis
let user = User::find_cached(&pool, 1i64).await?;
// Cache key: "users:1"

// Invalidate cached record
User::invalidate_cache(1i64).await?;

// Cache a whole query result
let users = User::query()
    .filter("active", true)
    .cache_as("active_users", 60)   // key = "active_users", ttl = 60s
    .get(&pool)
    .await?;

// Invalidate all cache keys for this model
User::flush_cache().await?;
```

### `CacheDriver` Trait

```rust
pub trait CacheDriver: Send + Sync {
    async fn get(&self, key: &str) -> OrmResult<Option<String>>;
    async fn set(&self, key: &str, value: &str, ttl_seconds: u64) -> OrmResult<()>;
    async fn del(&self, key: &str) -> OrmResult<()>;
    async fn del_pattern(&self, pattern: &str) -> OrmResult<()>;  // for flush_cache
}
```

### Tasks

- [ ] Define `CacheDriver` trait
- [ ] Implement `RedisCache` using `redis` crate (async, connection pool)
- [ ] Add `static CACHE_DRIVER: OnceLock<Box<dyn CacheDriver>>`
- [ ] Add `cache::set_driver(driver)` registration function
- [ ] Add `#[model(cache(...))]` macro attribute (parses `ttl` and `key_prefix`)
- [ ] Add `Model::find_cached(pool, id)` — get → deserialize or DB + cache set
- [ ] Add `Model::invalidate_cache(id)` — del from cache
- [ ] Add `Model::flush_cache()` — del_pattern on `{key_prefix}:*`
- [ ] Add `cache_as(key, ttl)` to `QueryBuilder` — caches the entire `Vec<T>` result
- [ ] Tests: cache hit, cache miss (DB fallback + write), invalidate, flush, query cache

---

## 13.3 Axum & Actix-Web Integration

### Feature Flags

```toml
rok-orm = { version = "1.0", features = ["postgres", "axum"] }
rok-orm = { version = "1.0", features = ["postgres", "actix"] }
```

### Axum API

```rust
use axum::{extract::{Path, State}, Json, Router};
use rok_orm::axum::{DbPool, OrmErrorResponse};
use sqlx::PgPool;

// Application state
#[derive(Clone)]
struct AppState { db: PgPool }

// DbPool extractor — pulls pool from state
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<User>, OrmErrorResponse> {
    let user = User::find_or_404(&state.db, id).await?;
    Ok(Json(user))
}

// OrmError → HTTP response
// OrmError::NotFound    → 404 JSON { "error": "User not found" }
// OrmError::Validation  → 422 JSON { "error": "...", "field": "..." }
// OrmError::Constraint  → 409 JSON { "error": "Constraint violation" }
// OrmError::Database    → 500 JSON { "error": "Internal server error" }

async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<User>, OrmErrorResponse> {
    let user = User::create_returning(&state.db, &[
        ("name",  body.name.into()),
        ("email", body.email.into()),
    ]).await?;
    Ok(Json(user))
}

// Router setup
let app = Router::new()
    .route("/users/:id", get(get_user))
    .route("/users",     post(create_user))
    .with_state(AppState { db: pool });
```

### Actix-Web API

```rust
use actix_web::{web, HttpResponse, Error};
use rok_orm::actix::{OrmError, DbPool};

async fn get_user(
    pool: web::Data<PgPool>,
    id: web::Path<i64>,
) -> Result<HttpResponse, OrmError> {
    let user = User::find_or_404(&pool, *id).await?;
    Ok(HttpResponse::Ok().json(user))
}
```

### Tasks

**Axum:**
- [ ] Add `rok_orm::axum` module behind `axum` feature
- [ ] Define `OrmErrorResponse` — implements `axum::response::IntoResponse`
- [ ] Map `OrmError` variants to HTTP status + JSON body
- [ ] Add `From<OrmError> for OrmErrorResponse`
- [ ] Provide typed response body `ErrorBody { error: String, field: Option<String> }`
- [ ] Tests: each OrmError variant → correct HTTP status code

**Actix-web:**
- [ ] Add `rok_orm::actix` module behind `actix` feature
- [ ] Define `OrmError` newtype implementing `actix_web::ResponseError`
- [ ] Map same `OrmError` variants to status codes
- [ ] Tests: same coverage as Axum

---

## Acceptance Criteria for Phase 13

- [ ] MSSQL: CRUD, pagination, upsert work correctly (tested via CI with Docker MSSQL image)
- [ ] Redis: cache hit/miss, invalidate, flush, query cache all verified
- [ ] Axum: all OrmError variants return correct HTTP status + JSON body
- [ ] Actix: same coverage as Axum
- [ ] No breaking changes to existing public API
- [ ] `cargo clippy -- -D warnings` clean for all feature combinations
- [ ] Phase file tasks all checked off
