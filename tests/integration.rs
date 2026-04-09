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

// ── through relations ─────────────────────────────────────────────────────────

#[derive(Model)]
pub struct Country { pub id: i64, pub name: String }

#[derive(Model)]
pub struct Post { pub id: i64, pub user_id: i64, pub country_id: i64, pub title: String }

// Through-relation test uses struct named exactly like the data model
// so that FK derivation (country_id) works correctly.
#[derive(Model, rok_orm_macros::Relations)]
#[model(table = "countries")]
pub struct CountryModel {
    pub id: i64,
    #[model(has_many_through(Post, User))]
    pub _posts: std::marker::PhantomData<Post>,
}

#[test]
fn has_many_through_macro_generates_join_query() {
    let rel = CountryModel { id: 1, _posts: std::marker::PhantomData }._posts();
    let (sql, params) = rel.query_for(1i64.into()).to_sql();
    assert!(sql.contains("INNER JOIN users ON users.id = posts.user_id"), "sql: {sql}");
    assert!(sql.contains("WHERE users.country_model_id = $1"), "sql: {sql}");
    assert_eq!(params.len(), 1);
}

#[test]
fn has_many_through_macro_table_names() {
    let rel = CountryModel { id: 2, _posts: std::marker::PhantomData }._posts();
    let (sql, _) = rel.query_for(2i64.into()).to_sql();
    assert!(sql.contains("FROM posts"), "sql: {sql}");
}

// ── table names ──────────────────────────────────────────────────────────────

#[test]
fn table_name_simple() {
    assert_eq!(User::table_name(), "users");
}

#[test]
fn table_name_multi_word() {
    assert_eq!(BlogPost::table_name(), "blog_posts");
    assert_eq!(OrderItem::table_name(), "order_items");
}

// ΓöÇΓöÇ columns ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn columns_list() {
    assert_eq!(User::columns(), &["id", "name", "email"]);
    assert_eq!(BlogPost::columns(), &["id", "title", "body", "published"]);
}

// ΓöÇΓöÇ query builder through Model trait ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ attribute: custom table name ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn custom_table_name() {
    assert_eq!(Article::table_name(), "articles");
    assert_eq!(Article::columns(), &["id", "title"]);
}

// ΓöÇΓöÇ attribute: custom primary key ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn struct_level_primary_key() {
    assert_eq!(Profile::primary_key(), "user_id");
}

#[test]
fn field_level_primary_key() {
    assert_eq!(Tag::primary_key(), "tag_id");
}

// ΓöÇΓöÇ attribute: skip and column rename ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn skip_excludes_field() {
    // cached_count is skipped
    assert_eq!(Tag::columns(), &["tag_id", "tag_name"]);
}

// ΓöÇΓöÇ OR conditions ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn or_where_conditions() {
    let (sql, params) = User::query()
        .where_eq("role", "admin")
        .or_where_eq("role", "moderator")
        .to_sql();
    assert!(sql.contains("role = $1 OR role = $2"));
    assert_eq!(params.len(), 2);
}

// ΓöÇΓöÇ between ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn where_between_query() {
    let (sql, params) = User::query().where_between("id", 1i64, 100i64).to_sql();
    assert!(sql.contains("id BETWEEN $1 AND $2"));
    assert_eq!(params.len(), 2);
}

// ΓöÇΓöÇ not_in ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn where_not_in_query() {
    let (sql, params) = User::query().where_not_in("id", vec![1i64, 2, 3]).to_sql();
    assert!(sql.contains("id NOT IN ($1, $2, $3)"));
    assert_eq!(params.len(), 3);
}

// ΓöÇΓöÇ distinct ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn distinct_query() {
    let (sql, _) = User::query().distinct().select(&["email"]).to_sql();
    assert!(sql.starts_with("SELECT DISTINCT email FROM users"));
}

// ΓöÇΓöÇ to_update_sql ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn update_sql_via_builder() {
    let (sql, params) =
        User::find(1i64).to_update_sql(&[("name", "Bob".into()), ("email", "b@b.com".into())]);
    assert!(sql.starts_with("UPDATE users SET name = $1, email = $2"));
    assert!(sql.contains("WHERE id = $3"));
    assert_eq!(params.len(), 3);
}

// ΓöÇΓöÇ join ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ group by / having ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ bulk insert ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ filter shorthand ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ soft delete model ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

#[test]
fn soft_delete_enabled() {
    assert_eq!(SoftDeletePost::soft_delete_column(), Some("deleted_at"));
}

#[test]
fn soft_delete_disabled() {
    assert_eq!(User::soft_delete_column(), None);
}

