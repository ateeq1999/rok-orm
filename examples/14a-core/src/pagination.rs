//! Example 6: Pagination
//! 
//! Demonstrates: Page<T>, paginate()

use rok_orm::{Model, pagination::Page};
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "posts")]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub user_id: i64,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Pagination\n");
    
    // Create some posts
    for i in 1..=15 {
        Post::create(pool, &[
            ("title", format!("Post #{}", i)),
            ("user_id", 1i64.into()),
        ]).await?;
    }
    println!("1. Created 15 posts");
    
    // Paginate
    println!("2. Paginate (page 1, per_page 5)...");
    let page: Page<Post> = Post::paginate(pool, 1, 5).await?;
    
    println!("   📄 Current page: {}", page.current_page);
    println!("   📄 Last page: {}", page.last_page);
    println!("   📄 Total items: {}", page.total);
    println!("   📄 Per page: {}", page.per_page);
    println!("   📄 Has next: {}", page.has_next());
    println!("   📄 Has prev: {}", page.has_prev());
    
    // Second page
    println!("3. Page 2...");
    let page2 = Post::paginate(pool, 2, 5).await?;
    println!("   Items on page 2: {}", page2.data.len());
    
    // Custom query pagination
    println!("4. Custom query with pagination...");
    let custom_page = Post::query()
        .order_by_desc("id")
        .paginate(pool, 1, 10)
        .await?;
    
    println!("   Custom query page: {}/{}", custom_page.current_page, custom_page.last_page);
    
    println!("\n✅ Pagination works correctly");
    Ok(())
}