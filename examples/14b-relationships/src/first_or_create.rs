//! Example: firstOrCreate / firstOrNew / updateOrCreate

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
    println!("\n📋 firstOrCreate / firstOrNew / updateOrCreate\n");
    
    println!("1. firstOrCreate - find or create...");
    let user1 = User::first_or_create(pool,
        &[("email", "new@example.com".into())],
        &[("name", "New User".into()), ("role", "user".into())],
    ).await?;
    println!("   Created/found: {} (id={})", user1.name, user1.id);
    
    println!("2. firstOrCreate - find existing...");
    let user2 = User::first_or_create(pool,
        &[("email", "new@example.com".into())],
        &[("name", "Different Name".into())],
    ).await?;
    println!("   Found existing: {} (id={}, same as first={})", 
        user2.name, user2.id, user1.id == user2.id);
    
    println!("3. firstOrNew - create in memory (no DB write)...");
    let new_user = User::first_or_new(
        &[("email", "memory@example.com".into())],
        &[("name", "In Memory".into())],
    );
    println!("   Created in memory: {} (id={} - not in DB)", 
        new_user.email, new_user.id);
    
    println!("4. updateOrCreate - update if found, create if not...");
    let updated = User::update_or_create(pool,
        &[("email", "update@example.com".into())],
        &[("name", "First".into())],
    ).await?;
    println!("   First call: {} (id={})", updated.name, updated.id);
    
    let updated2 = User::update_or_create(pool,
        &[("email", "update@example.com".into())],
        &[("name", "Updated".into())],
    ).await?;
    println!("   Second call: {} (id={}, same={})", 
        updated2.name, updated2.id, updated.id == updated2.id);
    
    println!("\n✅ firstOrCreate patterns work correctly");
    Ok(())
}