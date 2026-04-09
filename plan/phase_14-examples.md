# Phase 14: Examples Implementation

> **Target version:** v0.5.0
> **Status:** 🚧 In Progress
> **Note:** Comprehensive examples for all phases, organized into logical groups with Docker Compose support

---

## Overview

This phase provides practical, working examples for all rok-orm features. Examples are grouped into three implementation phases:

| Group | Phases | Focus | Docker Services |
|-------|--------|-------|-----------------|
| **14A** | 1-6 | Core Foundation | PostgreSQL, SQLite |
| **14B** | 7-8 | Relationships & Ergonomics | PostgreSQL, SQLite |
| **14C** | 9-13 | Advanced Features | PostgreSQL, MySQL, Redis |

---

# Phase 14A: Core Foundation Examples

> **Features from Phases 1-6**
> 
> Basic model definitions, query builder, CRUD operations, relationships, soft deletes, timestamps, pagination, aggregations, hooks, transactions.

## 14A.1 Project Setup

### Cargo.toml

```toml
[package]
name = "rok-orm-examples"
version = "0.5.0"
edition = "2021"

[dependencies]
rok-orm = { path = "../rok-orm", features = ["postgres", "sqlite"] }
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "postgres", "sqlite"] }
serde = { version = "1", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
```

### docker-compose.yml

```yaml
version: "3.8"

services:
  postgres:
    image: postgres:16-alpine
    container_name: rok-postgres
    environment:
      POSTGRES_USER: rok
      POSTGRES_PASSWORD: rokpass
      POSTGRES_DB: rok_orm_examples
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U rok -d rok_orm_examples"]
      interval: 5s
      timeout: 5s
      retries: 5

  sqlite:
    image: alpine:latest
    container_name: rok-sqlite
    volumes:
      - ./data:/data
    command: ["tail", "-f", "/dev/null"]

volumes:
  postgres_data:
```

### Run with Docker

```bash
docker-compose up -d
export DATABASE_URL="postgres://rok:rokpass@localhost:5432/rok_orm_examples"
```

---

## 14A.2 Basic Model Definition

```rust
use rok_orm::{Model, PgModel};
use chrono::{DateTime, Utc};

#[derive(Model, sqlx::FromRow)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
    pub active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

assert_eq!(User::table_name(), "users");
assert_eq!(User::columns(), &["id", "name", "email", "active", "created_at", "updated_at"]);
```

---

## 14A.3 Query Builder Basics

```rust
#[tokio::main]
async fn main() -> OrmResult<()> {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await?;

    // Build query
    let (sql, params) = User::query()
        .filter("active", true)
        .order_by_desc("created_at")
        .limit(10)
        .to_sql();

    println!("SQL: {}", sql);
    // SELECT * FROM users WHERE active = $1 ORDER BY created_at DESC LIMIT $2

    // Execute query
    let users: Vec<User> = User::query()
        .where_eq("active", true)
        .order_by_desc("created_at")
        .limit(10)
        .get(&pool)
        .await?;

    // First result or default
    let user = User::query()
        .filter("email", "alice@example.com")
        .first(&pool)
        .await?;

    Ok(())
}
```

---

## 14A.4 CRUD Operations

```rust
// CREATE
User::create(&pool, &[
    ("name", "Alice".into()),
    ("email", "alice@example.com".into()),
    ("active", true.into()),
]).await?;

// CREATE with RETURNING
let user = User::create_returning(&pool, &[
    ("name", "Bob".into()),
    ("email", "bob@example.com".into()),
    ("active", true.into()),
]).await?;
println!("Created user with id: {}", user.id);

// READ - find by primary key
let user = User::find_by_pk(&pool, 1).await?;

// READ - find or 404
let user = User::find_or_404(&pool, 1).await?;

// READ - all
let users = User::all(&pool).await?;

// UPDATE
User::update_by_pk(&pool, 1, &[
    ("name", "Alice Updated".into()),
]).await?;

// DELETE
User::delete_by_pk(&pool, 1).await?;

// Upsert (INSERT ON CONFLICT)
User::upsert(&pool, &[
    ("email", "admin@example.com".into()),
    ("name", "Admin".into()),
]).await?;

User::upsert(&pool, &[
    ("email", "admin@example.com".into()),
    ("name", "Admin Updated".into()),
], "email", &["name"]).await?;
```

---

## 14A.5 Basic Relationships

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "posts")]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub body: Option<String>,
    pub user_id: i64,
}

#[derive(Relations)]
pub struct UserRelations {
    #[has_many(target = "Post")]
    pub posts: HasMany<User, Post>,
}

#[derive(Relations)]
pub struct PostRelations {
    #[belongs_to(target = "User")]
    pub user: BelongsTo<Post, User>,
}

// Eager loading - prevents N+1
let users = User::query()
    .with("posts")
    .limit(10)
    .get(&pool)
    .await?;

