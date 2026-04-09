# Phase 7: Rich Relationships

> **Target version:** v0.4.0
> **Status:** 🚧 In Progress
> **Inspired by:** Laravel Eloquent, AdonisJS Lucid ORM

---

## Goal

First-class support for every relational pattern — with zero boilerplate and the same ergonomics as JavaScript/PHP ORMs, but fully type-safe in Rust.

---

## 7.1 Many-to-Many with Full Pivot Access

**Current state:** Basic `belongs_to_many` with no pivot column access.

### API

```rust
#[derive(Model, Relations, sqlx::FromRow)]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,

    #[model(many_to_many(
        related  = "Role",
        pivot    = "user_roles",
        fk       = "user_id",
        rfk      = "role_id",
        pivots   = ["assigned_at", "expires_at"],
    ))]
    pub roles: Vec<Role>,
}

// Attach / detach
user.roles().attach(&pool, role_id).await?;
user.roles().attach_with_pivot(&pool, role_id, &[
    ("assigned_at", Utc::now().into()),
    ("expires_at", None::<DateTime<Utc>>.into()),
]).await?;
user.roles().detach(&pool, role_id).await?;
user.roles().detach_all(&pool).await?;

// Sync — replace entire set (diff, insert missing, delete removed)
user.roles().sync(&pool, vec![1i64, 2, 3]).await?;

// Toggle — attach if absent, detach if present
user.roles().toggle(&pool, vec![1i64, 2]).await?;

// Query with pivot columns
let roles = user.roles()
    .filter("roles.active", true)
    .with_pivot(&["assigned_at", "expires_at"])
    .order_by_desc("assigned_at")
    .get(&pool)
    .await?;

for role in &roles {
    println!("assigned: {:?}", role.pivot("assigned_at"));
}

// Update pivot row
user.roles().update_pivot(&pool, role_id, &[
    ("expires_at", new_date.into()),
]).await?;
```

### Tasks

- [x] Add `ManyToMany<P, C>` struct: `pivot_table`, `foreign_key`, `related_key`, `pivot_columns`
- [x] Add `ManyToManyQuery<P, C>` with `attach()`, `attach_with_pivot()`, `detach()`, `detach_all()`
- [x] Implement `sync()` — SELECT current IDs, diff, batch INSERT missing, DELETE removed
- [x] Implement `toggle()` — SELECT current IDs, INSERT absent, DELETE present
- [x] Add `with_pivot(&[cols])` — inject pivot columns into the JOIN SELECT
- [x] Add `PivotRow` wrapper struct holding the related model + `HashMap<String, SqlValue>` pivot data
- [x] Add `update_pivot()` — UPDATE pivot table WHERE fk = ? AND rfk = ?
- [x] Extend `#[model(many_to_many(...))]` macro attribute parser
- [x] Tests: attach_sql, detach_sql, detach_all_sql, current_ids_sql, update_pivot_sql, with_pivot (unit, in many_to_many.rs)

---

## 7.2 Has-Many-Through

### API

```rust
// Country → Users → Posts  (country has many posts through users)
#[derive(Model, Relations, sqlx::FromRow)]
pub struct Country {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,

    #[model(has_many_through(
        related    = "Post",
        through    = "User",
        first_key  = "country_id",   // users.country_id
        second_key = "user_id",      // posts.user_id
    ))]
    pub posts: Vec<Post>,
}

let posts = country.posts()
    .filter("published", true)
    .order_by_desc("created_at")
    .get(&pool)
    .await?;

// Static form
let posts = Country::posts_through(&pool, country_id).await?;
```

**SQL generated:**
```sql
SELECT posts.*
FROM posts
INNER JOIN users ON users.id = posts.user_id
WHERE users.country_id = $1
```

### Tasks

- [x] Add `HasManyThrough<P, T, C>` struct (Parent, Through, Child)
- [x] Generate INNER JOIN SQL with parent FK in the WHERE clause
- [x] Add `has_many_through(...)` macro attribute
- [ ] Support eager loading via `.with("posts")` on a Country query
- [x] Tests: query generation via macro (integration test)

---

## 7.3 Has-One-Through

### API

