//! Example: Cursor Pagination
//!
//! Demonstrates: CursorPage, CursorResult, cursor_paginate

use rok_orm::{Model, cursor::{CursorPage, CursorResult}};
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "posts")]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Cursor Pagination\n");
    
    // Create test data
    for i in 1..=10 {
        Post::create(pool, &[
            ("title", format!("Post {}", i)),
        ]).await?;
    }
    println!("1. Created 10 posts");
    
    // First page
    println!("2. First page (no cursor)...");
    let result = Post::query()
        .order_by_desc("id")
        .cursor_paginate(pool, CursorPage { after: None, limit: 3 })
        .await?;
    
    println!("   Page 1: {} items", result.data.len());
    println!("   Has more: {}", result.has_more);
    println!("   Next cursor: {:?}", result.next_cursor);
    println!("   Prev cursor: {:?}", result.prev_cursor);
    
    // Next page
    if let Some(cursor) = &result.next_cursor {
        println!("3. Next page (with cursor)...");
        let next = Post::query()
            .order_by_desc("id")
            .cursor_paginate(pool, CursorPage { 
                after: Some(cursor.clone()), 
                limit: 3 
            })
            .await?;
        
        println!("   Page 2: {} items", next.data.len());
        println!("   Has more: {}", next.has_more);
        println!("   Next cursor: {:?}", next.next_cursor);
    }
    
    // Prev page
    if let Some(cursor) = &result.prev_cursor {
        println!("4. Previous page (with cursor)...");
        let prev = Post::query()
            .order_by_desc("id")
            .cursor_paginate(pool, CursorPage { 
                after: Some(cursor.clone()), 
                limit: 3 
            })
            .await?;
        
        println!("   Page 0: {} items", prev.data.len());
        println!("   Has more: {}", prev.has_more);
    }
    
    println!("\n✅ Cursor pagination works correctly");
    Ok(())
}