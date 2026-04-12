//! Example 10: Query Logging
//! 
//! Demonstrates: Logger, LogLevel, QueryTimer

use rok_orm::{Model, PgModel, logging::{Logger, LogLevel, QueryTimer}};
use serde::{Deserialize, Serialize};

#[derive(Model, sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
#[model(table = "users")]
pub struct User {
    #[model(primary_key)]
    pub id: i64,
    pub name: String,
    pub active: bool,
}

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Query Logging\n");
    
    // Create logger
    println!("1. Setting up logger...");
    let logger = Logger::new()
        .with_log_level(LogLevel::Debug)
        .with_slow_query_threshold(50); // ms
    
    // Time a query
    println!("2. Timing a query...");
    let timer = QueryTimer::new();
    
    let _users = User::query()
        .filter("active", true)
        .limit(10)
        .get(pool)
        .await?;
    
    let elapsed = timer.elapsed_ms();
    println!("   Query took: {}ms", elapsed);
    
    // Check if slow
    if logger.is_slow_query(elapsed) {
        println!("   ⚠️  Slow query detected!");
    } else {
        println!("   ✅ Query within normal range");
    }
    
    // Multiple queries
    println!("3. Multiple queries...");
    let timer2 = QueryTimer::new();
    
    for _ in 0..5 {
        let _ = User::all(pool).await?;
    }
    
    let elapsed2 = timer2.elapsed_ms();
    println!("   5 queries took: {}ms (avg: {:.1}ms)", elapsed2, elapsed2 as f64 / 5.0);
    
    println!("\n✅ Query logging works correctly");
    Ok(())
}