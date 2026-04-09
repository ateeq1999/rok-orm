//! Example 3: Basic Relationships
//! 
//! Demonstrates: has_many, belongs_to relationships

use rok_orm::{Model, PgModel, PgModelExt, Relations};
use serde::{Deserialize, Serialize};

#[derive(Model, Relations, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
    pub active: bool,
    #[model(has_many = Post)]
    _posts: (),
}

#[derive(Model, Relations, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "posts")]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub user_id: i64,
    pub published: bool,
    #[model(belongs_to = User)]
    _user: (),
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Basic Relationships\n");
    
    // Create a user first
    let user = User::create_returning(pool, &[
        ("name", "Author".into()),
        ("email", "author@example.com".into()),
        ("active", true.into()),
    ]).await?;
    
    // Create posts
    for i in 1..=3 {
        Post::create(pool, &[
            ("title", format!("Post {}", i)),
            ("user_id", user.id.into()),
            ("published", true.into()),
        ]).await?;
    }
    println!("1. Created user with 3 posts");
    
    // Query users and manually load their posts (eager loading pattern)
    println!("2. Fetch users with posts (manual eager load)...");
    let users = User::all(pool).await?;
    
    for u in &users {
        // Manually fetch related posts
        let posts = Post::get_where(
            pool,
            Post::query().filter("user_id", u.id),
        ).await?;
        println!("   📝 {} has {} posts", u.name, posts.len());
    }
    
    // Query posts with user info
    println!("3. Fetch posts with user info...");
    let posts = Post::all(pool).await?;
    for post in &posts {
        // Get the user for this post
        let users = User::get_where(
            pool,
            User::query().filter("id", post.user_id),
        ).await?;
        if let Some(user) = users.first() {
            println!("   - Post: {} by {}", post.title, user.name);
        }
    }
    
    println!("\n✅ Relationships work correctly");
    println!("   Note: Use user.posts() after adding fluent methods");
    Ok(())
}