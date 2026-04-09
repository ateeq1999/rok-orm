//! Example 1: Basic Model Definition
//!
//! Demonstrates: #[derive(Model)], table_name(), columns()

use rok_orm::Model;
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

pub async fn run(_pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Basic Model Definition\n");

    // Model metadata
    println!("Table name: {}", User::table_name());
    println!("Columns: {:?}", User::columns());
    println!("Primary key: id");

    // Verify assertions
    assert_eq!(User::table_name(), "users");
    assert_eq!(User::columns(), &["id", "name", "email", "active"]);

    println!("\n✅ Basic model metadata works correctly");
    Ok(())
}
