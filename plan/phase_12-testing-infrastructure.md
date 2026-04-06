# Phase 12: Testing Infrastructure

> **Target version:** v0.6.0
> **Status:** 🔜 Planned
> **Crate:** `rok-orm-test` (or feature flag `test-utils`)

---

## Goal

Writing tests for database-driven code should be as easy as writing the production code. Zero manual transaction setup, realistic fake data from a fluent factory API, and expressive assertion helpers.

---

## 12.1 Model Factories with Faker

### API

```rust
use rok_orm_test::{Factory, FactoryExt};
use fake::{Fake, Faker};
use fake::faker::{name::en::Name, internet::en::SafeEmail};

pub struct UserFactory;

impl Factory for UserFactory {
    type Model = User;

    fn definition() -> Vec<(&'static str, SqlValue)> {
        vec![
            ("name",   Name().fake::<String>().into()),
            ("email",  SafeEmail().fake::<String>().into()),
            ("active", true.into()),
            ("role",   "user".into()),
        ]
    }
}

// Create one
let user = UserFactory::new()
    .create(&pool)
    .await?;

// Override fields (state)
let admin = UserFactory::new()
    .state(&[("role", "admin".into()), ("active", true.into())])
    .create(&pool)
    .await?;

// Create many
let users = UserFactory::new()
    .count(10)
    .create_many(&pool)
    .await?;

// Make (in-memory only, no DB write)
let fields = UserFactory::new().make();

// Sequences — unique values per item
let users = UserFactory::new()
    .sequence("email", |i| format!("user{}@example.com", i).into())
    .count(5)
    .create_many(&pool)
    .await?;

// After-create hook (e.g., create related records)
let user = UserFactory::new()
    .after_create(|user, pool| async move {
        ProfileFactory::new()
            .state(&[("user_id", user.id.into())])
            .create(pool)
            .await?;
        Ok(user)
    })
    .create(&pool)
    .await?;
```

### New Crate Structure

```
rok-orm-test/
├── Cargo.toml
└── src/
    ├── lib.rs          re-exports: Factory, FactoryBuilder, TestDb, assert
    ├── factory.rs      Factory trait, FactoryBuilder<M>
    └── assert.rs       assertion helpers
```

### Tasks

- [ ] Create `rok-orm-test` as a workspace member (or `feature = "test-utils"` in main crate)
- [ ] Define `Factory` trait: `fn definition() -> Vec<(&'static str, SqlValue)>`
- [ ] Define `FactoryBuilder<M>` with: `state()`, `count()`, `sequence()`, `after_create()`
- [ ] Add `Factory::new() -> FactoryBuilder<Self>` default method
- [ ] Implement `FactoryBuilder::make() -> Vec<(&'static str, SqlValue)>` — applies state + sequences
- [ ] Implement `FactoryBuilder::create(pool) -> OrmResult<M>` — calls `M::create_returning`
- [ ] Implement `FactoryBuilder::create_many(pool) -> OrmResult<Vec<M>>`
- [ ] Add `fake` crate as optional dependency (behind `test-utils` feature)
- [ ] Tests: factory definition, state override, sequence, after_create hook

---

## 12.2 Database Transaction Per Test

### API

```rust
use rok_orm_test::prelude::*;

// PostgreSQL — transaction auto-rolls back after test
#[rok_orm_test::test(postgres)]
async fn test_user_creation(db: &TestDb) -> OrmResult<()> {
    let user = UserFactory::new().create(db.pool()).await?;

    assert_eq!(user.role, "user");
    assert!(user.active);

    // All DB changes here are ROLLED BACK when test ends (pass or fail)
    Ok(())
}

// SQLite in-memory — fresh DB per test
#[rok_orm_test::test(sqlite)]
async fn test_post_creation(db: &TestDb) -> OrmResult<()> {
    let post = PostFactory::new().create(db.pool()).await?;
    assert!(!post.title.is_empty());
    Ok(())
}

// With custom setup (run migrations before test)
#[rok_orm_test::test(postgres, migrate)]
async fn test_with_migrations(db: &TestDb) -> OrmResult<()> {
    // migrations have been run on db.pool()
    let user = UserFactory::new().create(db.pool()).await?;
    Ok(())
}
```

### `TestDb` Struct

```rust
pub struct TestDb {
    pool: Pool,                          // connection pool (or single connection in transaction)
    _transaction: Option<Transaction>,   // held open, rolled back on drop
}

impl TestDb {
    pub fn pool(&self) -> &Pool { &self.pool }
}
```

### Tasks

- [ ] Define `TestDb` struct with `pool()` accessor
- [ ] For PostgreSQL: begin a transaction in `TestDb::new()`, pass the transaction as pool
- [ ] For SQLite: create an in-memory SQLite pool, run migrations if `migrate` flag set
- [ ] Define `#[rok_orm_test::test(dialect)]` proc macro attribute:
  - Creates `TestDb` before the test
  - Wraps the test body in an async block
  - Rollback / drop `TestDb` after completion (pass or panic)
- [ ] Support `migrate` flag on the attribute — runs `Migrator` before returning `TestDb`
- [ ] Tests: verify rows inserted in test don't persist between tests (rollback works)

---

## 12.3 Assertion Helpers

### API

```rust
use rok_orm_test::assert_db;

// Model-level assertions
assert_db::model_exists::<User>(&pool, 1).await?;
assert_db::model_missing::<User>(&pool, 999).await?;
assert_db::model_count::<User>(&pool, 5).await?;
assert_db::model_count_where::<User>(
    &pool,
    User::query().filter("active", true),
    3,
).await?;

// Raw table assertions
assert_db::database_has("users", &[
    ("email", "alice@example.com".into()),
    ("active", true.into()),
], &pool).await?;

assert_db::database_missing("users", &[
    ("email", "deleted@example.com".into()),
], &pool).await?;

assert_db::table_count("users", 10, &pool).await?;
assert_db::table_empty("sessions", &pool).await?;
```

**On failure:** descriptive panic message with expected vs actual, table name, and conditions.

### Tasks

- [ ] Add `assert_db` module to `rok-orm-test`
- [ ] Implement `model_exists::<M>(pool, id)` — `M::find_by_pk` + assert Some
- [ ] Implement `model_missing::<M>(pool, id)` — `M::find_by_pk` + assert None
- [ ] Implement `model_count::<M>(pool, expected)` — `M::count` + assert eq
- [ ] Implement `model_count_where::<M>(pool, builder, expected)` — `M::count_where` + assert eq
- [ ] Implement `database_has(table, conditions, pool)` — `SELECT COUNT(*) WHERE ...` + assert > 0
- [ ] Implement `database_missing(table, conditions, pool)` — same + assert == 0
- [ ] Implement `table_count(table, n, pool)` — raw COUNT + assert eq
- [ ] Implement `table_empty(table, pool)` — raw COUNT + assert == 0
- [ ] Panic messages include: assertion name, table, conditions, expected, actual
- [ ] Tests: each assertion passes when correct, panics with descriptive message when wrong

---

## Acceptance Criteria for Phase 12

- [ ] All 3 sub-sections implemented
- [ ] Factory `fake` data changes each run (random)
- [ ] Transaction rollback verified: rows do not persist between tests
- [ ] All assertions produce clear failure messages
- [ ] Works with PG and SQLite
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Phase file tasks all checked off
