//! Lazy-loading helpers that execute relation queries against the database.

#[cfg(feature = "postgres")]
#[allow(dead_code)]
pub mod pg {
    use sqlx::PgPool;

    use crate::model::Model;
    use crate::executor::postgres;
    use crate::relations::traits::Relation;
    use crate::relations::{BelongsTo, HasMany, HasOne};

    pub async fn load_has_many<P, C>(
        pool: &PgPool,
        relation: &HasMany<P, C>,
        parent_ids: &[i64],
    ) -> Result<Vec<C>, sqlx::Error>
    where
        P: Model,
        C: Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let mut builder = relation.query_for(crate::query::SqlValue::Null);
        for id in parent_ids {
            builder = builder.or_where_eq(relation.foreign_key(), *id);
        }
        postgres::fetch_all(pool, builder).await
    }

    pub async fn load_has_one<P, C>(
        pool: &PgPool,
        relation: &HasOne<P, C>,
        parent_id: i64,
    ) -> Result<Option<C>, sqlx::Error>
    where
        P: Model,
        C: Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let builder = relation.query_for(crate::query::SqlValue::Integer(parent_id));
        postgres::fetch_optional(pool, builder).await
    }

    pub async fn load_belongs_to<P, C>(
        pool: &PgPool,
        relation: &BelongsTo<P, C>,
        parent: &P,
    ) -> Result<Option<C>, sqlx::Error>
    where
        P: Model,
        C: Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let fk_value = relation.foreign_key_value(parent);
        let builder = relation.query_for(fk_value);
        postgres::fetch_optional(pool, builder).await
    }
}