for user in &users {
    println!("User: {} has {} posts", user.name, user.posts.len());
}
```

---

## 14A.6 Soft Deletes

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "posts", soft_delete)]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub deleted_at: Option<DateTime<Utc>>,
}

// Excludes deleted by default
let posts = Post::all(&pool).await?;

// Include deleted
let all = Post::with_soft_delete().get(&pool).await?;
let all = Post::with_trashed().get(&pool).await?;

// Only deleted
let trashed = Post::only_trashed().get(&pool).await?;

// Restore
Post::restore(&pool, post_id).await?;

// Force delete (permanent)
Post::force_delete(&pool, post_id).await?;
```

---

## 14A.7 Auto Timestamps

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "articles", timestamps)]
pub struct Article {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// create_returning auto-adds created_at and updated_at
let article = Article::create_returning(&pool, &[
    ("title", "My Article".into()),
]).await?;
println!("Created at: {}", article.created_at);

// update_by_pk auto-updates updated_at
Article::update_by_pk(&pool, 1, &[
    ("title", "Updated Title".into()),
]).await?;
```

---

## 14A.8 Pagination

```rust
let page: Page<Post> = Post::paginate(&pool, 1, 20).await?;

println!("Total items: {}", page.total);
println!("Total pages: {}", page.last_page);
println!("Current page: {}", page.current_page);
println!("Per page: {}", page.per_page);
println!("Has next: {}", page.has_next());
println!("Has prev: {}", page.has_prev());

// From page 3
let page = Post::paginate(&pool, 3, 20).await?;

// Custom query with pagination
let page = Post::query()
    .filter("active", true)
    .order_by_desc("created_at")
    .paginate(&pool, 2, 50)
    .await?;
```

---

## 14A.9 Aggregations

```rust
let total: i64 = User::count(&pool).await?;
let active_count: i64 = User::query().filter("active", true).count(&pool).await?;

let revenue: f64 = Order::sum("total", &pool).await?;
let avg_age: f64 = User::avg("age", &pool).await?;
let oldest: Option<i64> = User::max("age", &pool).await?;
let youngest: Option<i64> = User::min("age", &pool).await?;

// Aggregate with query builder
let (sql, params) = User::query()
    .filter("active", true)
    .sum_sql("age");

let (sql, params) = User::query()
    .group_by(&["role"])
    .avg_sql("balance");
```

---

## 14A.10 Model Hooks

```rust
#[derive(ModelHooks)]
pub struct UserHooks;

#[async_trait]
impl ModelHooks<User> for UserHooks {
    async fn before_create(user: &mut User) -> OrmResult<()> {
        user.email = user.email.to_lowercase();
        Ok(())
    }

    async fn after_create(user: &User) -> OrmResult<()> {
        println!("Created user: {}", user.id);
        Ok(())
    }

    async fn before_update(user: &mut User) -> OrmResult<()> {
        println!("Updating user: {}", user.id);
        Ok(())
    }

    async fn after_update(user: &User) -> OrmResult<()> {
        Ok(())
    }

    async fn before_delete(user: &User) -> OrmResult<()> {
        Ok(())
    }

    async fn after_delete(user: &User) -> OrmResult<()> {
        Ok(())
    }
}
```

---

## 14A.11 Transactions

```rust
use rok_orm::Tx;

let mut tx = Tx::begin(&pool).await?;

tx.insert::<User>("users", &[
    ("name", "Alice".into()),
    ("email", "alice@example.com".into()),
]).await?;

tx.insert::<Post>("posts", &[
    ("title", "First Post".into()),
    ("user_id", 1i64.into()),
]).await?;

tx.commit().await?;

// Or rollback
let mut tx = Tx::begin(&pool).await?;
if something_failed {
    tx.rollback().await?;
} else {
    tx.commit().await?;
}

// Transaction with guard
{
    let mut tx = Tx::begin(&pool).await?;
    // ... operations ...
    tx.commit().await?;
} // automatic rollback if not committed
```

---

## 14A.12 Query Scopes

```rust
impl User {
    pub fn active() -> QueryBuilder<User> {
        User::query().filter("active", true)
    }
    
    pub fn admins() -> QueryBuilder<User> {
        User::query().filter("role", "admin")
    }
    
