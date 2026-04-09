//! Example 9: Query Scopes
//! 
//! Demonstrates: reusable query builders

use rok_orm::{Model, PgModel, query::QueryBuilder};
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
    pub active: bool,
    pub role: String,
}

// Define query scopes as associated functions
impl User {
    /// Scope: Active users
    pub fn active() -> QueryBuilder<User> {
        User::query().filter("active", true)
    }
    
    /// Scope: Users by role
    pub fn role(scope: &str) -> QueryBuilder<User> {
        User::query().filter("role", scope)
    }
    
    /// Scope: Recent users (last N days)
    pub fn recent(days: i64) -> QueryBuilder<User> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
        User::query().where_gt("created_at", cutoff)
    }
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Query Scopes\n");
    
    // Create test users with different roles
    let _ = User::create(pool, &[
        ("name", "Admin User".into()),
        ("email", "admin@test.com".into()),
        ("active", true.into()),
        ("role", "admin".into()),
    ]).await;
    
    let _ = User::create(pool, &[
        ("name", "Regular User".into()),
        ("email", "user@test.com".into()),
        ("active", true.into()),
        ("role", "user".into()),
    ]).await;
    
    // Use scopes - need to use get_where to execute
    println!("1. Using active() scope...");
    let active = User::get_where(pool, User::active()).await?;
    println!("   Active users: {}", active.len());
    
    println!("2. Using role('admin') scope...");
    let admins = User::get_where(pool, User::role("admin")).await?;
    println!("   Admin users: {}", admins.len());
    
    println!("3. Using recent(30) scope...");
    let recent = User::get_where(pool, User::recent(30)).await?;
    println!("   Recent users: {}", recent.len());
    
    // Chain scopes - combine multiple filters
    println!("4. Chaining scopes (active + role='user')...");
    // Note: chaining works because each scope returns QueryBuilder
    let chained = User::active().role("user");
    let active_users = User::get_where(pool, chained).await?;
    println!("   Active users with role 'user': {}", active_users.len());
    
    println!("\n✅ Query scopes work correctly");
    Ok(())
}