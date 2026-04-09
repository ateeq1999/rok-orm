//! Phase 14C: Advanced Features Examples
//!
//! Features from Phases 9-13
//!
//! Run specific example:
//!   cargo run -- schema_builder
//!   cargo run -- json_columns
//!   cargo run -- full_text_search
//!
//! Or run all:
//!   cargo run -- all

mod schema_builder;
mod json_columns;
mod full_text_search;
mod subqueries;
mod window_functions;

use std::env;

#[tokio::main]
async fn main() -> rok_orm::OrmResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter("rok_orm_examples=debug,sqlx=warn")
        .init();
    
    dotenv::dotenv().ok();
    
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://rok:rokpass@localhost:5432/rok_orm_14c".to_string());
    
    let pool = sqlx::PgPool::connect(&db_url).await?;
    println!("\n📦 Phase 14C: Advanced Features\n");
    println!("Connected to database\n");
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        show_menu();
        return Ok(());
    }
    
    let example = &args[1];
    println!("Running: {}\n", example);
    println!("{}", "=".repeat(50));
    
    match example.as_str() {
        "schema_builder" | "schema" => schema_builder::run(&pool).await?,
        "json" | "json_columns" => json_columns::run(&pool).await?,
        "fts" | "full_text" | "full_text_search" => full_text_search::run(&pool).await?,
        "subqueries" | "subquery" => subqueries::run(&pool).await?,
        "windows" | "window_functions" => window_functions::run(&pool).await?,
        "all" => run_all(&pool).await?,
        _ => {
            println!("Unknown example: {}\n", example);
            show_menu();
        }
    }
    
    println!("\n✅ Done!");
    Ok(())
}

fn show_menu() {
    println!("Available examples:");
    println!("  cargo run schema         - Schema builder / Blueprint API");
    println!("  cargo run json           - JSON column queries");
    println!("  cargo run fts            - Full-text search");
    println!("  cargo run subqueries     - Sub-queries and CTEs");
    println!("  cargo run windows        - Window functions");
    println!("  cargo run all            - Run all examples");
    println!();
    println!("Run with RUST_LOG=debug to see SQL queries:");
    println!("  RUST_LOG=debug cargo run json");
}

async fn run_all(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n🔄 Running all examples...\n");
    
    schema_builder::run(pool).await?;
    json_columns::run(pool).await?;
    full_text_search::run(pool).await?;
    subqueries::run(pool).await?;
    window_functions::run(pool).await?;
    
    Ok(())
}