    pub fn recent(days: i64) -> QueryBuilder<User> {
        let cutoff = Utc::now() - Duration::days(days);
        User::query().where_gt("created_at", cutoff)
    }
}

let users = User::active().get(&pool).await?;
let admins = User::admins().get(&pool).await?;
let recent = User::recent(30).get(&pool).await?;

// Chained scopes
let active_admins = User::active().admins().get(&pool).await?;
```

---

## 14A.13 Query Logging

```rust
use rok_orm::logging::{Logger, LogLevel, QueryTimer};

let logger = Logger::new()
    .with_log_level(LogLevel::Debug)
    .with_slow_query_threshold(100) // ms
    .with_query_callback(|sql, duration| {
        println!("Query took {}ms: {}", duration.as_millis(), sql);
    });

let timer = QueryTimer::new();

let users = User::query()
    .filter("active", true)
    .get(&pool)
    .await?;

let elapsed = timer.elapsed_ms();
if logger.is_slow_query(elapsed) {
    tracing::warn!("Slow query: {}ms - {}", elapsed, sql);
}
```

---

# Phase 14B: Rich Relationships & Developer Ergonomics

> **Features from Phases 7-8**
> 
> ManyToMany with pivot, HasManyThrough, HasOneThrough, Polymorphic, whereHas, withCount, firstOrCreate, when/when_else, raw expressions, tap/dd, chunking, cursor pagination, fillable/guarded, observers, global scopes, touches.

## 14B.1 Docker Compose for Phase 14B

```yaml
version: "3.8"

services:
  postgres:
    image: postgres:16-alpine
    container_name: rok-postgres
    environment:
      POSTGRES_USER: rok
      POSTGRES_PASSWORD: rokpass
      POSTGRES_DB: rok_orm_examples
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  sqlite:
    image: alpine:latest
    container_name: rok-sqlite
    volumes:
      - ./data:/data

volumes:
  postgres_data:
```

---

## 14B.2 Many-to-Many with Full Pivot Access

```rust
use rok_orm::{Model, PgModel, BelongsToMany};
use chrono::{DateTime, Utc};

#[derive(Model, sqlx::FromRow)]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
}

#[derive(Model, sqlx::FromRow)]
pub struct Role {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub active: bool,
}

#[derive(Relations)]
pub struct UserRelations {
    #[belongs_to_many(
        target = "Role",
        pivot = "user_roles",
        fk = "user_id",
        rfk = "role_id",
        pivots = ["assigned_at", "expires_at"],
    )]
    pub roles: BelongsToMany<User, Role>,
}

// Attach role to user
user.roles().attach(&pool, role_id).await?;

// Attach with pivot data
user.roles().attach_with_pivot(&pool, role_id, &[
    ("assigned_at", Utc::now().into()),
    ("expires_at", None::<DateTime<Utc>>.into()),
]).await?;

// Sync - replace entire set
user.roles().sync(&pool, vec![1i64, 2, 3]).await?;

// Toggle - attach if absent, detach if present
user.roles().toggle(&pool, vec![1i64, 2]).await?;

// Query with pivot columns
let roles = user.roles()
    .filter("roles.active", true)
    .with_pivot(&["assigned_at", "expires_at"])
    .order_by_desc("assigned_at")
    .get(&pool)
    .await?;

for role in &roles {
    if let Some(pivot) = role.pivot() {
        println!("assigned: {:?}", pivot.get("assigned_at"));
    }
}

// Update pivot row
user.roles().update_pivot(&pool, role_id, &[
    ("expires_at", new_date.into()),
]).await?;

// Detach
user.roles().detach(&pool, role_id).await?;
user.roles().detach_all(&pool).await?;
```

---

## 14B.3 Has-Many-Through

```rust
#[derive(Model, sqlx::FromRow)]
pub struct Country {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
}

#[derive(Model, sqlx::FromRow)]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub country_id: i64,
}

#[derive(Model, sqlx::FromRow)]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub user_id: i64,
}

#[derive(Relations)]
pub struct CountryRelations {
    #[has_many_through(
        related = "Post",
        through = "User",
        first_key = "country_id",
        second_key = "user_id",
    )]
    pub posts: HasManyThrough<Country, User, Post>,
}

// Query posts through country
let country = Country::find_by_pk(&pool, 1).await?;
let posts = country.posts()
    .filter("published", true)
    .order_by_desc("created_at")
    .get(&pool)
    .await?;

// SQL generated:
// SELECT posts.* FROM posts
// INNER JOIN users ON users.id = posts.user_id
// WHERE users.country_id = $1
```

---

## 14B.4 Has-One-Through

```rust
#[derive(Model, sqlx::FromRow)]
pub struct Mechanic {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
}

#[derive(Model, sqlx::FromRow)]
pub struct Car {
    #[model(primary_key)]
    pub id: i64,
    pub make: String,
    pub mechanic_id: i64,
}

#[derive(Model, sqlx::FromRow)]
pub struct CarOwner {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub car_id: i64,
}

#[derive(Relations)]
pub struct MechanicRelations {
    #[has_one_through(
        related = "CarOwner",
        through = "Car",
        first_key = "mechanic_id",
        second_key = "car_id",
    )]
    pub car_owner: HasOneThrough<Mechanic, Car, CarOwner>,
}

let mechanic = Mechanic::find_by_pk(&pool, 1).await?;
let owner = mechanic.car_owner().get(&pool).await?;
```

---

## 14B.5 Polymorphic Relationships

```rust
#[derive(Model, sqlx::FromRow)]
pub struct Image {
    #[model(primary_key)]
    pub id: i64,
    pub url: String,
    pub imageable_id: i64,
    pub imageable_type: String,
}

#[derive(Model, Relations, sqlx::FromRow)]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,

    #[model(morph_one(related = "Image", morph_key = "imageable"))]
    pub image: MorphOne<User, Image>,
}

