use rok_orm::{Dialect, Model, QueryBuilder};

#[derive(Model)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Model)]
pub struct BlogPost {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Model)]
pub struct OrderItem {
    pub id: i64,
    pub quantity: i32,
}

#[derive(Model)]
#[model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub title: String,
}

#[derive(Model)]
#[model(primary_key = "user_id")]
pub struct Profile {
    pub user_id: i64,
    pub bio: String,
}

#[derive(Model)]
pub struct Tag {
    #[model(primary_key)]
    pub tag_id: i64,
    #[model(column = "tag_name")]
    pub name: String,
    #[model(skip)]
    pub cached_count: i32,
}

#[derive(Model)]
#[model(soft_delete)]
pub struct SoftDeletePost {
    pub id: i64,
    pub title: String,
    pub deleted_at: Option<String>,
}

#[derive(Model)]
#[model(timestamps)]
pub struct TimestampedUser {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

// ── table names ───────────────────────────────────────────────────────────────

#[test]
fn table_name_simple() {
    assert_eq!(User::table_name(), "users");
}

#[test]
fn table_name_multi_word() {
    assert_eq!(BlogPost::table_name(), "blog_posts");
    assert_eq!(OrderItem::table_name(), "order_items");
}

// ── columns ───────────────────────────────────────────────────────────────────

#[test]
fn columns_list() {
    assert_eq!(User::columns(), &["id", "name", "email"]);
    assert_eq!(BlogPost::columns(), &["id", "title", "body", "published"]);
}

// ── query builder through Model trait ────────────────────────────────────────

#[test]
fn query_select_all() {
    let (sql, params) = User::query().to_sql();
    assert_eq!(sql, "SELECT * FROM users");
    assert!(params.is_empty());
}

#[test]
fn query_where_eq() {
    let (sql, params) = User::query().where_eq("id", 1i64).to_sql();
    assert!(sql.contains("WHERE id = $1"));
    assert_eq!(params.len(), 1);
}

#[test]
fn query_find() {
    let (sql, params) = User::find(42i64).to_sql();
    assert!(sql.contains("WHERE id = $1"));
    assert_eq!(params[0], rok_orm::SqlValue::Integer(42));
}

#[test]
fn query_chaining() {
    let (sql, params) = BlogPost::query()
        .where_eq("published", true)
        .where_like("title", "%rust%")
        .order_by_desc("id")
        .limit(5)
        .offset(10)
        .to_sql();

    assert!(sql.contains("FROM blog_posts"));
    assert!(sql.contains("WHERE published = $1 AND title LIKE $2"));
    assert!(sql.contains("ORDER BY id DESC"));
    assert!(sql.contains("LIMIT 5"));
    assert!(sql.contains("OFFSET 10"));
    assert_eq!(params.len(), 2);
}

#[test]
fn count_sql() {
    let (sql, _) = User::query().where_not_null("email").to_count_sql();
    assert!(sql.starts_with("SELECT COUNT(*) FROM users"));
    assert!(sql.contains("email IS NOT NULL"));
}

#[test]
fn insert_sql() {
    use rok_orm::SqlValue;
    let (sql, params) = rok_orm::QueryBuilder::<User>::insert_sql(
        "users",
        &[
            ("name", "Alice".into()),
            ("email", "alice@example.com".into()),
        ],
    );
    assert!(sql.contains("INSERT INTO users (name, email) VALUES ($1, $2)"));
    assert_eq!(
        params,
        vec![
            SqlValue::Text("Alice".into()),
            SqlValue::Text("alice@example.com".into()),
        ]
    );
}

// ── attribute: custom table name ──────────────────────────────────────────────

#[test]
fn custom_table_name() {
    assert_eq!(Article::table_name(), "articles");
    assert_eq!(Article::columns(), &["id", "title"]);
}

// ── attribute: custom primary key ─────────────────────────────────────────────

#[test]
fn struct_level_primary_key() {
    assert_eq!(Profile::primary_key(), "user_id");
}

#[test]
fn field_level_primary_key() {
    assert_eq!(Tag::primary_key(), "tag_id");
}

// ── attribute: skip and column rename ────────────────────────────────────────

#[test]
fn skip_excludes_field() {
    // cached_count is skipped
    assert_eq!(Tag::columns(), &["tag_id", "tag_name"]);
}

// ── OR conditions ─────────────────────────────────────────────────────────────

#[test]
fn or_where_conditions() {
    let (sql, params) = User::query()
        .where_eq("role", "admin")
        .or_where_eq("role", "moderator")
        .to_sql();
    assert!(sql.contains("role = $1 OR role = $2"));
    assert_eq!(params.len(), 2);
}

// ── between ───────────────────────────────────────────────────────────────────

#[test]
fn where_between_query() {
    let (sql, params) = User::query().where_between("id", 1i64, 100i64).to_sql();
    assert!(sql.contains("id BETWEEN $1 AND $2"));
    assert_eq!(params.len(), 2);
}

// ── not_in ────────────────────────────────────────────────────────────────────

#[test]
fn where_not_in_query() {
    let (sql, params) = User::query().where_not_in("id", vec![1i64, 2, 3]).to_sql();
    assert!(sql.contains("id NOT IN ($1, $2, $3)"));
    assert_eq!(params.len(), 3);
}

// ── distinct ──────────────────────────────────────────────────────────────────

#[test]
fn distinct_query() {
    let (sql, _) = User::query().distinct().select(&["email"]).to_sql();
    assert!(sql.starts_with("SELECT DISTINCT email FROM users"));
}

// ── to_update_sql ─────────────────────────────────────────────────────────────

#[test]
fn update_sql_via_builder() {
    let (sql, params) =
        User::find(1i64).to_update_sql(&[("name", "Bob".into()), ("email", "b@b.com".into())]);
    assert!(sql.starts_with("UPDATE users SET name = $1, email = $2"));
    assert!(sql.contains("WHERE id = $3"));
    assert_eq!(params.len(), 3);
}

// ── join ──────────────────────────────────────────────────────────────────────

#[test]
fn inner_join_query() {
    let (sql, params) = User::query()
        .inner_join("posts", "posts.user_id = users.id")
        .where_eq("users.active", true)
        .to_sql();
    assert!(sql.contains("INNER JOIN posts ON posts.user_id = users.id"));
    assert!(sql.contains("WHERE users.active = $1"));
    assert_eq!(params.len(), 1);
}

#[test]
fn left_join_query() {
    let (sql, _) = User::query()
        .left_join("profiles", "profiles.user_id = users.id")
        .to_sql();
    assert!(sql.contains("LEFT JOIN profiles ON profiles.user_id = users.id"));
}

// ── group by / having ─────────────────────────────────────────────────────────

#[test]
fn group_by_having_query() {
    use rok_orm::QueryBuilder;
    let (sql, _) = QueryBuilder::<User>::new("users")
        .select(&["role", "COUNT(*) as n"])
        .group_by(&["role"])
        .having("COUNT(*) > 1")
        .to_sql();
    assert!(sql.contains("GROUP BY role"));
    assert!(sql.contains("HAVING COUNT(*) > 1"));
}

// ── bulk insert ───────────────────────────────────────────────────────────────

#[test]
fn bulk_insert_sql() {
    use rok_orm::{QueryBuilder, SqlValue};

    let rows: Vec<Vec<(&str, SqlValue)>> = vec![
        vec![("name", "Alice".into()), ("email", "a@a.com".into())],
        vec![("name", "Bob".into()), ("email", "b@b.com".into())],
        vec![("name", "Carol".into()), ("email", "c@c.com".into())],
    ];
    let (sql, params) = QueryBuilder::<User>::bulk_insert_sql("users", &rows);
    assert!(sql.starts_with("INSERT INTO users (name, email) VALUES"));
    assert!(sql.contains("($1, $2), ($3, $4), ($5, $6)"));
    assert_eq!(params.len(), 6);
}

// ── filter shorthand ─────────────────────────────────────────────────────────────

#[test]
fn filter_shorthand() {
    let (sql, params) = User::query().filter("email", "test@example.com").to_sql();
    assert!(sql.contains("WHERE email = $1"));
    assert_eq!(params.len(), 1);
    assert_eq!(
        params[0],
        rok_orm::SqlValue::Text("test@example.com".into())
    );
}

// ── soft delete model ──────────────────────────────────────────────────────────

#[test]
fn soft_delete_enabled() {
    assert_eq!(SoftDeletePost::soft_delete_column(), Some("deleted_at"));
}

#[test]
fn soft_delete_disabled() {
    assert_eq!(User::soft_delete_column(), None);
}

// ── timestamps model ───────────────────────────────────────────────────────────

#[test]
fn timestamps_enabled() {
    assert!(TimestampedUser::timestamps_enabled());
}

#[test]
fn timestamps_disabled() {
    assert!(!User::timestamps_enabled());
}

#[test]
fn timestamps_columns() {
    assert_eq!(TimestampedUser::created_at_column(), Some("created_at"));
    assert_eq!(TimestampedUser::updated_at_column(), Some("updated_at"));
}

#[test]
fn no_timestamps_columns() {
    assert_eq!(User::created_at_column(), None);
    assert_eq!(User::updated_at_column(), None);
}

// ── query! macro with filter shorthand ─────────────────────────────────────

#[test]
fn query_macro_filter_shorthand() {
    use rok_orm_macros::query;

    let q = query!(User,
        filter "active" true,
        order_by_desc "created_at",
        limit 10
    );

    let (sql, params) = q.to_sql();
    assert!(sql.contains("WHERE active = $1"));
    assert!(sql.contains("ORDER BY created_at DESC"));
    assert!(sql.contains("LIMIT 10"));
    assert_eq!(params.len(), 1);
}

// ── eq shorthand ─────────────────────────────────────────────────────────────

#[test]
fn eq_shorthand() {
    let (sql, params) = User::query().eq("email", "test@example.com").to_sql();
    assert!(sql.contains("WHERE email = $1"));
    assert_eq!(params.len(), 1);
    assert_eq!(
        params[0],
        rok_orm::SqlValue::Text("test@example.com".into())
    );
}

#[test]
fn query_macro_eq_shorthand() {
    use rok_orm_macros::query;

    let q = query!(User,
        eq "active" true,
        order_by_desc "created_at"
    );

    let (sql, params) = q.to_sql();
    assert!(sql.contains("WHERE active = $1"));
    assert!(sql.contains("ORDER BY created_at DESC"));
    assert_eq!(params.len(), 1);
}

// ── ModelHooks derive ────────────────────────────────────────────────────────

use rok_orm::hooks::ModelHooks;

#[derive(Debug)]
pub struct HookTestModel {
    pub id: i64,
    pub name: String,
}

impl ModelHooks for HookTestModel {}

#[test]
fn model_hooks_derive() {
    let model = HookTestModel {
        id: 1,
        name: "Test".to_string(),
    };
    assert_eq!(model.name, "Test");
}

// ── BelongsToMany ────────────────────────────────────────────────────────────

use rok_orm::belongs_to_many::BelongsToMany;

#[derive(Debug)]
pub struct TestPost {
    pub id: i64,
    pub title: String,
}

#[derive(Debug)]
pub struct TestTag {
    pub id: i64,
    pub name: String,
}

impl rok_orm::Model for TestPost {
    fn table_name() -> &'static str {
        "posts"
    }
    fn primary_key() -> &'static str {
        "id"
    }
    fn columns() -> &'static [&'static str] {
        &["id", "title"]
    }
}

