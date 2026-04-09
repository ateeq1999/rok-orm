//! Phase 14C: Advanced Features Examples
//! 
//! Demonstrates features from Phases 9-13:
//! - Schema Builder
//! - Migration System
//! - JSON Column Support
//! - Full-Text Search
//! - Sub-queries and CTEs
//! - Window Functions
//! - MSSQL Support
//! - Redis Cache Integration
//! - Axum Integration

use chrono::{DateTime, Utc};
use rok_orm::{
    model::Model,
    query::QueryBuilder,
    schema::{Schema, Blueprint, ForeignAction},
    errors::{OrmResult, OrmError},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::main;

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
    pub metadata: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub body: Option<String>,
    pub user_id: i64,
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
    tracing::info!("Connected to PostgreSQL");
    
    run_examples(&pool).await?;
    Ok(())
}

async fn run_examples(pool: &PgPool) -> OrmResult<()> {
    
    // Schema Builder - Create Table
    tracing::info!("=== Schema Builder: Create Table ===");
    
    Schema::create("example_users", |t: &mut Blueprint| {
        t.id();
        t.string("name", 255);
        t.string("email", 255).unique();
        t.json("metadata").nullable();
        t.timestamps();
    }).execute(pool).await?;
    
    tracing::info!("Created example_users table");
    
    // Schema Builder - Column Types
    tracing::info!("\n=== Schema Builder: Column Types ===");
    
    Schema::create("example_products", |t| {
        t.increments("id");
        t.big_increments("big_id");
        t.uuid("uuid_id");
        t.string("name", 255);
        t.text("description");
        t.integer("quantity");
        t.big_integer("price");
        t.float("score");
        t.boolean("active");
        t.date("birthday");
        t.datetime("published_at");
        t.json("data").nullable();
        t.enum_col("status", &["draft", "published"]);
    }).execute(pool).await?;
    
    tracing::info!("Created example_products");
    
    // Schema Builder - Alter Table
    tracing::info!("\n=== Schema Builder: Alter Table ===");
    
    Schema::alter("example_users", |t| {
        t.add_column("avatar_url", |c| c.string(500).nullable());
    }).execute(pool).await?;
    
    tracing::info!("Added avatar_url column");
    
    // Schema Inspection
    let exists = Schema::has_table(pool, "example_users").await?;
    let has_col = Schema::has_column(pool, "example_users", "email").await?;
    tracing::info!("Table exists: {}, has email: {}", exists, has_col);
    
    // JSON Column Support
    tracing::info!("\n=== JSON Column Support ===");
    
    sqlx::query("INSERT INTO example_users (name, email, metadata) VALUES (?, ?, ?)")
        .bind("JSON User")
        .bind("json@example.com")
        .bind(r#"{"role": "admin"}"#)
        .execute(pool)
        .await?;
    
    tracing::info!("Inserted JSON data");
    
    // Full-Text Search
    tracing::info!("\n=== Full-Text Search ===");
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_posts_fts ON posts USING gin(to_tsvector('english', title || ' ' COALESCE(body, '')))")
        .execute(pool)
        .await
        .ok();
    
    let posts = Post::query()
        .where_raw("to_tsvector('english', title) @@ to_tsquery('english', 'test')", vec![])
        .limit(10)
        .get(pool)
        .await?;
    
    tracing::info!("Full-text search: {} results", posts.len());
    
    // Sub-queries
    tracing::info!("\n=== Sub-queries ===");
    
    let users = User::query()
        .where_exists(|sq| {
            sq.table("posts").select(&["1"]).where_raw("posts.user_id = users.id", vec![])
        })
        .get(pool)
        .await?;
    
    tracing::info!("Users with posts: {}", users.len());
    
    // Window Functions
    tracing::info!("\n=== Window Functions ===");
    
    let users = User::query()
        .select_raw("*, ROW_NUMBER() OVER (ORDER BY id) as rn")
        .limit(10)
        .get(pool)
        .await?;
    
    tracing::info!("Window function query: {} users", users.len());
    
    // Cleanup
    tracing::info!("\n=== Cleanup ===");
    
    sqlx::query("DROP TABLE IF EXISTS example_products")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DROP TABLE IF EXISTS example_users")
        .execute(pool)
        .await
        .ok();
    
    tracing::info!("Cleanup completed");
    tracing::info!("\n=== Phase 14C Examples Completed ===");
    
    Ok(())
}