//! Polymorphic relationships: MorphOne, MorphMany, MorphTo, MorphToMany, MorphedByMany.
//!
//! Polymorphic relations let a single child table belong to multiple parent types
//! via `{morph_key}_type` and `{morph_key}_id` columns.

use std::marker::PhantomData;
use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};

// ── MorphOne ─────────────────────────────────────────────────────────────────

/// One-to-one polymorphic: `Parent` → one `Child` row via `{morph_key}_type/id`.
///
/// SQL: `SELECT * FROM child WHERE {key}_type = 'parents' AND {key}_id = $1 LIMIT 1`
#[derive(Debug, Clone)]
pub struct MorphOne<P, C> {
    child_table: &'static str,
    morph_key: &'static str,
    parent_type: &'static str,
    _phantom: PhantomData<(P, C)>,
}

impl<P: Model, C: Model> MorphOne<P, C> {
    pub fn new(child_table: &'static str, morph_key: &'static str, parent_type: &'static str) -> Self {
        Self { child_table, morph_key, parent_type, _phantom: PhantomData }
    }

    pub fn query_for(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        QueryBuilder::<C>::new(self.child_table)
            .where_eq(&format!("{}_type", self.morph_key), self.parent_type)
            .where_eq(&format!("{}_id", self.morph_key), parent_id)
            .limit(1)
    }

    pub fn morph_key(&self) -> &'static str { self.morph_key }
    pub fn parent_type(&self) -> &'static str { self.parent_type }
    pub fn child_table(&self) -> &'static str { self.child_table }
}

// ── MorphMany ────────────────────────────────────────────────────────────────

/// One-to-many polymorphic: `Parent` → many `Child` rows via `{morph_key}_type/id`.
#[derive(Debug, Clone)]
pub struct MorphMany<P, C> {
    child_table: &'static str,
    morph_key: &'static str,
    parent_type: &'static str,
    _phantom: PhantomData<(P, C)>,
}

impl<P: Model, C: Model> MorphMany<P, C> {
    pub fn new(child_table: &'static str, morph_key: &'static str, parent_type: &'static str) -> Self {
        Self { child_table, morph_key, parent_type, _phantom: PhantomData }
    }

    pub fn query_for(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        QueryBuilder::<C>::new(self.child_table)
            .where_eq(&format!("{}_type", self.morph_key), self.parent_type)
            .where_eq(&format!("{}_id", self.morph_key), parent_id)
    }

    /// Build INSERT SQL with morph_type + morph_id injected.
    pub fn create_sql(&self, parent_id: SqlValue, data: &[(&str, SqlValue)]) -> (String, Vec<SqlValue>) {
        let type_col = format!("{}_type", self.morph_key);
        let id_col = format!("{}_id", self.morph_key);
        let mut full: Vec<(&str, SqlValue)> = vec![
            (Box::leak(type_col.into_boxed_str()), SqlValue::Text(self.parent_type.into())),
            (Box::leak(id_col.into_boxed_str()), parent_id),
        ];
        full.extend_from_slice(data);
        QueryBuilder::<C>::insert_sql(self.child_table, &full)
    }

    pub fn morph_key(&self) -> &'static str { self.morph_key }
    pub fn parent_type(&self) -> &'static str { self.parent_type }
    pub fn child_table(&self) -> &'static str { self.child_table }
}

// ── MorphToRef ───────────────────────────────────────────────────────────────

/// Inverse of MorphOne/MorphMany — resolves the polymorphic parent.
///
/// Provides SQL helpers; runtime resolution is done via `morph_type_map!`.
#[derive(Debug, Clone)]
pub struct MorphToRef {
    morph_key: &'static str,
}

impl MorphToRef {
    pub fn new(morph_key: &'static str) -> Self {
        Self { morph_key }
    }

    /// Build a SELECT for a specific parent type and ID.
    pub fn query_for_type<C: Model>(&self, morph_type_val: &str, morph_id: SqlValue) -> QueryBuilder<C> {
        let _ = morph_type_val; // type determines the table; caller picks C
        QueryBuilder::<C>::new(C::table_name()).where_eq(C::primary_key(), morph_id)
    }

    pub fn morph_key(&self) -> &'static str { self.morph_key }
    pub fn type_col(&self) -> String { format!("{}_type", self.morph_key) }
    pub fn id_col(&self) -> String { format!("{}_id", self.morph_key) }
}

