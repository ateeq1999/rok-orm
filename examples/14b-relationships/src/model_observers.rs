//! Example: Model Observers
//!
//! Demonstrates: ModelObserver trait, ObserverRegistry

use rok_orm::{Model, observer::ModelObserver};
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
}

pub struct UserObserver;

#[async_trait::async_trait]
impl ModelObserver for UserObserver {
    type Model = User;

    async fn creating(&self, user: &mut User) -> rok_orm::OrmResult<()> {
        user.email = user.email.to_lowercase();
        println!("   Observer: normalizing email to lowercase");
        Ok(())
    }
    
    async fn created(&self, user: &User) -> rok_orm::OrmResult<()> {
        println!("   Observer: user {} created", user.id);
        Ok(())
    }
    
    async fn updating(&self, user: &mut User) -> rok_orm::OrmResult<()> { Ok(()) }
    async fn updated(&self, user: &User) -> rok_orm::OrmResult<()> {
        println!("   Observer: user {} updated", user.id);
        Ok(())
    }
    async fn saving(&self, user: &mut User) -> rok_orm::OrmResult<()> { Ok(()) }
    async fn saved(&self, user: &User) -> rok_orm::OrmResult<()> { Ok(()) }
    async fn deleting(&self, user: &User) -> rok_orm::OrmResult<()> { Ok(()) }
    async fn deleted(&self, user: &User) -> rok_orm::OrmResult<()> {
        println!("   Observer: user {} deleted", user.id);
        Ok(())
    }
    async fn restoring(&self, user: &User) -> rok_orm::OrmResult<()> { Ok(()) }
    async fn restored(&self, user: &User) -> rok_orm::OrmResult<()> { Ok(()) }
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Model Observers\n");
    
    // Register observer
    println!("1. Registering observer...");
    User::observe(UserObserver);
    println!("   ✅ Observer registered");
    
    // Create - triggers creating and created
    println!("2. Creating user (triggers observer)...");
    let user = User::create_returning(pool, &[
        ("name", "Observed User".into()),
        ("email", "TEST@EXAMPLE.COM".into()), // Will be lowercased
    ]).await?;
    println!("   Created: {} (email normalized)", user.email);
    
    // Update - triggers updating and updated
    println!("3. Updating user (triggers observer)...");
    User::update_by_pk(pool, user.id, &[
        ("name", "Updated Name".into()),
    ]).await?;
    println!("   ✅ Update completed");
    
    println!("\n✅ Model observers work correctly");
    Ok(())
}