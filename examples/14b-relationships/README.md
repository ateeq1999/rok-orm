# Phase 14B: Rich Relationships & Developer Ergonomics Examples

Demonstrates features from Phases 7-8 of rok-orm.

## Features Covered

### Rich Relationships
- **ManyToMany with Pivot** — attach, detach, sync, toggle, with_pivot
- **HasManyThrough** — country → users → posts
- **HasOneThrough** — mechanic → car → owner
- **Polymorphic** — morphOne, morphMany, morphTo, morphToMany, morphedByMany
- **Relationship Writes** — create through, save, associate, dissociate
- **whereHas/whereDoesntHave** — filter by relationship existence
- **withCount/withSum/withAvg** — relationship aggregates as extras
- **firstOrCreate/updateOrCreate** — find-or-create patterns

### Advanced Model Features
- **UUID/ULID Primary Keys** — custom id generation
- **Per-Model Connections** — use different DB per model
- **withoutTimestamps** — suppress timestamp injection
- **Model Pruning** — automatic cleanup of old records
- **Event Muting** — without_events, save_quietly

### Developer Ergonomics
- **when()/when_else()** — conditional query chaining
- **Raw Expressions** — where_raw, select_raw, order_raw
- **tap()/dd()** — debug utilities
- **Chunking** — chunk, chunk_by_id for large datasets
- **Cursor Pagination** — efficient pagination for large tables
- **Mass Assignment** — fillable/guarded protection
- **Model Observers** — lifecycle event handlers
- **Global Query Scopes** — apply conditions to all queries
- **touches** — parent timestamp propagation

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

Expected output includes:
```
INFO Connected to database
INFO ManyToMany example - see documentation for full API
INFO HasManyThrough example - see documentation
INFO Posts with published comments: X
INFO Posts with >5 comments: X
INFO Users with no posts: X
...
```

## Viewing Database Data

### Option 1: Connect via psql

```bash
# Connect to the PostgreSQL container
docker exec -it rok-postgres-14b psql -U rok -d rok_orm_14b

# In psql:
# List tables: \dt
# Query data: SELECT * FROM users;
# View structure: \d users
# Check relationships: SELECT * FROM user_roles;
```

### Option 2: Run SQL from host

```bash
# List all tables
docker exec -i rok-postgres-14b psql -U rok -d rok_orm_14b -c "\dt"

# Query users
docker exec -i rok-postgres-14b psql -U rok -d rok_orm_14b -c "SELECT * FROM users LIMIT 10;"

# Query posts with user info
docker exec -i rok-postgres-14b psql -U rok -d rok_orm_14b -c "
  SELECT p.title, u.name as author 
  FROM posts p 
  JOIN users u ON p.user_id = u.id 
  LIMIT 10;
"

# Check pivot table (if ManyToMany used)
docker exec -i rok-postgres-14b psql -U rok -d rok_orm_14b -c "SELECT * FROM user_roles;"
```

### Option 3: Debug logging

Run with debug logging to see all SQL:

```bash
RUST_LOG=debug cargo run
```

This shows:
- Each SQL query executed
- Query parameters
- Execution time
- Row counts

### Option 4: Adminer (web UI)

Add to docker-compose.yml:

```yaml
services:
  adminer:
    image: adminer
    ports:
      - "8080:8080"
```

Access at http://localhost:8080
- System: PostgreSQL
- Server: postgres
- Username: rok
- Password: rokpass
- Database: rok_orm_14b

## Database Schema

This example creates tables based on the relationships:

```sql
-- Users table (with role, active fields for scope examples)
CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(255),
  email VARCHAR(255) UNIQUE,
  active BOOLEAN DEFAULT true,
  role VARCHAR(50),
  created_at TIMESTAMPTZ
);

-- Posts table (for has_many, whereHas examples)
CREATE TABLE posts (
  id BIGSERIAL PRIMARY KEY,
  title VARCHAR(255),
  body TEXT,
  user_id BIGINT REFERENCES users(id),
  country_id BIGINT,
  published BOOLEAN DEFAULT false,
  created_at TIMESTAMPTZ
);

-- Comments table (for withCount examples)
CREATE TABLE comments (
  id BIGSERIAL PRIMARY KEY,
  post_id BIGINT REFERENCES posts(id),
  body TEXT,
  published BOOLEAN DEFAULT true
);

-- Roles table (for ManyToMany examples)
CREATE TABLE roles (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(255),
  active BOOLEAN DEFAULT true
);

-- Pivot table for user_roles
CREATE TABLE user_roles (
  user_id BIGINT REFERENCES users(id),
  role_id BIGINT REFERENCES roles(id),
  assigned_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,
  PRIMARY KEY (user_id, role_id)
);

-- Activity logs (for pruning example)
CREATE TABLE activity_logs (
  id BIGSERIAL PRIMARY KEY,
  action VARCHAR(255),
  created_at TIMESTAMPTZ
);
```

## Project Structure

```
14b-relationships/
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
| DATABASE_URL | postgres://rok:rokpass@localhost:5432/rok_orm_14b | PostgreSQL connection |

## Stopping

```bash
# Stop containers
docker-compose down

# Remove data volumes
docker-compose down -v
```

## Common Queries to Try

```sql
-- View all users
SELECT * FROM users;

-- View posts with author
SELECT p.*, u.name as author_name FROM posts p JOIN users u ON p.user_id = u.id;

-- Count posts per user
SELECT u.name, COUNT(p.id) as post_count FROM users u LEFT JOIN posts p ON u.id = p.user_id GROUP BY u.id;

-- Users with published posts
SELECT DISTINCT u.* FROM users u JOIN posts p ON u.id = p.user_id WHERE p.published = true;

-- Posts with comment count
SELECT p.title, COUNT(c.id) as comment_count FROM posts p LEFT JOIN comments c ON p.id = c.post_id GROUP BY p.id;
```