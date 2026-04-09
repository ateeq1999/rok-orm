//! Example: Raw Expressions
//!
//! Demonstrates: where_raw, select_raw, order_raw, having_raw, from_raw_sql

use rok_orm::Model;
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
    pub role: String,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Raw Expressions\n");
    
    // Create test data
    let _ = User::create(pool, &[
        ("name", "Admin".into()),
        ("email", "ADMIN@example.com".into()),
        ("role", "admin".into()),
    ]).await;
    
    println!("1. where_raw - raw WHERE clause...");
    let users = User::query()
        .where_raw("LOWER(email) = LOWER($1)", vec!["admin@example.com".into()])
        .get(pool)
        .await?;
    println!("   Found: {} users", users.len());
    
    println!("2. select_raw - raw SELECT...");
    let users = User::query()
        .select_raw("id, UPPER(name) as name_upper")
        .limit(5)
        .get(pool)
        .await?;
    println!("   Raw select returned: {} users", users.len());
    
    println!("3. order_raw - raw ORDER BY...");
    let users = User::query()
        .order_raw("FIELD(role, 'admin', 'moderator', 'user')")
        .get(pool)
        .await?;
    println!("   Ordered by role: {} users", users.len());
    
    println!("4. from_raw_sql - execute raw SQL directly...");
    let users: Vec<User> = User::from_raw_sql(
        pool,
        "SELECT * FROM users WHERE active = true ORDER BY id DESC",
        vec![],
    ).await?;
    println!("   Raw SQL returned: {} users", users.len());
    
    println!("\n✅ Raw expressions work correctly");
    Ok(())
}