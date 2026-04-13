//! Phase 10 unit tests: JSON columns, full-text search, sub-queries, CTEs, window functions.

use crate::query::{Dialect, QueryBuilder};

// ── 10.1 JSON column support ──────────────────────────────────────────────────

#[test]
fn json_contains_postgres() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_json_contains("metadata", "role", "admin")
        .to_sql();
    assert!(sql.contains("metadata->>'role'"), "pg json key: {sql}");
    assert!(sql.contains("$1"), "pg placeholder: {sql}");
    assert_eq!(params.len(), 1);
}

#[test]
fn json_contains_sqlite() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .with_dialect(Dialect::Sqlite)
        .where_json_contains("metadata", "role", "admin")
        .to_sql_with_dialect(Dialect::Sqlite);
    assert!(sql.contains("json_extract(metadata,'$.role')"), "sqlite json: {sql}");
    assert!(sql.contains('?'), "sqlite placeholder: {sql}");
    assert_eq!(params.len(), 1);
}

#[test]
fn json_contains_mysql() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .with_dialect(Dialect::Mysql)
        .where_json_contains("metadata", "role", "admin")
        .to_sql_with_dialect(Dialect::Mysql);
    assert!(sql.contains("JSON_VALUE(metadata,'$.role')"), "mysql json: {sql}");
    assert_eq!(params.len(), 1);
}

#[test]
fn json_path_postgres() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_json_path("settings", "$.theme", "dark")
        .to_sql();
    assert!(sql.contains("settings #>> '{theme}'"), "pg json path: {sql}");
    assert_eq!(params.len(), 1);
}

#[test]
fn json_path_sqlite() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .with_dialect(Dialect::Sqlite)
        .where_json_path("settings", "$.theme", "dark")
        .to_sql_with_dialect(Dialect::Sqlite);
    assert!(sql.contains("json_extract(settings,'$.theme')"), "sqlite path: {sql}");
}

#[test]
fn json_array_contains_postgres() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_json_array_contains("permissions", "posts:write")
        .to_sql();
    assert!(sql.contains("permissions @>"), "pg array contains: {sql}");
    assert!(sql.contains("::jsonb"), "pg jsonb cast: {sql}");
    assert_eq!(params.len(), 1);
}

#[test]
fn json_array_contains_sqlite() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .with_dialect(Dialect::Sqlite)
        .where_json_array_contains("permissions", "posts:write")
        .to_sql_with_dialect(Dialect::Sqlite);
    assert!(sql.contains("json_each(permissions)"), "sqlite array: {sql}");
}

#[test]
fn json_array_contains_mysql() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .with_dialect(Dialect::Mysql)
        .where_json_array_contains("permissions", "posts:write")
        .to_sql_with_dialect(Dialect::Mysql);
    assert!(sql.contains("JSON_CONTAINS(permissions"), "mysql array: {sql}");
}

#[test]
fn select_json_field_postgres() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .select_json_field("metadata", "role", "user_role")
        .to_sql();
    assert!(sql.contains("metadata->>'role' AS user_role"), "pg select json: {sql}");
    assert!(sql.contains('*'), "star preserved: {sql}");
}

#[test]
fn select_json_field_sqlite() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .with_dialect(Dialect::Sqlite)
        .select_json_field("metadata", "role", "user_role")
        .to_sql_with_dialect(Dialect::Sqlite);
    assert!(
        sql.contains("json_extract(metadata,'$.role') AS user_role"),
        "sqlite select json: {sql}"
    );
}

// ── 10.2 Full-text search ─────────────────────────────────────────────────────

#[test]
fn fts_where_full_text_postgres() {
    let (sql, params) = QueryBuilder::<()>::new("posts")
        .where_full_text(&["title", "body"], "rust async orm")
        .to_sql();
    assert!(sql.contains("to_tsvector('english'"), "tsvector: {sql}");
    assert!(sql.contains("to_tsquery('english'"), "tsquery: {sql}");
    assert!(sql.contains("rust & async & orm"), "operands: {sql}");
    assert!(params.is_empty(), "no params for fts raw: {params:?}");
}

#[test]
fn fts_order_by_text_rank() {
    let (sql, _) = QueryBuilder::<()>::new("posts")
        .where_full_text(&["title", "body"], "rust orm")
        .order_by_text_rank(&["title", "body"], "rust orm")
        .to_sql();
    assert!(sql.contains("ts_rank"), "ts_rank: {sql}");
    assert!(sql.contains("ORDER BY"), "order by: {sql}");
}

#[test]
fn fts_where_match_mysql() {
    let (sql, _) = QueryBuilder::<()>::new("posts")
        .where_match(&["title", "body"], "rust async orm")
        .to_sql();
    assert!(sql.contains("MATCH(title, body)"), "match: {sql}");
    assert!(
        sql.contains("AGAINST('rust async orm' IN NATURAL LANGUAGE MODE)"),
        "against: {sql}"
    );
}

#[test]
fn fts_where_fts5_sqlite() {
    let (sql, _) = QueryBuilder::<()>::new("posts")
        .where_fts5("posts_fts", "rust async orm")
        .to_sql();
    assert!(sql.contains("posts_fts MATCH 'rust async orm'"), "fts5: {sql}");
}

