//! Example 5: Auto Timestamps
//! 
//! Demonstrates: timestamps attribute, created_at, updated_at

use rok_orm::Model;
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "articles", timestamps)]
pub struct Article {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Auto Timestamps\n");
    
    // Create - automatically adds created_at and updated_at
    println!("1. Create article (auto timestamps)...");
    let article = Article::create_returning(pool, &[
        ("title", "My Article".into()),
    ]).await?;
    
    if let Some(created) = article.created_at {
        println!("   ✅ created_at: {}", created.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(updated) = article.updated_at {
        println!("   ✅ updated_at: {}", updated.format("%Y-%m-%d %H:%M:%S"));
    }
    
    // Update - automatically updates updated_at
    println!("2. Update article (auto updates updated_at)...");
    let id = article.id;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Article::update_by_pk(pool, id, &[
        ("title", "Updated Title".into()),
    ]).await?;
    
    // Fetch and check
    let updated = Article::find_by_pk(pool, id).await?;
    println!("   ✅ updated_at changed to: {}", 
        updated.updated_at.unwrap().format("%Y-%m-%d %H:%M:%S"));
    
    println!("\n✅ Auto timestamps work correctly");
    Ok(())
}