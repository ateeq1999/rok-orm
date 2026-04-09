//! Phase 14B: Rich Relationships & Developer Ergonomics Examples
//! 
//! Demonstrates features from Phases 7-8:
//! - ManyToMany with pivot access
//! - HasManyThrough, HasOneThrough
//! - Polymorphic relationships
//! - Relationship write operations
//! - whereHas / whereDoesntHave
//! - withCount / withSum / withAvg
//! - firstOrCreate / firstOrNew / updateOrCreate
//! - UUID / ULID primary keys
//! - Per-model database connections
//! - withoutTimestamps / Custom column names
//! - Model Pruning
//! - Event Muting
//! - when() / when_else() conditional chaining
//! - Raw expressions
//! - tap() / dd() debugging
//! - Chunking for large datasets
//! - Cursor pagination
//! - fill() and mass assignment
//! - Model Observers
//! - Global Query Scopes
//! - touches - Parent timestamp propagation

use chrono::{DateTime, Duration, Utc};
use rok_orm::{
    model::Model,
    query::{QueryBuilder, CountOp},
    relations::{
        HasMany, BelongsTo, BelongsToMany, 
        HasManyThrough, HasOneThrough,
        MorphOne, MorphMany, MorphToRef,
        MorphToMany, MorphedByMany,
    },
    pagination::Page,
    cursor::{CursorPage, CursorResult},
    observer::{ModelObserver},
    global_scope::GlobalScope,
    errors::{OrmResult, OrmError},
    Prunable, PrunableRegistry,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::main;

// ============================================================================
// Model Definitions
// ============================================================================

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
    pub active: bool,
    pub role: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub active: bool,
}

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
}

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub body: Option<String>,
    pub user_id: i64,
    pub country_id: Option<i64>,
    pub published: bool,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    #[model(primary_key)]
    pub id: i64,
    pub post_id: i64,
    pub body: String,
    pub published: bool,
}

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    #[model(primary_key)]
    pub id: i64,
    pub url: String,
    pub imageable_id: i64,
    pub imageable_type: String,
}

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
}

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
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

// ============================================================================
// Relationship Definitions
// ============================================================================

#[derive(rok_orm::Relations)]
pub struct UserRelations {
    #[belongs_to_many(
        target = "Role",
        pivot = "user_roles",
        fk = "user_id",
        rfk = "role_id",
        pivots = ["assigned_at", "expires_at"],
    )]
    pub roles: BelongsToMany<User, Role>,
    
    #[has_many(target = "Post")]
    pub posts: HasMany<User, Post>,
}

#[derive(rok_orm::Relations)]
pub struct CountryRelations {
    #[has_many_through(
        related = "Post",
        through = "User",
        first_key = "country_id",
        second_key = "user_id",
    )]
    pub posts: HasManyThrough<Country, User, Post>,
}

#[derive(rok_orm::Relations)]
pub struct PostRelations {
    #[belongs_to(target = "User")]
    pub user: BelongsTo<Post, User>,
    
    #[has_many(target = "Comment")]
    pub comments: HasMany<Post, Comment>,
    
    #[morph_many(related = "Image", morph_key = "imageable")]
    pub images: MorphMany<Post, Image>,
    
    #[morph_to_many(
        related = "Tag",
        pivot = "taggables",
        morph_key = "taggable",
    )]
    pub tags: MorphToMany<Post, Tag>,
}

#[derive(rok_orm::Relations)]
pub struct UserMorphRelations {
    #[morph_one(related = "Image", morph_key = "imageable")]
    pub image: MorphOne<User, Image>,
}

#[derive(rok_orm::Relations)]
pub struct ImageRelations {
    #[morph_to(morph_key = "imageable")]
    pub imageable: MorphToRef,
}

#[derive(rok_orm::Relations)]
pub struct TagRelations {
    #[morphed_by_many(
        related = "Post",
        pivot = "taggables",
        morph_key = "taggable",
    )]
    pub posts: MorphedByMany<Tag, Post>,
}

#[derive(rok_orm::Relations)]
pub struct CommentRelations {
    #[belongs_to(target = "Post")]
    pub post: BelongsTo<Comment, Post>,
}

// ============================================================================
// Global Scope Definition
// ============================================================================

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

// ============================================================================
// Model Observer Definition
// ============================================================================

pub struct UserObserver;

