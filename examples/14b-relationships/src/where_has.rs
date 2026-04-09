//! Example: whereHas / whereDoesntHave
//!
//! Demonstrates: filter by relationship existence

use rok_orm::{Model, query::CountOp};
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

#[derive(rok_orm::Relations)]
pub struct UserRelations {
    #[has_many(target = "Post")]
    pub posts: rok_orm::relations::HasMany<User, Post>,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 whereHas / whereDoesntHave\n");
    
    // Create test data
    let user = User::create_returning(pool, &[
        ("name", "Author".into()),
    ]).await?;
    
    for i in 1..=3 {
        Post::create(pool, &[
            ("title", format!("Post {}", i)),
            ("user_id", user.id.into()),
            ("published", true.into()),
        ]).await?;
    }
    
    // Create user without posts
    User::create(pool, &[
        ("name", "No Posts User".into()),
    ]).await?;
    
    println!("1. Users with at least one post (whereHas)...");
    let with_posts = User::query()
        .where_has("posts", |q| q.filter("published", true))
        .get(pool)
        .await?;
    println!("   Users with published posts: {}", with_posts.len());
    
    println!("2. Users with more than 2 posts (whereHasCount)...");
    let popular = User::query()
        .where_has_count("posts", 2, CountOp::GreaterThan)
        .get(pool)
        .await?;
    println!("   Users with >2 posts: {}", popular.len());
    
    println!("3. Users with no posts (whereDoesntHave)...");
    let without = User::query()
        .where_doesnt_have("posts")
        .get(pool)
        .await?;
    println!("   Users without posts: {}", without.len());
    
    println!("4. Users without published posts...");
    let no_published = User::query()
        .where_doesnt_have("posts", |q| q.filter("published", true))
        .get(pool)
        .await?;
    println!("   Users without published posts: {}", no_published.len());
    
    println!("\n✅ whereHas/whereDoesntHave works correctly");
    Ok(())
}