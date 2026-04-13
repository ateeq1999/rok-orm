//! Unit tests for [`QueryBuilder`] — basic SELECT, WHERE, INSERT, UPDATE, DELETE.

use crate::query::{QueryBuilder, SqlValue};

#[test]
fn simple_select() {
    let (sql, params) = QueryBuilder::<()>::new("users").to_sql();
    assert_eq!(sql, "SELECT * FROM users");
    assert!(params.is_empty());
}

#[test]
fn distinct_select() {
    let (sql, _) = QueryBuilder::<()>::new("users").distinct().to_sql();
    assert!(sql.starts_with("SELECT DISTINCT * FROM users"));
}

#[test]
fn where_eq_generates_param() {
    let (sql, params) = QueryBuilder::<()>::new("users").where_eq("id", 42i64).to_sql();
    assert!(sql.contains("WHERE id = $1"));
    assert_eq!(params.len(), 1);
    assert_eq!(params[0], SqlValue::Integer(42));
}

#[test]
fn multiple_conditions() {
    let (sql, params) = QueryBuilder::<()>::new("posts")
        .where_eq("active", true)
        .where_like("title", "%rust%")
        .to_sql();
    assert!(sql.contains("WHERE active = $1 AND title LIKE $2"));
    assert_eq!(params.len(), 2);
}

#[test]
fn or_conditions() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_eq("role", "admin")
        .or_where_eq("role", "moderator")
        .to_sql();
    assert!(sql.contains("WHERE role = $1 OR role = $2"));
    assert_eq!(params.len(), 2);
}

#[test]
fn where_between() {
    let (sql, params) = QueryBuilder::<()>::new("orders")
        .where_between("amount", 10i64, 100i64)
        .to_sql();
    assert!(sql.contains("amount BETWEEN $1 AND $2"));
    assert_eq!(params.len(), 2);
}

#[test]
fn where_not_in() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_not_in("status", vec!["banned", "deleted"])
        .to_sql();
    assert!(sql.contains("status NOT IN ($1, $2)"));
    assert_eq!(params.len(), 2);
}

#[test]
fn where_not_like() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .where_not_like("email", "%@spam.com")
        .to_sql();
    assert!(sql.contains("email NOT LIKE $1"));
}

#[test]
fn to_update_sql() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_eq("id", 1i64)
        .to_update_sql(&[("name", "Bob".into()), ("active", true.into())]);
    assert!(sql.starts_with("UPDATE users SET name = $1, active = $2"));
    assert!(sql.contains("WHERE id = $3"));
    assert_eq!(params.len(), 3);
}

#[test]
fn order_limit_offset() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .order_by_desc("created_at")
        .order_by("name")
        .limit(10)
        .offset(20)
        .to_sql();
    assert!(sql.contains("ORDER BY created_at DESC, name ASC"));
    assert!(sql.contains("LIMIT 10"));
    assert!(sql.contains("OFFSET 20"));
}

#[test]
fn count_sql() {
    let (sql, _) = QueryBuilder::<()>::new("users")
        .where_eq("active", true)
        .to_count_sql();
    assert!(sql.starts_with("SELECT COUNT(*) FROM users"));
}

#[test]
fn delete_sql() {
    let (sql, params) = QueryBuilder::<()>::new("sessions")
        .where_eq("user_id", 5i64)
        .to_delete_sql();
    assert!(sql.contains("DELETE FROM sessions WHERE user_id = $1"));
    assert_eq!(params.len(), 1);
}

#[test]
fn insert_sql() {
    let (sql, params) = QueryBuilder::<()>::insert_sql(
        "users",
        &[("name", "Alice".into()), ("email", "a@a.com".into())],
    );
    assert!(sql.contains("INSERT INTO users (name, email) VALUES ($1, $2)"));
    assert_eq!(params.len(), 2);
}

#[test]
fn where_in() {
    let (sql, params) = QueryBuilder::<()>::new("users")
        .where_in("id", vec![1i64, 2, 3])
        .to_sql();
    assert!(sql.contains("id IN ($1, $2, $3)"));
    assert_eq!(params.len(), 3);
}

#[test]
fn select_specific_columns() {
    let (sql, _) = QueryBuilder::<()>::new("users").select(&["id", "email"]).to_sql();
    assert!(sql.starts_with("SELECT id, email FROM users"));
}

#[test]
fn option_value_null() {
    let val: SqlValue = Option::<i64>::None.into();
    assert_eq!(val, SqlValue::Null);
}

#[test]
fn option_value_some() {
    let val: SqlValue = Some(42i64).into();
    assert_eq!(val, SqlValue::Integer(42));
}