#[async_trait::async_trait]
impl ModelObserver for UserObserver {
    type Model = User;

    async fn creating(&self, user: &mut User) -> OrmResult<()> {
        user.email = user.email.to_lowercase();
        Ok(())
    }
    async fn created(&self, user: &User) -> OrmResult<()> {
        tracing::info!("UserObserver: created user {}", user.id);
        Ok(())
    }
    async fn updating(&self, user: &mut User) -> OrmResult<()> {
        Ok(())
    }
    async fn updated(&self, user: &User) -> OrmResult<()> {
        tracing::info!("UserObserver: updated user {}", user.id);
        Ok(())
    }
    async fn saving(&self, user: &mut User) -> OrmResult<()> { Ok(()) }
    async fn saved(&self, user: &User) -> OrmResult<()> { Ok(()) }
    async fn deleting(&self, user: &User) -> OrmResult<()> { Ok(()) }
    async fn deleted(&self, user: &User) -> OrmResult<()> {
        tracing::info!("UserObserver: deleted user {}", user.id);
        Ok(())
    }
    async fn restoring(&self, user: &User) -> OrmResult<()> { Ok(()) }
    async fn restored(&self, user: &User) -> OrmResult<()> { Ok(()) }
}

// ============================================================================
// Main Application
// ============================================================================

#[main]
async fn main() -> OrmResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter("rok_orm_examples=debug,sqlx=warn")
        .init();
    
    dotenv::dotenv().ok();
    
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://rok:rokpass@localhost:5432/rok_orm_examples".to_string());
    
    let pool = PgPool::connect(&db_url).await?;
    tracing::info!("Connected to database");
    
    run_examples(&pool).await?;
    
    Ok(())
}

