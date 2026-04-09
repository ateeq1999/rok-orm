//! Example: JSON Column Support
//!
//! Demonstrates: JSON column queries

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 JSON Column Support\n");
    
    // Create table with JSON column
    println!("1. Creating table with JSON columns...");
    sqlx::query("CREATE TABLE IF NOT EXISTS demo_users (
        id BIGSERIAL PRIMARY KEY,
        name VARCHAR(255),
        metadata JSONB,
        settings JSONB,
        permissions JSONB
    )").execute(pool).await?;
    println!("   ✅ Created demo_users with JSON columns");
    
    // Insert JSON data
    println!("2. Inserting JSON data...");
    sqlx::query("INSERT INTO demo_users (name, metadata, settings, permissions) VALUES (?, ?, ?, ?)")
        .bind("JSON User")
        .bind(r#"{"role": "admin", "verified": true}"#)
        .bind(r#"{"theme": "dark", "notifications": true}"#)
        .bind(r#"["read", "write", "delete"]"#)
        .execute(pool).await?;
    println!("   ✅ Inserted JSON data");
    
    // Query JSON - PostgreSQL specific operators
    println!("3. Querying JSON columns...");
    
    // ->> operator for text extraction
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT name, metadata->>'role' as role FROM demo_users WHERE metadata->>'role' = ?"
    )
        .bind("admin")
        .fetch_all(pool)
        .await?;
    println!("   Found {} users with role=admin", rows.len());
    
    // Check if JSON contains
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM demo_users WHERE metadata ? 'verified'"
    )
        .fetch_one(pool)
        .await?;
    println!("   Users with 'verified' key: {}", count.0);
    
    // Array contains
    let arr: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM demo_users WHERE permissions @> '\"read\"'"
    )
        .fetch_one(pool)
        .await?;
    println!("   Users with 'read' permission: {}", arr.0);
    
    // Cleanup
    println!("4. Cleaning up...");
    sqlx::query("DROP TABLE IF EXISTS demo_users")
        .execute(pool).await.ok();
    println!("   ✅ Table dropped");
    
    println!("\n✅ JSON column support works correctly");
    Ok(())
}