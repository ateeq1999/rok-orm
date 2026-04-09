//! Example: Window Functions
//!
//! Demonstrates: ROW_NUMBER, RANK, DENSE_RANK, LAG, LEAD, OVER clause

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Window Functions\n");
    
    // Setup
    println!("1. Setting up table...");
    sqlx::query("CREATE TABLE IF NOT EXISTS demo_users (
        id BIGSERIAL PRIMARY KEY,
        name VARCHAR(255),
        score INT,
        created_at TIMESTAMPTZ DEFAULT NOW()
    )").execute(pool).await?;
    println!("   ✅ Created demo_users");
    
    // Insert test data
    println!("2. Inserting test data...");
    let scores = [("Alice", 95), ("Bob", 85), ("Charlie", 95), ("Diana", 70), ("Eve", 85)];
    for (name, score) in scores {
        sqlx::query("INSERT INTO demo_users (name, score) VALUES (?, ?)")
            .bind(name)
            .bind(score)
            .execute(pool).await?;
    }
    println!("   ✅ Inserted 5 users with scores");
    
    // ROW_NUMBER - unique sequential number
    println!("3. ROW_NUMBER()...");
    let results: Vec<(String, i32, i64)> = sqlx::query_as(
        "SELECT name, score, ROW_NUMBER() OVER (ORDER BY score DESC) as rn 
         FROM demo_users"
    )
        .fetch_all(pool)
        .await?;
    
    println!("   Ranked by score:");
    for (name, score, rn) in &results {
        println!("     {}. {} - {} points", rn, name, score);
    }
    
    // RANK - with gaps
    println!("4. RANK() (with gaps)...");
    let ranks: Vec<(String, i32, i64)> = sqlx::query_as(
        "SELECT name, score, RANK() OVER (ORDER BY score DESC) as rank 
         FROM demo_users"
    )
        .fetch_all(pool)
        .await?;
    
    println!("   With gaps (95,95,85,85,70):");
    for (name, score, rank) in &ranks {
        println!("     Rank {}: {} - {}", rank, name, score);
    }
    
    // DENSE_RANK - without gaps
    println!("5. DENSE_RANK() (without gaps)...");
    let dense: Vec<(String, i32, i64)> = sqlx::query_as(
        "SELECT name, score, DENSE_RANK() OVER (ORDER BY score DESC) as dr 
         FROM demo_users"
    )
        .fetch_all(pool)
        .await?;
    
    println!("   Without gaps:");
    for (name, score, dr) in &dense {
        println!("     DensRank {}: {} - {}", dr, name, score);
    }
    
    // PARTITION BY - group by partition
    println!("6. PARTITION BY (grouped ranking)...");
    sqlx::query("CREATE TABLE IF NOT EXISTS demo_departments (
        id SERIAL PRIMARY KEY,
        dept VARCHAR(50),
        employee VARCHAR(50),
        salary INT
    )").execute(pool).await?;
    
    sqlx::query("INSERT INTO demo_departments (dept, employee, salary) VALUES 
        ('Engineering', 'Alice', 90000),
        ('Engineering', 'Bob', 85000),
        ('Engineering', 'Charlie', 95000),
        ('Sales', 'Diana', 70000),
        ('Sales', 'Eve', 75000)
    ").execute(pool).await?;
    
    let partitioned: Vec<(String, String, i32, i64)> = sqlx::query_as(
        "SELECT dept, employee, salary, 
                RANK() OVER (PARTITION BY dept ORDER BY salary DESC) as rn
         FROM demo_departments"
    )
        .fetch_all(pool)
        .await?;
    
    println!("   By department:");
    for (dept, emp, sal, rn) in &partitioned {
        println!("     {}: {} (${}) - Rank {}", dept, emp, sal, rn);
    }
    
    // LAG - previous row value
    println!("7. LAG() (previous row)...");
    let lag_results: Vec<(String, i32, Option<i32>)> = sqlx::query_as(
        "SELECT name, score, LAG(score) OVER (ORDER BY id) as prev_score 
         FROM demo_users ORDER BY id"
    )
        .fetch_all(pool)
        .await?;
    
    println!("   With previous score:");
    for (name, score, prev) in &lag_results {
        let diff = prev.map(|p| score - p);
        println!("     {}: {} (prev: {:?}, diff: {:?})", name, score, prev, diff);
    }
    
    // LEAD - next row value  
    println!("8. LEAD() (next row)...");
    let lead_results: Vec<(String, i32, Option<i32>)> = sqlx::query_as(
        "SELECT name, score, LEAD(score) OVER (ORDER BY id) as next_score 
         FROM demo_users ORDER BY id"
    )
        .fetch_all(pool)
        .await?;
    
    println!("   With next score:");
    for (name, score, next) in &lead_results {
        println!("     {}: {} (next: {:?})", name, score, next);
    }
    
    // Cleanup
    println!("9. Cleaning up...");
    sqlx::query("DROP TABLE IF EXISTS demo_departments").execute(pool).await.ok();
    sqlx::query("DROP TABLE IF EXISTS demo_users").execute(pool).await.ok();
    println!("   ✅ Tables dropped");
    
    println!("\n✅ Window functions work correctly");
    Ok(())
}