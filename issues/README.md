# rok-orm Example Bug Tracker

Issues found by running `cargo build` against `examples/14a-core`.

**Discovery date:** 2026-04-13
**Example project:** `examples/14a-core/src/`
**Build command:** `cd examples/14a-core && cargo build`

---

## Summary

| ID | Severity | File(s) | Title | Status |
|---|---|---|---|---|
| [bug-001](bug-001-missing-pgmodel-trait-imports.md) | critical | all except basic_model | Missing `PgModel` / `PgModelExt` trait imports | closed |
| [bug-002](bug-002-wrong-relations-derive-syntax.md) | critical | relationships.rs | Wrong `#[derive(Relations)]` syntax | closed |
| [bug-003](bug-003-querybuilder-missing-fluent-executor-methods.md) | critical | most files | `QueryBuilder` missing `.get()`, `.count()`, `.first()` | closed |
| [bug-004](bug-004-find-by-pk-returns-option-not-t.md) | high | crud_operations, timestamps | `find_by_pk` returns `Option<T>`, used as `T` | closed |
| [bug-005](bug-005-upsert-wrong-argument-signature.md) | high | crud_operations.rs | `upsert()` called with wrong / missing arguments | closed |
| [bug-006](bug-006-pagination-wrong-api-usage.md) | high | pagination.rs | `paginate()` on `QueryBuilder` receives pool argument | closed |
| [bug-007](bug-007-eager-loading-with-posts-field-missing.md) | high | relationships.rs | `.with("posts")` result not accessible as struct field | closed |
| [bug-008](bug-008-transactions-unescaped-format-braces.md) | low | transactions.rs | Unescaped `{` `}` in `println!` strings | closed |
| [bug-009](bug-009-query-scope-chaining-on-wrong-type.md) | medium | query_scopes.rs | Scope method `.role()` chained on `QueryBuilder` | closed |
| [bug-010](bug-010-aggregate-methods-wrong-arg-order-and-return-type.md) | high | aggregations.rs | `sum/avg/min/max` arg order inverted + wrong return types | closed |
| [bug-011](bug-011-soft-delete-wrong-static-method-names.md) | high | soft_deletes.rs | Soft delete called as static, not builder method | closed |
| [bug-012](bug-012-format-string-in-sqlvalue-context.md) | medium | relationships, pagination, aggregations | `.into()` inference ambiguity without trait in scope | closed |

---

## Fix Priority

### Must fix before `cargo build` succeeds

1. **bug-001** — Add `use rok_orm::{PgModel, PgModelExt};` to every example file.
   Unblocks most secondary errors automatically.

2. **bug-008** — Escape braces in `transactions.rs` `println!` calls. 2-line fix.

3. **bug-002** — Rewrite the `Relations` derive usage in `relationships.rs`.

4. **bug-003** — Largest change: either add fluent executor methods to `QueryBuilder`
   (API enhancement, affects the library) or rewrite example calls to use
   `PgModel` / `executor::postgres` APIs directly (affects only examples).

### Fix after build passes

5. **bug-004** — Use `find_or_404` or `.ok_or(RowNotFound)?` for `find_by_pk` results.
6. **bug-005** — Add missing `conflict_column` / `update_columns` to `upsert()` calls.
7. **bug-006** — Use `paginate_where(pool, builder, page, per_page)` for custom pagination.
8. **bug-010** — Swap arg order in `sum/avg/min/max`; fix `Option<i32>` → `Option<f64>`.
9. **bug-011** — Replace `Post::with_soft_delete()` with `Post::query().with_trashed()`.
10. **bug-009** — Replace `.role("user")` chain with `.filter("role", "user")`.
11. **bug-007** — Remove `u.posts.len()` or load posts separately; document limitation.
12. **bug-012** — Optional: use explicit `SqlValue::*` constructors for clarity.

---

## Root Cause Analysis

Three systemic problems account for almost all bugs:

### 1. Missing trait imports (bug-001)

Rust requires traits to be in scope before their methods are callable. The examples
were written as if `PgModel` / `PgModelExt` methods are auto-imported. All example
files need explicit `use rok_orm::{PgModel, PgModelExt};`.

### 2. Fluent executor API gap (bug-003)

The examples expect `QueryBuilder<T>` to be directly executable:

```rust
User::query().filter("active", true).get(pool).await?
```

The actual API separates query construction (`QueryBuilder`) from execution
(`PgModel` trait or `executor::postgres` free functions). This is the single most
impactful gap — it affects every example that chains query conditions with execution.

**Recommended long-term fix:** add feature-gated `.get(pool)`, `.count(pool)`,
`.first(pool)` methods to `QueryBuilder` so the fluent pattern works as written.

### 3. API mismatches from stale example code (bugs 004–012)

Several examples were written against an earlier or assumed API design that differed
from what was implemented: argument order for aggregates, `Option` vs `T` for
`find_by_pk`, static soft-delete methods, and relation derive syntax.