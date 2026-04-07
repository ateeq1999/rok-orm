//! [`HasMany`] — one-to-many relationship.

use std::marker::PhantomData;

use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};

use super::traits::Relation;

/// Represents a one-to-many association (`parent` → many `child` rows).
#[derive(Debug, Clone)]
pub struct HasMany<P, C>
where
    P: Model,
    C: Model,
{
    #[allow(dead_code)]
    parent_table: &'static str,
    #[allow(dead_code)]
    parent_pk: &'static str,
    #[allow(dead_code)]
    child_pk: &'static str,
    pub(crate) child_table: &'static str,
    pub(crate) foreign_key: String,
    _phantom: PhantomData<(P, C)>,
}

impl<P, C> HasMany<P, C>
where
    P: Model,
    C: Model,
{
    pub fn new(
        parent_table: &'static str,
        parent_pk: &'static str,
        child_table: &'static str,
        child_pk: &'static str,
        foreign_key: String,
    ) -> Self {
        Self {
            parent_table,
            parent_pk,
            child_table,
            child_pk,
            foreign_key,
            _phantom: PhantomData,
        }
    }

    pub fn query_for(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        QueryBuilder::new(self.child_table).where_eq(&self.foreign_key, parent_id)
    }

    pub fn foreign_key(&self) -> &str {
        &self.foreign_key
    }

    pub fn child_table(&self) -> &str {
        self.child_table
    }

    /// Build INSERT SQL for a new child with the FK already injected.
    ///
    /// Returns `(sql, params)` ready for `execute_raw`. Pass `data` as the
    /// other columns to insert (FK is prepended automatically).
    pub fn create_sql(
        &self,
        parent_id: SqlValue,
        data: &[(&str, SqlValue)],
    ) -> (String, Vec<SqlValue>) {
        let mut full_data: Vec<(&str, SqlValue)> = vec![(&self.foreign_key, parent_id)];
        full_data.extend_from_slice(data);
        QueryBuilder::<C>::insert_sql(self.child_table, &full_data)
    }

    /// Associate an existing child row with this parent by updating its FK.
    ///
    /// Returns `(sql, params)` — an `UPDATE child_table SET fk = $1 WHERE pk = $2`.
    pub fn associate_sql(&self, child_pk_val: SqlValue, parent_id: SqlValue) -> (String, Vec<SqlValue>) {
        let sql = format!(
            "UPDATE {} SET {} = $1 WHERE {} = $2",
            self.child_table,
            self.foreign_key,
            C::primary_key(),
        );
        (sql, vec![parent_id, child_pk_val])
    }

    /// Dissociate a child row by setting its FK to NULL.
    ///
    /// Returns `(sql, params)` — `UPDATE child_table SET fk = NULL WHERE pk = $1`.
    pub fn dissociate_sql(&self, child_pk_val: SqlValue) -> (String, Vec<SqlValue>) {
        let sql = format!(
            "UPDATE {} SET {} = NULL WHERE {} = $1",
            self.child_table,
            self.foreign_key,
            C::primary_key(),
        );
        (sql, vec![child_pk_val])
    }
}

impl<P, C> Relation<P, C> for HasMany<P, C>
where
    P: Model,
    C: Model,
{
    fn query(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        self.query_for(parent_id)
    }

    fn foreign_key_value(&self, _parent: &P) -> SqlValue {
        SqlValue::Null
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Model;

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
}
