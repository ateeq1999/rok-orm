//! Example 3: Basic Relationships
//! 
//! Demonstrates: has_many, belongs_to, with() eager loading

use rok_orm::{Model, relations::{HasMany, BelongsTo}};
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
    pub active: bool,
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
    
    // Eager loading - prevents N+1
    println!("2. Eager loading with with()...");
    let users = User::query()
        .with("posts")
        .limit(5)
        .get(pool)
        .await?;
    
    for u in &users {
        println!("   📝 {} has {} posts", u.name, u.posts.len());
    }
    
    // Lazy loading demonstration
    println!("3. Lazy loading (access relation on-demand)...");
    let posts = Post::query().limit(3).get(pool).await?;
    for post in &posts {
        // This would trigger a query if accessed
        println!("   - Post: {} (user_id: {})", post.title, post.user_id);
    }
    
    println!("\n✅ Relationships work correctly");
    Ok(())
}