#[derive(Model, Relations, sqlx::FromRow)]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,

    #[model(morph_many(related = "Image", morph_key = "imageable"))]
    pub images: MorphMany<Post, Image>,
}

// Usage
let image = user.image().get(&pool).await?;
let images = post.images().get(&pool).await?;

// Create fills imageable_type automatically
post.images().create(&pool, &[
    ("url", "https://cdn.example.com/img.png".into()),
]).await?;

// morphTo - inverse relationship
#[derive(Model, Relations, sqlx::FromRow)]
pub struct Image {
    #[model(primary_key)]
    pub id: i64,
    pub url: String,
    pub imageable_id: i64,
    pub imageable_type: String,

    #[model(morph_to(morph_key = "imageable"))]
    pub imageable: MorphToRef,
}

// Register types
morph_type_map! {
    ("users", User),
    ("posts", Post),
}

let parent = image.imageable().resolve(&pool).await?;
match parent {
    MorphParent::User(u) => println!("user: {}", u.name),
    MorphParent::Post(p) => println!("post: {}", p.title),
    MorphParent::Unknown(t, id) => eprintln!("unknown: {} #{}", t, id),
}

// morphToMany / morphedByMany
#[derive(Model, Relations, sqlx::FromRow)]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,

    #[model(morph_to_many(
        related = "Tag",
        pivot = "taggables",
        morph_key = "taggable",
    ))]
    pub tags: MorphToMany<Post, Tag>,
}

#[derive(Model, Relations, sqlx::FromRow)]
pub struct Tag {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,

    #[model(morphed_by_many(
        related = "Post",
        pivot = "taggables",
        morph_key = "taggable",
    ))]
    pub posts: MorphedByMany<Tag, Post>,
}

post.tags().attach(&pool, tag_id).await?;
post.tags().sync(&pool, vec![1i64, 5, 9]).await?;
let tags = post.tags().get(&pool).await?;
```

---

## 14B.6 Relationship Write Operations

```rust
// HasMany - create through relation (auto-injects FK)
let comment = post.comments().create_returning(&pool, &[
    ("body", "Great post!".into()),
    ("user_id", current_user_id.into()),
]).await?;

// Save an existing struct through relation
let mut comment = Comment { body: "hello".into(), ..Default::default() };
post.comments().save(&pool, &mut comment).await?;

// BelongsTo - associate / dissociate
comment.post().associate(&pool, &post).await?;
comment.post().dissociate(&pool).await?;

// HasOne - create or replace
user.profile().create_or_replace(&pool, &[
    ("bio", "Rust dev".into()),
]).await?;

// HasMany - create many
user.posts().create_many(&pool, &[
    vec![("title", "Post 1".into())],
    vec![("title", "Post 2".into())],
]).await?;
```

---

## 14B.7 whereHas / whereDoesntHave

```rust
use rok_orm::query::CountOp;

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

---

## 14B.8 withCount / withSum / withAvg

```rust
let posts = Post::query()
    .with_count("comments")
    .with_count_as("published_comments", "comments", |q| q.filter("published", true))
    .get(&pool)
    .await?;

for post in &posts {
    println!("comments: {:?}", post.extras.get("comments_count"));
    println!("published: {:?}", post.extras.get("published_comments_count"));
}

let users = User::query()
    .with_sum("orders", "total")
    .with_avg("orders", "total")
    .with_max("orders", "total")
    .with_min("orders", "total")
    .get(&pool)
    .await?;
```

---

## 14B.9 firstOrCreate / firstOrNew / updateOrCreate

```rust
use chrono::Utc;

// Find or create
let user = User::first_or_create(&pool,
    &[("email", "alice@example.com".into())],
    &[("name", "Alice".into()), ("role", "user".into())],
).await?;

// Find or new (no DB write)
let user = User::first_or_new(
    &[("email", "alice@example.com".into())],
    &[("name", "Alice".into())],
);
// user exists in memory but not saved

// Update if found, create if not
let user = User::update_or_create(&pool,
    &[("email", "alice@example.com".into())],
    &[("name", "Alice Updated".into()), ("last_login_at", Utc::now().into())],
).await?;
```

---

## 14B.10 UUID / ULID Primary Keys

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "articles", uuid)]
pub struct Article {
    pub id: String,
    pub title: String,
}

#[derive(Model, sqlx::FromRow)]
#[model(table = "events", ulid)]
pub struct Event {
    pub id: String,
    pub name: String,
}

// Create with auto-generated UUID
let article = Article::create_returning(&pool, &[
    ("title", "My Article".into()),
]).await?;
println!("Created with UUID: {}", article.id);

// Custom ID generation
#[derive(Model, sqlx::FromRow)]
#[model(table = "sessions", custom_id = "generate_token")]
pub struct Session {
    pub token: String,
    pub user_id: i64,
}

