# Phase 10: Advanced Query Features

> **Target version:** v0.5.0
> **Status:** ✅ Complete
> **Note:** 10.4 (Window Functions) depends on 8.2 (select_raw) from Phase 8

---

## Goal

Support complex SQL patterns — JSON columns, full-text search, subqueries, CTEs, and window functions — without forcing users to drop to raw strings for common operations.

---

## 10.1 JSON Column Support

### API

```rust
// PostgreSQL JSONB, MySQL JSON, SQLite TEXT (json_extract)
let users = User::query()
    .where_json_contains("metadata", "role", "admin")
    .where_json_path("settings", "$.theme", "dark")
    .get(&pool)
    .await?;

// Extract JSON field in SELECT
let users = User::query()
    .select_json_field("metadata", "role", "user_role")
    .get(&pool)
    .await?;

// Check if JSON array contains value
let users = User::query()
    .where_json_array_contains("permissions", "posts:write")
    .get(&pool)
    .await?;
```

**SQL per dialect:**

| Method | PostgreSQL | SQLite | MySQL |
|--------|-----------|--------|-------|
| `where_json_contains(col, key, val)` | `col->>'key' = $1` | `json_extract(col,'$.key') = ?` | `JSON_VALUE(col,'$.key') = ?` |
| `where_json_path(col, path, val)` | `col #>> '{path}' = $1` | `json_extract(col,'$.path') = ?` | `JSON_VALUE(col,'$.path') = ?` |
| `select_json_field(col, key, alias)` | `col->>'key' AS alias` | `json_extract(col,'$.key') AS alias` | `JSON_VALUE(col,'$.key') AS alias` |
| `where_json_array_contains(col, val)` | `col @> $1::jsonb` | `json_each(col).value = ?` | `JSON_CONTAINS(col,?)` |

### Tasks

- [x] Add `where_json_contains(col, key, val)` to `QueryBuilder`
- [x] Add `where_json_path(col, path, val)` to `QueryBuilder`
- [x] Add `select_json_field(col, key, alias)` to `QueryBuilder`
- [x] Add `where_json_array_contains(col, val)` to `QueryBuilder`
- [x] Dialect detection in SQL generation (read `QueryBuilder.dialect`)
- [x] Tests: all methods on PG + SQLite, verify generated SQL per dialect

---

## 10.2 Full-Text Search

### API

```rust
// PostgreSQL tsvector
let posts = Post::query()
    .where_full_text(&["title", "body"], "rust async orm")
    .order_by_text_rank(&["title", "body"], "rust async orm")
    .get(&pool)
    .await?;

// MySQL FULLTEXT
let posts = Post::query()
    .where_match(&["title", "body"], "rust async orm")
    .get(&pool)
    .await?;

// SQLite FTS5 (requires separate FTS virtual table)
let posts = Post::query()
    .where_fts5("posts_fts", "rust async orm")
    .get(&pool)
    .await?;
```

**SQL per dialect:**

**PostgreSQL:**
```sql
WHERE to_tsvector('english', title || ' ' || body) @@ to_tsquery('english', 'rust & async & orm')
ORDER BY ts_rank(to_tsvector('english', title || ' ' || body), to_tsquery('english', 'rust & async & orm')) DESC
```

**MySQL:**
```sql
WHERE MATCH(title, body) AGAINST('rust async orm' IN NATURAL LANGUAGE MODE)
```

**SQLite FTS5:**
```sql
WHERE posts_fts MATCH 'rust async orm'
```

### Tasks

- [x] Add `where_full_text(cols, query)` — PostgreSQL `tsvector @@`
- [x] Add `order_by_text_rank(cols, query)` — PostgreSQL `ts_rank` ORDER BY
- [x] Add `where_match(cols, query)` — MySQL `MATCH ... AGAINST`
- [x] Add `where_fts5(fts_table, query)` — SQLite FTS5 MATCH
- [x] Parse `query` string into appropriate form (split on spaces → `word1 & word2`)
- [x] Tests: PG full-text, MySQL MATCH, SQLite FTS5

---

## 10.3 Sub-queries and CTEs

### API

```rust
// WHERE col IN (subquery)
let power_users = User::query()
    .where_in_subquery("id", |sq| {
        sq.table("orders")
          .select(&["user_id"])
          .group_by(&["user_id"])
          .having_raw("COUNT(*) > 10")
    })
    .get(&pool)
    .await?;

// WHERE EXISTS (subquery)
let users_with_orders = User::query()
    .where_exists(|sq| {
        sq.table("orders")
          .select(&["1"])
          .where_raw("orders.user_id = users.id", vec![])
    })
    .get(&pool)
    .await?;

// Common Table Expression (WITH)
let result = User::query()
    .with_cte("ranked", |cte| {
        cte.table("users")
           .select_raw("*, ROW_NUMBER() OVER (ORDER BY created_at DESC) AS rn")
    })
    .from_cte("ranked")
    .where_raw("rn <= 10", vec![])
    .get(&pool)
    .await?;

// Nested subquery in FROM
let result = User::query()
    .from_subquery("active_users", |sq| {
        sq.table("users").filter("active", true)
    })
    .get(&pool)
    .await?;
```

### Tasks

- [x] Add `SubQueryBuilder` — a variant of `QueryBuilder` that generates `SELECT ... FROM ... WHERE ...` without the outer statement
- [x] Add `where_in_subquery(col, closure)` — `WHERE col IN (SELECT ...)`
- [x] Add `where_exists(closure)` — `WHERE EXISTS (SELECT 1 FROM ... WHERE ...)`
- [x] Add `where_not_exists(closure)` — `WHERE NOT EXISTS (...)`
- [x] Add `with_cte(name, closure)` — prepends `WITH name AS (SELECT ...)` to the query
- [x] Add `from_cte(name)` — sets FROM to the named CTE
- [x] Add `from_subquery(alias, closure)` — sets FROM to `(SELECT ...) AS alias`
- [x] Placeholder numbering: subquery params are offset by outer param count
- [x] Tests: where_in_subquery, where_exists, CTE, nested subquery

---

## 10.4 Window Functions

### API

```rust
// Use select_raw (from Phase 8.2) with OVER clause
let posts = Post::query()
    .select_raw("*, ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at DESC) AS rn")
    .from_subquery("ranked", |sq| sq.table("posts").select_raw("*"))
    .where_raw("rn = 1", vec![])
    .get(&pool)
    .await?;

// Latest post per user
let posts = Post::query()
    .window_rank_by("user_id", "created_at", "row_num")
    .having_rank(1)
    .get(&pool)
    .await?;
```

### Tasks

- [x] `select_raw()` and `from_subquery()` already tracked (8.2, 10.3) — window functions build on them
- [x] Add `window_rank_by(partition_col, order_col, alias)` — generates `ROW_NUMBER() OVER (PARTITION BY ... ORDER BY ...) AS alias` in SELECT
- [x] Add `having_rank(n)` — wraps in outer subquery with `WHERE alias = n`
- [x] Tests: row_number, partition by, rank filter

---

## Acceptance Criteria for Phase 10

- [x] All 4 sub-sections implemented
- [x] JSON queries tested per dialect
- [x] Full-text tested on PG, MySQL, SQLite
- [x] Subqueries and CTEs tested with complex nesting
- [x] `cargo clippy -- -D warnings` clean
- [x] Phase file tasks all checked off
