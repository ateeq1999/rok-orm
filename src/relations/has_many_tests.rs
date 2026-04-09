use super::*;
use crate::model::Model;
use crate::query::SqlValue;

struct User;
impl Model for User {
    fn table_name() -> &'static str { "users" }
    fn columns() -> &'static [&'static str] { &["id", "name"] }
}

struct Post;
impl Model for Post {
    fn table_name() -> &'static str { "posts" }
    fn columns() -> &'static [&'static str] { &["id", "user_id", "title"] }
}

fn posts_rel() -> HasMany<User, Post> {
    HasMany::new("users", "id", "posts", "id", "user_id".to_string())
}

#[test]
fn create_sql_prepends_fk() {
    let rel = posts_rel();
    let (sql, params) = rel.create_sql(
        SqlValue::Integer(1),
        &[("title", SqlValue::Text("Hello".into()))],
    );
    assert!(sql.contains("INSERT INTO posts"), "sql: {sql}");
    assert!(sql.contains("user_id"), "sql: {sql}");
    assert_eq!(params[0], SqlValue::Integer(1));
}

#[test]
fn associate_sql_updates_fk() {
    let rel = posts_rel();
    let (sql, _) = rel.associate_sql(SqlValue::Integer(5), SqlValue::Integer(1));
    assert!(sql.contains("UPDATE posts SET user_id"));
    assert!(sql.contains("WHERE id"));
}

#[test]
fn dissociate_sql_sets_null() {
    let rel = posts_rel();
    let (sql, params) = rel.dissociate_sql(SqlValue::Integer(5));
    assert!(sql.contains("SET user_id = NULL"));
    assert_eq!(params[0], SqlValue::Integer(5));
}

#[test]
fn save_sql_inserts_when_no_pk() {
    let rel = posts_rel();
    let (sql, params) = rel.save_sql(
        SqlValue::Integer(1),
        None,
        &[("title", SqlValue::Text("New Post".into()))],
    );
    assert!(sql.starts_with("INSERT INTO posts"), "sql: {sql}");
    assert!(sql.contains("user_id"), "sql: {sql}");
    assert_eq!(params[0], SqlValue::Integer(1)); // FK
}

#[test]
fn save_sql_updates_when_pk_given() {
    let rel = posts_rel();
    let (sql, params) = rel.save_sql(
        SqlValue::Integer(1),
        Some(SqlValue::Integer(42)),
        &[("title", SqlValue::Text("Updated".into()))],
    );
    assert!(sql.starts_with("UPDATE posts"), "sql: {sql}");
    assert!(sql.contains("WHERE id = $"), "sql: {sql}");
    assert_eq!(*params.last().unwrap(), SqlValue::Integer(42)); // PK at end
}

#[test]
fn create_many_sql_injects_fk_in_each_row() {
    let rel = posts_rel();
    let row1: &[(&str, SqlValue)] = &[("title", SqlValue::Text("Post 1".into()))];
    let row2: &[(&str, SqlValue)] = &[("title", SqlValue::Text("Post 2".into()))];
    let (sql, params) = rel.create_many_sql(SqlValue::Integer(7), &[row1, row2]);
    assert!(sql.starts_with("INSERT INTO posts"));
    assert!(sql.contains("user_id"));
    // 2 rows × 2 columns each = 4 params
    assert_eq!(params.len(), 4);
    assert_eq!(params[0], SqlValue::Integer(7)); // first row FK
    assert_eq!(params[2], SqlValue::Integer(7)); // second row FK
}