fn generate_token() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}
```

---

## 14B.11 Per-Model Database Connection

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "audit_logs", connection = "audit_db")]
pub struct AuditLog {
    #[model(primary_key)]
    pub id: i64,
    pub action: String,
    pub created_at: Option<DateTime<Utc>>,
}

// At application startup
ConnectionRegistry::register("audit_db", audit_pool);

// All AuditLog methods use the registered pool
let logs = AuditLog::all(&pool).await?;
let log = AuditLog::find_by_pk(&pool, 1).await?;
```

---

## 14B.12 withoutTimestamps + Custom Column Names

```rust
#[derive(Model, sqlx::FromRow)]
#[model(
    timestamps,
    created_at_col = "creation_date",
    updated_at_col = "modified_date"
)]
pub struct Flight {
    pub id: i64,
    pub name: String,
    pub creation_date: Option<DateTime<Utc>>,
    pub modified_date: Option<DateTime<Utc>>,
}

// Suppress timestamps for one call
User::without_timestamps(|| async {
    User::update_by_pk(&pool, 1, &[("views", 1000.into())]).await
}).await?;

// Increment without timestamps
User::increment_without_timestamps(&pool, 1, "views", 1).await?;
```

---

## 14B.13 Model Pruning

```rust
use chrono::Duration;

#[derive(Model, sqlx::FromRow)]
#[model(table = "activity_logs", prunable)]
pub struct ActivityLog {
    #[model(primary_key)]
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

// Prune old logs
let deleted = ActivityLog::prune(&pool).await?;
println!("Deleted {} old activity logs", deleted);

// Register for batch pruning
PrunableRegistry::register::<ActivityLog>();

// Prune all registered models
let results = PrunableRegistry::prune_all(&pool).await?;
```

---

## 14B.14 Event Muting

```rust
// Suppress all events in a block
User::without_events(|| async {
    User::create(&pool, &[
        ("name", "Seeded User".into()),
        ("email", "seed@example.com".into()),
    ]).await
}).await?;

// Instance-level quiet save
let user = User::find_or_404(&pool, 1).await?;
user.save_quietly(&pool, &[("name", "Quiet Update".into())]).await?;
```

---

## 14B.15 when() / when_else() Conditional Chaining

```rust
// Conditional query building
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

---

## 14B.16 Raw Expressions

```rust
// Raw WHERE
let users = User::query()
    .where_raw("LOWER(email) = LOWER($1)", vec!["admin@example.com".into()])
    .get(&pool)
    .await?;

// Raw SELECT
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

// Execute raw SQL directly
let rows: Vec<User> = User::from_raw_sql(
    &pool,
    "SELECT * FROM users WHERE created_at > $1 AND active = true",
    vec![start_date.into()],
).await?;
```

---

## 14B.17 tap() and dd() Debugging

```rust
// tap() - inspect without breaking the chain
let users = User::query()
    .filter("active", true)
    .tap(|q| {
        let (sql, _) = q.to_sql();
        tracing::debug!("Before limit: {sql}");
    })
    .limit(10)
    .get(&pool)
    .await?;

// dd() - print and panic in debug builds
#[cfg(debug_assertions)]
User::query()
    .filter("active", true)
    .dd();
```

---

## 14B.18 Chunking for Large Datasets

```rust
use tokio::try_join;

// Chunk with LIMIT/OFFSET
User::query()
    .filter("active", true)
    .chunk(&pool, 500, |batch| async move {
        for user in batch {
            send_email(&user).await;
        }
        Ok(())
    })
    .await?;

// chunk_by_id - stable even if rows are deleted
User::query()
    .chunk_by_id(&pool, 500, |batch| async move {
        process(batch).await
    })
    .await?;

// into_stream - async stream of rows
use futures::StreamExt;

let mut stream = User::query()
    .filter("active", true)
    .into_stream(&pool);

while let Some(user) = stream.next().await {
    let user = user?;
    process(user).await;
}
```

---

## 14B.19 Cursor Pagination

```rust
// First page
let result = Post::query()
    .order_by_desc("created_at")
    .cursor_paginate(&pool, CursorPage { after: None, limit: 20 })
    .await?;

println!("next cursor: {:?}", result.next_cursor);
println!("has more: {}", result.has_more);
println!("prev cursor: {:?}", result.prev_cursor);

// Next page
let next_result = Post::query()
    .order_by_desc("created_at")
    .cursor_paginate(&pool, CursorPage { after: result.next_cursor, limit: 20 })
    .await?;

// Previous page
let prev_result = Post::query()
    .order_by_desc("created_at")
    .cursor_paginate(&pool, CursorPage { after: result.prev_cursor, limit: 20 })
    .await?;
```

---

## 14B.20 fill() and Mass Assignment Protection

```rust
#[derive(Model, sqlx::FromRow)]
#[model(
    table = "users",
    fillable = ["name", "email", "bio"],
)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
    pub role: String,     // not fillable
    pub is_admin: bool,   // not fillable
}

// role and is_admin are silently dropped
let user = User::create_returning(&pool, &[
    ("name", "Alice".into()),
    ("email", "alice@example.com".into()),
    ("role", "admin".into()),
    ("is_admin", true.into()),
]).await?;