// ── 10.3 Sub-queries and CTEs ─────────────────────────────────────────────────

#[test]
fn where_in_subquery_no_params() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_in_subquery("id", |sq| {
            sq.table("orders")
                .select(&["user_id"])
                .group_by(&["user_id"])
                .having_raw("COUNT(*) > 10")
        })
        .to_sql();
    assert!(sql.contains("id IN (SELECT user_id FROM orders"), "in subquery: {sql}");
    assert!(sql.contains("GROUP BY user_id"), "group by: {sql}");
    assert!(sql.contains("HAVING COUNT(*) > 10"), "having: {sql}");
    assert!(params.is_empty(), "no params: {params:?}");
}

#[test]
fn where_in_subquery_with_filter() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_in_subquery("id", |sq| {
            sq.table("orders").select(&["user_id"]).filter("paid", true)
        })
        .to_sql();
    assert!(sql.contains("id IN (SELECT user_id FROM orders"), "in: {sql}");
    assert!(sql.contains("WHERE paid = $1"), "inner param: {sql}");
    assert_eq!(params.len(), 1, "one param");
}

#[test]
fn where_exists_basic() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .where_exists(|sq| {
            sq.table("orders")
                .select(&["1"])
                .where_raw("orders.user_id = users.id")
        })
        .to_sql();
    assert!(sql.contains("EXISTS (SELECT 1 FROM orders"), "exists: {sql}");
    assert!(sql.contains("orders.user_id = users.id"), "raw join: {sql}");
}

#[test]
fn where_not_exists_basic() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .where_not_exists(|sq| {
            sq.table("orders")
                .select(&["1"])
                .where_raw("orders.user_id = users.id")
        })
        .to_sql();
    assert!(sql.contains("NOT EXISTS"), "not exists: {sql}");
}

#[test]
fn with_cte_from_cte_where_raw() {
    let (sql, params) = QueryBuilder::<()>::new("ranked")
        .with_cte("ranked", |cte| {
            cte.table("users")
                .select_raw("*, ROW_NUMBER() OVER (ORDER BY created_at DESC) AS rn")
        })
        .from_cte("ranked")
        .where_raw("rn <= 10")
        .to_sql();
    assert!(sql.starts_with("WITH ranked AS ("), "with cte: {sql}");
    assert!(sql.contains("FROM ranked"), "from cte: {sql}");
    assert!(sql.contains("WHERE rn <= 10"), "where: {sql}");
    assert!(params.is_empty(), "no params: {params:?}");
}

#[test]
fn from_subquery_simple() {
    let (sql, params) = QueryBuilder::<()>::new("active_users")
        .from_subquery("active_users", |sq| {
            sq.table("users").filter("active", true)
        })
        .to_sql();
    assert!(
        sql.contains("FROM (SELECT * FROM users WHERE active = $1) AS active_users"),
        "from subquery: {sql}"
    );
    assert_eq!(params.len(), 1, "one param");
}

// ── 10.4 Window functions ─────────────────────────────────────────────────────

#[test]
fn window_rank_by_adds_select_expr() {
    let (sql, _) = QueryBuilder::<()>::new("posts")
        .window_rank_by("user_id", "created_at", "row_num")
        .to_sql();
    assert!(
        sql.contains(
            "ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at) AS row_num"
        ),
        "window rank: {sql}"
    );
}

#[test]
fn having_rank_wraps_in_outer_subquery() {
    let (sql, _) = QueryBuilder::<()>::new("posts")
        .window_rank_by("user_id", "created_at", "row_num")
        .having_rank(1)
        .to_sql();
    assert!(sql.contains("__ranked WHERE row_num = 1"), "outer where: {sql}");
    assert!(
        sql.contains("ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at) AS row_num"),
        "inner select: {sql}"
    );
    // No LIMIT / OFFSET on the inner query
    let inner_start = sql.find("SELECT *,").unwrap();
    let inner_end = sql.find(") AS __ranked").unwrap();
    let inner = &sql[inner_start..inner_end];
    assert!(!inner.contains("LIMIT"), "no limit on inner: {inner}");
}

#[test]
fn having_rank_without_window_rank_uses_default_alias() {
    // If someone calls having_rank without window_rank_by, alias defaults to "row_num"
    let (sql, _) = QueryBuilder::<()>::new("posts")
        .having_rank(2)
        .to_sql();
    assert!(sql.contains("row_num = 2"), "default alias: {sql}");
}

// ── Param offset correctness ──────────────────────────────────────────────────

#[test]
fn subquery_param_offset_in_outer_where() {
    // The outer WHERE should get $2 since the subquery consumes $1.
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_in_subquery("id", |sq| {
            sq.table("orders").select(&["user_id"]).filter("paid", true)
        })
        .where_eq("active", true)
        .to_sql();
    // inner: paid = $1, outer: active = $2
    assert!(sql.contains("paid = $1"), "inner offset: {sql}");
    assert!(sql.contains("active = $2"), "outer offset: {sql}");
    assert_eq!(params.len(), 2);
}