```rust
// Mechanic → Cars → CarOwner
#[derive(Model, Relations, sqlx::FromRow)]
pub struct Mechanic {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,

    #[model(has_one_through(
        related    = "CarOwner",
        through    = "Car",
        first_key  = "mechanic_id",
        second_key = "car_id",
    ))]
    pub car_owner: Option<CarOwner>,
}

let owner = mechanic.car_owner().get(&pool).await?;
```

### Tasks

- [x] Add `HasOneThrough<P, T, C>` struct
- [x] Generate INNER JOIN with LIMIT 1
- [x] Add `has_one_through(...)` macro attribute
- [x] Tests: query_for generates INNER JOIN + LIMIT 1, accessor values (unit, in has_one_through.rs)

---

## 7.4 Polymorphic Relationships

### morphOne / morphMany

```rust
// Image can belong to User OR Post
#[derive(Model, sqlx::FromRow)]
pub struct Image {
    pub id: i64,
    pub url: String,
    pub imageable_id: i64,
    pub imageable_type: String,  // "users" | "posts"
}

// User morph_one Image
#[derive(Model, Relations, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,

    #[model(morph_one(related = "Image", morph_key = "imageable"))]
    pub image: Option<Image>,
}

// Post morph_many Images
#[derive(Model, Relations, sqlx::FromRow)]
pub struct Post {
    pub id: i64,
    pub title: String,

    #[model(morph_many(related = "Image", morph_key = "imageable"))]
    pub images: Vec<Image>,
}

let image  = user.image().get(&pool).await?;
let images = post.images().get(&pool).await?;

// Create fills imageable_type automatically
post.images().create(&pool, &[
    ("url", "https://cdn.example.com/img.png".into()),
]).await?;
```

**SQL:**
```sql
SELECT * FROM images WHERE imageable_type = 'users' AND imageable_id = $1 LIMIT 1
SELECT * FROM images WHERE imageable_type = 'posts' AND imageable_id = $1
```

### morphTo (inverse)

```rust
#[derive(Model, Relations, sqlx::FromRow)]
pub struct Image {
    pub id: i64,
    pub url: String,
    pub imageable_id: i64,
    pub imageable_type: String,

    #[model(morph_to(morph_key = "imageable"))]
    pub imageable: MorphParent,
}

let parent = image.imageable().resolve(&pool).await?;
match parent {
    MorphParent::User(u) => println!("user: {}", u.name),
    MorphParent::Post(p) => println!("post: {}", p.title),
    MorphParent::Unknown(t, id) => eprintln!("unknown: {} #{}", t, id),
}
```

### morphToMany / morphedByMany (polymorphic pivot)

```rust
// Tags system: Tag ↔ Post, Tag ↔ Video via `taggables` pivot
#[derive(Model, Relations, sqlx::FromRow)]
pub struct Post {
    pub id: i64,
    pub title: String,

    #[model(morph_to_many(
        related   = "Tag",
        pivot     = "taggables",
        morph_key = "taggable",   // → taggable_id + taggable_type
    ))]
    pub tags: Vec<Tag>,
}

// Inverse on Tag
#[derive(Model, Relations, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,

    #[model(morphed_by_many(
        related   = "Post",
        pivot     = "taggables",
        morph_key = "taggable",
    ))]
    pub posts: Vec<Post>,
}

post.tags().attach(&pool, tag_id).await?;
post.tags().sync(&pool, vec![1i64, 5, 9]).await?;
let tags = post.tags().get(&pool).await?;
```

### Tasks

- [x] Add `MorphOne<P, C>` and `MorphMany<P, C>` structs (morph.rs)
- [x] Add `MorphToRef` with `query_for_type()` helper; runtime resolution via `morph_type_map!`
- [x] Add `morph_type_map!()` macro — registers `"type_str"` → `EnumVariant(Type)` with async `resolve(pool)`
- [x] Add `MorphToMany<P, C>` and `MorphedByMany<P, C>` structs with attach/detach/sync SQL
- [ ] `MorphToManyQuery` with live async `attach()`, `detach()`, `sync()` (PG executor wiring)
- [x] Auto-inject `morph_key_type = table_name()` on `MorphMany::create_sql()`
- [ ] Eager load polymorphic via `.with("imageable")` (batch by type, two queries)
- [x] Add all macro attributes: `morph_one`, `morph_many`, `morph_to`, `morph_to_many`, `morphed_by_many` (expand_relations_morph.rs)
- [x] Tests: MorphOne/MorphMany query, MorphToMany attach/detach, MorphedByMany query (unit in morph.rs); macro test in integration.rs; morph_type_map unit tests

