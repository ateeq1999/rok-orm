use rok_orm::Model;

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

#[derive(ModelHooks)]
pub struct HookTestModel {
    pub id: i64,
    pub name: String,
}

#[test]
fn model_hooks_derive() {
    let model = HookTestModel {
        id: 1,
        name: "Test".to_string(),
    };
    // ModelHooks is implemented via derive macro
    // The default implementations do nothing
    assert_eq!(model.name, "Test");
}

// ── BelongsToMany ────────────────────────────────────────────────────────────

use rok_orm::belongs_to_many::BelongsToMany;
use rok_orm::Model;

#[derive(Model)]
pub struct Post {
    pub id: i64,
    pub title: String,
}

#[derive(Model)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[test]
fn belongs_to_many_creation() {
    let relation = BelongsToMany::<Post, Tag>::new(
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
    let relation = BelongsToMany::<Post, Tag>::new(
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
    let relation = BelongsToMany::<Post, Tag>::new(
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