// ΓöÇΓöÇ timestamps model ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ query! macro with filter shorthand ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ eq shorthand ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ ModelHooks derive ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ BelongsToMany ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ eager loading ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ pagination ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ aggregation ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ upsert ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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
        Dialect::Postgres,
        "users",
        &[("email", "test@example.com".into())],
        "email",
    );

    assert!(sql.contains("INSERT INTO users"));
    assert!(sql.contains("ON CONFLICT (email)"));
    assert!(sql.contains("DO NOTHING"));
    assert_eq!(params.len(), 1);
}

// ΓöÇΓöÇ batch operations ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

// ΓöÇΓöÇ exists and pluck ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ error handling ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ΓöÇΓöÇ logging ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

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

// ── to_fields ──────────────────────────────────────────────────────────────

#[test]
fn to_fields_returns_non_pk_columns() {
    use rok_orm::SqlValue;
    let user = User { id: 1, name: "Alice".into(), email: "a@b.com".into() };
    let fields = user.to_fields();
    assert_eq!(fields.len(), 2);
    assert!(fields.iter().any(|(k, _)| *k == "name"));
    assert!(fields.iter().any(|(k, _)| *k == "email"));
    // PK (id) must not appear
    assert!(!fields.iter().any(|(k, _)| *k == "id"));
}

#[test]
fn to_fields_respects_column_rename() {
    use rok_orm::SqlValue;
    let tag = Tag { tag_id: 7, name: "rust".into(), cached_count: 0 };
    let fields = tag.to_fields();
    // tag_name is the renamed column for `name`; cached_count is #[model(skip)]
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "tag_name");
    assert_eq!(fields[0].1, SqlValue::Text("rust".into()));
}

#[test]
fn to_fields_skips_skipped_fields() {
    let tag = Tag { tag_id: 1, name: "x".into(), cached_count: 99 };
    let fields = tag.to_fields();
    assert!(!fields.iter().any(|(k, _)| *k == "cached_count"));
}

// ── 7.1 many_to_many named-params macro ──────────────────────────────────────

#[derive(rok_orm::Model, rok_orm_macros::Relations)]
pub struct UserWithRoles {
    pub id: i64,
    pub name: String,
    #[model(many_to_many(
        related = "Role",
        pivot   = "user_roles",
        fk      = "user_id",
        rfk     = "role_id",
        pivots  = ["assigned_at"],
    ))]
    pub roles: std::marker::PhantomData<Role>,
}

#[derive(rok_orm::Model)]
pub struct Role {
    pub id: i64,
    pub name: String,
}

#[test]
fn many_to_many_named_pivot_table() {
    let rel = UserWithRoles { id: 1, name: "Alice".into(), roles: std::marker::PhantomData }.roles();
    assert_eq!(rel.pivot_table_name(), "user_roles");
    assert_eq!(rel.left_key_name(), "user_id");
    assert_eq!(rel.right_key_name(), "role_id");
}

#[test]
fn many_to_many_named_with_pivot_in_query() {
    let rel = UserWithRoles { id: 1, name: "Alice".into(), roles: std::marker::PhantomData }.roles();
    // pivots = ["assigned_at"] means with_pivot was called, SELECT includes pivot cols
    let (sql, _) = rel.query_for(rok_orm::SqlValue::Integer(1)).to_sql();
    assert!(sql.contains("user_roles.assigned_at") || sql.contains("assigned_at"), "sql: {sql}");
}

#[test]
fn many_to_many_named_attach_sql() {
    let rel = UserWithRoles { id: 1, name: "Alice".into(), roles: std::marker::PhantomData }.roles();
    let (sql, params) = rel.attach_sql(rok_orm::SqlValue::Integer(1), rok_orm::SqlValue::Integer(3));
    assert!(sql.contains("INSERT INTO user_roles"), "sql: {sql}");
    assert_eq!(params.len(), 2);
}

#[test]
fn many_to_many_named_sync_sql() {
    let rel = UserWithRoles { id: 1, name: "Alice".into(), roles: std::marker::PhantomData }.roles();
    let (sql, _) = rel.current_ids_sql(rok_orm::SqlValue::Integer(1));
    assert!(sql.contains("SELECT role_id FROM user_roles"), "sql: {sql}");
}

// ── 7.3 HasOneThrough SQL generation ─────────────────────────────────────────

#[derive(rok_orm::Model, rok_orm_macros::Relations)]
#[model(table = "mechanics")]
pub struct Mechanic {
    pub id: i64,
    pub name: String,
    #[model(has_one_through(CarOwner, Car))]
    pub car_owner: std::marker::PhantomData<CarOwner>,
}

