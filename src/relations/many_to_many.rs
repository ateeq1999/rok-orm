//! [`ManyToMany`] — full pivot access for many-to-many relationships.
//!
//! Provides attach, detach, sync, toggle, with_pivot, and update_pivot operations.

use std::marker::PhantomData;
use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};

// ── struct ──────────────────────────────────────────────────────────────────

/// Many-to-many relationship with full pivot table access.
#[derive(Debug, Clone)]
pub struct ManyToMany<P, C>
where
    P: Model,
    C: Model,
{
    pub(crate) pivot_table: String,
    pub(crate) left_key: String,
    pub(crate) right_key: String,
    pub(crate) related_table: &'static str,
    pub(crate) related_pk: &'static str,
    pub(crate) pivot_columns: Vec<String>,
    _phantom: PhantomData<(P, C)>,
}

impl<P, C> ManyToMany<P, C>
where
    P: Model,
    C: Model,
{
    pub fn new(
        pivot_table: impl Into<String>,
        left_key: impl Into<String>,
        right_key: impl Into<String>,
        related_table: &'static str,
        related_pk: &'static str,
    ) -> Self {
        Self {
            pivot_table: pivot_table.into(),
            left_key: left_key.into(),
            right_key: right_key.into(),
            related_table,
            related_pk,
            pivot_columns: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn pivot_table_name(&self) -> &str { &self.pivot_table }
    pub fn left_key_name(&self) -> &str { &self.left_key }
    pub fn right_key_name(&self) -> &str { &self.right_key }

    /// Include extra pivot columns in the SELECT.
    pub fn with_pivot(mut self, cols: &[&str]) -> Self {
        self.pivot_columns = cols.iter().map(|s| s.to_string()).collect();
        self
    }

    // ── SQL generation ───────────────────────────────────────────────────────

    /// SELECT … INNER JOIN … WHERE pivot.left_key = $1
    pub fn query_for(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        let on = format!(
            "{}.{} = {}.{}",
            self.related_table, self.related_pk, self.pivot_table, self.right_key
        );
        let mut qb = QueryBuilder::<C>::new(self.related_table)
            .inner_join(&self.pivot_table, &on)
            .where_eq(
                &format!("{}.{}", self.pivot_table, self.left_key),
                parent_id,
            );
        if !self.pivot_columns.is_empty() {
            let pivot_cols: Vec<String> = self.pivot_columns
                .iter()
                .map(|c| format!("{}.{}", self.pivot_table, c))
                .collect();
            let mut all_cols = vec!["*".to_string()];
            all_cols.extend(pivot_cols);
            let col_refs: Vec<&str> = all_cols.iter().map(|s| s.as_str()).collect();
            qb = qb.select(&col_refs);
        }
        qb
    }

    pub fn attach_sql(&self, parent_id: SqlValue, related_id: SqlValue) -> (String, Vec<SqlValue>) {
        let sql = format!(
            "INSERT INTO {} ({}, {}) VALUES ($1, $2)",
            self.pivot_table, self.left_key, self.right_key
        );
        (sql, vec![parent_id, related_id])
    }

    pub fn attach_with_pivot_sql(
        &self,
        parent_id: SqlValue,
        related_id: SqlValue,
        extra: &[(&str, SqlValue)],
    ) -> (String, Vec<SqlValue>) {
        let mut cols = vec![self.left_key.as_str(), self.right_key.as_str()];
        let mut params = vec![parent_id, related_id];
        for (col, val) in extra {
            cols.push(col);
            params.push(val.clone());
        }
        let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("${i}")).collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.pivot_table,
            cols.join(", "),
            placeholders.join(", ")
        );
        (sql, params)
    }

    pub fn detach_sql(&self, parent_id: SqlValue, related_id: SqlValue) -> (String, Vec<SqlValue>) {
        let sql = format!(
            "DELETE FROM {} WHERE {} = $1 AND {} = $2",
            self.pivot_table, self.left_key, self.right_key
        );
        (sql, vec![parent_id, related_id])
    }

    pub fn detach_all_sql(&self, parent_id: SqlValue) -> (String, Vec<SqlValue>) {
        let sql = format!(
            "DELETE FROM {} WHERE {} = $1",
            self.pivot_table, self.left_key
        );
        (sql, vec![parent_id])
    }

    pub fn current_ids_sql(&self, parent_id: SqlValue) -> (String, Vec<SqlValue>) {
        let sql = format!(
            "SELECT {} FROM {} WHERE {} = $1",
            self.right_key, self.pivot_table, self.left_key
        );
        (sql, vec![parent_id])
    }

    pub fn update_pivot_sql(
        &self,
        parent_id: SqlValue,
        related_id: SqlValue,
        data: &[(&str, SqlValue)],
    ) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let set_clauses: Vec<String> = data
            .iter()
            .enumerate()
            .map(|(i, (col, val))| {
                params.push(val.clone());
                format!("{col} = ${}", i + 1)
            })
            .collect();
        params.push(parent_id);
        params.push(related_id);
        let pk_offset = data.len() + 1;
        let sql = format!(
            "UPDATE {} SET {} WHERE {} = ${} AND {} = ${}",
            self.pivot_table,
            set_clauses.join(", "),
            self.left_key,
            pk_offset,
            self.right_key,
            pk_offset + 1
        );
        (sql, params)
    }
}

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct User;
    impl Model for User {
        fn table_name() -> &'static str { "users" }
        fn columns() -> &'static [&'static str] { &["id", "name"] }
    }

    struct Role;
    impl Model for Role {
        fn table_name() -> &'static str { "roles" }
        fn columns() -> &'static [&'static str] { &["id", "name"] }
    }

    fn rel() -> ManyToMany<User, Role> {
        ManyToMany::new("user_roles", "user_id", "role_id", "roles", "id")
    }

    #[test]
    fn query_for_generates_inner_join() {
        let (sql, params) = rel().query_for(SqlValue::Integer(1)).to_sql();
        assert!(sql.contains("INNER JOIN user_roles"), "sql: {sql}");
        assert!(sql.contains("user_roles.user_id = $1"), "sql: {sql}");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], SqlValue::Integer(1));
    }

    #[test]
    fn attach_sql_inserts_pivot_row() {
        let (sql, params) = rel().attach_sql(SqlValue::Integer(1), SqlValue::Integer(5));
        assert!(sql.contains("INSERT INTO user_roles"), "sql: {sql}");
        assert!(sql.contains("user_id"), "sql: {sql}");
        assert!(sql.contains("role_id"), "sql: {sql}");
        assert_eq!(params[0], SqlValue::Integer(1));
        assert_eq!(params[1], SqlValue::Integer(5));
    }

    #[test]
    fn attach_with_pivot_sql_includes_extra_cols() {
        let (sql, params) = rel().attach_with_pivot_sql(
            SqlValue::Integer(1),
            SqlValue::Integer(5),
            &[("assigned_at", SqlValue::Text("2026-01-01".into()))],
        );
        assert!(sql.contains("assigned_at"), "sql: {sql}");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn detach_sql_deletes_pivot_row() {
        let (sql, params) = rel().detach_sql(SqlValue::Integer(1), SqlValue::Integer(5));
        assert!(sql.starts_with("DELETE FROM user_roles"), "sql: {sql}");
        assert!(sql.contains("user_id = $1"), "sql: {sql}");
        assert!(sql.contains("role_id = $2"), "sql: {sql}");
        assert_eq!(params[0], SqlValue::Integer(1));
        assert_eq!(params[1], SqlValue::Integer(5));
    }

    #[test]
    fn detach_all_sql_deletes_all_for_parent() {
        let (sql, params) = rel().detach_all_sql(SqlValue::Integer(7));
        assert!(sql.starts_with("DELETE FROM user_roles"), "sql: {sql}");
        assert!(sql.contains("user_id = $1"), "sql: {sql}");
        assert!(!sql.contains("role_id"), "sql: {sql}");
        assert_eq!(params[0], SqlValue::Integer(7));
    }

    #[test]
    fn current_ids_sql_selects_right_key() {
        let (sql, params) = rel().current_ids_sql(SqlValue::Integer(3));
        assert!(sql.contains("SELECT role_id FROM user_roles"), "sql: {sql}");
        assert!(sql.contains("user_id = $1"), "sql: {sql}");
        assert_eq!(params[0], SqlValue::Integer(3));
    }

    #[test]
    fn update_pivot_sql_sets_cols() {
        let (sql, params) = rel().update_pivot_sql(
            SqlValue::Integer(1),
            SqlValue::Integer(2),
            &[("expires_at", SqlValue::Text("2027-01-01".into()))],
        );
        assert!(sql.starts_with("UPDATE user_roles SET"), "sql: {sql}");
        assert!(sql.contains("expires_at = $1"), "sql: {sql}");
        assert!(sql.contains("user_id = $2"), "sql: {sql}");
        assert!(sql.contains("role_id = $3"), "sql: {sql}");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn with_pivot_includes_extra_cols_in_select() {
        let (sql, _) = rel()
            .with_pivot(&["assigned_at", "expires_at"])
            .query_for(SqlValue::Integer(1))
            .to_sql();
        assert!(sql.contains("user_roles.assigned_at"), "sql: {sql}");
        assert!(sql.contains("user_roles.expires_at"), "sql: {sql}");
    }
}

// ── PostgreSQL execution ─────────────────────────────────────────────────────

#[cfg(feature = "postgres")]
mod pg {
    use sqlx::{postgres::PgRow, PgPool, Row};
    use crate::executor::postgres;
    use crate::model::Model;
    use crate::query::SqlValue;
    use super::ManyToMany;

    impl<P, C> ManyToMany<P, C>
    where
        P: Model,
        C: Model + for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
    {
        pub async fn get(
            &self,
            pool: &PgPool,
            parent_id: impl Into<SqlValue>,
        ) -> Result<Vec<C>, sqlx::Error> {
            postgres::fetch_all(pool, self.query_for(parent_id.into())).await
        }

        pub async fn attach(
            &self,
            pool: &PgPool,
            parent_id: impl Into<SqlValue>,
            related_id: impl Into<SqlValue>,
        ) -> Result<u64, sqlx::Error> {
            let (sql, params) = self.attach_sql(parent_id.into(), related_id.into());
            postgres::execute_raw(pool, &sql, params).await
        }

        pub async fn attach_with_pivot(
            &self,
            pool: &PgPool,
            parent_id: impl Into<SqlValue>,
            related_id: impl Into<SqlValue>,
            extra: &[(&str, SqlValue)],
        ) -> Result<u64, sqlx::Error> {
            let (sql, params) =
                self.attach_with_pivot_sql(parent_id.into(), related_id.into(), extra);
            postgres::execute_raw(pool, &sql, params).await
        }

        pub async fn detach(
            &self,
            pool: &PgPool,
            parent_id: impl Into<SqlValue>,
            related_id: impl Into<SqlValue>,
        ) -> Result<u64, sqlx::Error> {
            let (sql, params) = self.detach_sql(parent_id.into(), related_id.into());
            postgres::execute_raw(pool, &sql, params).await
        }

        pub async fn detach_all(
            &self,
            pool: &PgPool,
            parent_id: impl Into<SqlValue>,
        ) -> Result<u64, sqlx::Error> {
            let (sql, params) = self.detach_all_sql(parent_id.into());
            postgres::execute_raw(pool, &sql, params).await
        }

        pub async fn sync(
            &self,
            pool: &PgPool,
            parent_id: impl Into<SqlValue> + Clone,
            desired_ids: Vec<i64>,
        ) -> Result<(), sqlx::Error> {
            let pid = parent_id.into();
            let (sql, params) = self.current_ids_sql(pid.clone());
            let rows = sqlx::query(&sql)
                .bind(match &params[0] {
                    SqlValue::Integer(n) => *n,
                    _ => 0,
                })
                .fetch_all(pool)
                .await?;
            let current: Vec<i64> = rows.iter().map(|r| r.get::<i64, _>(0)).collect();
            for id in &desired_ids {
                if !current.contains(id) {
                    self.attach(pool, pid.clone(), *id).await?;
                }
            }
            for id in &current {
                if !desired_ids.contains(id) {
                    self.detach(pool, pid.clone(), *id).await?;
                }
            }
            Ok(())
        }

        pub async fn toggle(
            &self,
            pool: &PgPool,
            parent_id: impl Into<SqlValue> + Clone,
            ids: Vec<i64>,
        ) -> Result<(), sqlx::Error> {
            let pid = parent_id.into();
            let (sql, params) = self.current_ids_sql(pid.clone());
            let rows = sqlx::query(&sql)
                .bind(match &params[0] {
                    SqlValue::Integer(n) => *n,
                    _ => 0,
                })
                .fetch_all(pool)
                .await?;
            let current: Vec<i64> = rows.iter().map(|r| r.get::<i64, _>(0)).collect();
            for id in ids {
                if current.contains(&id) {
                    self.detach(pool, pid.clone(), id).await?;
                } else {
                    self.attach(pool, pid.clone(), id).await?;
                }
            }
            Ok(())
        }

        pub async fn update_pivot(
            &self,
            pool: &PgPool,
            parent_id: impl Into<SqlValue>,
            related_id: impl Into<SqlValue>,
            data: &[(&str, SqlValue)],
        ) -> Result<u64, sqlx::Error> {
            let (sql, params) =
                self.update_pivot_sql(parent_id.into(), related_id.into(), data);
            postgres::execute_raw(pool, &sql, params).await
        }
    }
}