impl rok_orm::Model for TestTag {
    fn table_name() -> &'static str {
        "tags"
    }
    fn primary_key() -> &'static str {
        "id"
    }
    fn columns() -> &'static [&'static str] {
        &["id", "name"]
    }
}

#[test]
fn belongs_to_many_creation() {
    let relation = BelongsToMany::<TestPost, TestTag>::new(
        "posts",
        "id",
        "post_tags".to_string(),
        "post_id".to_string(),
        "tag_id".to_string(),
        "tags",
        "id",
    );

    assert_eq!(relation.pivot_table(), "post_tags");
    assert_eq!(relation.left_key(), "post_id");
    assert_eq!(relation.right_key(), "tag_id");
}

#[test]
fn belongs_to_many_query_generation() {
    let relation = BelongsToMany::<TestPost, TestTag>::new(
        "posts",
        "id",
        "post_tags".to_string(),
        "post_id".to_string(),
        "tag_id".to_string(),
        "tags",
        "id",
    );

    let (sql, params) = relation.get_sql_for(1i64.into());

    assert!(sql.contains("INNER JOIN post_tags ON"));
    assert!(params.len() >= 1);
}

#[test]
fn belongs_to_many_count_query() {
    let relation = BelongsToMany::<TestPost, TestTag>::new(
        "posts",
        "id",
        "post_tags".to_string(),
        "post_id".to_string(),
        "tag_id".to_string(),
        "tags",
        "id",
    );

    let (sql, _) = relation.count_sql_for(1i64.into());
    assert!(sql.starts_with("SELECT COUNT(*) FROM tags"));
    assert!(sql.contains("INNER JOIN post_tags"));
}