// Alternative: guarded approach
#[derive(Model, sqlx::FromRow)]
#[model(
    table = "users",
    guarded = ["id", "role", "is_admin"],
)]
pub struct UserGuarded {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub is_admin: bool,
}
```

---

## 14B.21 Model Observers

```rust
use rok_orm::observer::{ModelObserver, ObserverEvent};

pub struct UserObserver;

#[async_trait]
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

// Register observer
User::observe(UserObserver);
```

---

## 14B.22 Global Query Scopes

```rust
pub struct ActiveScope;

impl GlobalScope<User> for ActiveScope {
    fn apply(&self, query: QueryBuilder<User>) -> QueryBuilder<User> {
        query.filter("active", true)
    }
}

pub struct VerifiedScope;

impl GlobalScope<User> for VerifiedScope {
    fn apply(&self, query: QueryBuilder<User>) -> QueryBuilder<User> {
        query.filter("verified", true)
    }
}

// Register global scope
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

---

## 14B.23 touches — Parent Timestamp Propagation

```rust
#[derive(Model, sqlx::FromRow)]
#[model(
    timestamps,
    touches = ["post"],
)]
pub struct Comment {
    #[model(primary_key)]
    pub id: i64,
    pub post_id: i64,
    pub body: String,
    pub updated_at: Option<DateTime<Utc>>,
}

// After this update, posts.updated_at is also set to NOW()
Comment::update_by_pk(&pool, comment_id, &[
    ("body", "edited".into())
]).await?;
```

---

# Phase 14C: Advanced Features

> **Features from Phases 9-13**
> 
> Schema Builder, Migrations, JSON columns, full-text search, subqueries, CTEs, window functions, MSSQL, Redis caching, Axum integration.

## 14C.1 Docker Compose for Phase 14C

```yaml
version: "3.8"

services:
  postgres:
    image: postgres:16-alpine
    container_name: rok-postgres
    environment:
      POSTGRES_USER: rok
      POSTGRES_PASSWORD: rokpass
      POSTGRES_DB: rok_orm_examples
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  mysql:
    image: mysql:8
    container_name: rok-mysql
    environment:
      MYSQL_ROOT_PASSWORD: rokpass
      MYSQL_DATABASE: rok_orm_examples
      MYSQL_USER: rok
      MYSQL_PASSWORD: rokpass
    ports:
      - "3306:3306"
    volumes:
      - mysql_data:/var/lib/mysql

  redis:
    image: redis:7-alpine
    container_name: rok-redis
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  mysql_data:
  redis_data:
```

---

## 14C.2 Schema Builder - Create Table

```rust
use rok_orm::schema::{Schema, Blueprint, ForeignAction};

Schema::create("users", |t: &mut Blueprint| {
    t.id();
    t.string("name", 255);
    t.string("email", 255).unique();
    t.string("password", 255);
    t.boolean("active").default(true);
    t.string("role", 50).default("user");
    t.timestamps();
}).execute(&pool).await?;

Schema::create("posts", |t| {
    t.big_increments("id");
    t.string("title", 255);
    t.text("body");
    t.foreign("user_id")
        .references("users", "id")
        .on_delete(ForeignAction::Cascade);
    t.boolean("published").default(false);
    t.soft_deletes();
    t.timestamps();
}).execute(&pool).await?;
```

---

## 14C.3 Schema Builder - Column Types

```rust
Schema::create("products", |t| {
    // Primary keys
    t.increments("id");
    t.big_increments("big_id");
    t.uuid("uuid_id").primary();
    
    // Strings
    t.string("name", 255);
    t.text("description");
    
    // Numbers
    t.integer("quantity");
    t.big_integer("price");
    t.small_integer("priority");
    t.float("score");
    t.double("amount");
    t.decimal("total", 10, 2);
    
    // Boolean
    t.boolean("active");
    
    // Date/Time
    t.date("birthday");
    t.datetime("published_at");
    
    // Special
    t.json("metadata");
    t.binary("data");
    t.enum_col("status", &["draft", "published", "archived"]);
}).execute(&pool).await?;
```

---

## 14C.4 Schema Builder - Column Modifiers

```rust
Schema::create("samples", |t| {
    t.string("name", 255).nullable();
    t.integer("views").default(0);
    t.string("slug", 255).unique();
    t.string("code", 10).not_null();
    t.string("email", 255).unique().not_null();
    t.string("status", 20).default("active").not_null();
    
    // Composite unique
    t.unique_index(&["tenant_id", "slug"]);
    
    // Primary key
    t.primary_key(&["user_id", "role_id"]);
    
    // Indexes
    t.index(&["status", "created_at"]);
}).execute(&pool).await?;
```

---

## 14C.5 Schema Builder - Alter Table

