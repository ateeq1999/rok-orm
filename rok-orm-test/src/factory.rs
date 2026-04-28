//! Model factories for generating test data (Phase 12.1).
//!
//! # Quick start
//!
//! ```rust,ignore
//! use rok_orm_test::{Factory, FactoryBuilder};
//! use rok_orm::SqlValue;
//!
//! pub struct UserFactory;
//!
//! impl Factory for UserFactory {
//!     type Model = User;
//!
//!     fn definition() -> Vec<(&'static str, SqlValue)> {
//!         vec![
//!             ("name",  "Alice".into()),
//!             ("email", "alice@example.com".into()),
//!             ("role",  "user".into()),
//!         ]
//!     }
//! }
//!
//! // In-memory only — no DB write
//! let fields = UserFactory::new().make();
//!
//! // Persist with SQLite
//! let user = UserFactory::new().create_sqlite(&pool).await?;
//!
//! // Override fields ("state")
//! let admin = UserFactory::new()
//!     .state(&[("role", "admin".into())])
//!     .create_sqlite(&pool)
//!     .await?;
//!
//! // Sequence — unique value per record
//! let users = UserFactory::new()
//!     .sequence("email", |i| format!("user{}@example.com", i).into())
//!     .count(5)
//!     .create_many_sqlite(&pool)
//!     .await?;
//! ```

use std::marker::PhantomData;

use rok_orm::SqlValue;
#[allow(unused_imports)]
use rok_orm::OrmResult;

// ── Factory trait ─────────────────────────────────────────────────────────────

/// Implement this trait to describe default field values for a model.
///
/// Call [`Factory::new`] to obtain a [`FactoryBuilder`] for chaining.
pub trait Factory: Sized {
    /// The model type this factory produces.
    type Model;

