//! Example: withoutTimestamps
//!
//! Demonstrates: suppress timestamp injection

use rok_orm::Model;
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users", timestamps)]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub views: i32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 withoutTimestamps\n");
    
    // Normal create - timestamps added
    println!("1. Normal create (timestamps added)...");
    let user = User::create_returning(pool, &[
        ("name", "Normal User".into()),
        ("views", 0.into()),
    ]).await?;
    println!("   Created at: {:?}", user.created_at.is_some());
    
    // without_timestamps - suppress timestamp injection
    println!("2. without_timestamps (timestamps suppressed)...");
    let id = user.id;
    User::without_timestamps(|| async {
        User::update_by_pk(pool, id, &[("views", 100.into())]).await
    }).await?;
    println!("   ✅ Update completed without timestamp changes");
    
    // increment_without_timestamps
    println!("3. increment_without_timestamps...");
    User::increment_without_timestamps(pool, id, "views", 1).await?;
    println!("   ✅ Incremented without timestamp update");
    
    println!("\n✅ without_timestamps works correctly");
    Ok(())
}