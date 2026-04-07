//! [`Model`] trait — implemented automatically by `#[derive(Model)]`.

use std::cell::Cell;
use crate::query::QueryBuilder;

// ── Thread-local flags ──────────────────────────────────────────────────────

thread_local! {
    /// When set, executors skip injecting `created_at`/`updated_at` timestamps.
    static TIMESTAMPS_MUTED: Cell<bool> = Cell::new(false);
    /// When set, executors skip dispatching model hooks / observer events.
    static EVENTS_MUTED: Cell<bool> = Cell::new(false);
}

/// Returns `true` if timestamp injection is currently muted for this thread.
pub fn timestamps_muted() -> bool {
    TIMESTAMPS_MUTED.with(|f| f.get())
}

/// Returns `true` if event/hook dispatch is currently muted for this thread.
pub fn events_muted() -> bool {
    EVENTS_MUTED.with(|f| f.get())
}

pub trait Model: Sized {
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

    /// Run `f` with timestamp injection suppressed for this thread.
    ///
    /// Useful when you want to update a record without touching `updated_at`.
    fn without_timestamps<F, R>(f: F) -> R
    where F: FnOnce() -> R,
    {
        TIMESTAMPS_MUTED.with(|flag| flag.set(true));
        let result = f();
        TIMESTAMPS_MUTED.with(|flag| flag.set(false));
        result
    }

    /// Run `f` with model hook / observer dispatch suppressed for this thread.
    fn without_events<F, R>(f: F) -> R
    where F: FnOnce() -> R,
    {
        EVENTS_MUTED.with(|flag| flag.set(true));
        let result = f();
        EVENTS_MUTED.with(|flag| flag.set(false));
        result
    }

    /// Generate a new unique primary key value, or `None` for auto-increment.
    ///
    /// Override this in models with UUID or ULID primary keys.
    /// The executor will inject the returned value into INSERT data when `Some`.
    fn new_unique_id() -> Option<crate::query::SqlValue> {
        None
    }

    /// Compare two model instances by value equality.
    ///
    /// Requires `Self: PartialEq`. For models with `#[derive(PartialEq)]`,
    /// this compares all fields. For a PK-only comparison, use `is_same_pk()`.
    fn is(&self, other: &Self) -> bool where Self: PartialEq {
        self == other
    }

    fn find(id: impl Into<crate::query::SqlValue>) -> QueryBuilder<Self> {
        Self::query().where_eq(Self::primary_key(), id)
    }

    fn find_where(builder: QueryBuilder<Self>) -> QueryBuilder<Self> {
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::SqlValue;

    struct MockModel;
    impl Model for MockModel {
        fn table_name() -> &'static str { "mocks" }
        fn columns() -> &'static [&'static str] { &["id", "name", "email", "role"] }
        fn fillable() -> &'static [&'static str] { &["name", "email"] }
    }

    struct GuardedModel;
    impl Model for GuardedModel {
        fn table_name() -> &'static str { "guarded" }
        fn columns() -> &'static [&'static str] { &["id", "name", "role", "is_admin"] }
        fn guarded() -> &'static [&'static str] { &["role", "is_admin"] }
    }

    #[test]
    fn fillable_allows_only_listed_cols() {
        let data = [
            ("name", SqlValue::Text("Alice".into())),
            ("email", SqlValue::Text("alice@example.com".into())),
            ("role", SqlValue::Text("admin".into())),
        ];
        let filtered = MockModel::filter_fillable(&data);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|(c, _)| *c == "name"));
        assert!(filtered.iter().any(|(c, _)| *c == "email"));
        assert!(!filtered.iter().any(|(c, _)| *c == "role"));
    }

    #[test]
    fn guarded_blocks_listed_cols() {
        let data = [
            ("name", SqlValue::Text("Alice".into())),
            ("role", SqlValue::Text("admin".into())),
            ("is_admin", SqlValue::Bool(true)),
        ];
        let filtered = GuardedModel::filter_fillable(&data);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].0, "name");
    }

    #[test]
    fn no_filter_when_both_empty() {
        struct Open;
        impl Model for Open {
            fn table_name() -> &'static str { "open" }
            fn columns() -> &'static [&'static str] { &["id", "name"] }
        }
        let data = [("name", SqlValue::Text("x".into())), ("id", SqlValue::Integer(1))];
        let filtered = Open::filter_fillable(&data);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn without_timestamps_sets_and_resets_flag() {
        assert!(!timestamps_muted());
        MockModel::without_timestamps(|| {
            assert!(timestamps_muted());
        });
        assert!(!timestamps_muted());
    }

    #[test]
    fn without_events_sets_and_resets_flag() {
        assert!(!events_muted());
        MockModel::without_events(|| {
            assert!(events_muted());
        });
        assert!(!events_muted());
    }
}