```rust
// Add column
Schema::alter("users", |t| {
    t.add_column("avatar_url", |c| c.string(500).nullable());
}).execute(&pool).await?;

// Drop column
Schema::alter("users", |t| {
    t.drop_column("old_field");
}).execute(&pool).await?;

// Rename column
Schema::alter("users", |t| {
    t.rename_column("bio", "biography");
}).execute(&pool).await?;

// Change column
Schema::alter("users", |t| {
    t.change_column("name", |c| c.string(500));
}).execute(&pool).await?;

// Add index
Schema::alter("users", |t| {
    t.add_index(&["avatar_url"]);
}).execute(&pool).await?;

// Drop index
Schema::alter("users", |t| {
    t.drop_index("users_email_index");
}).execute(&pool).await?;
```

---

## 14C.6 Schema Builder - Drop/Rename

```rust
// Drop table if exists
Schema::drop_if_exists("users").execute(&pool).await?;

// Drop table
Schema::drop("users").execute(&pool).await?;

// Rename table
Schema::rename("old_name", "new_name").execute(&pool).await?;

// Check existence
let exists = Schema::has_table(&pool, "users").await?;
let has_col = Schema::has_column(&pool, "users", "email").await?;
```

---

## 14C.7 Migration System

```rust
use sqlx::any::AnyPool;
use async_trait::async_trait;

pub struct CreateUsersTable;

#[async_trait]
impl Migration for CreateUsersTable {
    fn name(&self) -> &'static str { "001_create_users_table" }

    async fn up(&self, pool: &AnyPool) -> OrmResult<()> {
        Schema::create("users", |t| {
            t.id();
            t.string("name", 255);
            t.string("email", 255).unique();
            t.timestamps();
        }).execute(pool).await
    }

    async fn down(&self, pool: &AnyPool) -> OrmResult<()> {
        Schema::drop_if_exists("users").execute(pool).await
    }
}

pub struct AddAvatarToUsers;

#[async_trait]
impl Migration for AddAvatarToUsers {
    fn name(&self) -> &'static str { "002_add_avatar_to_users" }

    async fn up(&self, pool: &AnyPool) -> OrmResult<()> {
        Schema::alter("users", |t| {
            t.add_column("avatar_url", |c| c.string(500).nullable());
        }).execute(pool).await
    }

    async fn down(&self, pool: &AnyPool) -> OrmResult<()> {
        Schema::alter("users", |t| {
            t.drop_column("avatar_url");
        }).execute(pool).await
    }
}
```

---

## 14C.8 Running Migrations

```rust
let migrator = Migrator::new(&pool)
    .add(CreateUsersTable)
    .add(CreatePostsTable)
    .add(AddAvatarToUsers);

// Run all pending migrations
migrator.run().await?;

// Rollback last N batches
migrator.rollback(1).await?;

// Reset - rollback all
migrator.reset().await?;

// Fresh - drop all and re-run
migrator.fresh().await?;

// Get status
let statuses = migrator.status().await?;

for status in &statuses {
    println!("{} - batch {:?} - {:?}", status.name, status.batch, status.run_at);
}
```

---

## 14C.9 JSON Column Support

```rust
// PostgreSQL JSONB, MySQL JSON, SQLite TEXT
let users = User::query()
    .where_json_contains("metadata", "role", "admin")
    .where_json_path("settings", "$.theme", "dark")
    .get(&pool)
    .await?;

// Extract JSON field in SELECT
let users = User::query()
    .select_json_field("metadata", "role", "user_role")
    .get(&pool)
    .await?;

// Check if JSON array contains value
let users = User::query()
    .where_json_array_contains("permissions", "posts:write")
    .get(&pool)
    .await?;
```

---

## 14C.10 Full-Text Search

```rust
// PostgreSQL tsvector
let posts = Post::query()
    .where_full_text(&["title", "body"], "rust async orm")
    .order_by_text_rank(&["title", "body"], "rust async orm")
    .get(&pool)
    .await?;

// MySQL FULLTEXT
let posts = Post::query()
    .where_match(&["title", "body"], "rust async orm")
    .get(&pool)
    .await?;

// SQLite FTS5 (requires separate FTS virtual table)
let posts = Post::query()
    .where_fts5("posts_fts", "rust async orm")
    .get(&pool)
    .await?;
```

---

## 14C.11 Sub-queries and CTEs

```rust
// WHERE col IN (subquery)
let power_users = User::query()
    .where_in_subquery("id", |sq| {
        sq.table("orders")
          .select(&["user_id"])
          .group_by(&["user_id"])
          .having_raw("COUNT(*) > 10")
    })
    .get(&pool)
    .await?;

// WHERE EXISTS (subquery)
let users_with_orders = User::query()
    .where_exists(|sq| {
        sq.table("orders")
          .select(&["1"])
          .where_raw("orders.user_id = users.id", vec![])
    })
    .get(&pool)
    .await?;

// WHERE NOT EXISTS
let users_without_posts = User::query()
    .where_not_exists(|sq| {
        sq.table("posts")
          .select(&["1"])
          .where_raw("posts.user_id = users.id", vec![])
    })
    .get(&pool)
    .await?;

// CTE (WITH clause)
let result = User::query()
    .with_cte("ranked", |cte| {
        cte.table("users")
           .select_raw("*, ROW_NUMBER() OVER (ORDER BY created_at DESC) AS rn")
    })
    .from_cte("ranked")
    .where_raw("rn <= 10", vec![])
    .get(&pool)
    .await?;

// Nested subquery in FROM
let result = User::query()
    .from_subquery("active_users", |sq| {
        sq.table("users").filter("active", true)
    })
    .get(&pool)
    .await?;
```

