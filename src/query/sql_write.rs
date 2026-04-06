//! Static INSERT / UPSERT / batch-UPDATE SQL helpers for [`QueryBuilder`].

use super::builder::{Dialect, QueryBuilder};
use super::condition::{Condition, JoinOp, SqlValue};

impl<T> QueryBuilder<T> {
    // ── INSERT ──────────────────────────────────────────────────────────────

    /// Build an `INSERT INTO` statement (PostgreSQL `$N` placeholders).
    pub fn insert_sql(table: &str, data: &[(&str, SqlValue)]) -> (String, Vec<SqlValue>) {
        Self::insert_sql_with_dialect(Dialect::Postgres, table, data)
    }

    /// Build an `INSERT INTO` statement for the given dialect.
    pub fn insert_sql_with_dialect(
        dialect: Dialect,
        table: &str,
        data: &[(&str, SqlValue)],
    ) -> (String, Vec<SqlValue>) {
        let cols: Vec<&str> = data.iter().map(|(c, _)| *c).collect();
        let placeholders: Vec<String> = match dialect {
            Dialect::Postgres => (1..=data.len()).map(|i| format!("${i}")).collect(),
            Dialect::Sqlite | Dialect::Mysql => (0..data.len()).map(|_| "?".to_string()).collect(),
        };
        let params: Vec<SqlValue> = data.iter().map(|(_, v)| v.clone()).collect();
        (
            format!(
                "INSERT INTO {table} ({}) VALUES ({})",
                cols.join(", "),
                placeholders.join(", ")
            ),
            params,
        )
    }

    /// Build an `INSERT INTO … VALUES …, …` statement for multiple rows.
    ///
    /// All rows must have the same columns in the same order as the first row.
    pub fn bulk_insert_sql(table: &str, rows: &[Vec<(&str, SqlValue)>]) -> (String, Vec<SqlValue>) {
        assert!(!rows.is_empty(), "bulk_insert_sql requires at least one row");
        let cols: Vec<&str> = rows[0].iter().map(|(c, _)| *c).collect();
        let mut params: Vec<SqlValue> = Vec::new();
        let mut value_groups: Vec<String> = Vec::new();
        let mut offset = 1usize;

        for row in rows {
            let placeholders: Vec<String> = (offset..offset + row.len())
                .map(|i| format!("${i}"))
                .collect();
            value_groups.push(format!("({})", placeholders.join(", ")));
            for (_, v) in row.iter() {
                params.push(v.clone());
            }
            offset += row.len();
        }

        (
            format!(
                "INSERT INTO {table} ({}) VALUES {}",
                cols.join(", "),
                value_groups.join(", ")
            ),
            params,
        )
    }

    // ── UPDATE (static) ─────────────────────────────────────────────────────