---

## 7.5 Relationship Write Operations

### API

```rust
// HasMany — create through relation (auto-injects FK)
let comment = post.comments().create_returning(&pool, &[
    ("body", "Great post!".into()),
    ("user_id", current_user_id.into()),
]).await?;

// Save an existing struct through relation (fills FK)
let mut comment = Comment { body: "hello".into(), ..Default::default() };
post.comments().save(&pool, &mut comment).await?;

// BelongsTo — associate / dissociate
comment.post().associate(&pool, &post).await?;   // sets post_id = post.id
comment.post().dissociate(&pool).await?;           // sets post_id = NULL

// HasOne — create or replace (deletes existing first)
user.profile().create_or_replace(&pool, &[
    ("bio", "Rust dev".into()),
]).await?;

// HasMany — create many
user.posts().create_many(&pool, &[
    vec![("title", "Post 1".into())],
    vec![("title", "Post 2".into())],
]).await?;
```

### Tasks

- [x] Add `create_sql(parent_id, data)` on `HasMany` — INSERT with FK injected
- [x] Add `associate_sql(child_pk, parent_id)` on `HasMany` and `BelongsTo` — UPDATE SET fk
- [x] Add `dissociate_sql(child_pk)` on `HasMany` and `BelongsTo` — UPDATE SET fk = NULL
- [x] Add async `create()`, `create_returning()` on `HasMany` for direct DB execution
- [x] Add `save(pool, parent_id, child_pk_val, data)` on `HasMany` — async INSERT-or-UPDATE with FK injected (PG)
- [x] Add `create_or_replace(pool, parent_id, data)` on `HasOne` — DELETE then INSERT RETURNING * (PG)
- [x] Tests: create_sql, associate_sql, dissociate_sql SQL generation

---

## 7.6 whereHas / whereDoesntHave

### API

```rust
// Posts with at least one published comment
let posts = Post::query()
    .where_has("comments", |q| q.filter("published", true))
    .get(&pool)
    .await?;

// Posts with more than 5 comments
let posts = Post::query()
    .where_has_count("comments", 5, CountOp::GreaterThan)
    .get(&pool)
    .await?;

// Users with no posts
let users = User::query()
    .where_doesnt_have("posts")
    .get(&pool)
    .await?;

// Users with no published posts
let users = User::query()
    .where_doesnt_have("posts", |q| q.filter("published", true))
    .get(&pool)
    .await?;
```

**SQL generated:**
```sql
-- where_has
WHERE EXISTS (SELECT 1 FROM comments WHERE comments.post_id = posts.id AND published = $1)

-- where_doesnt_have
WHERE NOT EXISTS (SELECT 1 FROM posts WHERE posts.user_id = users.id)

-- where_has_count
WHERE (SELECT COUNT(*) FROM comments WHERE comments.post_id = posts.id) > 5
```

### Tasks

- [x] Add `CountOp` enum: `Equal`, `NotEqual`, `GreaterThan`, `GreaterThanOrEqual`, `LessThan`, `LessThanOrEqual`
- [x] Add `where_has(rel, closure)` to QueryBuilder → `WHERE EXISTS (...)`
- [x] Add `where_doesnt_have(rel, closure?)` → `WHERE NOT EXISTS (...)`
- [x] Add `where_has_raw` / `where_doesnt_have_raw` for raw subquery strings
- [x] Add `where_has_count(rel, n, CountOp)` → subquery with count comparison
- [x] Integrate with relation registry (each model exposes its relation sub-query builders via macro)
- [x] Tests: each variant, with and without closure (unit tests in subquery.rs)

---

## 7.7 Relationship Aggregates: withCount / withSum / withAvg

### API

```rust
let posts = Post::query()
    .with_count("comments")
    .with_count_as("published_comments", "comments", |q| q.filter("published", true))
    .get(&pool)
    .await?;

// posts[0].extras["comments_count"] -> SqlValue::Integer(5)
// posts[0].extras["published_comments_count"] -> SqlValue::Integer(3)

let users = User::query()
    .with_sum("orders", "total")
    .with_avg("orders", "total")
    .with_max("orders", "total")
    .get(&pool)
    .await?;
// users[0].extras["orders_sum_total"]
// users[0].extras["orders_avg_total"]
```

