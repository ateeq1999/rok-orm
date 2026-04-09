//! Phase 14B: Rich Relationships & Developer Ergonomics Examples
//!
//! Features from Phases 7-8
//!
//! Run specific example:
//!   cargo run -- many_to_many
//!   cargo run -- where_has
//!   cargo run -- with_count
//!
//! Or run all:
//!   cargo run -- all

mod many_to_many;
mod where_has;
mod with_count;
mod first_or_create;
mod without_timestamps;
mod event_muting;
mod when_conditional;
mod raw_expressions;
mod cursor_pagination;
mod model_observers;
mod global_scopes;

use std::env;

#[tokio::main]
async fn main() -> rok_orm::OrmResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter("rok_orm_examples=debug,sqlx=warn")
        .init();
    
    dotenv::dotenv().ok();
    
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://rok:rokpass@localhost:5432/rok_orm_14b".to_string());
    
    let pool = sqlx::PgPool::connect(&db_url).await?;
    println!("\n📦 Phase 14B: Rich Relationships & Developer Ergonomics\n");
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
        "many_to_many" | "m2m" => many_to_many::run(&pool).await?,
        "where_has" => where_has::run(&pool).await?,
        "with_count" | "count" => with_count::run(&pool).await?,
        "first_or_create" | "foa" => first_or_create::run(&pool).await?,
        "without_timestamps" | "no_ts" => without_timestamps::run(&pool).await?,
        "event_muting" | "mute" => event_muting::run(&pool).await?,
        "when" | "conditional" => when_conditional::run(&pool).await?,
        "raw" | "raw_expressions" => raw_expressions::run(&pool).await?,
        "cursor" | "cursor_pagination" => cursor_pagination::run(&pool).await?,
        "observers" | "model_observers" => model_observers::run(&pool).await?,
        "scopes" | "global_scopes" => global_scopes::run(&pool).await?,
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
    println!("  cargo run many_to_many       - BelongsToMany with pivot access");
    println!("  cargo run where_has          - Filter by relationship existence");
    println!("  cargo run with_count         - Relationship aggregates as extras");
    println!("  cargo run first_or_create    - Find-or-create patterns");
    println!("  cargo run without_timestamps - Suppress timestamp injection");
    println!("  cargo run event_muting       - Suppress model events");
    println!("  cargo run when              - Conditional query chaining");
    println!("  cargo run raw               - Raw SQL expressions");
    println!("  cargo run cursor            - Cursor-based pagination");
    println!("  cargo run observers         - Model lifecycle observers");
    println!("  cargo run scopes            - Global query scopes");
    println!("  cargo run all               - Run all examples");
    println!();
    println!("Run with RUST_LOG=debug to see SQL queries:");
    println!("  RUST_LOG=debug cargo run where_has");
}

async fn run_all(pool: &sqlx::PgPool) -> rok_orm::OrmResult<()> {
    println!("\n🔄 Running all examples...\n");
    
    many_to_many::run(pool).await?;
    where_has::run(pool).await?;
    with_count::run(pool).await?;
    first_or_create::run(pool).await?;
    without_timestamps::run(pool).await?;
    event_muting::run(pool).await?;
    when_conditional::run(pool).await?;
    raw_expressions::run(pool).await?;
    cursor_pagination::run(pool).await?;
    model_observers::run(pool).await?;
    global_scopes::run(pool).await?;
    
    Ok(())
}