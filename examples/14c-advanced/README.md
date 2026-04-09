# Phase 14C: Advanced Features Examples

Demonstrates features from Phases 9-13 of rok-orm.

## Features Covered

### Schema & Migrations (Phase 9)
- **Schema Builder** — Blueprint API for create, alter, drop tables
- **Column Types** — id, string, text, integer, bigint, float, double, decimal, boolean, date, datetime, json, binary, enum
- **Column Modifiers** — nullable, default, unique, not_null, primary
- **Foreign Keys** — references, on_delete, on_update
- **Indexes** — index, unique_index
- **Migration System** — Migration trait, Migrator with run/rollback/reset/fresh
- **Schema Inspection** — has_table, has_column

### Advanced Query (Phase 10)
- **JSON Column Support** — where_json_contains, where_json_path, select_json_field
- **Full-Text Search** — PostgreSQL tsvector, MySQL MATCH, SQLite FTS5
- **Sub-queries** — where_in_subquery, where_exists, where_not_exists
- **CTEs** — with_cte, from_cte, from_subquery
- **Window Functions** — ROW_NUMBER, RANK, LAG, LEAD

### Ecosystem (Phase 13)
- **MSSQL Support** — TOP, OUTPUT INSERTED, MERGE
- **Redis Cache** — find_cached, invalidate_cache, cache_as, flush_cache
- **Axum Integration** — DbPool extractor, OrmErrorResponse

## Prerequisites

- Docker & Docker Compose
- Rust 1.78+
- Cargo

## Quick Start

### 1. Start Services

```bash
docker-compose up -d
```

This starts:
- PostgreSQL (port 5432)
- MySQL (port 3306) — optional
- Redis (port 6379) — optional

### 2. Configure Environment

```bash
cp .env.example .env
# Edit .env if needed
```

### 3. Run the Example

```bash
cargo run
```

Expected output:
```
INFO Connected to PostgreSQL
INFO === Schema Builder: Create Table ===
INFO Created example_users table
INFO === Schema Builder: Column Types ===
INFO Created example_products
INFO === JSON Column Support ===
INFO Inserted JSON data
INFO === Full-Text Search ===
...
```

## Viewing Database Data

### PostgreSQL (Primary)

```bash
# Connect to PostgreSQL container
docker exec -it rok-postgres-14c psql -U rok -d rok_orm_14c

# List all tables
\d

# Query tables
SELECT * FROM example_users;
SELECT * FROM example_products;
SELECT * FROM posts;

# Check indexes
SELECT indexname, indexdef FROM pg_indexes WHERE schemaname = 'public';

# Check constraints
SELECT conname, contype FROM pg_constraint;
```

### MySQL (Secondary)

```bash
# Connect to MySQL container
docker exec -it rok-mysql-14c mysql -u rok -prokpass rok_orm_14c

# List tables
SHOW TABLES;

# Query data
SELECT * FROM example_users;
```

### Redis (Cache)

```bash
# Connect to Redis
docker exec -it rok-redis-14c redis-cli

# List keys
KEYS *

# Get value
GET users:1

# Check all keys pattern
KEYS users:*
```

### Debug Logging

```bash
RUST_LOG=debug cargo run
```

Shows all SQL queries with parameters and timing.

### Adminer (Web UI)

Add to docker-compose.yml:

```yaml
services:
  adminer:
    image: adminer
    ports:
      - "8080:8080"
```

Access at http://localhost:8080
- PostgreSQL: Server=postgres, User=rok, Pass=rokpass, DB=rok_orm_14c
- MySQL: Server=mysql, User=rok, Pass=rokpass, DB=rok_orm_14c

## Database Schema Created

The example creates these tables:

### PostgreSQL

```sql
-- Users with JSON columns
CREATE TABLE example_users (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(255),
  email VARCHAR(255) UNIQUE,
  avatar_url VARCHAR(500),
  metadata JSONB,
  settings JSONB,
  permissions JSONB,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ
);

-- Products with various column types
CREATE TABLE example_products (
  id SERIAL PRIMARY KEY,
  big_id BIGSERIAL PRIMARY KEY,
  uuid_id UUID,
  name VARCHAR(255),
  description TEXT,
  quantity INTEGER,
  price BIGINT,
  priority SMALLINT,
  score REAL,
  amount DOUBLE PRECISION,
  total DECIMAL(10,2),
  active BOOLEAN,
  birthday DATE,
  published_at TIMESTAMPTZ,
  metadata JSONB,
  data BYTEA,
  status VARCHAR(20)
);

-- Posts for full-text search
CREATE TABLE posts (
  id BIGSERIAL PRIMARY KEY,
  title VARCHAR(255),
  body TEXT,
  tags JSONB,
  created_at TIMESTAMPTZ
);

-- Index for full-text search
CREATE INDEX idx_posts_fts ON posts USING gin(to_tsvector('english', title || ' ' COALESCE(body, '')));
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| DATABASE_URL | postgres://rok:rokpass@localhost:5432/rok_orm_14c | PostgreSQL connection |
| MYSQL_URL | mysql://rok:rokpass@localhost:3306/rok_orm_14c | MySQL connection (optional) |
| REDIS_URL | redis://localhost:6379 | Redis connection (optional) |

## Project Structure

```
14c-advanced/
├── Cargo.toml
├── docker-compose.yml
├── .env.example
├── .env (generated)
└── src/
    └── main.rs
```

## Common Queries

### PostgreSQL

```sql
-- View all tables
SELECT table_name FROM information_schema.tables WHERE table_schema = 'public';

-- Query JSON column
SELECT name, metadata->>'role' as role FROM example_users WHERE metadata->>'verified' = 'true';

-- Full-text search
SELECT * FROM posts WHERE to_tsvector('english', title || ' ' || COALESCE(body, '')) @@ to_tsquery('english', 'rust & orm');

-- Window function example
SELECT id, name, ROW_NUMBER() OVER (ORDER BY created_at DESC) as rn FROM example_users;
```

### MySQL

```sql
-- JSON queries
SELECT name, JSON_EXTRACT(metadata, '$.role') FROM example_users;

-- Full-text search
SELECT * FROM posts WHERE MATCH(title, body) AGAINST('rust orm' IN NATURAL LANGUAGE MODE);
```

## Stopping

```bash
# Stop containers
docker-compose down

# Remove data volumes
docker-compose down -v

# Remove only specific volume
docker volume rm examples_postgres_data examples_mysql_data examples_redis_data
```

## Troubleshooting

### PostgreSQL won't start
```bash
# Check logs
docker logs rok-postgres-14c

# Check port availability
lsof -i :5432
```

### Connection refused
```bash
# Wait for container to be ready
docker-compose up -d
sleep 5

# Test connection
docker exec -it rok-postgres-14c pg_isready -U rok -d rok_orm_14c
```

### Data cleanup
```bash
# Drop and recreate database
docker exec -it rok-postgres-14c psql -U rok -d postgres -c "DROP DATABASE IF EXISTS rok_orm_14c; CREATE DATABASE rok_orm_14c;"
```