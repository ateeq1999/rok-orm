//! Unit tests for the schema builder — verifies DDL SQL generation per dialect.

use super::{Schema, SchemaDialect};
use super::column::ForeignAction;

// ── CREATE TABLE ─────────────────────────────────────────────────────────────

#[test]
fn test_create_table_postgres_basic() {
    let op = Schema::create_with_dialect("users", SchemaDialect::Postgres, |t| {
        t.id();
        t.string("name", 255);
        t.string("email", 255).unique();
        t.boolean("active").default("true");
        t.timestamps();
    });
    let sql = op.to_sql();
    assert!(sql.contains("CREATE TABLE users"), "missing CREATE TABLE: {sql}");
    assert!(sql.contains("id BIGSERIAL PRIMARY KEY"), "missing PK: {sql}");
    assert!(sql.contains("name VARCHAR(255) NOT NULL"), "missing name: {sql}");
    assert!(sql.contains("email VARCHAR(255) NOT NULL UNIQUE"), "missing unique email: {sql}");
    assert!(sql.contains("active BOOLEAN NOT NULL DEFAULT true"), "missing default: {sql}");
    assert!(sql.contains("created_at TIMESTAMPTZ NULL"), "missing created_at: {sql}");
    assert!(sql.contains("updated_at TIMESTAMPTZ NULL"), "missing updated_at: {sql}");
}

#[test]
fn test_create_table_sqlite_basic() {
    let op = Schema::create_with_dialect("posts", SchemaDialect::Sqlite, |t| {
        t.id();
        t.string("title", 255);
        t.boolean("published").default("0");
        t.soft_deletes();
    });
    let sql = op.to_sql();
    assert!(sql.contains("CREATE TABLE posts"), "missing CREATE TABLE: {sql}");
    assert!(sql.contains("id INTEGER PRIMARY KEY"), "missing PK: {sql}");
    assert!(sql.contains("published INTEGER NOT NULL DEFAULT 0"), "missing boolean default: {sql}");
    assert!(sql.contains("deleted_at TEXT NULL"), "missing soft_deletes: {sql}");
}

#[test]
fn test_create_table_mysql_basic() {
    let op = Schema::create_with_dialect("orders", SchemaDialect::Mysql, |t| {
        t.id();
        t.big_integer("user_id");
        t.decimal("total", 10, 2);
        t.boolean("paid");
    });
    let sql = op.to_sql();
    assert!(sql.contains("CREATE TABLE orders"), "missing CREATE TABLE: {sql}");
    assert!(sql.contains("id BIGINT AUTO_INCREMENT PRIMARY KEY NOT NULL"), "missing PK: {sql}");
    assert!(sql.contains("user_id BIGINT NOT NULL"), "missing bigint: {sql}");
    assert!(sql.contains("DECIMAL(10,2)"), "missing decimal: {sql}");
    assert!(sql.contains("paid TINYINT(1) NOT NULL"), "missing bool: {sql}");
}

#[test]
fn test_create_table_with_foreign_key() {
    let op = Schema::create_with_dialect("posts", SchemaDialect::Postgres, |t| {
        t.id();
        t.string("title", 255);
        t.integer("user_id");
        t.foreign("user_id")
            .references("users", "id")
            .on_delete(ForeignAction::Cascade);
    });
    let sql = op.to_sql();
    assert!(sql.contains("FOREIGN KEY (user_id) REFERENCES users (id)"), "missing FK: {sql}");
    assert!(sql.contains("ON DELETE CASCADE"), "missing ON DELETE CASCADE: {sql}");
}

#[test]
fn test_create_table_with_indexes() {
    let op = Schema::create_with_dialect("users", SchemaDialect::Postgres, |t| {
        t.id();
        t.string("email", 255);
        t.string("role", 50);
        t.index(&["role"]);
        t.unique_index(&["email"]);
    });
    let sql = op.to_sql();
    assert!(sql.contains("CREATE INDEX users_role_idx ON users (role)"), "missing index: {sql}");
    assert!(
        sql.contains("CREATE UNIQUE INDEX users_email_unique ON users (email)"),
        "missing unique index: {sql}"
    );
}

