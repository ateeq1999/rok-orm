//! Query scopes for reusable query constraints.
//!
//! Scopes are a pattern for encapsulating reusable query logic. They are plain
//! functions (or methods) that return a `QueryBuilder`.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::{Model, QueryBuilder};
//!
//! #[derive(Model)]
//! pub struct User {
//!     pub id: i64,
//!     pub name: String,
//!     pub email: String,
//!     pub role: String,
//!     pub active: bool,
//! }
//!
//! // Define scopes as impl methods on the model
//! impl User {
//!     /// Active users only
//!     pub fn active() -> QueryBuilder<User> {
//!         User::query().filter("active", true)
//!     }
//!
//!     /// Users with a specific role
//!     pub fn by_role(role: &str) -> QueryBuilder<User> {
//!         User::query().filter("role", role)
//!     }
//!
//!     /// Search by name or email
//!     pub fn search(term: &str) -> QueryBuilder<User> {
//!         User::query()
//!             .or_where_like("name", &format!("%{}%", term))
//!             .or_where_like("email", &format!("%{}%", term))
//!     }
//!
//!     /// Combine scopes
//!     pub fn active_admins() -> QueryBuilder<User> {
//!         Self::active().filter("role", "admin")
//!     }
//! }
//!
//! // Usage
//! let admins = User::active_admins().get(&pool).await?;
//! let users = User::search("john").get(&pool).await?;
//! ```
//!
//! # Global Scopes
//!
//! Global scopes are automatically applied to all queries for a model.
//!
//! ```rust,ignore
//! // In your model definition
//! impl Model for User {
//!     fn query() -> QueryBuilder<Self> {
//!         // Auto-exclude soft-deleted records
//!         let builder = QueryBuilder::new(Self::table_name());
//!         if let Some(col) = Self::soft_delete_column() {
//!             builder.with_soft_delete(col)
//!         } else {
//!             builder
//!         }
//!     }
//! }
//! ```
//!
//! # Scope Traits
//!
//! For more reusable scopes, you can define a trait:
//!
//! ```rust,ignore
//! pub trait ActiveScope<T: Model> {
//!     fn active() -> QueryBuilder<T>;
//! }
//!
//! impl<T: Model> ActiveScope<T> for T {
//!     fn active() -> QueryBuilder<T> {
//!         T::query().filter("active", true)
//!     }
//! }
//! ```

use crate::{Model, QueryBuilder};

pub trait Scope<T: Model> {
    fn apply(builder: QueryBuilder<T>) -> QueryBuilder<T>;
}

pub trait ScopeMut<T: Model> {
    fn apply(&self, builder: QueryBuilder<T>) -> QueryBuilder<T>;
}

pub struct AndScope<S1, S2, T>
where
    S1: Scope<T>,
    S2: Scope<T>,
    T: Model,
{
    _marker: std::marker::PhantomData<(S1, S2, T)>,
}

impl<S1, S2, T> Scope<T> for AndScope<S1, S2, T>
where
    S1: Scope<T>,
    S2: Scope<T>,
    T: Model,
{
    fn apply(builder: QueryBuilder<T>) -> QueryBuilder<T> {
        let builder = S1::apply(builder);
        S2::apply(builder)
    }
}

pub struct OrScope<S1, S2, T>
where
    S1: Scope<T>,
    S2: Scope<T>,
    T: Model,
{
    _marker: std::marker::PhantomData<(S1, S2, T)>,
}

impl<S1, S2, T> Scope<T> for OrScope<S1, S2, T>
where
    S1: Scope<T>,
    S2: Scope<T>,
    T: Model,
{
    fn apply(builder: QueryBuilder<T>) -> QueryBuilder<T> {
        builder
    }
}
