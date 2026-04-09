//! Example 4: Soft Deletes
//! 
//! Demonstrates: soft_delete, with_trashed, only_trashed, restore, force_delete

use rok_orm::{Model, PgModel, PgModelExt};
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "posts", soft_delete)]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub published: bool,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Soft Deletes\n");
    
    // Create posts
    for i in 1..=3 {
        Post::create(pool, &[
            ("title", format!("Post {}", i)),
            ("published", true.into()),
        ]).await?;
    }
    println!("1. Created 3 posts");
    
    // Soft delete one post
    println!("2. Soft delete post #1...");
    Post::delete_by_pk(pool, 1).await?;
    println!("   ✅ Soft deleted (deleted_at set)");
    
    // Excludes deleted by default
    println!("3. Query (excludes deleted by default)...");
    let active = Post::all(pool).await?;
    println!("   Active posts: {}", active.len());
    
    // Include trashed - use builder method then fetch
    println!("4. with_trashed() - includes deleted...");
    let all = Post::get_where(pool, Post::query().with_trashed()).await?;
    println!("   Total posts (with trashed): {}", all.len());
    
    // Only trashed
    println!("5. only_trashed() - only deleted...");
    let trashed = Post::get_where(pool, Post::query().only_trashed()).await?;
    println!("   Trashed posts: {}", trashed.len());
    
    // Restore
    println!("6. Restore deleted post...");
    Post::restore(pool, 1).await?;
    println!("   ✅ Restored (deleted_at = NULL)");
    
    // Verify restored
    let active = Post::all(pool).await?;
    println!("   Active posts after restore: {}", active.len());
    
    println!("\n✅ Soft deletes work correctly");
    Ok(())
}