# rok-orm — Master Plan

> Implementation tracker, sequence guide, acceptance criteria, and contribution guidelines.

---

## Progress Overview

| Phase                                         | Title                                       | Version        | Status         |
| --------------------------------------------- | ------------------------------------------- | -------------- | -------------- |
| [1–6](./phase_01-foundation-and-core.md)      | Foundation & Core (all prior work)          | v0.1–0.3       | ✅ Complete    |
| —                                             | File-size refactor (500-line rule enforced) | v0.4.0         | ✅ Complete    |
| [7](./phase_07-rich-relationships.md)         | Rich Relationships                          | v0.4.0         | ✅ Complete    |
| [8](./phase_08-developer-ergonomics.md)       | Developer Ergonomics                        | v0.4.0         | ✅ Complete    |
| [9](./phase_09-schema-and-migrations.md)      | Schema Builder & Migrations                 | v0.5.0         | ✅ Complete    |
| [10](./phase_10-advanced-query.md)            | Advanced Query Features                     | v0.5.0         | ✅ Complete    |
| [11](./phase_11-casting-and-serialization.md) | Model Casting & Serialization               | v0.6.0         | 🔜 Planned     |
| [12](./phase_12-testing-infrastructure.md)    | Testing Infrastructure                      | v0.6.0         | 🔜 Planned     |
| [13](./phase_13-ecosystem.md)                 | Ecosystem (MSSQL, Redis, Axum)              | v1.0.0         | 🔜 Planned     |
| [14](./phase_14-examples.md)                  | Examples Implementation                     | v0.5.0         | 🚧 In Progress |
| [CLI](./orm-cli.md)                           | rok-cli — Command-Line Tool                 | separate crate | 🔜 Planned     |

---

## Version Targets

| Version | Target  | Phases    |
| ------- | ------- | --------- |
| v0.4.0  | Q3 2026 | 7, 8      |
| v0.5.0  | Q4 2026 | 9, 10, 14 |

### v0.5.0 — Sprint 3: Schema & Advanced Query (Phases 9, 10, 14)

Dependency order matters — Phase 9.1 (Schema Builder) must come before 9.2 (Migrations). Phase 10 is independent. Phase 14 documents examples.

```
9.1  Schema builder / Blueprint API
9.2  Migration system (depends on 9.1)
9.3  Auto-model generation from DB (depends on 9.1 inspector)
10.1 JSON column support
10.2 Full-text search
10.3 Sub-queries and CTEs
10.4 Window functions (extends 8.2 select_raw)
14.1 Rich Relationships examples (Phase 7)
14.2 Developer Ergonomics examples (Phase 8)
14.3 Schema Builder examples (Phase 9)
14.4 Migration System examples (Phase 9)
14.5 Advanced Query examples (Phase 10)
```

Phase 9.1 (Schema Builder) must come before 9.2 (Migrations). Phase 10 is independent.

```
9.1  Schema builder / Blueprint API
9.2  Migration system (depends on 9.1)
9.3  Auto-model generation from DB (depends on 9.1 inspector)
10.1 JSON column support
10.2 Full-text search
10.3 Sub-queries and CTEs
10.4 Window functions (extends 8.2 select_raw)
```

### v0.6.0 — Casting, Serialization & Testing (Phases 11, 12)

Phase 11 and 12 are independent of each other.

```
11.1 Attribute casting (json / datetime / bool / csv / encrypted)
11.2 Serialization control (hidden / visible / appends)
11.3 Accessors and mutators
12.1 Model factories with faker
12.2 Database transaction per test
12.3 Assertion helpers
```

### v1.0.0 — Ecosystem (Phase 13)

All sub-sections are independent.

```
13.1 MSSQL / SQL Server
13.2 Redis cache integration
13.3 Axum / Actix-web integration
```

---

## Acceptance Criteria

Every feature merged into `main` must satisfy **all** of the following:

### 1. Correctness

- [ ] Feature works as documented in the phase file
- [ ] SQL generated is syntactically correct and tested on target dialects (PG / SQLite / MySQL)
- [ ] Edge cases handled: empty inputs, NULL values, large datasets, concurrent access

### 2. Tests

- [ ] Unit tests for SQL generation (no DB required) — in `src/` next to the code
- [ ] Integration tests against a real DB — in `tests/`
- [ ] Each `- [ ]` task in the phase file has at least one corresponding test
- [ ] `cargo test --workspace` passes with zero failures

### 3. Code Quality

- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt --all` applied
- [ ] No `unwrap()` / `expect()` in library code (use `?` or return `OrmResult`)
- [ ] No `println!` / `eprintln!` left in library code (use `tracing::debug!` / `log::debug!`)

### 4. Public API

- [ ] All public types, traits, and methods have doc comments with at least one example
- [ ] New macro attributes are documented in the `#[model(...)]` attribute list in README
- [ ] No breaking changes to existing public API without a version bump discussion
- [ ] Feature flags documented in `Cargo.toml` and README

### 5. Documentation

- [ ] Phase file updated: all `- [ ]` tasks changed to `- [x]` when done
- [ ] `plan/README.md` progress table updated
- [ ] README.md updated if user-visible API changed
- [ ] USER_MANUAL.md updated with usage examples for new features

---

## Implementation Guidelines

### Struct Naming Convention

