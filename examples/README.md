# rok-orm Examples

This directory contains working examples for all rok-orm features, organized into three phases.

## Directory Structure

```
examples/
├── 14a-core/                 # Core Foundation examples (Phases 1-6)
│   ├── Cargo.toml
│   ├── docker-compose.yml
│   ├── .env.example
│   ├── README.md
│   └── src/main.rs
│
├── 14b-relationships/        # Rich Relationships examples (Phases 7-8)
│   ├── Cargo.toml
│   ├── docker-compose.yml
│   ├── .env.example
│   ├── README.md
│   └── src/main.rs
│
└── 14c-advanced/            # Advanced Features (Phases 9-13)
    ├── Cargo.toml
    ├── docker-compose.yml
    ├── .env.example
    ├── README.md
    └── src/main.rs
```

## Quick Start

Choose an example based on what you want to learn:

### 14A - Core Foundation (Beginner)
Basic features from Phases 1-6: models, CRUD, relationships, timestamps, pagination, hooks.

```bash
cd 14a-core
docker-compose up -d
cp .env.example .env
cargo run
```

### 14B - Relationships & Ergonomics (Intermediate)
Advanced relationships and developer ergonomics from Phases 7-8.

```bash
cd 14b-relationships
docker-compose up -d
cp .env.example .env
cargo run
```

### 14C - Advanced Features (Advanced)
Schema builder, migrations, JSON, full-text search from Phases 9-13.

```bash
cd 14c-advanced
docker-compose up -d
cp .env.example .env
cargo run
```

## Features by Phase

| Phase | Group | Topics |
|-------|-------|--------|
| 1-6 | 14A | Model, QueryBuilder, CRUD, relationships, soft deletes, timestamps, pagination, aggregations, hooks, transactions, scopes, logging |
| 7-8 | 14B | ManyToMany, HasManyThrough, Polymorphic, whereHas, withCount, firstOrCreate, when/when_else, raw expressions, chunking, cursor pagination, observers, scopes |
| 9-13 | 14C | Schema builder, migrations, JSON columns, full-text search, subqueries, CTEs, window functions, MSSQL, Redis cache, Axum |

## Viewing Database Data

Each example includes instructions for:

1. **psql/mysql CLI** — connect directly to database
2. **Docker exec** — run queries from host
3. **Debug logging** — RUST_LOG=debug cargo run
4. **Adminer** — optional web UI (see individual READMEs)

## Requirements

- Docker & Docker Compose
- Rust 1.78+
- Cargo

## Individual Documentation

See each example's README for detailed information:
- [14a-core/README.md](14a-core/README.md)
- [14b-relationships/README.md](14b-relationships/README.md)
- [14c-advanced/README.md](14c-advanced/README.md)