// ── eager loading ─────────────────────────────────────────────────────────────

#[test]
fn query_builder_with_single_relation() {
    let q = User::query().with("posts");
    assert_eq!(q.eager_loads(), &["posts"]);
}

#[test]
fn query_builder_with_multiple_relations() {
    let q = User::query().with("posts").with("comments").with("profile");
    assert_eq!(q.eager_loads(), &["posts", "comments", "profile"]);
}

#[test]
fn query_builder_with_many_relations() {
    let q = User::query().with_many(vec!["posts".to_string(), "tags".to_string()]);
    assert_eq!(q.eager_loads(), &["posts", "tags"]);
}

#[test]
fn eager_has_many_build_query() {
    use rok_orm::eager::HasManyEager;

    let loader = HasManyEager::<User>::new("posts", "user_id".to_string(), "id");
    let (sql, params) = loader
        .build_query::<BlogPost>(&[1i64.into(), 2i64.into()])
        .to_sql();

    assert!(sql.contains("SELECT * FROM posts"));
    assert!(sql.contains("WHERE user_id IN ($1, $2)"));
    assert_eq!(params.len(), 2);
}

#[test]
fn eager_belongs_to_build_query() {
    use rok_orm::eager::BelongsToEager;

    let loader = BelongsToEager::<BlogPost>::new("posts", "user_id".to_string(), "users", "id");
    let (sql, params) = loader
        .build_query::<User>(&[1i64.into(), 2i64.into()])
        .to_sql();

    assert!(sql.contains("SELECT * FROM users"));
    assert!(sql.contains("WHERE id IN ($1, $2)"));
    assert_eq!(params.len(), 2);
}