| Concept                    | Rust name                                |
| -------------------------- | ---------------------------------------- |
| Relationship type          | `HasMany<P, C>`, `MorphOne<P, C>`        |
| Relationship query builder | `HasManyQuery<P, C>`                     |
| Pivot row                  | `PivotRow`                               |
| Cursor state               | `CursorPage`, `CursorResult<T>`          |
| Schema operation           | `Schema`, `Blueprint`, `ColumnDef`       |
| Migration                  | `Migration` (trait), `Migrator` (runner) |
| Observer                   | `ModelObserver` (trait)                  |
| Global scope               | `GlobalScope<M>` (trait)                 |

### Error Handling

- Always return `OrmResult<T>` (alias for `Result<T, OrmError>`) in public async methods
- Map `sqlx::Error` using `OrmError::from_sqlx_error()` at executor boundaries
- Internal helpers may return `Result<T, sqlx::Error>` if they don't cross the public boundary

### Feature Flag Discipline

- Database-specific code lives behind `#[cfg(feature = "postgres")]` etc.
- Test utilities live behind `#[cfg(any(test, feature = "test-utils"))]`
- Keep core query logic (`src/query/`) feature-flag-free

### Macro Attribute Extension Pattern

When adding a new `#[model(...)]` attribute:

1. Add parsing in `proc-macro/src/lib.rs` — new `ModelAttr` variant
2. Add the generated method to `model.rs` `Model` trait with a default impl
3. Override in the macro's generated `impl Model for Struct`
4. Document the new attribute in the macro's doc comment

### SQL Dialect Checklist

For any new SQL clause, verify correctness on all three dialects:

| Clause       | PostgreSQL    | SQLite         | MySQL                   |
| ------------ | ------------- | -------------- | ----------------------- |
| Placeholders | `$1, $2`      | `?, ?`         | `?, ?`                  |
| RETURNING    | ✅            | ✅ (3.35+)     | ❌ (use last_insert_id) |
| ON CONFLICT  | ✅            | ✅             | ❌ (ON DUPLICATE KEY)   |
| JSON extract | `->>`         | `json_extract` | `JSON_VALUE`            |
| Full-text    | `@@` tsvector | FTS5           | MATCH AGAINST           |

### File Size Rule

> **Hard limit: no `.rs` source file may exceed 500 lines.**

When a file reaches this limit, split it into focused sub-modules before committing:

| Too large                         | Split into                                                                 |
| --------------------------------- | -------------------------------------------------------------------------- |
| `query.rs` (builder + SQL gen)    | `query/builder.rs` + `query/sql_gen.rs`                                    |
| `pg_model.rs` (CRUD + aggregates) | `pg_model/crud.rs` + `pg_model/aggregates.rs`                              |
| `relations.rs` (all types)        | `relations/has_many.rs`, `relations/has_one.rs`, `relations/belongs_to.rs` |
| `proc-macro/lib.rs`               | `derive_model.rs`, `derive_relations.rs`, `query_macro.rs`                 |

Each sub-module must have a single clear responsibility. `mod.rs` files only re-export — no logic.

---

### Commit Style

```
feat(phase-7): add ManyToMany pivot attach/detach/sync
fix(relations): correct foreign key injection in has_many_through
test(phase-8): add chunk_by_id integration test
docs(readme): document withCount usage
```

---

## Architecture Reference

```
rok-orm/
├── plan/                       ← You are here
│   ├── README.md               ← This file (master tracker)
│   ├── phase_01-foundation-and-core.md
│   ├── phase_07-rich-relationships.md
│   ├── phase_08-developer-ergonomics.md
│   ├── phase_09-schema-and-migrations.md
│   ├── phase_10-advanced-query.md
│   ├── phase_11-casting-and-serialization.md
│   ├── phase_12-testing-infrastructure.md
│   ├── phase_14-examples.md
│   ├── phase_13-ecosystem.md
│   └── orm-cli.md
│
├── examples/                    ← Working examples with Docker
│   ├── docker-compose.yml       ← All services combined
│   ├── docker-compose.14a.yml   ← Phase 14A: Core Foundation
│   ├── docker-compose.14b.yml   ← Phase 14B: Relationships
│   ├── docker-compose.14c.yml   ← Phase 14C: Advanced Features
│   ├── README.md
│   ├── 14a-core/                ← Core examples (Phases 1-6)
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   ├── 14b-relationships/        ← Relationship examples (Phases 7-8)
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── 14c-advanced/            ← Advanced examples (Phases 9-13)
│       ├── Cargo.toml
│       └── src/main.rs
│
├── src/
│   ├── model/          model.rs, pg_model.rs, sqlite_model.rs, mysql_model.rs
│   ├── query/          query.rs, condition.rs
│   ├── executor/       postgres.rs, sqlite.rs, mysql.rs, sqlx_pg.rs, sqlx_sqlite.rs
│   ├── relations/      relations.rs, belongs_to_many.rs, eager.rs
│   ├── schema/         (planned) blueprint.rs, column.rs, inspector.rs
│   ├── errors.rs
│   ├── hooks.rs
│   ├── logging.rs
│   ├── pagination.rs
│   ├── scopes.rs
│   └── transaction.rs
│
└── proc-macro/
    └── src/lib.rs      #[derive(Model)], #[derive(Relations)], query!()
```

---

## PR Checklist (copy into every PR description)

```markdown
## Checklist

- [ ] Phase file tasks checked off
- [ ] Unit tests added (SQL generation)
- [ ] Integration tests added (real DB)
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt --all` applied
- [ ] Public API has doc comments
- [ ] README / USER_MANUAL updated if needed
- [ ] plan/README.md progress table updated
```
