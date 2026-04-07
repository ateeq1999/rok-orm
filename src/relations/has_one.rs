//! [`HasOne`] — one-to-one relationship (parent owns foreign key on child).

use std::marker::PhantomData;

use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};

use super::traits::Relation;

/// Represents a one-to-one association (`parent` → one optional `child` row).
#[derive(Debug, Clone)]
pub struct HasOne<P, C>
where
    P: Model,
    C: Model,
{
    #[allow(dead_code)]
    parent_table: &'static str,
    #[allow(dead_code)]
    parent_pk: &'static str,
    pub(crate) child_table: &'static str,
    pub(crate) foreign_key: String,
    _phantom: PhantomData<(P, C)>,
}

impl<P, C> HasOne<P, C>
where
    P: Model,
    C: Model,
{
    pub fn new(
        parent_table: &'static str,
        parent_pk: &'static str,
        child_table: &'static str,
        foreign_key: String,
    ) -> Self {
        Self {
            parent_table,
            parent_pk,
            child_table,
            foreign_key,
            _phantom: PhantomData,
        }
    }

    pub fn query_for(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        QueryBuilder::new(self.child_table).where_eq(&self.foreign_key, parent_id)
    }

    pub fn foreign_key(&self) -> &str { &self.foreign_key }
    pub fn child_table(&self) -> &str { self.child_table }

    /// Returns `(delete_sql, delete_params, insert_sql, insert_params)`.
    ///
    /// Execute the delete first, then the insert, ideally inside a transaction.
    pub fn create_or_replace_sql(
        &self,
        parent_id: SqlValue,
        data: &[(&str, SqlValue)],
    ) -> (String, Vec<SqlValue>, String, Vec<SqlValue>) {
        // DELETE FROM child WHERE fk = ?
        let del_sql = format!(
            "DELETE FROM {} WHERE {} = $1",
            self.child_table, self.foreign_key,
        );
        let del_params = vec![parent_id.clone()];

        // Build INSERT: inject FK then data columns
        let mut cols = vec![self.foreign_key.as_str()];
        let mut vals: Vec<SqlValue> = vec![parent_id];
        for (col, val) in data {
            cols.push(col);
            vals.push(val.clone());
        }
        let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("${i}")).collect();
        let ins_sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.child_table,
            cols.join(", "),
            placeholders.join(", "),
        );
        (del_sql, del_params, ins_sql, vals)
    }
}

impl<P, C> Relation<P, C> for HasOne<P, C>
where
    P: Model,
    C: Model,
{
    fn query(&self, parent_id: SqlValue) -> QueryBuilder<C> { self.query_for(parent_id) }
    fn foreign_key_value(&self, _parent: &P) -> SqlValue { SqlValue::Null }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct User;
    impl Model for User {
        fn table_name() -> &'static str { "users" }
        fn columns() -> &'static [&'static str] { &["id", "name"] }
    }

    struct Profile;
    impl Model for Profile {
        fn table_name() -> &'static str { "profiles" }
        fn columns() -> &'static [&'static str] { &["id", "user_id", "bio"] }
    }

    fn profile_rel() -> HasOne<User, Profile> {
        HasOne::new("users", "id", "profiles", "user_id".to_string())
    }

    #[test]
    fn query_for_filters_by_fk() {
        let rel = profile_rel();
        let (sql, params) = rel.query_for(SqlValue::Integer(5)).to_sql();
        assert!(sql.contains("FROM profiles"));
        assert!(sql.contains("user_id = $1"));
        assert_eq!(params[0], SqlValue::Integer(5));
    }

    #[test]
    fn create_or_replace_sql_generates_delete_and_insert() {
        let rel = profile_rel();
        let data = [("bio", SqlValue::Text("Rust dev".into()))];
        let (del_sql, del_params, ins_sql, ins_params) =
            rel.create_or_replace_sql(SqlValue::Integer(1), &data);
        assert!(del_sql.contains("DELETE FROM profiles"));
        assert!(del_sql.contains("user_id = $1"));
        assert_eq!(del_params[0], SqlValue::Integer(1));
        assert!(ins_sql.contains("INSERT INTO profiles"));
        assert!(ins_sql.contains("user_id"));
        assert!(ins_sql.contains("bio"));
        assert_eq!(ins_params.len(), 2);
    }
}
