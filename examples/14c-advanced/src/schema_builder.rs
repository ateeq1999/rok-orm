//! Example: Schema Builder / Blueprint API
//!
//! Demonstrates: Schema::create, Blueprint, column types, foreign keys, indexes

use rok_orm::schema::{Schema, Blueprint, ForeignAction};

pub async fn run(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n📋 Schema Builder / Blueprint API\n");
    
    // Create table
    println!("1. Creating table with Blueprint...");
    Schema::create("demo_users", |t: &mut Blueprint| {
        t.id();
        t.string("name", 255);
        t.string("email", 255).unique();
        t.boolean("active").default(true);
        t.string("role", 50).default("user");
        t.timestamps();
    }).execute(pool).await?;
    println!("   ✅ Created: demo_users");
    
    // Create with foreign key
    println!("2. Creating table with foreign key...");
    Schema::create("demo_posts", |t| {
        t.big_increments("id");
        t.string("title", 255);
        t.text("body").nullable();
        t.foreign("user_id")
            .references("demo_users", "id")
            .on_delete(ForeignAction::Cascade);
        t.boolean("published").default(false);
        t.timestamps();
    }).execute(pool).await?;
    println!("   ✅ Created: demo_posts with FK");
    
    // Column types
    println!("3. Creating table with various column types...");
    Schema::create("demo_products", |t| {
        t.increments("id");
        t.big_increments("big_id");
        t.uuid("uuid_id");
        t.string("name", 255);
        t.text("description");
        t.integer("quantity");
        t.big_integer("price");
        t.float("score");
        t.double("amount");
        t.decimal("total", 10, 2);
        t.boolean("active");
        t.date("birthday");
        t.datetime("published_at");
        t.json("metadata").nullable();
        t.enum_col("status", &["draft", "published"]);
    }).execute(pool).await?;
    println!("   ✅ Created: demo_products with all types");
    
    // Alter table
    println!("4. Altering table (add column)...");
    Schema::alter("demo_users", |t| {
        t.add_column("avatar_url", |c| c.string(500).nullable());
    }).execute(pool).await?;
    println!("   ✅ Added avatar_url column");
    
    // Check existence
    println!("5. Schema inspection...");
    let exists = Schema::has_table(pool, "demo_users").await?;
    let has_col = Schema::has_column(pool, "demo_users", "email").await?;
    println!("   demo_users exists: {}", exists);
    println!("   has email column: {}", has_col);
    
    // Cleanup
    println!("6. Cleaning up...");
    sqlx::query("DROP TABLE IF EXISTS demo_products")
        .execute(pool).await.ok();
    sqlx::query("DROP TABLE IF EXISTS demo_posts")
        .execute(pool).await.ok();
    sqlx::query("DROP TABLE IF EXISTS demo_users")
        .execute(pool).await.ok();
    println!("   ✅ Tables dropped");
    
    println!("\n✅ Schema builder works correctly");
    Ok(())
}