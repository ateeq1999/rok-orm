//! Fluent async executor methods for [`QueryBuilder`] (PostgreSQL feature gate).
//!
//! These methods were split out of `builder.rs` to keep that file under 300 lines.

cfg_if::cfg_if! {
    if #[cfg(feature = "postgres")] {
        impl<T> super::builder::QueryBuilder<T> {
            /// Execute the query and return all matching rows.
            pub async fn get(self, pool: &sqlx::PgPool) -> Result<Vec<T>, sqlx::Error>
            where
                T: crate::Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
            {
                crate::executor::postgres::fetch_all(pool, self).await
            }

            /// Execute the query and return the first matching row, if any.
            pub async fn first(self, pool: &sqlx::PgPool) -> Result<Option<T>, sqlx::Error>
            where
                T: crate::Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
            {
                crate::executor::postgres::fetch_optional(pool, self).await
            }

            /// Execute the query and return the count of matching rows.
            pub async fn count(self, pool: &sqlx::PgPool) -> Result<i64, sqlx::Error>
            where
                T: crate::Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
            {
                crate::executor::postgres::count(pool, self).await
            }

            /// Execute the query and return all matching rows (alias for `get`).
            pub async fn get_optional(self, pool: &sqlx::PgPool) -> Result<Vec<T>, sqlx::Error>
            where
                T: crate::Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
            {
                crate::executor::postgres::fetch_all(pool, self).await
            }

            /// Execute the query with offset pagination and return a paginated result.
            pub async fn execute_paginated(
                self,
                pool: &sqlx::PgPool,
                page: i64,
                per_page: i64,
            ) -> Result<crate::pagination::Page<T>, sqlx::Error>
            where
                T: crate::Model + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
            {
                let total = crate::executor::postgres::count(pool, self.clone()).await?;
                let data = crate::executor::postgres::fetch_all(
                    pool,
                    self.paginate(page, per_page),
                )
                .await?;
                Ok(crate::pagination::Page::new(data, total, per_page, page))
            }
        }
    }
}
