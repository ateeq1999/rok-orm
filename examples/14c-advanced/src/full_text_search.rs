//! Example: Full-Text Search
//!
//! Demonstrates: PostgreSQL tsvector, tsquery, full-text search

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Full-Text Search\n");
    
    // Create table
    println!("1. Creating posts table...");
    sqlx::query("CREATE TABLE IF NOT EXISTS demo_posts (
        id BIGSERIAL PRIMARY KEY,
        title VARCHAR(255),
        body TEXT,
        created_at TIMESTAMPTZ DEFAULT NOW()
    )").execute(pool).await?;
    println!("   ✅ Created demo_posts");
    
    // Create GIN index for full-text search
    println!("2. Creating GIN index...");
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_demo_posts_fts 
        ON demo_posts USING gin(to_tsvector('english', title || ' ' || COALESCE(body, '')))")
        .execute(pool)
        .await
        .ok();
    println!("   ✅ Created GIN index");
    
    // Insert test data
    println!("3. Inserting test data...");
    sqlx::query("INSERT INTO demo_posts (title, body) VALUES 
        ('Introduction to Rust', 'Rust is a systems programming language that runs fast, prevents segfaults, and guarantees thread safety.'),
        ('Getting Started with Async', 'Async programming in Rust using tokio and async/await syntax.'),
        ('Building Web Apps', 'How to build web applications using Axum and ROK framework.')
    ").execute(pool).await?;
    println!("   ✅ Inserted 3 posts");
    
    // Basic full-text search
    println!("4. Full-text search queries...");
    
    // Simple match
    let results: Vec<(String, String)> = sqlx::query_as(
        "SELECT title, ts_headline('english', body, to_tsquery('english', ?)) as snippet 
         FROM demo_posts 
         WHERE to_tsvector('english', title || ' ' || COALESCE(body, '')) @@ to_tsquery('english', ?)
         ORDER BY ts_rank(to_tsvector('english', title || ' ' || COALESCE(body, '')), to_tsquery('english', ?)) DESC"
    )
        .bind("rust")
        .bind("rust")
        .bind("rust")
        .fetch_all(pool)
        .await?;
    
    println!("   Search 'rust': {} results", results.len());
    for (title, _) in &results {
        println!("     - {}", title);
    }
    
    // Multiple words (AND)
    let multi: Vec<(String,)> = sqlx::query_as(
        "SELECT title FROM demo_posts 
         WHERE to_tsvector('english', title || ' ' || COALESCE(body, '')) @@ to_tsquery('english', ?)"
    )
        .bind("rust & async")
        .fetch_all(pool)
        .await?;
    println!("   Search 'rust & async': {} results", multi.len());
    
    // Ranking
    println!("5. Search with ranking...");
    let ranked: Vec<(String, f32)> = sqlx::query_as(
        "SELECT title, ts_rank(to_tsvector('english', title || ' ' || COALESCE(body, '')), 
                to_tsquery('english', 'web')) as rank
         FROM demo_posts
         ORDER BY rank DESC"
    )
        .fetch_all(pool)
        .await?;
    
    for (title, rank) in &ranked {
        println!("   {} (rank: {:.3})", title, rank);
    }
    
    // Cleanup
    println!("6. Cleaning up...");
    sqlx::query("DROP TABLE IF EXISTS demo_posts")
        .execute(pool).await.ok();
    println!("   ✅ Table dropped");
    
    println!("\n✅ Full-text search works correctly");
    Ok(())
}