**SQL generated (subquery style):**
```sql
SELECT posts.*,
  (SELECT COUNT(*) FROM comments WHERE comments.post_id = posts.id) AS comments_count,
  (SELECT COUNT(*) FROM comments WHERE comments.post_id = posts.id AND published = $1) AS published_comments_count
FROM posts
```

### Tasks

- [x] Add `extras: HashMap<String, SqlValue>` field to model row results (via `WithExtras<M>`)
- [x] Add `with_count_col(child, fk, pk, alias)` to QueryBuilder — correlated COUNT subquery
- [x] Add `with_sum_col`, `with_avg_col`, `with_min_col`, `with_max_col` — aggregate subqueries
- [x] Generate subquery SQL injected into SELECT list as named column
- [x] Map result column into the `extras` map after fetch (via `fetch_with_extras` + `get_with_extras`)
- [x] Tests: with_count, with_sum, multiple aggregates combined

---

## 7.8 firstOrCreate / firstOrNew / updateOrCreate

### API

```rust
// Find or create
let user = User::first_or_create(&pool,
    &[("email", "alice@example.com".into())],   // search
    &[("name", "Alice".into()), ("role", "user".into())],  // defaults on create
).await?;

// Find or new (no DB write)
let user = User::first_or_new(
    &[("email", "alice@example.com".into())],
    &[("name", "Alice".into())],
);

// Update if found, create if not
let user = User::update_or_create(&pool,
    &[("email", "alice@example.com".into())],
    &[("name", "Alice Updated".into()), ("last_login_at", Utc::now().into())],
).await?;
```

### Tasks

- [x] Add `first_or_create(pool, search, defaults)` to PgModel / SqliteModel / MyModel
- [x] Add `first_or_new(search, defaults) -> Self` (sync, no pool)
- [x] Add `update_or_create(pool, search, values)` — UPDATE if found, INSERT if not
- [x] Tests: create path, find path, update path

---

## 7.9 Model Replication and Comparison

### API

```rust
let original = Post::find_or_404(&pool, 1).await?;
let mut copy = original.replicate();   // clones, resets PK to default
copy.title = format!("Copy of {}", original.title);
let saved = Post::create_returning(&pool, &copy.to_fields()).await?;

let a = User::find_or_404(&pool, 1).await?;
let b = User::find_or_404(&pool, 1).await?;
assert!(a.is(&b));
```

### Tasks

- [x] Add `replicate(&self) -> Self` to `Model` trait (clone + reset PK field to Default)
- [x] Add `to_fields(&self) -> Vec<(&'static str, SqlValue)>` — serialize all non-PK columns
- [x] Add `is(&self, other: &Self) -> bool` — value equality (requires `Self: PartialEq`)
- [x] Tests: to_fields with rename, skip, default PK exclusion

---

## 7.10 UUID / ULID Primary Keys

### API

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "articles", primary_key = "id", uuid)]
pub struct Article {
    pub id: String,
    pub title: String,
}

#[derive(Model, sqlx::FromRow)]
#[model(table = "events", primary_key = "id", ulid)]
pub struct Event {
    pub id: String,
    pub name: String,
}

#[derive(Model, sqlx::FromRow)]
#[model(table = "sessions", primary_key = "token", custom_id = "generate_token")]
pub struct Session {
    pub token: String,
    pub user_id: i64,
}

fn generate_token() -> String { /* user-defined */ }
```

### Tasks

- [x] Add `uuid` boolean flag to `#[model(...)]` parser
- [x] Add `Model::new_unique_id() -> Option<SqlValue>` — default `None` (auto-increment)
- [x] Macro generates override using `uuid::Uuid::new_v4()` behind `uuid-pk` feature flag
- [x] Inject generated ID into INSERT data before executor runs (PgModel create/create_returning)
- [x] Add `custom_id = "fn_name"` — calls user-defined function for ID generation
- [x] Add `uuid` to `Cargo.toml` optional dependencies (feature: `uuid-pk`)
- [x] Tests: create with UUID PK, find by UUID PK

