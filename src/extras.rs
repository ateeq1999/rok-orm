//! [`WithExtras<M>`] — wraps a model row with additional aggregate columns.
//!
//! Used with `with_count_col`, `with_sum_col`, etc. to carry the computed
//! values alongside the main model without modifying the model struct.
//!
//! # Example
//!
//! ```rust
//! use rok_orm::extras::WithExtras;
//! use rok_orm::SqlValue;
//!
//! // If Post had a `comments_count` extra loaded via with_count_col:
//! // let post_with_count: WithExtras<Post> = ...;
//! // assert_eq!(post_with_count.extra_i64("comments_count"), Some(5));
//! ```

use std::collections::HashMap;
use std::ops::Deref;

use crate::query::SqlValue;

/// A model row paired with a map of extra computed columns (aggregates, etc.).
#[derive(Debug, Clone)]
pub struct WithExtras<M> {
    /// The underlying model row.
    pub model: M,
    /// Extra columns keyed by alias name (e.g. `"comments_count"` → `SqlValue::Integer(5)`).
    pub extras: HashMap<String, SqlValue>,
}

impl<M> WithExtras<M> {
    pub fn new(model: M) -> Self {
        Self { model, extras: HashMap::new() }
    }

    pub fn with_extra(mut self, key: impl Into<String>, val: SqlValue) -> Self {
        self.extras.insert(key.into(), val);
        self
    }

    /// Get an extra value by key.
    pub fn extra(&self, key: &str) -> Option<&SqlValue> {
        self.extras.get(key)
    }

    /// Convenience: get an integer extra (e.g. counts).
    pub fn extra_i64(&self, key: &str) -> Option<i64> {
        match self.extras.get(key)? {
            SqlValue::Integer(n) => Some(*n),
            _ => None,
        }
    }

    /// Convenience: get a float extra (e.g. averages).
    pub fn extra_f64(&self, key: &str) -> Option<f64> {
        match self.extras.get(key)? {
            SqlValue::Float(f) => Some(*f),
            SqlValue::Integer(n) => Some(*n as f64),
            _ => None,
        }
    }

    /// Convenience: get a text extra.
    pub fn extra_str(&self, key: &str) -> Option<&str> {
        match self.extras.get(key)? {
            SqlValue::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl<M> Deref for WithExtras<M> {
    type Target = M;
    fn deref(&self) -> &Self::Target { &self.model }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Post { pub id: i64, pub title: String }

    #[test]
    fn extras_stores_and_retrieves_values() {
        let p = Post { id: 1, title: "Hello".into() };
        let we = WithExtras::new(p)
            .with_extra("comments_count", SqlValue::Integer(5))
            .with_extra("avg_rating", SqlValue::Float(4.2));

        assert_eq!(we.extra_i64("comments_count"), Some(5));
        assert_eq!(we.extra_f64("avg_rating"), Some(4.2));
        assert_eq!(we.extra("missing"), None);
    }

    #[test]
    fn deref_accesses_model_fields() {
        let p = Post { id: 42, title: "Rust".into() };
        let we = WithExtras::new(p);
        assert_eq!(we.id, 42);
        assert_eq!(we.title, "Rust");
    }

    #[test]
    fn extra_i64_from_integer_extra() {
        let p = Post { id: 1, title: "x".into() };
        let we = WithExtras::new(p).with_extra("n", SqlValue::Integer(99));
        assert_eq!(we.extra_i64("n"), Some(99));
    }

    #[test]
    fn extra_f64_promotes_integer() {
        let p = Post { id: 1, title: "x".into() };
        let we = WithExtras::new(p).with_extra("count", SqlValue::Integer(3));
        assert_eq!(we.extra_f64("count"), Some(3.0));
    }
}