#[test]
fn eager_query_empty_ids_returns_limit_zero() {
    use rok_orm::eager::HasManyEager;

    let loader = HasManyEager::<User>::new("posts", "user_id".to_string(), "id");
    let (sql, _) = loader.build_query::<BlogPost>(&[]).to_sql();

    assert!(sql.contains("LIMIT 0"));
}

// ── pagination ────────────────────────────────────────────────────────────────

#[test]
fn page_new_calculates_last_page() {
    use rok_orm::pagination::Page;

    let page: Page<i32> = Page::new(vec![1, 2, 3], 25, 10, 1);

    assert_eq!(page.total, 25);
    assert_eq!(page.per_page, 10);
    assert_eq!(page.current_page, 1);
    assert_eq!(page.last_page, 3);
    assert!(page.has_next());
    assert!(!page.has_prev());
}

#[test]
fn page_has_next_and_prev() {
    use rok_orm::pagination::Page;

    let page1: Page<i32> = Page::new(vec![], 100, 10, 1);
    assert!(page1.has_next());
    assert!(!page1.has_prev());

    let page5: Page<i32> = Page::new(vec![], 100, 10, 5);
    assert!(page5.has_next());
    assert!(page5.has_prev());

    let page10: Page<i32> = Page::new(vec![], 100, 10, 10);
    assert!(!page10.has_next());
    assert!(page10.has_prev());
}

