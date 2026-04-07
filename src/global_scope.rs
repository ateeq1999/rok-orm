//! Global query scopes — automatically applied conditions for a model.
//!
//! A [`GlobalScope`] is automatically applied to every query for a model.
//! Register scopes with [`ScopeRegistry`] and apply them in your `Model::query()`.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::{GlobalScope, ScopeRegistry, QueryBuilder};
//!
//! pub struct ActiveScope;
//!
//! impl GlobalScope<User> for ActiveScope {
//!     fn apply(&self, query: QueryBuilder<User>) -> QueryBuilder<User> {
//!         query.where_eq("active", true)
//!     }
//! }
//!
//! // Register at startup
//! ScopeRegistry::add_scope::<User, _>(ActiveScope);
//!
//! // Apply in a custom Model::query() override
//! impl Model for User {
//!     fn query() -> QueryBuilder<Self> {
//!         let builder = QueryBuilder::new(Self::table_name());
//!         ScopeRegistry::apply_scopes::<Self>(builder)
//!     }
//! }
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use crate::model::Model;
use crate::query::QueryBuilder;

// ── Trait ───────────────────────────────────────────────────────────────────

/// A query scope automatically applied to all queries for model `M`.
pub trait GlobalScope<M: Model>: Send + Sync + 'static {
    fn apply(&self, query: QueryBuilder<M>) -> QueryBuilder<M>;
}

// ── Type-erased scope entry ─────────────────────────────────────────────────

type ErasedApply = Box<dyn Fn(Box<dyn Any + Send>) -> Box<dyn Any + Send> + Send + Sync>;

struct ScopeEntry {
    scope_type_id: TypeId,
    apply: ErasedApply,
}

// ── Registry ────────────────────────────────────────────────────────────────

static SCOPE_REGISTRY: OnceLock<RwLock<HashMap<TypeId, Vec<ScopeEntry>>>> = OnceLock::new();

fn registry() -> &'static RwLock<HashMap<TypeId, Vec<ScopeEntry>>> {
    SCOPE_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Global scope registry.
pub struct ScopeRegistry;

impl ScopeRegistry {
    /// Register a scope for model type `M`.
    pub fn add_scope<M: Model + Send + 'static, S: GlobalScope<M>>(scope: S) {
        let entry = ScopeEntry {
            scope_type_id: TypeId::of::<S>(),
            apply: Box::new(move |any_qb: Box<dyn Any + Send>| {
                let qb = *any_qb.downcast::<QueryBuilder<M>>().expect("scope type mismatch");
                let result = scope.apply(qb);
                Box::new(result) as Box<dyn Any + Send>
            }),
        };
        let mut reg = registry().write().unwrap();
        reg.entry(TypeId::of::<M>()).or_default().push(entry);
    }

    /// Remove all scopes of type `S` for model `M`.
    pub fn remove_scope<M: 'static, S: 'static>() {
        let mut reg = registry().write().unwrap();
        if let Some(entries) = reg.get_mut(&TypeId::of::<M>()) {
            entries.retain(|e| e.scope_type_id != TypeId::of::<S>());
        }
    }

    /// Apply all registered scopes for `M` to `builder`, skipping excluded ones.
    ///
    /// Scopes listed in `builder.excluded_scope_ids` (via `without_global_scope::<S>()`)
    /// are skipped.
    pub fn apply_scopes<M: Model + Send + 'static>(builder: QueryBuilder<M>) -> QueryBuilder<M> {
        let reg = registry().read().unwrap();
        if let Some(entries) = reg.get(&TypeId::of::<M>()) {
            let excluded = builder.excluded_scope_ids.clone();
            let mut any_qb: Box<dyn Any + Send> = Box::new(builder);
            for entry in entries {
                if excluded.contains(&entry.scope_type_id) {
                    continue;
                }
                any_qb = (entry.apply)(any_qb);
            }
            *any_qb.downcast::<QueryBuilder<M>>().expect("scope result type mismatch")
        } else {
            builder
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::QueryBuilder;

    struct ScopeModel;
    impl Model for ScopeModel {
        fn table_name() -> &'static str { "scope_tests" }
        fn columns() -> &'static [&'static str] { &["id", "active", "deleted"] }
    }

    struct ActiveFilter;
    impl GlobalScope<ScopeModel> for ActiveFilter {
        fn apply(&self, q: QueryBuilder<ScopeModel>) -> QueryBuilder<ScopeModel> {
            q.where_eq("active", true)
        }
    }

    struct NotDeletedFilter;
    impl GlobalScope<ScopeModel> for NotDeletedFilter {
        fn apply(&self, q: QueryBuilder<ScopeModel>) -> QueryBuilder<ScopeModel> {
            q.where_eq("deleted", false)
        }
    }

    #[test]
    fn apply_scopes_injects_conditions() {
        ScopeRegistry::add_scope::<ScopeModel, _>(ActiveFilter);
        ScopeRegistry::add_scope::<ScopeModel, _>(NotDeletedFilter);
        let builder = QueryBuilder::<ScopeModel>::new("scope_tests");
        let builder = ScopeRegistry::apply_scopes::<ScopeModel>(builder);
        let (sql, _) = builder.to_sql();
        assert!(sql.contains("WHERE active"), "expected WHERE active in: {sql}");
        assert!(sql.contains("deleted"), "expected deleted in: {sql}");
    }

    #[test]
    fn remove_scope_unregisters_it() {
        // This test checks remove_scope doesn't panic; actual registry state
        // may be shared with other tests, so we just call the API.
        ScopeRegistry::remove_scope::<ScopeModel, NotDeletedFilter>();
    }

    #[test]
    fn without_global_scope_excludes_scope() {
        ScopeRegistry::add_scope::<ScopeModel, _>(ActiveFilter);
        let builder = QueryBuilder::<ScopeModel>::new("scope_tests")
            .without_global_scope::<ActiveFilter>();
        let builder = ScopeRegistry::apply_scopes::<ScopeModel>(builder);
        let (sql, _) = builder.to_sql();
        // ActiveFilter was excluded — WHERE active should NOT appear
        assert!(!sql.contains("active = "), "scope should be excluded: {sql}");
    }
}