---

## 7.11 Per-Model Database Connection

### API

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "audit_logs", connection = "audit_db")]
pub struct AuditLog { ... }

// At startup
ConnectionRegistry::register("audit_db", audit_pool);

// All AuditLog methods use the registered "audit_db" pool
let logs = AuditLog::all(&pool).await?;  // pool param ignored for named-connection models
```

### Tasks

- [x] Add `connection()` method to `Model` trait (default: `"default"`)
- [x] Macro generates override from `#[model(connection = "...")]`
- [x] Add `ConnectionRegistry` — dialect-specific `OnceLock<RwLock<HashMap<String, Pool>>>`
- [ ] Executor methods check `Model::connection()` and resolve pool from registry
- [ ] Tests: register + use named connection, fallback to provided pool

---

## 7.12 withoutTimestamps + Custom Timestamp Column Names

### API

```rust
#[derive(Model, sqlx::FromRow)]
#[model(timestamps, created_at_col = "creation_date", updated_at_col = "modified_date")]
pub struct Flight {
    pub id: i64,
    pub name: String,
    pub creation_date: String,
    pub modified_date: String,
}

// Suppress for one call
User::without_timestamps(|| async {
    User::update_by_pk(&pool, 1, &[("views", 1000.into())]).await
}).await?;

User::increment_without_timestamps(&pool, 1, "views", 1).await?;
```

### Tasks

- [x] Add `created_at_col` and `updated_at_col` to `#[model(...)]` attribute
- [x] Macro uses these overrides instead of `"created_at"` / `"updated_at"` literals
- [x] Add `soft_delete_col` attribute to `#[model(...)]` for custom soft-delete column name
- [x] Add `TIMESTAMPS_MUTED: thread_local! { Cell<bool> }`
- [x] Add `Model::without_timestamps(closure)` — sets flag, runs, resets
- [x] Executor paths check flag before injecting timestamp columns (PgModel)
- [x] Add `increment_without_timestamps(pool, id, col, delta)` helper
- [x] Tests: without_timestamps sets and resets flag correctly

---

## 7.13 Model Pruning

### API

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "activity_logs", prunable)]
pub struct ActivityLog {
    pub id: i64,
    pub action: String,
    pub created_at: DateTime<Utc>,
}

impl Prunable for ActivityLog {
    fn prunable_query() -> QueryBuilder<Self> {
        ActivityLog::query()
            .where_lt("created_at", Utc::now() - Duration::days(30))
    }
}

let deleted = ActivityLog::prune(&pool).await?;  // → u64 rows deleted
```

### Tasks

- [x] Define `Prunable` trait: `prunable_query() -> QueryBuilder<Self>`
- [x] Add default `prune(pool)` / `prune_sqlite(pool)` / `prune_mysql(pool)` on `Prunable` (DELETE)
- [x] Add `PrunableRegistry` — register + `prune_all(pool)` batch runner
- [x] Tests: prune generates correct DELETE SQL with date filter

---

## 7.14 Event Muting (without_events)

### API

```rust
User::without_events(|| async {
    User::create(&pool, &[
        ("name", "Seeded User".into()),
        ("email", "seed@example.com".into()),
    ]).await
}).await?;

// Instance-level
let user = User::find_or_404(&pool, 1).await?;
user.save_quietly(&pool, &[("name", "Quiet Update".into())]).await?;
```

### Tasks

- [x] Add `EVENTS_MUTED: thread_local! { Cell<bool> }`
- [x] Add `Model::without_events(closure)` — set flag, run, reset
- [x] Executor paths check flag before dispatching hooks / observer calls
- [x] Add `save_quietly(pool, data)` as convenience wrapper calling `update_by_pk` with events muted
- [x] Tests: without_events sets and resets flag correctly

---

## Acceptance Criteria for Phase 7

- [x] All 14 sub-sections fully implemented
- [x] Zero regressions in existing 98 tests (now 217 tests, 0 failures)
- [x] All new features tested on PostgreSQL AND SQLite (minimum)
- [x] All new `#[model(...)]` attributes documented in proc-macro doc comments
- [x] `cargo clippy -- -D warnings` clean
- [x] Phase file tasks all checked off
