//! [`HasOneThrough`] — access a single distant relation through an intermediate model.
//!
//! Example: `Mechanic` has one `CarOwner` **through** `Car`.

use std::marker::PhantomData;
use crate::model::Model;
use crate::query::{QueryBuilder, SqlValue};

/// Represents a has-one-through relationship: `Parent` → through `Middle` → `Child`.
#[derive(Debug, Clone)]
pub struct HasOneThrough<P, M, C>
where
    P: Model,
    M: Model,
    C: Model,
{
    through_table: &'static str,
    through_pk: &'static str,
    /// FK on through table pointing at parent (e.g. "mechanic_id")
    first_key: String,
    /// FK on child table pointing at through table (e.g. "car_id")
    second_key: String,
    child_table: &'static str,
    _phantom: PhantomData<(P, M, C)>,
}

impl<P, M, C> HasOneThrough<P, M, C>
where
    P: Model,
    M: Model,
    C: Model,
{
    pub fn new(
        through_table: &'static str,
        through_pk: &'static str,
        first_key: impl Into<String>,
        second_key: impl Into<String>,
        child_table: &'static str,
    ) -> Self {
        Self {
            through_table,
            through_pk,
            first_key: first_key.into(),
            second_key: second_key.into(),
            child_table,
            _phantom: PhantomData,
        }
    }

    /// Build the query (returns at most one row).
    ///
    /// Generates:
    /// ```sql
    /// SELECT child.* FROM child
    /// INNER JOIN through ON through.pk = child.second_key
    /// WHERE through.first_key = $1
    /// LIMIT 1
    /// ```
    pub fn query_for(&self, parent_id: SqlValue) -> QueryBuilder<C> {
        let on = format!(
            "{}.{} = {}.{}",
            self.through_table, self.through_pk, self.child_table, self.second_key
        );
        QueryBuilder::<C>::new(self.child_table)
            .inner_join(self.through_table, &on)
            .where_eq(
                &format!("{}.{}", self.through_table, self.first_key),
                parent_id,
            )
            .limit(1)
    }

    pub fn through_table(&self) -> &'static str {
        self.through_table
    }

    pub fn first_key(&self) -> &str {
        &self.first_key
    }

    pub fn second_key(&self) -> &str {
        &self.second_key
    }

    pub fn child_table(&self) -> &'static str {
        self.child_table
    }
}

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct Mechanic;
    impl Model for Mechanic {
        fn table_name() -> &'static str { "mechanics" }
        fn columns() -> &'static [&'static str] { &["id", "name"] }
    }

    struct Car;
    impl Model for Car {
        fn table_name() -> &'static str { "cars" }
        fn columns() -> &'static [&'static str] { &["id", "mechanic_id"] }
    }

    struct CarOwner;
    impl Model for CarOwner {
        fn table_name() -> &'static str { "car_owners" }
        fn columns() -> &'static [&'static str] { &["id", "car_id", "name"] }
    }

    fn rel() -> HasOneThrough<Mechanic, Car, CarOwner> {
        HasOneThrough::new("cars", "id", "mechanic_id", "car_id", "car_owners")
    }

    #[test]
    fn query_for_generates_inner_join_and_limit_1() {
        let (sql, params) = rel().query_for(SqlValue::Integer(1)).to_sql();
        assert!(sql.contains("FROM car_owners"), "sql: {sql}");
        assert!(sql.contains("INNER JOIN cars"), "sql: {sql}");
        assert!(sql.contains("cars.mechanic_id = $1"), "sql: {sql}");
        assert!(sql.contains("LIMIT 1"), "sql: {sql}");
        assert_eq!(params[0], SqlValue::Integer(1));
    }

    #[test]
    fn query_for_none_id_still_generates_valid_sql() {
        // Verifies SQL structure holds for any parent ID
        let (sql, _) = rel().query_for(SqlValue::Integer(42)).to_sql();
        assert!(sql.contains("car_owners.car_id"), "join condition: {sql}");
    }

    #[test]
    fn accessors_return_expected_values() {
        let r = rel();
        assert_eq!(r.through_table(), "cars");
        assert_eq!(r.first_key(), "mechanic_id");
        assert_eq!(r.second_key(), "car_id");
        assert_eq!(r.child_table(), "car_owners");
    }
}

// ── PostgreSQL execution ─────────────────────────────────────────────────────

#[cfg(feature = "postgres")]
impl<P, M, C> HasOneThrough<P, M, C>
where
    P: Model,
    M: Model,
    C: Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    pub async fn get(
        &self,
        pool: &sqlx::PgPool,
        parent_id: impl Into<SqlValue>,
    ) -> Result<Option<C>, sqlx::Error> {
        crate::executor::postgres::fetch_optional(pool, self.query_for(parent_id.into())).await
    }
}
