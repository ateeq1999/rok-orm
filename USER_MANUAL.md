# rok-orm User Manual

> A comprehensive guide to building database-driven applications with rok-orm.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Defining Models](#defining-models)
3. [Query Builder](#query-builder)
4. [CRUD Operations](#crud-operations)
5. [Relationships](#relationships)
6. [Transactions](#transactions)
7. [Configuration](#configuration)
8. [Best Practices](#best-practices)
9. [Troubleshooting](#troubleshooting)

---

## Getting Started

### Installation

Add rok-orm to your `Cargo.toml`:

```toml
[dependencies]
# For PostgreSQL
rok-orm = { version = "0.2", features = ["postgres"] }
sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "postgres"] }

# For SQLite
rok-orm = { version = "0.2", features = ["sqlite"] }
sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "sqlite"] }
```

### Basic Setup

```rust
use rok_orm::{Model, PgModel};
use sqlx::PgPool;

#[derive(Model, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connection pool
    let pool = PgPool::connect("postgres://user:pass@localhost/mydb").await?;

    // Use the model
    let users = User::all(&pool).await?;
    println!("Found {} users", users.len());

    Ok(())
}
```

---

## Defining Models

### Basic Model

```rust
use serde::{Deserialize, Serialize};
use rok_orm::Model;

#[derive(Model, sqlx::FromRow, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub active: bool,
}
```

**Auto-generated:**
- `table_name()` → `"users"` (struct name + "s" in snake_case)
- `primary_key()` → `"id"`
- `columns()` → `&["id", "name", "email", "active"]`

### Custom Table Name

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "blog_posts")]  // Custom table name
pub struct BlogPost {
    pub id: i64,
    pub title: String,
    pub content: String,
}
```

### Custom Primary Key

```rust
// Struct-level override
#[derive(Model, sqlx::FromRow)]
#[model(primary_key = "user_id")]
pub struct Profile {
    pub user_id: i64,
    pub bio: String,
}

// Field-level override
#[derive(Model, sqlx::FromRow)]
pub struct Post {
    #[model(primary_key)]
    pub post_id: i64,
    pub title: String,
}
```

### Field Attributes

```rust
#[derive(Model, sqlx::FromRow)]
pub struct Article {
    #[model(primary_key)]
    pub id: i64,

    #[model(column = "article_title")]  // Map to different column
    pub title: String,

    #[model(skip)]  // Exclude from columns
    pub cache_data: String,

    pub content: String,
    pub created_at: String,
}
```

### Soft Deletes

```rust
#[derive(Model, sqlx::FromRow)]
#[model(soft_delete)]  // Adds deleted_at filtering
pub struct Post {
    pub id: i64,
    pub title: String,
    pub deleted_at: Option<String>,  // Required column
}
```

### Auto Timestamps

```rust
#[derive(Model, sqlx::FromRow)]
#[model(timestamps)]  // Auto-manage created_at/updated_at
pub struct User {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}
```

---

## Query Builder

### Starting a Query

```rust
use rok_orm::{QueryBuilder, SqlValue};

// From model
let query = User::query();

// From scratch
let query = QueryBuilder::<User>::new("users");
```

### WHERE Conditions

```rust
// Shorthand (recommended)
User::query()
    .filter("active", true)
    .filter("role", "admin")
    .get(&pool)
    .await?;

// Explicit conditions
User::query()
    .where_eq("email", "admin@example.com")
    .where_ne("status", "banned")
    .where_gt("age", 18)
    .where_gte("score", 100)
    .where_lt("price", 50.00)
    .where_lte("quantity", 10)
    .get(&pool)
    .await?;
```

### String Conditions

```rust
User::query()
    .where_like("name", "%John%")      // LIKE '%John%'
    .where_not_like("email", "%spam%")
    .get(&pool)
    .await?;
```

### NULL Conditions

```rust
User::query()
    .where_null("deleted_at")        // WHERE deleted_at IS NULL
    .where_not_null("email")          // WHERE email IS NOT NULL
    .get(&pool)
    .await?;
```

### IN and BETWEEN

```rust
// IN clause
User::query()
    .where_in("role", vec!["admin", "moderator", "editor"])
    .get(&pool)
    .await?;

// NOT IN
User::query()
    .where_not_in("status", vec!["banned", "deleted"])
    .get(&pool)
    .await?;

// BETWEEN
Post::query()
    .where_between("price", 10.0, 100.0)
    .get(&pool)
    .await?;
```

### OR Conditions

```rust
User::query()
    .filter("role", "admin")
    .or_where_eq("role", "moderator")
    .or_where_eq("verified", true)
    .get(&pool)
    .await?;
```

### Ordering and Pagination

```rust
User::query()
    .order_by("name")           // ASC (default)
    .order_by_desc("created_at") // DESC
    .limit(10)
    .offset(20)
    .get(&pool)
    .await?;
```

### Selecting Columns

```rust
User::query()
    .select(&["id", "name", "email"])  // SELECT id, name, email
    .get(&pool)
    .await?;
```

### DISTINCT

```rust
User::query()
    .distinct()
    .select(&["email"])
    .get(&pool)
    .await?;
```

### Joins

```rust
Post::query()
    .inner_join("users", "users.id = posts.user_id")
    .left_join("categories", "categories.id = posts.category_id")
    .where_eq("users.active", true)
    .get(&pool)
    .await?;
```

### Grouping and Having

```rust
User::query()
    .select(&["role", "COUNT(*) as count"])
    .group_by(&["role"])
    .having("COUNT(*) > 5")
    .get(&pool)
    .await?;
```

### Using the `query!` Macro

```rust
use rok_orm_macros::query;

let q = query!(User,
    where_eq "active" true,
    where_in "role" vec!["admin", "user"],
    order_by_desc "created_at",
    limit 10,
    offset 0
);

let users = User::find_where(&pool, q).await?;
```

---

## CRUD Operations

### Create

```rust
// Insert and get rows affected
User::create(&pool, &[
    ("name", "Alice".into()),
    ("email", "alice@example.com".into()),
    ("active", true.into()),
]).await?;

// Insert and get the created row
let user: User = User::create_returning(&pool, &[
    ("name", "Alice".into()),
    ("email", "alice@example.com".into()),
]).await?;

println!("Created user with id: {}", user.id);
```

### Read

```rust
// All records
let users: Vec<User> = User::all(&pool).await?;

// Find by primary key
let user: Option<User> = User::find_by_pk(&pool, 1).await?;

// Find or fail with error
let user = User::find_or_404(&pool, 1).await?;

// First record
let user: Option<User> = User::first(&pool).await?;

// First or fail
let user = User::first_or_404(&pool).await?;

// Custom query
let admins: Vec<User> = User::find_where(
    &pool,
    User::query()
        .filter("role", "admin")
        .order_by_desc("created_at")
).await?;
```

### Update

```rust
// Update by primary key
User::update_by_pk(&pool, 1, &[
    ("name", "Bob".into()),
    ("email", "bob@example.com".into()),
]).await?;

// Update with custom WHERE
User::update_where(
    &pool,
    User::query().filter("role", "guest"),
    &[("active", false.into())],
).await?;
```

### Delete

```rust
// Delete by primary key
User::delete_by_pk(&pool, 1).await?;

// Delete with custom WHERE
User::delete_where(
    &pool,
    User::query().filter("active", false),
).await?;
```

### Count

```rust
let total: i64 = User::count(&pool).await?;

let admin_count: i64 = User::count_where(
    &pool,
    User::query().filter("role", "admin"),
).await?;
```

### Bulk Operations

```rust
// Bulk insert
User::bulk_create(&pool, &[
    vec![
        ("name", "Alice".into()),
        ("email", "alice@example.com".into()),
    ],
    vec![
        ("name", "Bob".into()),
        ("email", "bob@example.com".into()),
    ],
]).await?;
```

---

## Relationships

### Define Relationships

```rust
use rok_orm::{Model, Relations};

#[derive(Model, sqlx::FromRow)]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
}

#[derive(Model, sqlx::FromRow)]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub user_id: i64,      // Foreign key
    pub title: String,
    pub body: String,
}

#[derive(Model, Relations, sqlx::FromRow)]
pub struct Profile {
    #[model(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub bio: String,

    #[model(belongs_to(User))]
    pub user: User,
}
```

### Using Relationships

```rust
// HasMany: User has many Posts
let user = User::find_or_404(&pool, 1).await?;
let posts: Vec<Post> = Post::find_where(
    &pool,
    Post::query().filter("user_id", user.id)
).await?;

// BelongsTo: Profile belongs to User
let profile = Profile::find_or_404(&pool, 1).await?;
let user = User::find_by_pk(&pool, profile.user_id).await?;
```

---

## Transactions

### Basic Transaction

```rust
use rok_orm::{Tx, PgModel};

let mut tx = Tx::begin(&pool).await?;

tx.insert::<User>("users", &[
    ("name", "Alice".into()),
    ("email", "alice@example.com".into()),
]).await?;

tx.insert::<Post>("posts", &[
    ("user_id", 1i64.into()),
    ("title", "Hello".into()),
    ("body", "World".into()),
]).await?;

tx.commit().await?;
```

### Automatic Rollback

```rust
// If commit() is not called, transaction rolls back automatically
{
    let mut tx = Tx::begin(&pool).await?;
    
    // Operations...
    
    // Don't call commit() - will rollback on drop
}
```

### Transaction with Result

```rust
let result: Result<(User, Post), sqlx::Error> = async {
    let mut tx = Tx::begin(&pool).await?;
    
    let user: User = tx.insert_returning::<User>("users", &[
        ("name", "Alice".into()),
        ("email", "alice@example.com".into()),
    ]).await?;
    
    let post: Post = tx.insert_returning::<Post>("posts", &[
        ("user_id", user.id.into()),
        ("title", "My Post".into()),
        ("body", "Content".into()),
    ]).await?;
    
    tx.commit().await?;
    
    Ok((user, post))
}.await;
```

---

## Configuration

### Environment Variables

```bash
# .env
DATABASE_URL=postgres://user:pass@localhost/mydb
```

### Connection Pool

```rust
let pool = PgPoolOptions::new()
    .max_connections(10)
    .min_connections(2)
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .connect(&std::env::var("DATABASE_URL")?)
    .await?;
```

### Feature Flags

```toml
[dependencies]
rok-orm = { version = "0.2", default-features = false, features = ["postgres"] }
```

---

## Best Practices

### 1. Use Explicit Conditions

```rust
// ✅ Good: Clear intent
User::query()
    .filter("status", "active")
    .order_by_desc("created_at")
    .limit(20)
    .get(&pool)
    .await?;

// ⚠️ Avoid: Implicit boolean
User::query()
    .filter("active", true)  // Clear
    // vs
    .filter("active", 1)     // Magic number
```

### 2. Index Your Queries

```sql
-- Add indexes for frequently queried columns
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_posts_user_id ON posts(user_id);
CREATE INDEX idx_posts_created_at ON posts(created_at);
```

### 3. Use Pagination for Large Datasets

```rust
// ✅ Good: Paginated
let users = User::query()
    .order_by_desc("created_at")
    .limit(50)
    .offset(offset)
    .get(&pool)
    .await?;

// ⚠️ Avoid: Loading everything
let users = User::all(&pool).await?;  // May be millions!
```

### 4. Handle Errors Gracefully

```rust
match User::find_by_pk(&pool, id).await {
    Ok(Some(user)) => Ok(Json(user)),
    Ok(None) => Err(StatusCode::NOT_FOUND),
    Err(e) => {
        tracing::error!("Database error: {:?}", e);
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
```

### 5. Use Transactions for Related Operations

```rust
// ✅ Good: Atomic operation
let mut tx = Tx::begin(&pool).await?;
tx.insert::<User>(...)?;
tx.insert::<Profile>(...)?;
tx.commit().await?;

// ⚠️ Avoid: Partial failures
User::create(&pool, &user_data).await?;
// If Profile::create fails, User is already created!
```

---

## Troubleshooting

### Common Errors

#### "Row not found"

```rust
// ⚠️ Error
let user = User::find_by_pk(&pool, 999).await?;
// user is None

// ✅ Fix: Handle Option
match User::find_by_pk(&pool, 999).await? {
    Some(user) => Ok(user),
    None => Err(StatusCode::NOT_FOUND),
}
```

#### "Relation does not exist"

```rust
// Check your table names match
#[derive(Model)]
#[model(table = "users")]  // Must match actual table name
pub struct User { ... }
```

#### "Column not found"

```rust
// Ensure column names match database
// Database: created_at
// Model:   created_at (✅) vs createdAt (❌)
```

### Debug SQL Generation

```rust
// Print generated SQL
let query = User::query()
    .filter("active", true)
    .limit(10);

let (sql, params) = query.to_sql();
println!("SQL: {}", sql);
println!("Params: {:?}", params);
```

### Enable Logging

```rust
// In your main.rs
env_logger::Builder::from_env(
    env_logger::Env::default().default_filter_or("info")
).init();

// Or with tracing
tracing_subscriber::fmt()
    .with_env_filter("rok_orm=debug")
    .init();
```

---

## Examples

### User Registration

```rust
#[derive(Model, sqlx::FromRow, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub active: bool,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

async fn register(
    pool: &PgPool,
    body: Json<RegisterRequest>,
) -> Result<Created<Json<User>>, StatusCode> {
    // Check if email exists
    if User::find_where(
        &pool,
        User::query().filter("email", &body.email),
    ).await?.len() > 0 {
        return Err(StatusCode::CONFLICT);
    }

    // Hash password
    let hash = hash_password(&body.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create user
    let user: User = User::create_returning(&pool, &[
        ("email", body.email.clone().into()),
        ("password_hash", hash.into()),
        ("active", false.into()),
    ]).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Created::new("/users").body(Json(user)))
}
```

### Blog with Posts and Comments

```rust
#[derive(Model, sqlx::FromRow)]
pub struct Post {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: String,
    pub published: bool,
    pub created_at: String,
}

#[derive(Model, sqlx::FromRow)]
pub struct Comment {
    pub id: i64,
    pub post_id: i64,
    pub user_id: i64,
    pub content: String,
    pub created_at: String,
}

async fn get_post_with_comments(
    pool: &PgPool,
    post_id: i64,
) -> Result<Json<PostWithComments>, StatusCode> {
    // Get post
    let post = Post::find_or_404(&pool, post_id).await?;

    // Get comments
    let comments = Comment::find_where(
        &pool,
        Comment::query()
            .filter("post_id", post_id)
            .order_by_desc("created_at")
    ).await?;

    Ok(Json(PostWithComments { post, comments }))
}

#[derive(Serialize)]
pub struct PostWithComments {
    post: Post,
    comments: Vec<Comment>,
}
```

### Dashboard Statistics

```rust
async fn dashboard_stats(pool: &PgPool) -> Result<Json<DashboardStats>, StatusCode> {
    let total_users: i64 = User::count(&pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_users: i64 = User::count_where(
        &pool,
        User::query().filter("active", true)
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_posts: i64 = Post::count(&pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let published_posts: i64 = Post::count_where(
        &pool,
        Post::query().filter("published", true)
    ).await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(DashboardStats {
        total_users,
        active_users,
        total_posts,
        published_posts,
    }))
}

#[derive(Serialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub active_users: i64,
    pub total_posts: i64,
    pub published_posts: i64,
}
```

---

## API Reference

### Model Trait

```rust
pub trait Model: Sized {
    fn table_name() -> &'static str;
    fn primary_key() -> &'static str;
    fn columns() -> &'static [&'static str];
    fn soft_delete_column() -> Option<&'static str>;
    fn timestamps_enabled() -> bool;
    fn query() -> QueryBuilder<Self>;
    fn find(id: impl Into<SqlValue>) -> QueryBuilder<Self>;
}
```

### PgModel Trait

```rust
pub trait PgModel: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin {
    fn all(pool: &PgPool) -> impl Future<Output = Result<Vec<Self>, sqlx::Error>>;
    fn find_by_pk(pool: &PgPool, id: impl Into<SqlValue>) -> impl Future<Output = Result<Option<Self>, sqlx::Error>>;
    async fn find_or_404(pool: &PgPool, id: impl Into<SqlValue>) -> Result<Self, sqlx::Error>;
    async fn first(pool: &PgPool) -> Result<Option<Self>, sqlx::Error>;
    async fn first_or_404(pool: &PgPool) -> Result<Self, sqlx::Error>;
    async fn get(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error>;
    async fn count(pool: &PgPool) -> Result<i64, sqlx::Error>;
    fn create(pool: &PgPool, data: &[(&str, SqlValue)]) -> impl Future<Output = Result<u64, sqlx::Error>>;
    fn create_returning(pool: &PgPool, data: &[(&str, SqlValue)]) -> impl Future<Output = Result<Self, sqlx::Error>>;
    fn update_by_pk(pool: &PgPool, id: impl Into<SqlValue>, data: &[(&str, SqlValue)]) -> impl Future<Output = Result<u64, sqlx::Error>>;
    fn delete_by_pk(pool: &PgPool, id: impl Into<SqlValue>) -> impl Future<Output = Result<u64, sqlx::Error>>;
    fn bulk_create(pool: &PgPool, rows: &[Vec<(&str, SqlValue)>]) -> impl Future<Output = Result<u64, sqlx::Error>>;
}
```

---

## License

rok-orm is dual-licensed under MIT or Apache-2.0.