---

## 14C.12 Window Functions

```rust
// Using select_raw with OVER clause
let posts = Post::query()
    .select_raw("*, ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at DESC) AS rn")
    .from_subquery("ranked", |sq| sq.table("posts").select_raw("*"))
    .where_raw("rn = 1", vec![])
    .get(&pool)
    .await?;

// Convenience method
let posts = Post::query()
    .window_rank_by("user_id", "created_at", "row_num")
    .having_rank(1)
    .get(&pool)
    .await?;

// Other window functions
let stats = User::query()
    .select_raw("*, COUNT(*) OVER (PARTITION BY role) as role_count")
    .get(&pool)
    .await?;

let users = User::query()
    .select_raw("*, LAG(created_at) OVER (ORDER BY created_at) as prev_login")
    .get(&pool)
    .await?;
```

---

## 14C.13 MSSQL Support (Phase 13.1)

```toml
# Cargo.toml
rok-orm = { version = "1.0", features = ["mssql"] }
```

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
let user = User::find_or_404(&pool, 1i64).await?;
User::create(&pool, &[("name", "Alice".into())]).await?;

// MSSQL-specific: TOP instead of LIMIT
let users = User::query().limit(10).get(&pool).await?;
```

---

## 14C.14 Redis Cache Integration (Phase 13.2)

```toml
rok-orm = { version = "1.0", features = ["postgres", "redis"] }
```

```rust
use rok_orm::cache::RedisCache;

#[derive(Model, sqlx::FromRow)]
#[model(table = "users", cache(ttl = 300, key_prefix = "users"))]
pub struct User {
    pub id: i64,
    pub name: String,
}

// Register cache driver at startup
rok_orm::cache::set_driver(RedisCache::new("redis://127.0.0.1:6379").await?);

// Find with cache - checks Redis first, falls back to DB
let user = User::find_cached(&pool, 1i64).await?;

// Invalidate cached record
User::invalidate_cache(1i64).await?;

// Cache a whole query result
let users = User::query()
    .filter("active", true)
    .cache_as("active_users", 60)
    .get(&pool)
    .await?;

// Invalidate all cache for this model
User::flush_cache().await?;
```

---

## 14C.15 Axum Integration (Phase 13.3)

```toml
rok-orm = { version = "1.0", features = ["postgres", "axum"] }
```

```rust
use axum::{
    extract::{Path, State},
    Json, Router,
};
use rok_orm::axum::{DbPool, OrmErrorResponse};
use sqlx::PgPool;

#[derive(Clone)]
struct AppState { db: PgPool }

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<User>, OrmErrorResponse> {
    let user = User::find_or_404(&state.db, id).await?;
    Ok(Json(user))
}

async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<User>, OrmErrorResponse> {
    let user = User::create_returning(&state.db, &[
        ("name", body.name.into()),
        ("email", body.email.into()),
    ]).await?;
    Ok(Json(user))
}

let app = Router::new()
    .route("/users/:id", get(get_user))
    .route("/users", post(create_user))
    .with_state(AppState { db: pool });
```

---

## 14C.16 Auto-Generated Models from Database

```rust
use rok_orm::schema::{ModelGenerator, inspector::inspect_table};

#[tokio::main]
async fn main() -> OrmResult<()> {
    let generator = ModelGenerator::from_pool(&pool)
        .tables(&["users", "posts", "comments"])
        .output_dir("src/models/")
        .with_derives(&["Debug", "Clone", "Serialize", "Deserialize"])
        .detect_timestamps(true)
        .detect_soft_delete(true);

    generator.generate().await?;

    Ok(())
}

// Generated output: src/models/user.rs
// use rok_orm::Model;
// use chrono::{DateTime, Utc};
// use serde::{Deserialize, Serialize};
//
// #[derive(Debug, Clone, Model, sqlx::FromRow, Serialize, Deserialize)]
// #[model(table = "users", timestamps, soft_delete)]
// pub struct User {
//     #[model(primary_key)]
//     pub id: i64,
//     pub name: String,
//     pub email: String,
//     pub created_at: Option<DateTime<Utc>>,
//     pub updated_at: Option<DateTime<Utc>>,
//     pub deleted_at: Option<DateTime<Utc>>,
// }
```

---

# Acceptance Criteria for Phase 14

- [x] 14A: Core Foundation examples (Phases 1-6) with Docker Compose
- [x] 14B: Rich Relationships & Developer Ergonomics (Phases 7-8) with Docker Compose
- [x] 14C: Advanced Features (Phases 9-13) with Docker Compose
- [ ] All examples compile without errors
- [ ] Each example includes usage context and SQL generated
- [ ] Docker Compose files for each phase group
- [ ] Cross-references to phase documentation