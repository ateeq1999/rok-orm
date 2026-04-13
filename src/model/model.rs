//! [`Model`] trait — implemented automatically by `#[derive(Model)]`.

use std::cell::Cell;
use crate::query::QueryBuilder;

// ── Thread-local flags ──────────────────────────────────────────────────────

thread_local! {
    /// When set, executors skip injecting `created_at`/`updated_at` timestamps.
    static TIMESTAMPS_MUTED: Cell<bool> = const { Cell::new(false) };
    /// When set, executors skip dispatching model hooks / observer events.
    static EVENTS_MUTED: Cell<bool> = const { Cell::new(false) };
}

/// Returns `true` if timestamp injection is currently muted for this thread.
pub fn timestamps_muted() -> bool {
    TIMESTAMPS_MUTED.with(|f| f.get())
}

/// Returns `true` if event/hook dispatch is currently muted for this thread.
pub fn events_muted() -> bool {
    EVENTS_MUTED.with(|f| f.get())
}

#[allow(async_fn_in_trait)] pub trait Model: Sized {
    fn table_name() -> &'static str;

    fn primary_key() -> &'static str {
        "id"
    }

    fn columns() -> &'static [&'static str];

    fn soft_delete_column() -> Option<&'static str> {
        None
    }

    fn timestamps_enabled() -> bool {
        false
    }

    fn created_at_column() -> Option<&'static str> {
        None
    }

    fn updated_at_column() -> Option<&'static str> {
        None
    }

    /// Columns allowed for mass assignment. Empty slice means all columns are allowed.
    fn fillable() -> &'static [&'static str] {
        &[]
    }

    /// Columns blocked from mass assignment. Empty slice means nothing is guarded.
    fn guarded() -> &'static [&'static str] {
        &[]
    }

    /// Relation names whose parent's `updated_at` should be touched after writes.
    fn touches() -> &'static [&'static str] {
        &[]
    }

    /// Named connection key for this model. Default is `"default"`.
    ///
    /// Override with `#[model(connection = "audit_db")]` to use a named pool
    /// registered in [`ConnectionRegistry`].
    fn connection() -> &'static str {
        "default"
    }

    /// Filter `data` through the fillable/guarded lists.
    ///
    /// If `fillable()` is non-empty, only listed columns pass through.
    /// Otherwise, `guarded()` columns are blocked.
    fn filter_fillable<'a>(data: &'a [(&'a str, crate::query::SqlValue)]) -> Vec<(&'a str, crate::query::SqlValue)> {
        let fillable = Self::fillable();
        let guarded = Self::guarded();
        if !fillable.is_empty() {
            data.iter()
                .filter(|(col, _)| fillable.contains(col))
                .cloned()
                .collect()
        } else if !guarded.is_empty() {
            data.iter()
                .filter(|(col, _)| !guarded.contains(col))
                .cloned()
                .collect()
        } else {
            data.to_vec()
        }
    }

    fn query() -> QueryBuilder<Self> {
        let builder = QueryBuilder::new(Self::table_name());
        if let Some(col) = Self::soft_delete_column() {
            builder.with_soft_delete(col)
        } else {
            builder
        }
    }

    /// Like `query()` but with all registered global scopes applied.
    ///
    /// Used automatically by `all()`, `get()`, `first()`, and `count()` on PgModel/SqliteModel/MyModel.
    fn scoped_query() -> QueryBuilder<Self>
    where Self: Send + 'static,
    {
        crate::global_scope::ScopeRegistry::apply_scopes::<Self>(Self::query())
    }

    /// Register a global scope for this model. Convenience for `ScopeRegistry::add_scope`.
    fn add_global_scope<S>(scope: S)
    where Self: Send + 'static, S: crate::global_scope::GlobalScope<Self>,
    {
        crate::global_scope::ScopeRegistry::add_scope::<Self, S>(scope);
    }

    /// Register an observer for this model. Convenience for `ObserverRegistry::observe`.
    fn observe<O>(observer: O)
    where Self: 'static, O: crate::observer::ModelObserver<Self>,
    {
        crate::observer::ObserverRegistry::observe::<Self, O>(observer);
    }

    /// Run `f` with timestamp injection suppressed.
    fn without_timestamps<F, R>(f: F) -> R where F: FnOnce() -> R {
        TIMESTAMPS_MUTED.with(|flag| flag.set(true));
        let result = f();
        TIMESTAMPS_MUTED.with(|flag| flag.set(false));
        result
    }

    /// Async variant: flag stays set for the full duration of the future.
    async fn without_timestamps_async<F, Fut, R>(f: F) -> R
    where F: FnOnce() -> Fut, Fut: std::future::Future<Output = R> {
        TIMESTAMPS_MUTED.with(|flag| flag.set(true));
        let result = f().await;
        TIMESTAMPS_MUTED.with(|flag| flag.set(false));
        result
    }

    /// Run `f` with model observer dispatch suppressed.
    fn without_events<F, R>(f: F) -> R where F: FnOnce() -> R {
        EVENTS_MUTED.with(|flag| flag.set(true));
        let result = f();
        EVENTS_MUTED.with(|flag| flag.set(false));
        result
    }

    /// Async variant: flag stays set for the full duration of the future.
    async fn without_events_async<F, Fut, R>(f: F) -> R
    where F: FnOnce() -> Fut, Fut: std::future::Future<Output = R> {
        EVENTS_MUTED.with(|flag| flag.set(true));
        let result = f().await;
        EVENTS_MUTED.with(|flag| flag.set(false));
        result
    }

    /// Generate a new unique primary key, or `None` for auto-increment.
    /// Override for UUID/ULID primary keys; the executor injects the value on INSERT.
    fn new_unique_id() -> Option<crate::query::SqlValue> { None }

    /// Compare two model instances by value equality (requires `Self: PartialEq`).
    fn is(&self, other: &Self) -> bool where Self: PartialEq { self == other }

    /// Merge `conditions` + `data` into a field list for building a new unsaved record.
    /// Duplicate keys from `data` that already appear in `conditions` are ignored.
    fn first_or_new<'a>(
        conditions: &[(&'a str, crate::query::SqlValue)],
        data: &[(&'a str, crate::query::SqlValue)],
    ) -> Vec<(&'a str, crate::query::SqlValue)> {
        let mut merged: Vec<(&str, crate::query::SqlValue)> = conditions.to_vec();
        for row in data {
            if !merged.iter().any(|(c, _)| c == &row.0) {
                merged.push(row.clone());
            }
        }
        merged
    }

    /// Serialize non-PK columns for INSERT/UPDATE. Generated by `#[derive(Model)]`.
    fn to_fields(&self) -> Vec<(&'static str, crate::query::SqlValue)> { vec![] }

    // ── Phase 11: casting & serialization ────────────────────────────────────

    /// Columns excluded from `serialize()` output. Override via `#[model(hidden = [...])]`.
    fn hidden() -> &'static [&'static str] { &[] }

    /// Column whitelist for `serialize()`. Empty means all non-hidden. Override via `#[model(visible = [...])]`.
    fn visible() -> &'static [&'static str] { &[] }

    /// Computed field names appended to `serialize()`. Override via `#[model(appends = [...])]`.
    fn appends() -> &'static [&'static str] { &[] }

    /// Apply post-load decoding for cast fields.
    ///
    /// Generated by `#[derive(Model)]` when any field has a `cast` attribute that
    /// requires decoding (e.g. `cast = "csv"`, `cast = "encrypted"`).
    /// Call this after `sqlx::FromRow` if you use such casts.
    fn post_process(&mut self) {}

    /// Clone this record. To also reset the primary key, use `pk_reset()` on the clone
    /// (generated by `#[derive(Model)]`):
    /// ```ignore
    /// let mut copy = record.replicate();
    /// copy.pk_reset();
    /// ```
    fn replicate(&self) -> Self where Self: Clone { self.clone() }

    fn find(id: impl Into<crate::query::SqlValue>) -> QueryBuilder<Self> {
        Self::query().where_eq(Self::primary_key(), id)
    }

    fn find_where(builder: QueryBuilder<Self>) -> QueryBuilder<Self> { builder }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
