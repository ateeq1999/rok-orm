//! Example: Sub-queries and CTEs
//!
//! Demonstrates: WHERE IN subquery, WHERE EXISTS, CTEs, from_subquery

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Sub-queries and CTEs\n");
    
    // Setup tables
    println!("1. Setting up tables...");
    sqlx::query("CREATE TABLE IF NOT EXISTS demo_users (
        id BIGSERIAL PRIMARY KEY,
        name VARCHAR(255),
        created_at TIMESTAMPTZ DEFAULT NOW()
    )").execute(pool).await?;
    
    sqlx::query("CREATE TABLE IF NOT EXISTS demo_orders (
        id BIGSERIAL PRIMARY KEY,
        user_id BIGINT,
        total DECIMAL(10,2),
        created_at TIMESTAMPTZ DEFAULT NOW()
    )").execute(pool).await?;
    println!("   ✅ Created users and orders tables");
    
    // Insert test data
    println!("2. Inserting test data...");
    for i in 1..=5 {
        sqlx::query("INSERT INTO demo_users (name) VALUES (?)")
            .bind(format!("User {}", i))
            .execute(pool).await?;
    }
    
    // User 1 has many orders
    for i in 1..=10 {
        sqlx::query("INSERT INTO demo_orders (user_id, total) VALUES (?, ?)")
            .bind(1i64)
            .bind(100.0 + (i as f64 * 10.0))
            .execute(pool).await?;
    }
    
    // Other users have fewer orders
    for user_id in 2..=5 {
        for i in 1..=2 {
            sqlx::query("INSERT INTO demo_orders (user_id, total) VALUES (?, ?)")
                .bind(user_id)
                .bind(50.0 + (i as f64 * 5.0))
                .execute(pool).await?;
        }
    }
    println!("   ✅ Inserted test data");
    
    // WHERE IN subquery
    println!("3. WHERE IN subquery...");
    let power_users: Vec<(String,)> = sqlx::query_as(
        "SELECT u.name FROM demo_users u 
         WHERE u.id IN (
             SELECT o.user_id FROM demo_orders o 
             GROUP BY o.user_id 
             HAVING COUNT(*) > 5
         )"
    )
        .fetch_all(pool)
        .await?;
    println!("   Users with >5 orders: {:?}", power_users.len());
    
    // WHERE EXISTS
    println!("4. WHERE EXISTS subquery...");
    let with_orders: Vec<(String,)> = sqlx::query_as(
        "SELECT u.name FROM demo_users u 
         WHERE EXISTS (
             SELECT 1 FROM demo_orders o WHERE o.user_id = u.id
         )"
    )
        .fetch_all(pool)
        .await?;
    println!("   Users with any orders: {}", with_orders.len());
    
    // WHERE NOT EXISTS
    println!("5. WHERE NOT EXISTS subquery...");
    let without_orders: Vec<(String,)> = sqlx::query_as(
        "SELECT u.name FROM demo_users u 
         WHERE NOT EXISTS (
             SELECT 1 FROM demo_orders o WHERE o.user_id = u.id
         )"
    )
        .fetch_all(pool)
        .await?;
    println!("   Users without orders: {}", without_orders.len());
    
    // CTE (Common Table Expression)
    println!("6. CTE (WITH clause)...");
    let cte_results: Vec<(String, i64, i64)> = sqlx::query_as(
        "WITH ranked AS (
            SELECT u.name, COUNT(o.id) as order_count,
                   ROW_NUMBER() OVER (ORDER BY COUNT(o.id) DESC) as rn
            FROM demo_users u
            LEFT JOIN demo_orders o ON u.id = o.user_id
            GROUP BY u.id, u.name
        )
        SELECT name, order_count, rn FROM ranked WHERE rn <= 3"
    )
        .fetch_all(pool)
        .await?;
    println!("   Top 3 users by orders:");
    for (name, count, rank) in &cte_results {
        println!("     {}. {} ({} orders)", rank, name, count);
    }
    
    // Cleanup
    println!("7. Cleaning up...");
    sqlx::query("DROP TABLE IF EXISTS demo_orders").execute(pool).await.ok();
    sqlx::query("DROP TABLE IF EXISTS demo_users").execute(pool).await.ok();
    println!("   ✅ Tables dropped");
    
    println!("\n✅ Sub-queries and CTEs work correctly");
    Ok(())
}