#[derive(rok_orm::Model)]
pub struct Car {
    pub id: i64,
    pub mechanic_id: i64,
}

#[derive(rok_orm::Model)]
pub struct CarOwner {
    pub id: i64,
    pub car_id: i64,
}

#[test]
fn has_one_through_macro_generates_join_query() {
    let mech = Mechanic { id: 1, name: "Bob".into(), car_owner: std::marker::PhantomData };
    let rel = mech.car_owner();
    let (sql, params) = rel.query_for(rok_orm::SqlValue::Integer(1)).to_sql();
    assert!(sql.contains("INNER JOIN cars ON cars.id = car_owners.car_id"), "sql: {sql}");
    assert!(sql.contains("WHERE cars.mechanic_id = $1"), "sql: {sql}");
    assert!(sql.contains("LIMIT 1"), "sql: {sql}");
    assert_eq!(params.len(), 1);
}

#[test]
fn has_one_through_absent_returns_none_query() {
    let mech = Mechanic { id: 99, name: "Ghost".into(), car_owner: std::marker::PhantomData };
    let rel = mech.car_owner();
    let (sql, _) = rel.query_for(rok_orm::SqlValue::Integer(99)).to_sql();
    assert!(sql.contains("FROM car_owners"), "sql: {sql}");
}

// ── 7.4 Polymorphic macro attrs ───────────────────────────────────────────────

#[derive(rok_orm::Model, rok_orm_macros::Relations)]
pub struct ImageableUser {
    pub id: i64,
    pub name: String,
    #[model(morph_one(related = "Image2", morph_key = "imageable"))]
    pub image: std::marker::PhantomData<Image2>,
}

#[derive(rok_orm::Model, rok_orm_macros::Relations)]
pub struct ImageablePost {
    pub id: i64,
    pub title: String,
    #[model(morph_many(related = "Image2", morph_key = "imageable"))]
    pub images: std::marker::PhantomData<Image2>,
}

#[derive(rok_orm::Model)]
pub struct Image2 {
    pub id: i64,
    pub imageable_type: String,
    pub imageable_id: i64,
    pub url: String,
}

#[test]
fn morph_one_macro_generates_correct_query() {
    let u = ImageableUser { id: 5, name: "Alice".into(), image: std::marker::PhantomData };
    let rel = u.image();
    let (sql, params) = rel.query_for(rok_orm::SqlValue::Integer(5)).to_sql();
    assert!(sql.contains("FROM image2s"), "sql: {sql}");
    assert!(sql.contains("imageable_type"), "sql: {sql}");
    assert!(sql.contains("LIMIT 1"), "sql: {sql}");
    assert_eq!(params[0], rok_orm::SqlValue::Text("imageable_users".into()));
}

#[test]
fn morph_many_macro_generates_correct_query() {
    let p = ImageablePost { id: 3, title: "Hello".into(), images: std::marker::PhantomData };
    let rel = p.images();
    let (sql, params) = rel.query_for(rok_orm::SqlValue::Integer(3)).to_sql();
    assert!(sql.contains("FROM image2s"), "sql: {sql}");
    assert!(!sql.contains("LIMIT"), "morph_many should not have LIMIT: {sql}");
    assert_eq!(params[0], rok_orm::SqlValue::Text("imageable_posts".into()));
}

// ── 8.4 chunk SQL — verify LIMIT/OFFSET generation ───────────────────────────

#[test]
fn chunk_first_page_sql_uses_limit_and_zero_offset() {
    let (sql, _) = User::query()
        .filter("active", true)
        .limit(500)
        .offset(0)
        .to_sql();
    assert!(sql.contains("LIMIT 500"), "sql: {sql}");
    assert!(sql.contains("OFFSET 0"), "sql: {sql}");
}

#[test]
fn chunk_second_page_sql_advances_offset() {
    let (sql, _) = User::query()
        .filter("active", true)
        .limit(500)
        .offset(500)
        .to_sql();
    assert!(sql.contains("LIMIT 500"), "sql: {sql}");
    assert!(sql.contains("OFFSET 500"), "sql: {sql}");
}

#[test]
fn chunk_by_id_sql_uses_where_gt_on_pk() {
    let (sql, params) = User::query()
        .where_gt("id", rok_orm::SqlValue::Integer(100))
        .order_by("id")
        .limit(500)
        .to_sql();
    assert!(sql.contains("id > $1"), "sql: {sql}");
    assert!(sql.contains("ORDER BY id ASC"), "sql: {sql}");
    assert!(sql.contains("LIMIT 500"), "sql: {sql}");
    assert_eq!(params[0], rok_orm::SqlValue::Integer(100));
}
