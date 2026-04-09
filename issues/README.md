# Bug Fix Progress

This document tracks the status of bugs found in the example code.

## Overall Status

- **Total Bugs**: 12
- **Fixed**: 3
- **In Progress**: 0
- **Pending**: 9

## Bug Status Detail

### Fixed ✓

| Bug ID | Title | Status |
|--------|-------|--------|
| bug-001 | Missing PgModel/PgModelExt trait imports | **FIXED** |
| bug-004 | find_by_pk returns Option not T | **FIXED** |
| bug-006 | Pagination API used incorrectly | **FIXED** |

### In Progress 🔄

None currently.

### Pending ⏳

| Bug ID | Title | Priority | Status |
|--------|-------|----------|--------|
| bug-002 | Wrong Relations derive syntax | critical | ⏳ Pending |
| bug-003 | QueryBuilder missing fluent executor methods | critical | ⏳ Pending |
| bug-005 | upsert() wrong argument signature | high | ⏳ Pending |
| bug-007 | Eager loading with posts field missing | high | ⏳ Pending |
| bug-008 | Transactions unescaped format braces | low | ⏳ Pending |
| bug-009 | Query scope chaining on wrong type | medium | ⏳ Pending |
| bug-010 | Aggregate methods wrong arg order and return type | high | ⏳ Pending |
| bug-011 | Soft delete wrong static method names | high | ⏳ Pending |
| bug-012 | Format string in SqlValue context | medium | ⏳ Pending |

## Next Phase

The next phase of fixes focuses on **QueryBuilder executor methods** (bug-003) and **Relations derive syntax** (bug-002).

Once these are resolved, the following examples can be uncommented:
- `relationships`
- `soft_deletes`
- `timestamps`
- `transactions`
- `query_scopes`
- `query_logging`

## How to Help

1. Pick a pending bug from the list above
2. Read the bug description in `issues/bug-XXX-*.md`
3. Implement the fix in the relevant example file
4. Test with `cargo run <example-name>`
5. Update this README with status change

## Running Examples

From `examples/14a-core`:

```bash
# Run all currently working examples
cargo run all

# Run individual examples
cargo run basic_model
cargo run crud
cargo run pagination
cargo run aggregations
```

## Progress Timeline

- **Phase 1** (COMPLETE): Fixed trait imports (bug-001), Option handling (bug-004), Pagination API (bug-006)
- **Phase 2** (NEXT): Fix QueryBuilder executor methods (bug-003) and Relations syntax (bug-002)
- **Phase 3**: Remaining bug fixes and full example suite enablement