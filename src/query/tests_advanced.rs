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

#[test]
fn distinct_adds_keyword() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .distinct()
        .select(&["email"])
        .to_sql();
    assert!(sql.contains("SELECT DISTINCT email FROM users"));
}

#[test]
fn select_specific_columns() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .select(&["id", "name", "email"])
        .to_sql();
    assert!(sql.contains("SELECT id, name, email FROM users"));
    assert!(!sql.contains("SELECT *"));
}

#[test]
fn order_by_asc_and_desc() {
    let (sql, _) = QueryBuilder::<()>::new("posts")
        .order_by("title")
        .order_by_desc("created_at")
        .to_sql();
    assert!(sql.contains("ORDER BY title ASC, created_at DESC"));
}

#[test]
fn paginate_sets_limit_and_offset() {
    let (sql, _) = QueryBuilder::<()>::new("posts")
        .paginate(3, 10)
        .to_sql();
    assert!(sql.contains("LIMIT 10"));
    assert!(sql.contains("OFFSET 20"));
}

#[test]
fn where_null_generates_is_null() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_null("deleted_at")
        .to_sql();
    assert!(sql.contains("deleted_at IS NULL"));
    assert!(params.is_empty());
}

#[test]
fn where_not_null_generates_is_not_null() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_not_null("email")
        .to_sql();
    assert!(sql.contains("email IS NOT NULL"));
    assert!(params.is_empty());
}

#[test]
fn where_between_generates_correct_sql() {
    let (sql, params) = QueryBuilder::<()>::new("orders")
        .where_between("amount", 10i64, 100i64)
        .to_sql();
    assert!(sql.contains("amount BETWEEN $1 AND $2"));
    assert_eq!(params.len(), 2);
}

#[test]
fn cursor_sql_adds_where_gt_and_limit_plus_one() {
    let (sql, params) = QueryBuilder::<()>::new("posts")
        .cursor_sql("id", Some(42i64), 20)
        .to_sql();
    assert!(sql.contains("id > $1"));
    assert!(sql.contains("LIMIT 21"));
    assert_eq!(params.len(), 1);
    assert_eq!(params[0], SqlValue::Integer(42));
}

#[test]
fn cursor_sql_without_after_adds_only_limit() {
    let (sql, params) = QueryBuilder::<()>::new("posts")
        .cursor_sql("id", None, 10)
        .to_sql();
    assert!(!sql.contains("WHERE"));
    assert!(sql.contains("LIMIT 11"));
    assert!(params.is_empty());
}

#[test]
fn exists_sql_wraps_in_select_exists() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .where_eq("active", true)
        .exists_sql();
    assert!(sql.starts_with("SELECT EXISTS(SELECT 1 FROM users"));
    assert!(sql.contains("active = $1"));
}

#[test]
fn pluck_sql_selects_single_column() {
    let (sql, params) = QueryBuilder::<()>::new("tags")
        .where_eq("post_id", 5i64)
        .pluck_sql("name");
    assert!(sql.contains("SELECT name FROM tags"));
    assert!(sql.contains("post_id = $1"));
    assert_eq!(params.len(), 1);
}

#[test]
fn or_where_joins_with_or() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_eq("role", "admin")
        .or_where_eq("role", "moderator")
        .to_sql();
    assert!(sql.contains("role = $1 OR role = $2"));
    assert_eq!(params.len(), 2);
}

#[test]
fn update_sql_generates_set_clause() {
    let data = [("name", SqlValue::Text("Bob".into())), ("active", SqlValue::Bool(false))];
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_eq("id", 7i64)
        .to_update_sql(&data);
    assert!(sql.starts_with("UPDATE users SET"));
    assert!(sql.contains("name = $1"));
    assert!(sql.contains("WHERE id = $3"));
    assert_eq!(params.len(), 3);
}

#[test]
fn delete_sql_with_condition() {
    let (sql, params) = QueryBuilder::<()>::new("sessions")
        .where_eq("user_id", 99i64)
        .to_delete_sql();
    assert!(sql.starts_with("DELETE FROM sessions"));
    assert!(sql.contains("WHERE user_id = $1"));
    assert_eq!(params.len(), 1);
}
