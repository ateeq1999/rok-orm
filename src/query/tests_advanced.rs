//! Unit tests for [`QueryBuilder`] — joins, GROUP BY, bulk INSERT, soft-delete, dialects.

use crate::query::{Dialect, QueryBuilder, SqlValue};

#[test]
fn inner_join() {
    let (sql, _) = QueryBuilder::<()>::new("orders")
        .inner_join("users", "users.id = orders.user_id")
        .to_sql();
    assert!(sql.contains("INNER JOIN users ON users.id = orders.user_id"));
}

#[test]
fn left_join_with_where() {
    let (sql, params) = QueryBuilder::<()>::new("orders")
        .left_join("users", "users.id = orders.user_id")
        .where_eq("orders.status", "paid")
        .to_sql();
    assert!(sql.contains("LEFT JOIN users ON users.id = orders.user_id"));
    assert!(sql.contains("WHERE orders.status = $1"));
    assert_eq!(params.len(), 1);
}

#[test]
fn right_join() {
    let (sql, _) = QueryBuilder::<()>::new("orders")
        .right_join("products", "products.id = orders.product_id")
        .to_sql();
    assert!(sql.contains("RIGHT JOIN products ON products.id = orders.product_id"));
}

#[test]
fn group_by_and_having() {
    let (sql, _) = QueryBuilder::<()>::new("orders")
        .select(&["user_id", "COUNT(*) as total"])
        .group_by(&["user_id"])
        .having("COUNT(*) > 5")
        .to_sql();
    assert!(sql.contains("GROUP BY user_id"));
    assert!(sql.contains("HAVING COUNT(*) > 5"));
    assert!(sql.find("GROUP BY").unwrap() < sql.find("HAVING").unwrap());
}

#[test]
fn count_sql_with_join() {
    let (sql, _) = QueryBuilder::<()>::new("orders")
        .inner_join("users", "users.id = orders.user_id")
        .where_eq("users.active", true)
        .to_count_sql();
    assert!(sql.contains("INNER JOIN users ON users.id = orders.user_id"));
    assert!(sql.contains("SELECT COUNT(*) FROM orders"));
}

#[test]
fn bulk_insert_sql_two_rows() {
    let rows: Vec<Vec<(&str, SqlValue)>> = vec![
        vec![("name", "Alice".into()), ("email", "a@a.com".into())],
        vec![("name", "Bob".into()), ("email", "b@b.com".into())],
    ];
    let (sql, params) = QueryBuilder::<()>::bulk_insert_sql("users", &rows);
    assert!(sql.starts_with("INSERT INTO users (name, email) VALUES"));
    assert!(sql.contains("($1, $2), ($3, $4)"));
    assert_eq!(params.len(), 4);
}

#[test]
fn bulk_insert_sql_single_row() {
    let rows = vec![vec![("x", SqlValue::Integer(1))]];
    let (sql, params) = QueryBuilder::<()>::bulk_insert_sql("t", &rows);
    assert!(sql.contains("($1)"));
    assert_eq!(params.len(), 1);
}

#[test]
fn soft_delete_auto_filter() {
    let (sql, _) = QueryBuilder::<()>::new("posts")
        .with_soft_delete("deleted_at")
        .to_sql();
    assert!(sql.contains("WHERE deleted_at IS NULL"));
}

#[test]
fn with_trashed_includes_deleted() {
    let (sql, _) = QueryBuilder::<()>::new("posts")
        .with_soft_delete("deleted_at")
        .with_trashed()
        .to_sql();
    assert!(!sql.contains("WHERE"));
    assert!(sql.contains("SELECT * FROM posts"));
}

#[test]
fn only_trashed_filters_deleted_only() {
    let (sql, _) = QueryBuilder::<()>::new("posts")
        .with_soft_delete("deleted_at")
        .only_trashed()
        .to_sql();
    assert!(sql.contains("WHERE deleted_at IS NOT NULL"));
}

#[test]
fn soft_delete_with_conditions() {
    let (sql, params) = QueryBuilder::<()>::new("posts")
        .with_soft_delete("deleted_at")
        .where_eq("author_id", 42i64)
        .to_sql();
    assert!(sql.contains("WHERE author_id = $1 AND deleted_at IS NULL"));
    assert_eq!(params.len(), 1);
}

#[test]
fn restore_sql() {
    let (sql, params) = QueryBuilder::<()>::new("posts")
        .with_soft_delete("deleted_at")
        .where_eq("id", 1i64)
        .push_update_column("deleted_at", SqlValue::Null)
        .to_restore_sql();
    assert!(sql.starts_with("UPDATE posts SET deleted_at = $1"));
    assert!(sql.contains("WHERE id = $2"));
    assert_eq!(params.len(), 2);
}

#[test]
fn force_delete_sql_bypasses_soft_delete() {
    let (sql, params) = QueryBuilder::<()>::new("posts")
        .with_soft_delete("deleted_at")
        .where_eq("id", 1i64)
        .to_force_delete_sql();
    assert!(sql.starts_with("DELETE FROM posts"));
    assert!(sql.contains("WHERE id = $1"));
    assert!(!sql.contains("deleted_at"));
    assert_eq!(params.len(), 1);
}

#[test]
fn soft_delete_count_sql() {
    let (sql, _) = QueryBuilder::<()>::new("posts")
        .with_soft_delete("deleted_at")
        .to_count_sql();
    assert!(sql.starts_with("SELECT COUNT(*) FROM posts"));
    assert!(sql.contains("WHERE deleted_at IS NULL"));
}

#[test]
fn sqlite_dialect_uses_question_mark() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_eq("id", 1i64)
        .where_eq("active", true)
        .to_sql_with_dialect(Dialect::Sqlite);
    assert!(sql.contains("WHERE id = ? AND active = ?"));
    assert_eq!(params.len(), 2);
}

#[test]
fn mysql_dialect_uses_question_mark() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_in("id", vec![1i64, 2, 3])
        .to_sql_with_dialect(Dialect::Mysql);
    assert!(sql.contains("id IN (?, ?, ?)"));
    assert_eq!(params.len(), 3);
}
