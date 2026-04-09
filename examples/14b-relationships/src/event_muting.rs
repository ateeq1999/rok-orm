//! Example: Event Muting
//!
//! Demonstrates: without_events, save_quietly

use rok_orm::Model;
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Event Muting\n");
    
    println!("1. without_events - suppress all hooks...");
    User::without_events(|| async {
        User::create(pool, &[
            ("name", "Silent Create".into()),
            ("email", "silent@example.com".into()),
        ]).await
    }).await?;
    println!("   ✅ Created without triggering events");
    
    // save_quietly (update without events)
    println!("2. save_quietly - update without events...");
    let user = User::create_returning(pool, &[
        ("name", "Quiet Update".into()),
        ("email", "quiet@example.com".into()),
    ]).await?;
    
    user.save_quietly(pool, &[
        ("name", "Updated Quietly".into()),
    ]).await?;
    println!("   ✅ Updated without triggering events");
    
    println!("\n✅ Event muting works correctly");
    Ok(())
}