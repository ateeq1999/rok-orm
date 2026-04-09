---
id: bug-001
type: bug
severity: critical
affects: all example files except basic_model.rs
source: examples/14a-core/src/
---

# Missing PgModel / PgModelExt Trait Imports

## Description

Every example file that calls CRUD or aggregate methods (`create`, `find_by_pk`,
`update_by_pk`, `delete_by_pk`, `all`, `count`, `sum`, `paginate`, etc.) fails to
compile because these methods come from the `PgModel` and `PgModelExt` traits, which
must be explicitly imported before they are usable.

## Affected Files

| File | Missing import |
|---|---|
| `crud_operations.rs` | `PgModel`, `PgModelExt` |
| `soft_deletes.rs` | `PgModel` |
| `timestamps.rs` | `PgModel` |
| `pagination.rs` | `PgModel`, `PgModelExt` |
| `aggregations.rs` | `PgModel`, `PgModelExt` |
| `transactions.rs` | `PgModel` |
| `query_scopes.rs` | `PgModel` |
| `query_logging.rs` | `PgModel` |
| `relationships.rs` | `PgModel` |

## Compiler Error (representative)

```
error[E0599]: no function or associated item named `create` found for struct
              `crud_operations::User` in the current scope
  --> src/crud_operations.rs:23:11
   |
   = help: the following traits which provide `create` are implemented but not
     in scope; perhaps you want to import one of them:
   |
 5 + use rok_orm::PgModel;
```

## Fix

Add the following to the top of each affected file:

```rust
use rok_orm::{PgModel, PgModelExt};
```

`PgModel` provides: `create`, `create_returning`, `find_by_pk`, `find_or_404`,
`update_by_pk`, `delete_by_pk`, `all`, `count`, `restore`, `force_delete`.

`PgModelExt` provides: `paginate`, `sum`, `avg`, `min`, `max`, `upsert`,
`upsert_returning`, `exists`, `pluck`, `chunk`, `cursor_paginate`.
