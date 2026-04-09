//! Example 7: Aggregations
//! 
//! Demonstrates: count, sum, avg, min, max

use rok_orm::{Model, PgModel, PgModelExt};
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub active: bool,
    pub age: i32,
}

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    #[model(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub total: f64,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Aggregations\n");
    
    // Create users
    for i in 1..=5 {
        User::create(pool, &[
            ("name", format!("User {}", i).into()),
            ("active", true.into()),
            ("age", (20 + i).into()),
        ]).await?;
    }
    println!("1. Created 5 users (ages 21-25)");
    
    // Create orders
    for i in 1..=3 {
        Order::create(pool, &[
            ("user_id", 1i64.into()),
            ("total", (100.0 * i as f64).into()),
        ]).await?;
    }
    println!("2. Created 3 orders for user #1 (100, 200, 300)");
    
    // Count
    println!("3. Count operations...");
    let total: i64 = User::count(pool).await?;
    println!("   Total users: {}", total);
    
    let active_count = User::count_where(pool, User::query().filter("active", true)).await?;
    println!("   Active users: {}", active_count);
    
    // Sum
    println!("4. Sum operations...");
    let revenue: Option<f64> = Order::sum(pool, "total").await?;
    println!("   Total revenue: ${:.2}", revenue.unwrap_or(0.0));
    
    // Avg
    println!("5. Average operations...");
    let avg_age: Option<f64> = User::avg(pool, "age").await?;
    println!("   Average user age: {:.1}", avg_age.unwrap_or(0.0));
    
    let avg_order: Option<f64> = Order::avg(pool, "total").await?;
    println!("   Average order value: ${:.2}", avg_order.unwrap_or(0.0));
    
    // Min/Max
    println!("6. Min/Max operations...");
    let oldest: Option<f64> = User::min(pool, "age").await?;
    let youngest: Option<f64> = User::max(pool, "age").await?;
    println!("   Youngest user age: {:?}", oldest);
    println!("   Oldest user age: {:?}", youngest);
    
    // With query builder - get SQL for aggregation (not executed here)
    println!("7. Query builder aggregations (SQL generation)...");
    let (sql, _) = User::query()
        .filter("active", true)
        .sum_sql("age");
    println!("   SUM SQL: {}", sql);
    
    println!("\n✅ Aggregations work correctly");
    Ok(())
}