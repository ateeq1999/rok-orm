//! Phase 14A: Core Foundation Examples
//!
//! Features from Phases 1-6
//!
//! Run specific example:
//!   cargo run basic_model
//!   cargo run crud
//!   cargo run relations
//!   cargo run soft_delete
//!   cargo run timestamps
//!   cargo run pagination
//!   cargo run aggregations
//!   cargo run transactions
//!   cargo run scopes
//!   cargo run logging
//!
//! Or run all:
//!   cargo run all

mod basic_model;
mod crud;
mod relations;
mod soft_delete;
mod timestamps;
mod pagination;
mod aggregations;
mod transactions;
mod scopes;
mod logging;

use std::env;

#[tokio::main]
async fn main() -> rok_orm::OrmResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter("rok_orm_examples=debug,sqlx=warn")
        .init();

    dotenv::dotenv().ok();

    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://rok:rokpass@localhost:5432/rok_orm_14a".to_string());

    let pool = sqlx::PgPool::connect(&db_url).await?;
    println!("\n📦 Phase 14A: Core Foundation Examples\n");
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
        "basic_model" => basic_model::run(&pool).await?,
        "crud" | "crud_operations" => crud::run(&pool).await?,
        "relations" | "relationships" => relations::run(&pool).await?,
        "soft_delete" | "soft_deletes" => soft_delete::run(&pool).await?,
        "timestamps" => timestamps::run(&pool).await?,
        "pagination" => pagination::run(&pool).await?,
        "aggregations" => aggregations::run(&pool).await?,
        "transactions" => transactions::run(&pool).await?,
        "scopes" | "query_scopes" => scopes::run(&pool).await?,
        "logging" | "query_logging" => logging::run(&pool).await?,
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
    println!("  cargo run basic_model   - Model definition and metadata");
    println!("  cargo run crud          - Create, Read, Update, Delete");
    println!("  cargo run relations     - has_many, belongs_to relationships");
    println!("  cargo run soft_delete   - Soft delete patterns");
    println!("  cargo run timestamps    - Auto timestamps");
    println!("  cargo run pagination    - Page<T> pagination");
    println!("  cargo run aggregations  - count, sum, avg, min, max");
    println!("  cargo run transactions  - Tx wrapper");
    println!("  cargo run scopes        - Query scopes");
    println!("  cargo run logging       - Query logging");
    println!("  cargo run all           - Run all examples");
    println!();
    println!("Or run with RUST_LOG=debug to see all SQL queries:");
    println!("  RUST_LOG=debug cargo run crud");
}

async fn run_all(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n🔄 Running all examples...\n");

    basic_model::run(pool).await?;
    crud::run(pool).await?;
    relations::run(pool).await?;
    soft_delete::run(pool).await?;
    timestamps::run(pool).await?;
    pagination::run(pool).await?;
    aggregations::run(pool).await?;
    transactions::run(pool).await?;
    scopes::run(pool).await?;
    logging::run(pool).await?;

    Ok(())
}
