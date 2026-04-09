---
id: bug-008
type: bug
severity: low
affects: examples/14a-core/src/transactions.rs:53-54
---

# Unescaped `{` `}` in `println!` Format Strings

## Description

Lines 53–54 of `transactions.rs` print literal Rust code that contains `{` and `}`
characters. In a `println!` format string these are interpreted as format
placeholders, causing a compile error.

## Current (broken) Code

```rust
println!("     if condition { tx.rollback().await?; }");
//                          ^ treated as format placeholder — compile error
println!("     else { tx.commit().await?; }");
//                  ^ same issue
```

## Compiler Error

```
error: invalid format string: expected `}`, found `t`
  --> src/transactions.rs:53:35
   |
53 |     println!("     if condition { tx.rollback().await?; }");
   |                                 - ^ expected `}` in format string
```

## Fix

Escape literal braces by doubling them (`{{` and `}}`):

```rust
println!("     if condition {{ tx.rollback().await?; }}");
println!("     else {{ tx.commit().await?; }}");
```