// ── MorphToMany ──────────────────────────────────────────────────────────────

/// Polymorphic many-to-many: parent → pivot({morph_key}_type, {morph_key}_id, related_fk) → related.
#[derive(Debug, Clone)]
pub struct MorphToMany<P, C> {
    pivot_table: &'static str,
    morph_key: &'static str,
    parent_type: &'static str,
    related_fk: &'static str,
    related_table: &'static str,
    related_pk: &'static str,
    _phantom: PhantomData<(P, C)>,
}

impl<P: Model, C: Model> MorphToMany<P, C> {
    pub fn new(
        pivot_table: &'static str,
        morph_key: &'static str,
        parent_type: &'static str,
        related_fk: &'static str,
        related_table: &'static str,
        related_pk: &'static str,
    ) -> Self {
        Self { pivot_table, morph_key, parent_type, related_fk, related_table, related_pk, _phantom: PhantomData }
    }

    pub fn query_for(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        let on = format!(
            "{}.{} = {}.{}",
            self.related_table, self.related_pk, self.pivot_table, self.related_fk
        );
        QueryBuilder::<C>::new(self.related_table)
            .inner_join(self.pivot_table, &on)
            .where_eq(&format!("{}.{}_type", self.pivot_table, self.morph_key), self.parent_type)
            .where_eq(&format!("{}.{}_id", self.pivot_table, self.morph_key), parent_id)
    }

    pub fn attach_sql(&self, parent_id: SqlValue, related_id: SqlValue) -> (String, Vec<SqlValue>) {
        let type_col = format!("{}_type", self.morph_key);
        let id_col = format!("{}_id", self.morph_key);
        let sql = format!(
            "INSERT INTO {} ({}, {}, {}) VALUES ($1, $2, $3)",
            self.pivot_table, type_col, id_col, self.related_fk
        );
        (sql, vec![SqlValue::Text(self.parent_type.into()), parent_id, related_id])
    }

    pub fn detach_sql(&self, parent_id: SqlValue, related_id: SqlValue) -> (String, Vec<SqlValue>) {
        let sql = format!(
            "DELETE FROM {} WHERE {}_type = $1 AND {}_id = $2 AND {} = $3",
            self.pivot_table, self.morph_key, self.morph_key, self.related_fk
        );
        (sql, vec![SqlValue::Text(self.parent_type.into()), parent_id, related_id])
    }

    pub fn sync_current_ids_sql(&self, parent_id: SqlValue) -> (String, Vec<SqlValue>) {
        let sql = format!(
            "SELECT {} FROM {} WHERE {}_type = $1 AND {}_id = $2",
            self.related_fk, self.pivot_table, self.morph_key, self.morph_key
        );
        (sql, vec![SqlValue::Text(self.parent_type.into()), parent_id])
    }
}

// ── MorphedByMany ─────────────────────────────────────────────────────────────

/// Inverse of MorphToMany: related model → pivot → many parent types.
#[derive(Debug, Clone)]
pub struct MorphedByMany<P, C> {
    pivot_table: &'static str,
    morph_key: &'static str,
    related_type: &'static str,   // the type string for Self (e.g. "posts")
    left_fk: &'static str,        // FK in pivot pointing at Self
    parent_table: &'static str,
    parent_pk: &'static str,
    _phantom: PhantomData<(P, C)>,
}

impl<P: Model, C: Model> MorphedByMany<P, C> {
    pub fn new(
        pivot_table: &'static str,
        morph_key: &'static str,
        related_type: &'static str,
        left_fk: &'static str,
        parent_table: &'static str,
        parent_pk: &'static str,
    ) -> Self {
        Self { pivot_table, morph_key, related_type, left_fk, parent_table, parent_pk, _phantom: PhantomData }
    }

    pub fn query_for(&self, self_id: SqlValue) -> QueryBuilder<C> {
        let on = format!(
            "{}.{} = {}.{}_id AND {}.{}_type = '{}'",
            self.parent_table, self.parent_pk,
            self.pivot_table, self.morph_key,
            self.pivot_table, self.morph_key, self.related_type,
        );
        QueryBuilder::<C>::new(self.parent_table)
            .inner_join(self.pivot_table, &on)
            .where_eq(&format!("{}.{}", self.pivot_table, self.left_fk), self_id)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct User;
    impl Model for User {
        fn table_name() -> &'static str { "users" }
        fn columns() -> &'static [&'static str] { &["id", "name"] }
    }

