//! Example: when() / when_else() Conditional Query Building

use rok_orm::Model;
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub role: String,
    pub active: bool,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 when() / when_else() Conditional Chaining\n");
    
    // Create test data
    let _ = User::create(pool, &[
        ("name", "Admin User".into()),
        ("role", "admin".into()),
        ("active", true.into()),
    ]).await;
    
    let _ = User::create(pool, &[
        ("name", "Regular User".into()),
        ("role", "user".into()),
        ("active", true.into()),
    ]).await;
    
    // Simulate query parameters
    let filter_role = Some("admin".to_string());
    let active_only = true;
    let search = None::<String>;
    
    println!("1. Conditional with when()...");
    let users = User::query()
        .when(filter_role.is_some(), |q| {
            q.filter("role", filter_role.clone().unwrap())
        })
        .when(active_only, |q| q.filter("active", true))
        .when(search.is_some(), |q| {
            q.where_like("name", &format!("%{}%", search.clone().unwrap()))
        })
        .get(pool)
        .await?;
    println!("   Filtered users: {}", users.len());
    
    // with_else branch
    println!("2. Conditional with when_else()...");
    let is_admin = false;
    let users = User::query()
        .when_else(
            is_admin,
            |q| q.filter("role", "admin"),
            |q| q.filter("role", "user"),
        )
        .get(pool)
        .await?;
    println!("   when_else (is_admin={}): {} users", is_admin, users.len());
    
    let is_admin = true;
    let users = User::query()
        .when_else(
            is_admin,
            |q| q.filter("role", "admin"),
            |q| q.filter("role", "user"),
        )
        .get(pool)
        .await?;
    println!("   when_else (is_admin={}): {} users", is_admin, users.len());
    
    println!("\n✅ Conditional chaining works correctly");
    Ok(())
}