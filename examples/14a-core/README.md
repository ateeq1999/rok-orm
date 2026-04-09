# Phase 14A: Core Foundation Examples

Demonstrates features from Phases 1-6 of rok-orm.

## Features Covered

- **Basic Model Definition** — `#[derive(Model)]` with table/column mapping
- **Query Builder** — Fluent API with conditions, ordering, limits
- **CRUD Operations** — create, read, update, delete, upsert
- **Basic Relationships** — has_many, belongs_to, eager loading
- **Soft Deletes** — with_trashed, only_trashed, restore, force_delete
- **Auto Timestamps** — created_at, updated_at management
- **Pagination** — Page<T> with page numbers
- **Aggregations** — count, sum, avg, min, max
- **Model Hooks** — before_create, after_create, etc.
- **Transactions** — Tx wrapper with commit/rollback
- **Query Scopes** — reusable query builders
- **Query Logging** — Logger with slow query detection

## Prerequisites

- Docker & Docker Compose
- Rust 1.78+
- Cargo

## Quick Start

### 1. Start PostgreSQL

```bash
docker-compose up -d
```

### 2. Configure Environment

```bash
cp .env.example .env
# Edit .env if needed (defaults should work)
```

### 3. Run the Example

```bash
cargo run
```

You should see output like:

```
INFO Connected to database
INFO Table: users, Columns: ["id", "name", "email", "active", "created_at", "updated_at"]
INFO SQL: SELECT * FROM "users" WHERE "active" = $1 ORDER BY "created_at" DESC LIMIT $2
INFO Created user id=1
...
```

## Viewing Database Data

### Option 1: Connect via psql

```bash
# Connect to the PostgreSQL container
docker exec -it rok-postgres-14a psql -U rok -d rok_orm_14a

# In psql, you can run:
# List tables: \dt
# Query data: SELECT * FROM users;
# View schema: \d users
```

### Option 2: Run SQL from host

```bash
# Query users table
docker exec -i rok-postgres-14a psql -U rok -d rok_orm_14a -c "SELECT * FROM users;"

# List all tables
docker exec -i rok-postgres-14a psql -U rok -d rok_orm_14a -c "\dt"
```

### Option 3: Check logs in application

The example logs all database operations. Run with:

```bash
RUST_LOG=debug cargo run
```

This shows:
- SQL queries executed
- Parameters passed
- Query execution time
- Rows affected

### Option 4: Use adminer (optional)

Add to docker-compose.yml for web-based database access:

```yaml
services:
  adminer:
    image: adminer
    ports:
      - "8080:8080"
```

Then visit http://localhost:8080
- System: PostgreSQL
- Server: postgres
- Username: rok
- Password: rokpass
- Database: rok_orm_14a

## Project Structure

```
14a-core/
├── Cargo.toml
├── docker-compose.yml
├── .env.example
├── .env (generated)
└── src/
    └── main.rs
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| DATABASE_URL | postgres://rok:rokpass@localhost:5432/rok_orm_14a | PostgreSQL connection string |

## Stopping

```bash
# Stop containers
docker-compose down

# Remove data volumes (will delete all data)
docker-compose down -v
```