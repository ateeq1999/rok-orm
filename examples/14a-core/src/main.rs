//! Phase 14A: Core Foundation Examples
//! 
//! Demonstrates features from Phases 1-6:
//! - Basic model definition
//! - Query builder
//! - CRUD operations
//! - Basic relationships
//! - Soft deletes
//! - Auto timestamps
//! - Pagination
//! - Aggregations
//! - Model hooks
//! - Transactions
//! - Query scopes
//! - Query logging

use chrono::{DateTime, Duration, Utc};
use rok_orm::{
    model::Model,
    query::QueryBuilder,
    relations::{HasMany, BelongsTo},
    pagination::Page,
    hooks::ModelHooks,
    errors::{OrmResult, OrmError},
    logging::{Logger, LogLevel},
    PgModel,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::main;

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
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

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "posts", timestamps, soft_delete)]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub body: Option<String>,
    pub user_id: i64,
    pub published: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    #[model(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub total: f64,
    pub status: String,
}

#[derive(rok_orm::Relations)]
pub struct UserRelations {
    #[has_many(target = "Post")]
    pub posts: HasMany<User, Post>,
}

#[derive(rok_orm::Relations)]
pub struct PostRelations {
    #[belongs_to(target = "User")]
    pub user: BelongsTo<Post, User>,
}

impl User {
    pub fn active() -> QueryBuilder<User> {
        User::query().filter("active", true)
    }
    
    pub fn recent(days: i64) -> QueryBuilder<User> {
        let cutoff = Utc::now() - Duration::days(days);
        User::query().where_gt("created_at", cutoff)
    }
}

pub struct UserHooks;

#[async_trait::async_trait]
impl ModelHooks<User> for UserHooks {
    async fn before_create(user: &mut User) -> OrmResult<()> {
        user.email = user.email.to_lowercase();
        Ok(())
    }
    async fn after_create(user: &User) -> OrmResult<()> {
        tracing::info!("Created user {}", user.id);
        Ok(())
    }
    async fn before_update(user: &mut User) -> OrmResult<()> { Ok(()) }
    async fn after_update(user: &User) -> OrmResult<()> { Ok(()) }
    async fn before_delete(user: &User) -> OrmResult<()> { Ok(()) }
    async fn after_delete(user: &User) -> OrmResult<()> { Ok(()) }
}

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
    // Basic model
    assert_eq!(User::table_name(), "users");
    tracing::info!("Table: {}, Columns: {:?}", User::table_name(), User::columns());
    
    // Query builder
    let (sql, params) = User::query()
        .filter("active", true)
        .order_by_desc("created_at")
        .limit(10)
        .to_sql();
    tracing::info!("SQL: {}", sql);
    
    // CRUD
    User::create(pool, &[
        ("name", "Alice".into()),
        ("email", "alice@example.com".into()),
        ("active", true.into()),
    ]).await?;
    
    let user = User::create_returning(pool, &[
        ("name", "Bob".into()),
        ("email", "bob@example.com".into()),
        ("active", true.into()),
    ]).await?;
    tracing::info!("Created user id={}", user.id);
    
    // Relationships
    let users = User::query().with("posts").limit(5).get(pool).await?;
    for u in &users {
        tracing::info!("User {} has {} posts", u.name, u.posts.len());
    }
    
    // Soft deletes
    let posts = Post::all(pool).await?;
    let all = Post::with_soft_delete().get(pool).await?;
    let trashed = Post::only_trashed().get(pool).await?;
    tracing!("Active: {}, All: {}, Trashed: {}", posts.len(), all.len(), trashed.len());
    
    // Timestamps
    let post = Post::create_returning(pool, &[
        ("title", "Test".into()),
        ("user_id", 1i64.into()),
        ("published", false.into()),
    ]).await?;
    tracing::info!("Created at: {:?}", post.created_at);
    
    // Pagination
    let page: Page<Post> = Post::paginate(pool, 1, 5).await?;
    tracing::info!("Page {}/{}", page.current_page, page.last_page);
    
    // Aggregations
    let total: i64 = User::count(pool).await?;
    tracing::info!("Total users: {}", total);
    
    let revenue: f64 = Order::sum("total", pool).await?;
    tracing::info!("Total revenue: ${:.2}", revenue);
    
    // Transactions
    use rok_orm::Tx;
    let mut tx = Tx::begin(pool).await?;
    tx.insert::<User>("users", &[
        ("name", "Tx User".into()),
        ("email", "tx@example.com".into()),
        ("active", true.into()),
    ]).await?;
    tx.commit().await?;
    
    // Scopes
    let active = User::active().get(pool).await?;
    let recent = User::recent(30).get(pool).await?;
    tracing::info!("Active: {}, Recent: {}", active.len(), recent.len());
    
    // Logging
    let logger = Logger::new().with_log_level(LogLevel::Debug);
    let start = std::time::Instant::now();
    let _ = User::all(pool).await?;
    tracing::info!("Query took {}ms", start.elapsed().as_millis());
    
    tracing::info!("\n=== Phase 14A Examples Completed ===");
    Ok(())
}