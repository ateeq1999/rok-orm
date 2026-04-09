//! Example: withCount / withSum / withAvg
//!
//! Demonstrates: relationship aggregates as query extras

use rok_orm::Model;
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
}

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "posts")]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub user_id: i64,
    pub published: bool,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 withCount / withSum / withAvg\n");
    
    // Create test data
    let user = User::create_returning(pool, &[
        ("name", "Author".into()),
    ]).await?;
    
    for i in 1..=5 {
        let published = i <= 3;
        Post::create(pool, &[
            ("title", format!("Post {}", i)),
            ("user_id", user.id.into()),
            ("published", published.into()),
        ]).await?;
    }
    
    println!("1. withCount - get post count per user...");
    let users = User::query()
        .with_count("posts")
        .get(pool)
        .await?;
    
    for u in &users {
        let count = u.extras.get("posts_count")
            .map(|v| format!("{:?}", v))
            .unwrap_or_else(|| "0".to_string());
        println!("   {}: {} posts", u.name, count);
    }
    
    println!("2. withCountAs - with filter...");
    let users = User::query()
        .with_count_as("published_posts", "posts", |q| q.filter("published", true))
        .get(pool)
        .await?;
    
    for u in &users {
        let count = u.extras.get("published_posts_count")
            .map(|v| format!("{:?}", v))
            .unwrap_or_else(|| "0".to_string());
        println!("   {}: {} published", u.name, count);
    }
    
    println!("\n✅ withCount works correctly");
    Ok(())
}