async fn run_examples(pool: &PgPool) -> OrmResult<()> {
    
    // -------------------------------------------------------------------------
    // Example 1: Many-to-Many with Pivot Access
    // -------------------------------------------------------------------------
    tracing::info!("=== Example 1: ManyToMany with Pivot ===");
    
    // This would require the pivot table setup
    // user.roles().attach(&pool, role_id).await?;
    // user.roles().sync(&pool, vec![1i64, 2, 3]).await?;
    // user.roles().toggle(&pool, vec![1i64, 2]).await?;
    
    // With pivot columns
    // let roles = user.roles()
    //     .with_pivot(&["assigned_at", "expires_at"])
    //     .get(&pool)
    //     .await?;
    
    tracing::info!("ManyToMany example - see documentation for full API");
    
    // -------------------------------------------------------------------------
    // Example 2: Has-Many-Through
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 2: HasManyThrough ===");
    
    // Country has many Posts through User
    // let posts = country.posts()
    //     .filter("published", true)
    //     .get(&pool)
    //     .await?;
    
    tracing::info!("HasManyThrough example - see documentation");
    
    // -------------------------------------------------------------------------
    // Example 3: whereHas / whereDoesntHave
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 3: whereHas / whereDoesntHave ===");
    
    // Posts with at least one published comment
    let posts_with_comments = Post::query()
        .where_has("comments", |q| q.filter("published", true))
        .limit(5)
        .get(pool)
        .await?;
    
    tracing::info!("Posts with published comments: {}", posts_with_comments.len());
    
    // Posts with more than 5 comments
    let popular_posts = Post::query()
        .where_has_count("comments", 5, CountOp::GreaterThan)
        .get(pool)
        .await?;
    
    tracing::info!("Posts with >5 comments: {}", popular_posts.len());
    
    // Users with no posts
    let users_without_posts = User::query()
        .where_doesnt_have("posts")
        .get(pool)
        .await?;
    
    tracing::info!("Users with no posts: {}", users_without_posts.len());
    
    // -------------------------------------------------------------------------
    // Example 4: withCount / withSum / withAvg
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 4: withCount / withSum ===");
    
    let posts_with_counts = Post::query()
        .with_count("comments")
        .with_count_as("published_comments", "comments", |q| q.filter("published", true))
        .limit(5)
        .get(pool)
        .await?;
    
    for post in &posts_with_counts {
        let total = post.extras.get("comments_count")
            .map(|v| format!("{:?}", v))
            .unwrap_or_else(|| "N/A".to_string());
        tracing::info!("Post {} comments: {}", post.id, total);
    }
    
    // -------------------------------------------------------------------------
    // Example 5: firstOrCreate / firstOrNew / updateOrCreate
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 5: firstOrCreate ===");
    
    let user = User::first_or_create(pool,
        &[("email", "newuser@example.com".into())],
        &[("name", "New User".into()), ("role", "user".into())],
    ).await;
    
    match user {
        Ok(u) => tracing::info!("first_or_create: user {} ({})", u.name, u.id),
        Err(e) => tracing::warn!("first_or_create failed: {:?}", e),
    }
    
    // first_or_new (no DB write)
    let new_user = User::first_or_new(
        &[("email", "another@example.com".into())],
        &[("name", "Another")],
    );
    tracing::info!("first_or_new created: {:?}", new_user.email);
    
    // update_or_create
    let updated = User::update_or_create(pool,
        &[("email", "admin@example.com".into())],
        &[("name", "Admin Updated".into())],
    ).await;
    
    match updated {
        Ok(u) => tracing::info!("update_or_create: {} (id={})", u.name, u.id),
        Err(e) => tracing::warn!("update_or_create: {:?}", e),
    }
    
    // -------------------------------------------------------------------------
    // Example 6: UUID / ULID Primary Keys
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 6: UUID/ULID (requires feature flag) ===");
    
    // Requires: #[model(table = "articles", uuid)]
    // let article = Article::create_returning(&pool, &[
    //     ("title", "My Article".into()),
    // ]).await?;
    // tracing::info!("Created with UUID: {}", article.id);
    
    tracing::info!("UUID/ULID - see documentation for setup");
    
    // -------------------------------------------------------------------------
    // Example 7: withoutTimestamps / Custom Column Names
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 7: withoutTimestamps ===");
    
    // Suppress timestamps for one call
    User::without_timestamps(|| async {
        User::update_by_pk(pool, 1, &[("active", true.into())]).await
    }).await?;
    
    tracing::info!("without_timestamps completed");
    
    // -------------------------------------------------------------------------
    // Example 8: Model Pruning
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 8: Model Pruning ===");
    
    // Register for batch pruning
    PrunableRegistry::register::<ActivityLog>();
    
    // Prune old logs
    // let deleted = ActivityLog::prune(&pool).await?;
    // tracing::info!("Pruned {} old activity logs", deleted);
    
    tracing::info!("Pruning example - see documentation");
    
    // -------------------------------------------------------------------------
    // Example 9: Event Muting
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 9: Event Muting ===");
    
    // Suppress all events in a block
    User::without_events(|| async {
        User::create(pool, &[
            ("name", "Silent User".into()),
            ("email", "silent@example.com".into()),
            ("active", true.into()),
            ("role", "user".into()),
        ]).await
    }).await?;
    
    tracing::info!("Event muting completed");
    
    // -------------------------------------------------------------------------
    // Example 10: when() / when_else() Conditional Chaining
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 10: when() / when_else() ===");
    
    let role_filter = Some("admin".to_string());
    let active_only = true;
    let search_term = Some("test".to_string());
    
    let users = User::query()
        .when(role_filter.is_some(), |q| {
            q.filter("role", role_filter.clone().unwrap())
        })
        .when(active_only, |q| q.filter("active", true))
        .when(search_term.is_some(), |q| {
            q.where_like("name", &format!("%{}%", search_term.clone().unwrap()))
        })
        .limit(10)
        .get(pool)
        .await?;
    
    tracing::info!("Conditional query returned {} users", users.len());
    
    // With else branch
    let admin_filter = false;
    let users = User::query()
        .when_else(
            admin_filter,
            |q| q.filter("role", "admin"),
            |q| q.filter("role", "user"),
        )
        .get(pool)
        .await?;
    
    tracing::info!("when_else query returned {} users", users.len());
    
    // -------------------------------------------------------------------------
    // Example 11: Raw Expressions
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 11: Raw Expressions ===");
    
    let users = User::query()
        .where_raw("LOWER(email) = LOWER($1)", vec!["Admin@Example.com".into()])
        .get(pool)
        .await?;
    
    tracing::info!("Raw WHERE query returned {} users", users.len());
    
    let users = User::query()
        .select_raw("id, name, UPPER(name) as name_upper")
        .limit(5)
        .get(pool)
        .await?;
    
    tracing::info!("Raw SELECT query returned {} users", users.len());
    
    // Raw ORDER BY
    let users = User::query()
        .order_raw("FIELD(role, 'admin', 'moderator', 'user')")
        .get(pool)
        .await?;
    
    tracing::info!("Raw ORDER BY query returned {} users", users.len());
    
    // -------------------------------------------------------------------------
    // Example 12: tap() and dd() Debugging
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 12: tap() / dd() ===");
    
    let users = User::query()
        .filter("active", true)
        .tap(|q| {
            let (sql, _) = q.to_sql();
            tracing::debug!("Before limit: {}", sql);
        })
        .limit(10)
        .get(pool)
        .await?;
    
    tracing::info!("tap() example - checked query without modification");
    
    // dd() only works in debug builds
    // #[cfg(debug_assertions)]
    // User::query().filter("active", true).dd();
    
    // -------------------------------------------------------------------------
    // Example 13: Chunking for Large Datasets
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 13: Chunking ===");
    
    // Chunk with LIMIT/OFFSET
    // User::query()
    //     .chunk(&pool, 100, |batch| async move {
    //         for user in batch {
    //             process(&user).await;
    //         }
    //         Ok(())
    //     })
    //     .await?;
    
    // chunk_by_id - stable even if rows are deleted
    // User::query()
    //     .chunk_by_id(&pool, 100, |batch| async move {
    //         process(batch).await
    //     })
    //     .await?;
    
    tracing::info!("Chunking example - see documentation");
    
    // -------------------------------------------------------------------------
    // Example 14: Cursor Pagination
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 14: Cursor Pagination ===");
    
    // First page
    let result = Post::query()
        .order_by_desc("created_at")
        .cursor_paginate(pool, CursorPage { after: None, limit: 5 })
        .await?;
    
    tracing::info!("First page: {} items, has_more: {}", result.data.len(), result.has_more);
    tracing::info!("Next cursor: {:?}", result.next_cursor);
    
    // Next page
    if let Some(cursor) = &result.next_cursor {
        let next = Post::query()
            .order_by_desc("created_at")
            .cursor_paginate(pool, CursorPage { after: Some(cursor.clone()), limit: 5 })
            .await?;
        
        tracing::info!("Next page: {} items, has_more: {}", next.data.len(), next.has_more);
    }
    
    // -------------------------------------------------------------------------
    // Example 15: fill() and Mass Assignment Protection
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 15: Mass Assignment ===");
    
    // Note: fillable model requires proper setup in derive macro
    // This is the API usage pattern
    let data = vec![
        ("name", "Fillable User".into()),
        ("email", "fillable@example.com".into()),
        ("role", "admin".into()),   // This would be ignored if not in fillable
        ("active", true.into()),
    ];
    
    // The fillable filter is applied during create/update
    tracing::info!("Mass assignment example - data prepared");
    
    // -------------------------------------------------------------------------
    // Example 16: Model Observers
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 16: Model Observers ===");
    
    // Register observer
    User::observe(UserObserver);
    
    // Now all User operations will trigger observer callbacks
    let _ = User::create(pool, &[
        ("name", "Observed User".into()),
        ("email", "observed@example.com".into()),
        ("active", true.into()),
        ("role", "user".into()),
    ]).await;
    
    tracing::info!("Model observer registered and triggered");
    
    // -------------------------------------------------------------------------
    // Example 17: Global Query Scopes
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 17: Global Query Scopes ===");
    
    // Register global scope
    User::add_global_scope(ActiveScope);
    
    // All queries automatically include WHERE active = true
    let active_users = User::all(pool).await?;
    tracing::info!("Global scope applied: {} active users", active_users.len());
    
    // Opt out per query
    let all_users = User::query()
        .without_global_scope::<ActiveScope>()
        .get(pool)
        .await?;
    
    tracing::info!("Without scope: {} total users", all_users.len());
    
    // Remove permanently
    // User::remove_global_scope::<ActiveScope>();
    
    // -------------------------------------------------------------------------
    // Example 18: touches - Parent Timestamp Propagation
    // -------------------------------------------------------------------------
    tracing::info!("\n=== Example 18: touches ===");
    
    // Requires: #[model(touches = ["post"])] on Comment model
    // After comment update, posts.updated_at is also set to NOW()
    // Comment::update_by_pk(&pool, comment_id, &[("body", "edited".into())]).await?;
    
    tracing::info!("touches example - see documentation");
    
    // -------------------------------------------------------------------------
    tracing::info!("\n=== All Phase 14B Examples Completed ===");
    
    Ok(())
}