#[test]
fn test_create_table_multi_pk() {
    let op = Schema::create_with_dialect("user_roles", SchemaDialect::Postgres, |t| {
        t.integer("user_id");
        t.integer("role_id");
        t.primary_key(&["user_id", "role_id"]);
    });
    let sql = op.to_sql();
    assert!(sql.contains("PRIMARY KEY (user_id, role_id)"), "missing multi PK: {sql}");
}

// ── Column types ─────────────────────────────────────────────────────────────

#[test]
fn test_all_column_types_postgres() {
    let op = Schema::create_with_dialect("all_types", SchemaDialect::Postgres, |t| {
        t.increments("id");
        t.uuid("uid");
        t.text("body");
        t.big_integer("views");
        t.small_integer("priority");
        t.float("score");
        t.double("price");
        t.date("birthday");
        t.json("meta");
        t.binary("data");
        t.enum_col("status", &["draft", "published"]);
    });
    let sql = op.to_sql();
    assert!(sql.contains("id SERIAL NOT NULL"));
    assert!(sql.contains("uid UUID NOT NULL"));
    assert!(sql.contains("body TEXT NOT NULL"));
    assert!(sql.contains("views BIGINT NOT NULL"));
    assert!(sql.contains("priority SMALLINT NOT NULL"));
    assert!(sql.contains("score REAL NOT NULL"));
    assert!(sql.contains("price DOUBLE PRECISION NOT NULL"));
    assert!(sql.contains("birthday DATE NOT NULL"));
    assert!(sql.contains("meta JSONB NOT NULL"));
    assert!(sql.contains("data BYTEA NOT NULL"));
    assert!(sql.contains("status VARCHAR(255)"));
}

// ── ALTER TABLE ──────────────────────────────────────────────────────────────

#[test]
fn test_alter_table_add_column() {
    let op = Schema::alter_with_dialect("users", SchemaDialect::Postgres, |t| {
        t.add_column("avatar_url", super::column::ColumnType::String(500)).nullable();
    });
    let sql = op.to_sql();
    assert!(
        sql.contains("ALTER TABLE users ADD COLUMN avatar_url VARCHAR(500) NULL"),
        "bad alter: {sql}"
    );
}

#[test]
fn test_alter_table_drop_rename() {
    let op = Schema::alter_with_dialect("users", SchemaDialect::Postgres, |t| {
        t.drop_column("old_field");
        t.rename_column("bio", "biography");
    });
    let sql = op.to_sql();
    assert!(sql.contains("DROP COLUMN old_field"), "missing drop: {sql}");
    assert!(sql.contains("RENAME COLUMN bio TO biography"), "missing rename: {sql}");
}

#[test]
fn test_alter_table_drop_column_sqlite_comment() {
    let op = Schema::alter_with_dialect("users", SchemaDialect::Sqlite, |t| {
        t.drop_column("old_field");
    });
    let sql = op.to_sql();
    // SQLite can't drop columns prior to 3.35 — we emit a comment
    assert!(sql.contains("-- SQLite: recreate table"), "missing SQLite comment: {sql}");
}

// ── DROP / RENAME ─────────────────────────────────────────────────────────────

#[test]
fn test_drop_table() {
    let sql = Schema::drop("users").to_sql();
    assert_eq!(sql, "DROP TABLE users");
}

#[test]
fn test_drop_table_if_exists() {
    let sql = Schema::drop_if_exists("users").to_sql();
    assert_eq!(sql, "DROP TABLE IF EXISTS users");
}

#[test]
fn test_rename_table() {
    let sql = Schema::rename("old_name", "new_name").to_sql();
    assert_eq!(sql, "ALTER TABLE old_name RENAME TO new_name");
}

// ── Standalone index helpers ──────────────────────────────────────────────────

#[test]
fn test_create_index_standalone() {
    let sql = Schema::create_index("users", &["email"], true).to_sql();
    assert!(sql.contains("CREATE UNIQUE INDEX"), "missing UNIQUE INDEX: {sql}");
    assert!(sql.contains("ON users (email)"), "missing table/col: {sql}");
}

#[test]
fn test_drop_index_standalone() {
    let sql = Schema::drop_index("users_email_unique").to_sql();
    assert_eq!(sql, "DROP INDEX IF EXISTS users_email_unique");
}