#[test]
fn query_builder_paginate() {
    let (sql, params) = User::query().paginate(2, 15).to_sql();

    assert!(sql.contains("LIMIT 15"));
    assert!(sql.contains("OFFSET 15"));
    assert!(params.is_empty());
}

#[test]
fn query_builder_paginate_page_1() {
    let (sql, _) = User::query().paginate(1, 10).to_sql();

    assert!(sql.contains("LIMIT 10"));
    assert!(sql.contains("OFFSET 0"));
}

#[test]
fn query_builder_paginate_caps_at_100() {
    let (sql, _) = User::query().paginate(1, 500).to_sql();

    assert!(sql.contains("LIMIT 100"));
}

// ── aggregation ────────────────────────────────────────────────────────────────

#[test]
fn query_builder_sum_sql() {
    let (sql, params) = User::query().sum_sql("age");

    assert!(sql.contains("SELECT SUM(age) FROM users"));
    assert!(params.is_empty());
}

#[test]
fn query_builder_avg_sql() {
    let (sql, params) = User::query().avg_sql("price");

    assert!(sql.contains("SELECT AVG(price) FROM users"));
    assert!(params.is_empty());
}

#[test]
fn query_builder_min_sql() {
    let (sql, params) = User::query().min_sql("created_at");

    assert!(sql.contains("SELECT MIN(created_at) FROM users"));
    assert!(params.is_empty());
}

#[test]
fn query_builder_max_sql() {
    let (sql, params) = User::query().max_sql("score");

    assert!(sql.contains("SELECT MAX(score) FROM users"));
    assert!(params.is_empty());
}

#[test]
fn query_builder_aggregate_with_where() {
    let (sql, params) = User::query().filter("active", true).sum_sql("amount");

    assert!(sql.contains("SELECT SUM(amount) FROM users"));
    assert!(sql.contains("WHERE active = $1"));
    assert_eq!(params.len(), 1);
}

// ── upsert ────────────────────────────────────────────────────────────────────

#[test]
fn upsert_sql_generates_on_conflict() {
    let (sql, params) = QueryBuilder::<()>::upsert_sql(
        "users",
        &[
            ("email", "test@example.com".into()),
            ("name", "Test".into()),
        ],
        "email",
        &["name"],
    );

    assert!(sql.contains("INSERT INTO users"));
    assert!(sql.contains("ON CONFLICT (email)"));
    assert!(sql.contains("DO UPDATE SET"));
    assert!(sql.contains("name = EXCLUDED.name"));
    assert_eq!(params.len(), 2);
}

#[test]
fn upsert_do_nothing_sql() {
    let (sql, params) = QueryBuilder::<()>::upsert_do_nothing_sql(
        "users",
        &[("email", "test@example.com".into())],
        "email",
    );

    assert!(sql.contains("INSERT INTO users"));
    assert!(sql.contains("ON CONFLICT (email)"));
    assert!(sql.contains("DO NOTHING"));
    assert_eq!(params.len(), 1);
}

// ── batch operations ────────────────────────────────────────────────────────────────

// ── exists and pluck ───────────────────────────────────────────────────────────

