---
id: bug-012
type: bug
severity: medium
affects: examples/14a-core/src/relationships.rs:53-56
         examples/14a-core/src/pagination.rs:22-25
         examples/14a-core/src/aggregations.rs:40-45
---

# `format!(...)` and Loop-Variable `.into()` Used as `SqlValue` Without Explicit Type

## Description

Several examples pass `format!(...)` results and numeric loop variables with `.into()`
as `SqlValue` pairs. While `SqlValue` has `From<String>`, `From<i64>`, and
`From<f64>` implementations, two patterns can cause inference failures when the
surrounding method is not yet resolved (e.g. due to bug-001 / bug-003):

1. **`format!(...).into()`** — `String` → `SqlValue::Text` via `From<String>` ✓
   (works once PgModel is in scope)

2. **`(100.0 * i as f64).into()`** — `f64` → `SqlValue::Float` via `From<f64>` ✓
   (works once PgModel is in scope)

3. **`true.into()`** in aggregations.rs — compiles once method is resolved,
   since `SqlValue: From<bool>` ✓

These are **not standalone bugs** — they resolve automatically once bug-001 is fixed.
However, if explicit types are preferred for clarity, use the `SqlValue` constructors:

```rust
// Explicit — always unambiguous:
("title", SqlValue::Text(format!("Post {}", i)))
("user_id", SqlValue::Integer(user.id))
("active", SqlValue::Bool(true))
("total", SqlValue::Float(100.0 * i as f64))

// Implicit via From — works once trait is in scope:
("title", format!("Post {}", i).into())
("user_id", user.id.into())     // i64 → SqlValue::Integer
("active", true.into())          // bool → SqlValue::Bool
("total", (100.0 * i as f64).into())
```

## Recommendation

Prefer explicit `SqlValue::*` constructors in examples to make the type mapping
self-documenting and avoid ambiguity during future API changes.
