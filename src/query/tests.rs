//! Unit tests for [`QueryBuilder`] SQL generation (no database required).

use crate::query::{Dialect, QueryBuilder, SqlValue};

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
        let (sql, params) = QueryBuilder::<()>::new("users")
            .where_eq("id", 42i64)
            .to_sql();
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
        let (sql, _) = QueryBuilder::<()>::new("users")
            .select(&["id", "email"])
            .to_sql();
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
        let gpos = sql.find("GROUP BY").unwrap();
        let hpos = sql.find("HAVING").unwrap();
        assert!(gpos < hpos);
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
