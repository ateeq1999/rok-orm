//! [`Model`] trait ΓÇö implemented automatically by `#[derive(Model)]`.

use crate::query::QueryBuilder;

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

    /// Generate a new unique primary key value, or `None` for auto-increment.
    ///
    /// Override this in models with UUID or ULID primary keys.
    /// The executor will inject the returned value into INSERT data when `Some`.
    fn new_unique_id() -> Option<crate::query::SqlValue> {
        None
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
}
