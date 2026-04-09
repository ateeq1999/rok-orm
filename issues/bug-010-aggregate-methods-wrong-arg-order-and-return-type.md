---
id: bug-010
type: bug
severity: high
affects: examples/14a-core/src/aggregations.rs:58-72
---

# Aggregate Methods Called with Wrong Argument Order and Wrong Return Types

## Description

`aggregations.rs` calls `sum`, `avg`, `min`, `max` with arguments in the wrong order
and uses incorrect return-type annotations for `min`/`max`.

### 1. Argument order inverted

`PgModelExt` defines:
```rust
async fn sum(pool: &PgPool, column: &str) -> Result<Option<f64>, sqlx::Error>
async fn avg(pool: &PgPool, column: &str) -> Result<Option<f64>, sqlx::Error>
async fn min(pool: &PgPool, column: &str) -> Result<Option<f64>, sqlx::Error>
async fn max(pool: &PgPool, column: &str) -> Result<Option<f64>, sqlx::Error>
```

The examples pass `(column, pool)` — reversed:

```rust
let revenue: f64 = Order::sum("total", pool).await?;   // ❌ reversed
let avg_age: f64 = User::avg("age", pool).await?;       // ❌ reversed
let oldest: Option<i32> = User::min("age", pool).await?;// ❌ reversed + wrong type
let youngest: Option<i32> = User::max("age", pool).await?;// ❌ reversed + wrong type
```

### 2. Return type mismatch

`min` and `max` return `Option<f64>`, not `Option<i32>`. The type annotations will
cause a type mismatch error even after fixing the arg order.

### 3. Unwrapping Option<f64> as f64

Lines 58–67 bind into `f64` but the methods return `Option<f64>`. Assigning
`Option<f64>` to `f64` fails.

## Fix

```rust
// pool first, column second; unwrap the Option:
let revenue = Order::sum(pool, "total").await?.unwrap_or(0.0);
println!("   Total revenue: ${:.2}", revenue);

let avg_age = User::avg(pool, "age").await?.unwrap_or(0.0);
println!("   Average user age: {:.1}", avg_age);

let avg_order = Order::avg(pool, "total").await?.unwrap_or(0.0);
println!("   Average order value: ${:.2}", avg_order);

// min/max return Option<f64>:
let youngest_age: Option<f64> = User::min(pool, "age").await?;
let oldest_age: Option<f64>   = User::max(pool, "age").await?;
println!("   Youngest user age: {:?}", youngest_age);
println!("   Oldest user age: {:?}", oldest_age);
```
