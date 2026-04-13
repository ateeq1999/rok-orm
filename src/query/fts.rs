//! 10.2 Full-text search methods for [`QueryBuilder`].
//!
//! | Method | Dialect | SQL generated |
//! |--------|---------|---------------|
//! | `where_full_text` | PostgreSQL | `to_tsvector('english', cols) @@ to_tsquery('english', 'word1 & word2')` |
//! | `order_by_text_rank` | PostgreSQL | `ORDER BY ts_rank(…) DESC` |
//! | `where_match` | MySQL | `MATCH(cols) AGAINST('query' IN NATURAL LANGUAGE MODE)` |
//! | `where_fts5` | SQLite | `fts_table MATCH 'query'` |

use super::builder::QueryBuilder;
use super::condition::{Condition, JoinOp, OrderDir};

/// Convert a space-separated query string into a `&`-joined tsquery operand.
///
/// `"rust async orm"` → `"rust & async & orm"`
fn to_tsquery_operand(query: &str) -> String {
    query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" & ")
}

impl<T> QueryBuilder<T> {
    /// PostgreSQL full-text search using `tsvector @@ tsquery`.
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("posts")
    ///     .where_full_text(&["title", "body"], "rust async orm")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("to_tsvector"));
    /// assert!(sql.contains("to_tsquery"));
    /// ```
    pub fn where_full_text(self, cols: &[&str], query: &str) -> Self {
        let col_expr = cols.join(" || ' ' || ");
        let ts_query = to_tsquery_operand(query);
        let frag = format!(
            "to_tsvector('english', {col_expr}) @@ to_tsquery('english', '{ts_query}')"
        );
        self.push(JoinOp::And, Condition::Raw(frag))
    }

    /// PostgreSQL: add a `ts_rank` `ORDER BY` expression (descending).
    ///
    /// Call after [`where_full_text`](Self::where_full_text) for ranked results.
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("posts")
    ///     .where_full_text(&["title", "body"], "rust orm")
    ///     .order_by_text_rank(&["title", "body"], "rust orm")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("ts_rank"));
    /// assert!(sql.contains("ORDER BY"));
    /// ```
    pub fn order_by_text_rank(mut self, cols: &[&str], query: &str) -> Self {
        let col_expr = cols.join(" || ' ' || ");
        let ts_query = to_tsquery_operand(query);
        let expr = format!(
            "ts_rank(to_tsvector('english', {col_expr}), to_tsquery('english', '{ts_query}')) DESC"
        );
        self.order.push((String::new(), OrderDir::Raw(expr)));
        self
    }

    /// MySQL full-text search using `MATCH … AGAINST`.
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("posts")
    ///     .where_match(&["title", "body"], "rust async orm")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("MATCH(title, body)"));
    /// assert!(sql.contains("AGAINST"));
    /// ```
    pub fn where_match(self, cols: &[&str], query: &str) -> Self {
        let col_list = cols.join(", ");
        let frag = format!(
            "MATCH({col_list}) AGAINST('{query}' IN NATURAL LANGUAGE MODE)"
        );
        self.push(JoinOp::And, Condition::Raw(frag))
    }

    /// SQLite FTS5 search using the `MATCH` operator against a virtual table.
    ///
    /// The `fts_table` must be a pre-created FTS5 virtual table name.
    ///
    /// # Example
    /// ```rust
    /// use rok_orm::QueryBuilder;
    ///
    /// let (sql, _) = QueryBuilder::<()>::new("posts")
    ///     .where_fts5("posts_fts", "rust async orm")
    ///     .to_sql();
    ///
    /// assert!(sql.contains("posts_fts MATCH"));
    /// ```
    pub fn where_fts5(self, fts_table: &str, query: &str) -> Self {
        let frag = format!("{fts_table} MATCH '{query}'");
        self.push(JoinOp::And, Condition::Raw(frag))
    }
}