    /// Default column → value mapping for the model.
    ///
    /// These values are used unless overridden with [`FactoryBuilder::state`]
    /// or [`FactoryBuilder::sequence`].
    fn definition() -> Vec<(&'static str, SqlValue)>;

    /// Create a new [`FactoryBuilder`] from this factory.
    fn new() -> FactoryBuilder<Self> {
        FactoryBuilder {
            state: Vec::new(),
            sequences: Vec::new(),
            count: 1,
            _factory: PhantomData,
        }
    }
}

// ── Sequence callback type ────────────────────────────────────────────────────

type SeqFn = Box<dyn Fn(usize) -> SqlValue + Send + Sync>;

// ── FactoryBuilder ────────────────────────────────────────────────────────────

/// Builder returned by [`Factory::new`].
///
/// Chain methods to customise each factory run, then call
/// [`make`](FactoryBuilder::make), a dialect `create_*`, or
/// `create_many_*`.
pub struct FactoryBuilder<F: Factory> {
    state: Vec<(&'static str, SqlValue)>,
    sequences: Vec<(&'static str, SeqFn)>,
    /// Number of records [`create_many_*`] will produce.
    count: usize,
    _factory: PhantomData<fn() -> F>,
}

impl<F: Factory> FactoryBuilder<F> {
    // ── Configuration ─────────────────────────────────────────────────────

    /// Override specific columns for every record produced.
    ///
    /// Merges on top of [`Factory::definition`]; later values win.
    pub fn state(mut self, overrides: &[(&'static str, SqlValue)]) -> Self {
        self.state.extend_from_slice(overrides);
        self
    }

    /// Set the number of records produced by `create_many_*`.
    pub fn count(mut self, n: usize) -> Self {
        self.count = n;
        self
    }

    /// Add a sequence: `f` receives the 0-based record index and returns
    /// a unique value for `col`.
    ///
    /// ```rust,ignore
    /// UserFactory::new()
    ///     .sequence("email", |i| format!("u{}@example.com", i).into())
    ///     .count(5)
    ///     .create_many_sqlite(&pool)
    ///     .await?;
    /// ```
    pub fn sequence(
        mut self,
        col: &'static str,
        f: impl Fn(usize) -> SqlValue + Send + Sync + 'static,
    ) -> Self {
        self.sequences.push((col, Box::new(f)));
        self
    }

    // ── In-memory building ────────────────────────────────────────────────

    /// Build the field list for record `index` (0-based), without touching the DB.
    ///
    /// Applies definition → state overrides → sequence overrides.
    pub fn make_one(&self, index: usize) -> Vec<(&'static str, SqlValue)> {
        let mut fields = F::definition();

        for (col, val) in &self.state {
            if let Some(entry) = fields.iter_mut().find(|(c, _)| c == col) {
                *entry = (col, val.clone());
            } else {
                fields.push((col, val.clone()));
            }
        }

        for (col, seq_fn) in &self.sequences {
            let val = seq_fn(index);
            if let Some(entry) = fields.iter_mut().find(|(c, _)| c == col) {
                *entry = (col, val);
            } else {
                fields.push((col, val));
            }
        }

        fields
    }

    /// Build field lists for [`count`](FactoryBuilder::count) records without
    /// writing to the DB.
    pub fn make(&self) -> Vec<Vec<(&'static str, SqlValue)>> {
        (0..self.count).map(|i| self.make_one(i)).collect()
    }
}

// ── SQLite create methods ─────────────────────────────────────────────────────

#[cfg(feature = "sqlite")]
impl<F, M> FactoryBuilder<F>
where
    F: Factory<Model = M>,
    M: rok_orm::SqliteModel
        + Send
        + Unpin
        + 'static
        + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    /// Persist one record to a SQLite database and return the hydrated model.
    pub async fn create_sqlite(self, pool: &sqlx::SqlitePool) -> OrmResult<M> {
        let fields = self.make_one(0);
        M::create_returning(pool, &fields).await.map_err(rok_orm::OrmError::from)
    }

    /// Persist [`count`](FactoryBuilder::count) records to a SQLite database.
    pub async fn create_many_sqlite(self, pool: &sqlx::SqlitePool) -> OrmResult<Vec<M>> {
        let mut results = Vec::with_capacity(self.count);
        for i in 0..self.count {
            let fields = self.make_one(i);
            results.push(M::create_returning(pool, &fields).await.map_err(rok_orm::OrmError::from)?);
        }
        Ok(results)
    }
}

// ── Postgres create methods ───────────────────────────────────────────────────

#[cfg(feature = "postgres")]
impl<F, M> FactoryBuilder<F>
where
    F: Factory<Model = M>,
    M: rok_orm::PgModel
        + Send
        + Unpin
        + 'static
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>,
{
    /// Persist one record to a Postgres database and return the hydrated model.
    pub async fn create_pg(self, pool: &sqlx::PgPool) -> OrmResult<M> {
        let fields = self.make_one(0);
        M::create_returning(pool, &fields).await.map_err(rok_orm::OrmError::from)
    }

    /// Persist [`count`](FactoryBuilder::count) records to a Postgres database.
    pub async fn create_many_pg(self, pool: &sqlx::PgPool) -> OrmResult<Vec<M>> {
        let mut results = Vec::with_capacity(self.count);
        for i in 0..self.count {
            let fields = self.make_one(i);
            results.push(M::create_returning(pool, &fields).await.map_err(rok_orm::OrmError::from)?);
        }
        Ok(results)
    }
}

// ── TestDb create convenience methods ────────────────────────────────────────
//
// These let users write `factory.create(db.pool())` with the `TestDb` pool ref.

#[cfg(feature = "sqlite")]
impl<F, M> FactoryBuilder<F>
where
    F: Factory<Model = M>,
    M: rok_orm::SqliteModel
        + Send
        + Unpin
        + 'static
        + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    /// Create one record using a `TestDb` pool (SQLite variant).
    pub async fn create(self, pool: &crate::TestDbPool<'_>) -> OrmResult<M> {
        match pool {
            crate::TestDbPool::Sqlite(p) => {
                M::create_returning(p, &self.make_one(0)).await.map_err(rok_orm::OrmError::from)
            }
            #[allow(unreachable_patterns)]
            _ => Err(rok_orm::OrmError::Other(
                "factory::create called with non-SQLite pool".into(),
            )),
        }
    }

    /// Create many records using a `TestDb` pool (SQLite variant).
    pub async fn create_many(self, pool: &crate::TestDbPool<'_>) -> OrmResult<Vec<M>> {
        match pool {
            crate::TestDbPool::Sqlite(p) => {
                let mut results = Vec::with_capacity(self.count);
                for i in 0..self.count {
                    results.push(M::create_returning(p, &self.make_one(i)).await.map_err(rok_orm::OrmError::from)?);
                }
                Ok(results)
            }
            #[allow(unreachable_patterns)]
            _ => Err(rok_orm::OrmError::Other(
                "factory::create_many called with non-SQLite pool".into(),
            )),
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeModel;

    struct FakeFactory;

    impl Factory for FakeFactory {
        type Model = FakeModel;

        fn definition() -> Vec<(&'static str, SqlValue)> {
            vec![
                ("name",  SqlValue::Text("Default".into())),
                ("email", SqlValue::Text("default@example.com".into())),
            ]
        }
    }

    #[test]
    fn make_one_returns_definition() {
        let fields = FakeFactory::new().make_one(0);
        assert_eq!(fields.len(), 2);
        assert!(fields.iter().any(|(c, _)| *c == "name"));
        assert!(fields.iter().any(|(c, _)| *c == "email"));
    }

    #[test]
    fn state_overrides_existing_column() {
        let fields = FakeFactory::new()
            .state(&[("name", SqlValue::Text("Bob".into()))])
            .make_one(0);
        let name = fields.iter().find(|(c, _)| *c == "name").map(|(_, v)| v);
        assert_eq!(name, Some(&SqlValue::Text("Bob".into())));
        assert_eq!(fields.len(), 2); // no extra column added
    }

    #[test]
    fn state_can_add_new_column() {
        let fields = FakeFactory::new()
            .state(&[("role", SqlValue::Text("admin".into()))])
            .make_one(0);
        let role = fields.iter().find(|(c, _)| *c == "role").map(|(_, v)| v);
        assert_eq!(role, Some(&SqlValue::Text("admin".into())));
        assert_eq!(fields.len(), 3); // definition(2) + new role
    }

    #[test]
    fn sequence_varies_per_index() {
        let builder = FakeFactory::new()
            .count(3)
            .sequence("email", |i| SqlValue::Text(format!("u{}@test.com", i)));
        let all = builder.make();
        assert_eq!(all.len(), 3);
        for (i, fields) in all.iter().enumerate() {
            let email = fields.iter().find(|(c, _)| *c == "email").map(|(_, v)| v);
            assert_eq!(email, Some(&SqlValue::Text(format!("u{}@test.com", i))));
        }
    }

    #[test]
    fn make_returns_count_records() {
        let records = FakeFactory::new().count(5).make();
        assert_eq!(records.len(), 5);
    }

    #[test]
    fn sequence_overrides_definition_column() {
        let builder = FakeFactory::new()
            .sequence("name", |i| SqlValue::Text(format!("Name{}", i)));
        let fields = builder.make_one(7);
        let name = fields.iter().find(|(c, _)| *c == "name").map(|(_, v)| v);
        assert_eq!(name, Some(&SqlValue::Text("Name7".into())));
        assert_eq!(fields.len(), 2); // no extra column added
    }
}
