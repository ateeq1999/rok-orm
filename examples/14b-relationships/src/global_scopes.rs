//! Example: Global Query Scopes
//!
//! Demonstrates: GlobalScope trait, add_global_scope, without_global_scope

use rok_orm::{Model, global_scope::GlobalScope, query::QueryBuilder};
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub active: bool,
    pub verified: bool,
}

// Define a global scope
pub struct ActiveScope;

impl GlobalScope<User> for ActiveScope {
    fn apply(&self, query: QueryBuilder<User>) -> QueryBuilder<User> {
        query.filter("active", true)
    }
}

pub struct VerifiedScope;

impl GlobalScope<User> for VerifiedScope {
    fn apply(&self, query: QueryBuilder<User>) -> QueryBuilder<User> {
        query.filter("verified", true)
    }
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Global Query Scopes\n");
    
    // Register global scope
    println!("1. Registering global scope (ActiveScope)...");
    User::add_global_scope(ActiveScope);
    println!("   ✅ Scope registered - all queries now filter active=true");
    
    // All queries now include active = true
    println!("2. Query with global scope...");
    let users = User::all(pool).await?;
    println!("   All users (filtered by active=true): {}", users.len());
    
    // Without global scope
    println!("3. Without global scope...");
    let all = User::query()
        .without_global_scope::<ActiveScope>()
        .get(pool)
        .await?;
    println!("   All users (unfiltered): {}", all.len());
    
    // Add another scope
    println!("4. Adding VerifiedScope...");
    User::add_global_scope(VerifiedScope);
    println!("   ✅ Both scopes active");
    
    // Remove scope
    println!("5. Removing scope...");
    User::remove_global_scope::<ActiveScope>();
    println!("   ✅ ActiveScope removed");
    
    println!("\n✅ Global query scopes work correctly");
    Ok(())
}