    struct Post;
    impl Model for Post {
        fn table_name() -> &'static str { "posts" }
        fn columns() -> &'static [&'static str] { &["id", "title"] }
    }

    struct Image;
    impl Model for Image {
        fn table_name() -> &'static str { "images" }
        fn columns() -> &'static [&'static str] { &["id", "imageable_type", "imageable_id", "url"] }
    }

    struct Tag;
    impl Model for Tag {
        fn table_name() -> &'static str { "tags" }
        fn columns() -> &'static [&'static str] { &["id", "name"] }
    }

    #[test]
    fn morph_one_query_filters_type_and_id() {
        let rel: MorphOne<User, Image> = MorphOne::new("images", "imageable", "users");
        let (sql, params) = rel.query_for(SqlValue::Integer(5)).to_sql();
        assert!(sql.contains("FROM images"), "sql: {sql}");
        assert!(sql.contains("imageable_type = $1"), "sql: {sql}");
        assert!(sql.contains("imageable_id = $2"), "sql: {sql}");
        assert!(sql.contains("LIMIT 1"), "sql: {sql}");
        assert_eq!(params[0], SqlValue::Text("users".into()));
        assert_eq!(params[1], SqlValue::Integer(5));
    }

    #[test]
    fn morph_many_query_filters_type_and_id_no_limit() {
        let rel: MorphMany<Post, Image> = MorphMany::new("images", "imageable", "posts");
        let (sql, params) = rel.query_for(SqlValue::Integer(3)).to_sql();
        assert!(sql.contains("FROM images"), "sql: {sql}");
        assert!(sql.contains("imageable_type = $1"), "sql: {sql}");
        assert!(sql.contains("imageable_id = $2"), "sql: {sql}");
        assert!(!sql.contains("LIMIT"), "sql should not have LIMIT: {sql}");
        assert_eq!(params[0], SqlValue::Text("posts".into()));
        assert_eq!(params[1], SqlValue::Integer(3));
    }

    #[test]
    fn morph_to_ref_columns() {
        let r = MorphToRef::new("imageable");
        assert_eq!(r.type_col(), "imageable_type");
        assert_eq!(r.id_col(), "imageable_id");
    }

    #[test]
    fn morph_to_many_query_joins_pivot() {
        let rel: MorphToMany<Post, Tag> = MorphToMany::new(
            "taggables", "taggable", "posts", "tag_id", "tags", "id"
        );
        let (sql, params) = rel.query_for(SqlValue::Integer(1)).to_sql();
        assert!(sql.contains("INNER JOIN taggables"), "sql: {sql}");
        assert!(sql.contains("taggable_type"), "sql: {sql}");
        assert_eq!(params[0], SqlValue::Text("posts".into()));
    }

    #[test]
    fn morph_to_many_attach_sql() {
        let rel: MorphToMany<Post, Tag> = MorphToMany::new(
            "taggables", "taggable", "posts", "tag_id", "tags", "id"
        );
        let (sql, params) = rel.attach_sql(SqlValue::Integer(1), SqlValue::Integer(5));
        assert!(sql.contains("INSERT INTO taggables"), "sql: {sql}");
        assert_eq!(params[0], SqlValue::Text("posts".into()));
    }

    #[test]
    fn morph_to_many_detach_sql() {
        let rel: MorphToMany<Post, Tag> = MorphToMany::new(
            "taggables", "taggable", "posts", "tag_id", "tags", "id"
        );
        let (sql, params) = rel.detach_sql(SqlValue::Integer(1), SqlValue::Integer(5));
        assert!(sql.contains("DELETE FROM taggables"), "sql: {sql}");
        assert_eq!(params[0], SqlValue::Text("posts".into()));
    }

    #[test]
    fn morphed_by_many_query_joins_pivot() {
        let rel: MorphedByMany<Tag, Post> = MorphedByMany::new(
            "taggables", "taggable", "posts", "tag_id", "posts", "id"
        );
        let (sql, _) = rel.query_for(SqlValue::Integer(2)).to_sql();
        assert!(sql.contains("INNER JOIN taggables"), "sql: {sql}");
    }
}
