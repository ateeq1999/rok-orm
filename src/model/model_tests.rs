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

#[test]
fn first_or_new_merges_conditions_and_data() {
    let conditions = [("email", SqlValue::Text("a@b.com".into()))];
    let data = [
        ("name", SqlValue::Text("Alice".into())),
        ("email", SqlValue::Text("ignored@b.com".into())), // duplicate key
    ];
    let merged = MockModel::first_or_new(&conditions, &data);
    assert_eq!(merged.len(), 2); // email + name (no duplicate)
    assert!(merged.iter().any(|(k, _)| *k == "email"));
    assert!(merged.iter().any(|(k, _)| *k == "name"));
}

#[derive(Clone)]
struct CloneModel { id: i64, name: String }
impl Model for CloneModel {
    fn table_name() -> &'static str { "clone_models" }
    fn columns() -> &'static [&'static str] { &["id", "name"] }
}
// Simulate what the macro generates for pk_reset
impl CloneModel {
    fn pk_reset(&mut self) { self.id = Default::default(); }
}

#[test]
fn replicate_returns_clone() {
    let m = CloneModel { id: 5, name: "Alice".into() };
    let copy = m.replicate();
    assert_eq!(copy.id, 5); // default replicate just clones
    assert_eq!(copy.name, "Alice");
}

#[test]
fn pk_reset_zeroes_primary_key() {
    let m = CloneModel { id: 5, name: "Alice".into() };
    let mut copy = m.replicate();
    copy.pk_reset();
    assert_eq!(copy.id, 0); // reset to Default
    assert_eq!(copy.name, "Alice"); // non-PK preserved
}

#[test]
fn first_or_new_conditions_take_priority_over_defaults() {
    let conditions = [("email", SqlValue::Text("a@b.com".into()))];
    let data = [("email", SqlValue::Text("other@b.com".into())), ("name", SqlValue::Text("Alice".into()))];
    let merged = MockModel::first_or_new(&conditions, &data);
    let email_val: Vec<_> = merged.iter().filter(|(k, _)| *k == "email").collect();
    assert_eq!(email_val.len(), 1);
    assert_eq!(email_val[0].1, SqlValue::Text("a@b.com".into()));
}

#[test]
fn first_or_new_includes_all_defaults_when_no_overlap() {
    let conditions = [("email", SqlValue::Text("a@b.com".into()))];
    let data = [("name", SqlValue::Text("Bob".into())), ("role", SqlValue::Text("user".into()))];
    let merged = MockModel::first_or_new(&conditions, &data);
    assert_eq!(merged.len(), 3);
}

#[test]
fn first_or_new_empty_defaults() {
    let conditions = [("id", SqlValue::Integer(1))];
    let merged = MockModel::first_or_new(&conditions, &[]);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].0, "id");
}

#[test]
fn is_compares_by_value() {
    struct Cmp { val: i64 }
    impl Model for Cmp {
        fn table_name() -> &'static str { "cmp" }
        fn columns() -> &'static [&'static str] { &["val"] }
    }
    impl PartialEq for Cmp {
        fn eq(&self, other: &Self) -> bool { self.val == other.val }
    }
    let a = Cmp { val: 1 };
    let b = Cmp { val: 1 };
    let c = Cmp { val: 2 };
    assert!(a.is(&b));
    assert!(!a.is(&c));
}