#[test]
fn exists_sql_generates_subquery() {
    let (sql, params) = User::query().filter("active", true).exists_sql();

    assert!(sql.contains("SELECT EXISTS(SELECT 1 FROM users"));
    assert!(sql.contains("WHERE active = $1"));
    assert_eq!(params.len(), 1);
}

#[test]
fn exists_sql_simple() {
    let (sql, params) = User::query().exists_sql();

    assert!(sql.contains("SELECT EXISTS(SELECT 1 FROM users"));
    assert!(params.is_empty());
}

#[test]
fn pluck_sql_generates_single_column() {
    let (sql, params) = User::query().pluck_sql("email");

    assert!(sql.contains("SELECT email FROM users"));
    assert!(params.is_empty());
}

#[test]
fn pluck_sql_with_where() {
    let (sql, params) = User::query().filter("active", true).pluck_sql("email");

    assert!(sql.contains("SELECT email FROM users"));
    assert!(sql.contains("WHERE active = $1"));
    assert_eq!(params.len(), 1);
}

#[test]
fn pluck_sql_with_limit() {
    let (sql, _) = User::query()
        .filter("active", true)
        .limit(5)
        .pluck_sql("name");

    assert!(sql.contains("SELECT name FROM users"));
    assert!(sql.contains("WHERE active = $1"));
    assert!(sql.contains("LIMIT 5"));
}

// ── error handling ─────────────────────────────────────────────────────────────

#[test]
fn orm_error_not_found() {
    use rok_orm::errors::OrmError;

    let err = OrmError::not_found("User", "id", "42");
    assert!(err.is_not_found());
    assert!(!err.is_validation());
    assert!(!err.is_constraint());
    assert_eq!(err.to_string(), "Record not found: User::id=42");
}

#[test]
fn orm_error_validation() {
    use rok_orm::errors::OrmError;

    let err = OrmError::validation("Name cannot be empty");
    assert!(err.is_validation());
    assert_eq!(err.to_string(), "Validation failed: Name cannot be empty");
}

#[test]
fn orm_error_constraint() {
    use rok_orm::errors::OrmError;

    let err = OrmError::constraint("unique_email");
    assert!(err.is_constraint());
}

// ── logging ───────────────────────────────────────────────────────────────────

#[test]
fn log_level_ordering() {
    use rok_orm::logging::LogLevel;

    assert!(LogLevel::Debug.should_log(LogLevel::Debug));
    assert!(LogLevel::Debug.should_log(LogLevel::Warn));
    assert!(!LogLevel::Warn.should_log(LogLevel::Debug));
}

#[test]
fn query_timer() {
    use rok_orm::logging::QueryTimer;
    use std::time::Duration;

    let timer = QueryTimer::new();
    std::thread::sleep(Duration::from_millis(10));

    assert!(timer.elapsed_ms() >= 10);
    assert!(timer.elapsed() >= Duration::from_millis(10));
}

#[test]
fn logger_slow_query_detection() {
    use rok_orm::logging::Logger;

    let logger = Logger::new().with_slow_query_threshold(100);

    assert!(!logger.is_slow_query(50));
    assert!(logger.is_slow_query(150));
}

#[test]
fn delete_in_sql_empty_values() {
    let builder = QueryBuilder::<()>::new("users");
    let (sql, params) = builder.to_delete_in_sql_with_dialect(Dialect::Postgres, "id", &[]);

    assert!(sql.contains("DELETE FROM users"));
    assert!(params.is_empty());
}

#[test]
fn delete_in_sql_sqlite_dialect() {
    let builder = QueryBuilder::<()>::new("users");
    let (sql, params) =
        builder.to_delete_in_sql_with_dialect(Dialect::Sqlite, "id", &[1i64.into(), 2i64.into()]);

    assert!(sql.contains("WHERE id IN (?, ?)"));
    assert_eq!(params.len(), 2);
}
