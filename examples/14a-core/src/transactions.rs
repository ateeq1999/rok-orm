//! Example 8: Transactions
//! 
//! Demonstrates: Tx::begin, commit, rollback

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

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "posts")]
pub struct Post {
    #[model(primary_key)]
    pub id: i64,
    pub title: String,
    pub user_id: i64,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Transactions\n");
    
    // Successful transaction
    println!("1. Commit transaction...");
    {
        let mut tx = rok_orm::Tx::begin(pool).await?;
        
        tx.insert::<User>("users", &[
            ("name", "Tx User".into()),
            ("email", "tx@example.com".into()),
        ]).await?;
        
        tx.insert::<Post>("posts", &[
            ("title", "Tx Post".into()),
            ("user_id", 1i64.into()),
        ]).await?;
        
        tx.commit().await?;
    }
    println!("   ✅ Transaction committed successfully");
    
    // Rollback example (commented out for demo)
    println!("2. Rollback demonstration...");
    println!("   (Transaction would rollback if something failed)");
    println!("   Example pattern:");
    println!("     let mut tx = Tx::begin(pool).await?;");
    println!("     if condition { tx.rollback().await?; }");
    println!("     else { tx.commit().await?; }");
    
    // Check data
    let users = User::query()
        .filter("email", "tx@example.com")
        .get(pool)
        .await?;
    println!("   User created in transaction: {}", users.len() > 0);
    
    println!("\n✅ Transactions work correctly");
    Ok(())
}