# rok-orm Plan

> This file is the top-level index. Detailed plans live in [`plan/`](./plan/).

---

## Current Version: 0.5.0

## Quick Status

| Phase                                              | Title                            | Version        | Status      |
| -------------------------------------------------- | -------------------------------- | -------------- | ----------- |
| [1–6](./plan/phase_01-foundation-and-core.md)      | Foundation & Core                | v0.1–0.3       | ✅ Complete |
| [7](./plan/phase_07-rich-relationships.md)         | Rich Relationships               | v0.4.0         | ✅ Complete |
| [8](./plan/phase_08-developer-ergonomics.md)       | Developer Ergonomics             | v0.4.0         | ✅ Complete |
| [9](./plan/phase_09-schema-and-migrations.md)      | Schema Builder & Migrations      | v0.5.0         | ✅ Complete |
| [10](./plan/phase_10-advanced-query.md)            | Advanced Query Features          | v0.5.0         | ✅ Complete |
| [11](./plan/phase_11-casting-and-serialization.md) | Model Casting & Serialization    | v0.6.0         | 🔜 Planned  |
| [12](./plan/phase_12-testing-infrastructure.md)    | Testing Infrastructure           | v0.6.0         | 🔜 Planned  |
| [13](./plan/phase_13-ecosystem.md)                 | Ecosystem (MSSQL / Redis / Axum) | v1.0.0         | 🔜 Planned  |
| [CLI](./plan/orm-cli.md)                           | rok-cli Commands                 | separate crate | 🔜 Planned  |

---

## Master Plan

→ **[plan/README.md](./plan/README.md)** — implementation sequence, acceptance criteria, guidelines, PR checklist

---

## Code File Size Rule

> **No source file (`.rs`) may exceed 300 lines.**

If a file grows beyond 500 lines it must be split into focused sub-modules before the PR is merged. Each sub-module should own one clear responsibility (e.g. SQL generation, executor glue, a single relation type). This rule applies to all crates in the workspace including `proc-macro/`.

---

## Version History

| Version | Date    | Changes                                                                                                                                                                                                                 |
| ------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0.1.0   | 2024    | Initial release with QueryBuilder                                                                                                                                                                                       |
| 0.2.0   | 2026    | Soft delete, timestamps, relations, find_or_404, Eloquent-style API, model hooks, belongs_to_many                                                                                                                       |
| 0.3.0   | 2026    | Full soft delete, auto timestamps, eager loading, pagination, aggregates (sum/avg/min/max), upsert, batch ops, relation chaining, lazy loading, exists/pluck/update_all, query scopes, OrmError, logging, MySQL support |
| 0.4.0   | 2026    | Rich relationships (many-to-many pivot, has-many-through, polymorphic), whereHas, withCount, ergonomics (when, raw, tap, chunk, cursor pagination), mass assignment, observers, global scopes                           |
| 0.5.0   | 2026    | Schema builder (Blueprint API, all dialects), migration system (run/rollback/reset/fresh), auto-model generation from DB, SqlValue From<DateTime<Utc>>, Relations macro FK fix                                          |
| 0.6.0   | Q1 2027 | Model casting, serialization control, accessors/mutators, factories, transaction-per-test                                                                                                                               |
| 1.0.0   | Q2 2027 | MSSQL, Redis cache, Axum/Actix integration, stable public API                                                                                                                                                           |
