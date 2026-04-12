//! Unit tests for the migration system (SQL-level logic only — no live DB required).
//!
//! Integration tests that actually run against a DB live in `tests/integration.rs`.

/// Verify that `MigrationStatus` can be constructed and read back.
#[test]
fn test_migration_status_fields() {
    use super::MigrationStatus;

    let pending = MigrationStatus {
        name: "001_create_users".to_string(),
        batch: None,
        run_at: None,
        is_pending: true,
    };
    assert!(pending.is_pending);
    assert_eq!(pending.name, "001_create_users");
    assert!(pending.batch.is_none());

    let applied = MigrationStatus {
        name: "001_create_users".to_string(),
        batch: Some(1),
        run_at: Some(chrono::Utc::now()),
        is_pending: false,
    };
    assert!(!applied.is_pending);
    assert_eq!(applied.batch, Some(1));
    assert!(applied.run_at.is_some());
}
