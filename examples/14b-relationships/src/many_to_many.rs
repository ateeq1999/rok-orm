//! Example: ManyToMany with Pivot Access
//!
//! Demonstrates: belongs_to_many, attach, sync, toggle, with_pivot, update_pivot

use rok_orm::{Model, relations::BelongsToMany};
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
}

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
}

#[derive(rok_orm::Relations)]
pub struct UserRelations {
    #[belongs_to_many(
        target = "Role",
        pivot = "user_roles",
        fk = "user_id",
        rfk = "role_id",
        pivots = ["assigned_at"],
    )]
    pub roles: BelongsToMany<User, Role>,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 ManyToMany with Pivot Access\n");
    
    // Note: Requires pivot table setup in database
    // For demo, show API usage
    
    println!("1. ManyToMany API demonstration:");
    println!("   user.roles().attach(&pool, role_id).await?");
    println!("   user.roles().attach_with_pivot(&pool, role_id, &[...]).await?");
    println!("   user.roles().sync(&pool, vec![1, 2, 3]).await?");
    println!("   user.roles().toggle(&pool, vec![1, 2]).await?");
    println!("   user.roles().with_pivot(&[\"assigned_at\"]).get(&pool).await?");
    println!("   user.roles().update_pivot(&pool, role_id, &[...]).await?");
    println!("   user.roles().detach(&pool, role_id).await?");
    
    println!("\n✅ ManyToMany API documented");
    println!("   (Requires pivot table setup for full demo)");
    Ok(())
}