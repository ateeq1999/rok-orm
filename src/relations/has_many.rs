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

    /// Insert a new child row with the FK already set (PostgreSQL).
    #[cfg(feature = "postgres")]
    pub async fn create_pg(
        &self,
        pool: &sqlx::PgPool,
        parent_id: SqlValue,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error> {
        let mut full: Vec<(&str, SqlValue)> = vec![(&self.foreign_key, parent_id)];
        full.extend_from_slice(data);
        let (sql, params) = QueryBuilder::<C>::insert_sql_with_dialect(
            crate::query::Dialect::Postgres, self.child_table, &full,
        );
        crate::executor::postgres::execute_raw(pool, &sql, params).await
    }

    /// Insert a new child row and return it (PostgreSQL `RETURNING *`).
    #[cfg(feature = "postgres")]
    pub async fn create_returning_pg(
        &self,
        pool: &sqlx::PgPool,
        parent_id: SqlValue,
        data: &[(&str, SqlValue)],
    ) -> Result<C, sqlx::Error>
    where C: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let mut full: Vec<(&str, SqlValue)> = vec![(&self.foreign_key, parent_id)];
        full.extend_from_slice(data);
        crate::executor::postgres::insert_returning::<C>(pool, self.child_table, &full).await
    }

    /// Insert a new child row with the FK already set (SQLite).
    #[cfg(feature = "sqlite")]
    pub async fn create_sqlite(
        &self,
        pool: &sqlx::SqlitePool,
        parent_id: SqlValue,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error> {
        let mut full: Vec<(&str, SqlValue)> = vec![(&self.foreign_key, parent_id)];
        full.extend_from_slice(data);
        let (sql, params) = QueryBuilder::<C>::insert_sql_with_dialect(
            crate::query::Dialect::Sqlite, self.child_table, &full,
        );
        crate::executor::sqlite::execute_raw(pool, &sql, params).await
    }

    /// Build UPDATE-or-INSERT SQL for a child: sets its FK to the parent ID.
    ///
    /// If `child_pk_val` is `Some`, it's an UPDATE (the child already has a PK).
    /// If `None`, it's an INSERT.
    pub fn save_sql(
        &self,
        parent_id: SqlValue,
        child_pk_val: Option<SqlValue>,
        data: &[(&str, SqlValue)],
    ) -> (String, Vec<SqlValue>) {
        if let Some(pk_val) = child_pk_val {
            // UPDATE: inject FK then remaining data columns
            let mut all = vec![(&self.foreign_key as &str, parent_id)];
            all.extend_from_slice(data);
            let set_clauses: Vec<String> = all.iter().enumerate()
                .map(|(i, (col, _))| format!("{col} = ${}", i + 1))
                .collect();
            let mut params: Vec<SqlValue> = all.into_iter().map(|(_, v)| v).collect();
            params.push(pk_val);
            let sql = format!(
                "UPDATE {} SET {} WHERE {} = ${}",
                self.child_table,
                set_clauses.join(", "),
                C::primary_key(),
                params.len()
            );
            (sql, params)
        } else {
            // INSERT: use create_sql
            self.create_sql(parent_id, data)
        }
    }

    /// Generate bulk INSERT SQL for multiple child rows, each with FK injected.
    pub fn create_many_sql(
        &self,
        parent_id: SqlValue,
        rows: &[&[(&str, SqlValue)]],
    ) -> (String, Vec<SqlValue>) {
        let full_rows: Vec<Vec<(&str, SqlValue)>> = rows.iter().map(|r| {
            let mut row = vec![(&self.foreign_key as &str, parent_id.clone())];
            row.extend_from_slice(r);
            row
        }).collect();
        QueryBuilder::<C>::bulk_insert_sql(self.child_table, &full_rows)
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

// ── PostgreSQL async execution: save ─────────────────────────────────────────

#[cfg(feature = "postgres")]
impl<P, C> HasMany<P, C>
where
    P: Model,
    C: Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    /// Insert or update a child row with the parent FK injected.
    ///
    /// - `child_pk_val = None` → INSERT (new child)
    /// - `child_pk_val = Some(id)` → UPDATE (existing child by PK)
    ///
    /// `data` is the child's non-FK columns. The FK is automatically prepended.
    pub async fn save(
        &self,
        pool: &sqlx::PgPool,
        parent_id: impl Into<SqlValue>,
        child_pk_val: Option<SqlValue>,
        data: &[(&str, SqlValue)],
    ) -> Result<u64, sqlx::Error> {
        let (sql, params) = self.save_sql(parent_id.into(), child_pk_val, data);
        crate::executor::postgres::execute_raw(pool, &sql, params).await
    }
}

#[cfg(test)]
#[path = "has_many_tests.rs"]
mod tests;