    /// Build an `UPDATE … SET … WHERE …` from explicit conditions.
    pub fn update_sql(
        table: &str,
        data: &[(&str, SqlValue)],
        conditions: &[(JoinOp, Condition)],
    ) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let set_clauses: Vec<String> = data
            .iter()
            .enumerate()
            .map(|(i, (col, val))| {
                params.push(val.clone());
                format!("{col} = ${}", i + 1)
            })
            .collect();
        let mut sql = format!("UPDATE {table} SET {}", set_clauses.join(", "));
        if !conditions.is_empty() {
            sql.push_str(&super::build_where_from(conditions, &mut params));
        }
        (sql, params)
    }

    // ── UPSERT ──────────────────────────────────────────────────────────────

    pub fn upsert_sql(
        table: &str,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> (String, Vec<SqlValue>) {
        let cols: Vec<&str> = data.iter().map(|(c, _)| *c).collect();
        let placeholders: Vec<String> = (1..=data.len()).map(|i| format!("${i}")).collect();
        let params: Vec<SqlValue> = data.iter().map(|(_, v)| v.clone()).collect();
        let update_clauses: Vec<String> = update_columns
            .iter()
            .map(|col| format!("{col} = EXCLUDED.{col}"))
            .collect();
        let sql = format!(
            "INSERT INTO {table} ({}) VALUES ({}) ON CONFLICT ({conflict_column}) DO UPDATE SET {}",
            cols.join(", "),
            placeholders.join(", "),
            update_clauses.join(", ")
        );
        (sql, params)
    }

    pub fn upsert_sql_with_dialect(
        dialect: Dialect,
        table: &str,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
        update_columns: &[&str],
    ) -> (String, Vec<SqlValue>) {
        let cols: Vec<&str> = data.iter().map(|(c, _)| *c).collect();
        let placeholders: Vec<String> = match dialect {
            Dialect::Postgres => (1..=data.len()).map(|i| format!("${i}")).collect(),
            Dialect::Sqlite | Dialect::Mysql => (0..data.len()).map(|_| "?".to_string()).collect(),
        };
        let params: Vec<SqlValue> = data.iter().map(|(_, v)| v.clone()).collect();
        let update_clauses: Vec<String> = update_columns
            .iter()
            .map(|col| format!("{col} = VALUES({col})"))
            .collect();
        let sql = match dialect {
            Dialect::Postgres | Dialect::Sqlite => format!(
                "INSERT INTO {table} ({}) VALUES ({}) ON CONFLICT ({conflict_column}) DO UPDATE SET {}",
                cols.join(", "),
                placeholders.join(", "),
                update_clauses.join(", ")
            ),
            Dialect::Mysql => format!(
                "INSERT INTO {table} ({}) VALUES ({}) ON DUPLICATE KEY UPDATE {}",
                cols.join(", "),
                placeholders.join(", "),
                update_clauses.join(", ")
            ),
        };
        (sql, params)
    }

    pub fn upsert_do_nothing_sql(
        dialect: Dialect,
        table: &str,
        data: &[(&str, SqlValue)],
        conflict_column: &str,
    ) -> (String, Vec<SqlValue>) {
        let cols: Vec<&str> = data.iter().map(|(c, _)| *c).collect();
        let placeholders: Vec<String> = match dialect {
            Dialect::Postgres => (1..=data.len()).map(|i| format!("${i}")).collect(),
            Dialect::Sqlite | Dialect::Mysql => (0..data.len()).map(|_| "?".to_string()).collect(),
        };
        let params: Vec<SqlValue> = data.iter().map(|(_, v)| v.clone()).collect();
        let sql = match dialect {
            Dialect::Mysql => format!(
                "INSERT IGNORE INTO {table} ({}) VALUES ({})",
                cols.join(", "),
                placeholders.join(", ")
            ),
            _ => format!(
                "INSERT INTO {table} ({}) VALUES ({}) ON CONFLICT ({conflict_column}) DO NOTHING",
                cols.join(", "),
                placeholders.join(", ")
            ),
        };
        (sql, params)
    }

    pub fn insert_ignore_sql(
        dialect: Dialect,
        table: &str,
        data: &[(&str, SqlValue)],
    ) -> (String, Vec<SqlValue>) {
        let cols: Vec<&str> = data.iter().map(|(c, _)| *c).collect();
        let placeholders: Vec<String> = match dialect {
            Dialect::Postgres => (1..=data.len()).map(|i| format!("${i}")).collect(),
            Dialect::Sqlite | Dialect::Mysql => (0..data.len()).map(|_| "?".to_string()).collect(),
        };
        let params: Vec<SqlValue> = data.iter().map(|(_, v)| v.clone()).collect();
        let sql = match dialect {
            Dialect::Mysql => format!(
                "INSERT IGNORE INTO {table} ({}) VALUES ({})",
                cols.join(", "),
                placeholders.join(", ")
            ),
            _ => format!(
                "INSERT INTO {table} ({}) VALUES ({})",
                cols.join(", "),
                placeholders.join(", ")
            ),
        };
        (sql, params)
    }

    // ── DELETE IN ───────────────────────────────────────────────────────────

    pub fn delete_in_sql(&self, column: &str, values: &[SqlValue]) -> (String, Vec<SqlValue>) {
        self.to_delete_in_sql_with_dialect(Dialect::Postgres, column, values)
    }

    pub fn to_delete_in_sql_with_dialect(
        &self,
        dialect: Dialect,
        column: &str,
        values: &[SqlValue],
    ) -> (String, Vec<SqlValue>) {
        let params = values.to_vec();
        let placeholders: Vec<String> = match dialect {
            Dialect::Postgres => (1..=values.len()).map(|i| format!("${}", i)).collect(),
            Dialect::Sqlite | Dialect::Mysql => {
                (0..values.len()).map(|_| "?".to_string()).collect()
            }
        };
        let sql = format!(
            "DELETE FROM {} WHERE {} IN ({})",
            self.table,
            column,
            placeholders.join(", ")
        );
        (sql, params)
    }

    // ── batch UPDATE ────────────────────────────────────────────────────────

    pub fn update_batch_sql(
        table: &str,
        id_column: &str,
        updates: &[(i64, &str, SqlValue)],
    ) -> (String, Vec<SqlValue>) {
        let mut params: Vec<SqlValue> = Vec::new();
        let mut where_clauses: Vec<String> = Vec::new();
        let mut param_offset = 0;

        for (id, _column, value) in updates {
            params.push(value.clone());
            param_offset += 1;
            where_clauses.push(format!("${}", updates.len() + param_offset));
            params.push(SqlValue::Integer(*id));
        }

        let columns: Vec<&str> = updates.iter().map(|(_, col, _)| *col).collect();
        let case_sql = if !columns.is_empty() {
            let cases: Vec<String> = columns
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    let cases: Vec<String> = updates
                        .iter()
                        .enumerate()
                        .map(|(j, _)| {
                            let param_idx = j + 1;
                            let val_idx = updates.len() + j * 2 + 1 + i;
                            format!("WHEN ${} THEN ${}", param_idx, val_idx)
                        })
                        .collect();
                    format!("{} = CASE {} END", col, cases.join(" "))
                })
                .collect();
            cases.join(", ")
        } else {
            String::new()
        };

        let sql = if !case_sql.is_empty() {
            format!(
                "UPDATE {table} SET {case_sql} WHERE {id_column} IN ({})",
                where_clauses.join(", ")
            )
        } else {
            format!("DELETE FROM {table} WHERE 1=0")
        };

        (sql, params)
    }
}
