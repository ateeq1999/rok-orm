//! Example 2: CRUD Operations
//! 
//! Demonstrates: create, create_returning, find_by_pk, find_or_404, update_by_pk, delete_by_pk, upsert

use rok_orm::{Model, PgModel, PgModelExt};
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

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 CRUD Operations\n");
    
    // CREATE
    println!("1. Create user...");
    User::create(pool, &[
        ("name", "Alice".into()),
        ("email", "alice@example.com".into()),
        ("active", true.into()),
    ]).await?;
    println!("   ✅ Created: Alice");
    
    // CREATE with RETURNING
    println!("2. Create with returning...");
    let bob = User::create_returning(pool, &[
        ("name", "Bob".into()),
        ("email", "bob@example.com".into()),
        ("active", true.into()),
    ]).await?;
    println!("   ✅ Created with ID: {} (Bob)", bob.id);
    
    // READ - find by pk (returns Option<T>)
    println!("3. Find by primary key...");
    let user = User::find_by_pk(pool, bob.id).await?
        .expect("User should exist");
    println!("   ✅ Found: {} (id={})", user.name, user.id);
    
    // READ - first row (no filter)
    println!("4. Find first...");
    let first = User::first(pool).await?
        .expect("Should have at least one user");
    println!("   ✅ Found first: {}", first.name);
    
    // Alternative: filter and get (returns Vec)
    let filtered = User::get_where(pool, User::query().filter("email", "alice@example.com")).await?;
    println!("   Filtered users: {}", filtered.len());
    
    // UPDATE
    println!("5. Update user...");
    User::update_by_pk(pool, bob.id, &[
        ("name", "Robert".into()),
    ]).await?;
    println!("   ✅ Updated Bob -> Robert");
    
    // UPSERT (insert or update)
    println!("6. Upsert...");
    User::upsert(pool, &[
        ("email", "admin@example.com".into()),
        ("name", "Admin".into()),
    ], "email", &["name"]).await?;
    
    User::upsert(pool, &[
        ("email", "admin@example.com".into()),
        ("name", "Admin Updated".into()),
    ], "email", &["name"]).await?;
    println!("   ✅ Upsert completed");
    
    // READ - all
    println!("7. Get all users...");
    let users = User::all(pool).await?;
    println!("   ✅ Total users: {}", users.len());
    
    println!("\n✅ CRUD operations work correctly");
    Ok(())
}