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
