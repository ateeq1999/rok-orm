//! Phase 11.2 — `SerializeOverride<T>` for `make_visible` / `make_hidden`.
//!
//! The generated `serialize()`, `make_visible()`, and `make_hidden()` inherent methods
//! on each model use this type.  The generated code calls
//! `SerializeOverride::visible(self, fields)` or `SerializeOverride::hidden(self, fields)`.

use crate::model::Model;

/// Wraps a model reference with field-visibility overrides for one-off serialization.
///
/// Created by the generated `make_visible()` / `make_hidden()` inherent methods.
/// Call `.serialize()` to get the final [`serde_json::Value`].
///
/// # Example (generated code usage)
///
/// ```rust
/// // The generated make_visible / make_hidden produce a SerializeOverride.
/// // Users just call:
/// // let json = serde_json::to_string(&user.make_visible(&["password"]).serialize())?;
/// ```
pub struct SerializeOverride<'a, T> {
    model: &'a T,
    /// Hidden fields temporarily made visible.
    extra_visible: Vec<String>,
    /// Visible fields temporarily made hidden.
    extra_hidden: Vec<String>,
}

impl<'a, T: Model + serde::Serialize> SerializeOverride<'a, T> {
    /// Create an override that reveals `fields` which are normally hidden.
    pub fn visible(model: &'a T, fields: &[&str]) -> Self {
        SerializeOverride {
            model,
            extra_visible: fields.iter().map(|s| s.to_string()).collect(),
            extra_hidden: Vec::new(),
        }
    }

    /// Create an override that hides `fields` in addition to the model's defaults.
    pub fn hidden(model: &'a T, fields: &[&str]) -> Self {
        SerializeOverride {
            model,
            extra_visible: Vec::new(),
            extra_hidden: fields.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Serialize the model applying field-visibility overrides.
    ///
    /// 1. Serializes all struct fields via [`serde_json::to_value`].
    /// 2. Removes fields in [`Model::hidden`] unless they appear in `extra_visible`.
    /// 3. Removes fields in `extra_hidden`.
    /// 4. If [`Model::visible`] is non-empty, keeps only those fields
    ///    (plus `extra_visible`).
    ///
    /// Appended fields (from `{Model}Appends`) are not included — call the
    /// model's own `.serialize()` for appends support.
    pub fn serialize(&self) -> serde_json::Value {
        let val = serde_json::to_value(self.model).unwrap_or(serde_json::Value::Null);
        let mut map = match val {
            serde_json::Value::Object(m) => m,
            other => return other,
        };

        // Remove base-hidden fields unless temporarily revealed
        for col in T::hidden() {
            if !self.extra_visible.iter().any(|v| v == *col) {
                map.remove(*col);
            }
        }

        // Remove extra-hidden fields
        for col in &self.extra_hidden {
            map.remove(col.as_str());
        }

        // Apply visible whitelist (intersect with extra_visible)
        let vis = T::visible();
        if !vis.is_empty() {
            map.retain(|k, _| {
                vis.contains(&k.as_str())
                    || self.extra_visible.iter().any(|v| v == k)
            });
        }

        serde_json::Value::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct SimpleModel {
        id: i64,
        name: String,
        password: String,
    }

    impl Model for SimpleModel {
        fn table_name() -> &'static str { "simple" }
        fn columns() -> &'static [&'static str] { &["id", "name", "password"] }
        fn hidden() -> &'static [&'static str] { &["password"] }
    }

    #[test]
    fn visible_override_reveals_hidden_field() {
        let m = SimpleModel { id: 1, name: "Alice".into(), password: "secret".into() };
        let json = SerializeOverride::visible(&m, &["password"]).serialize();
        assert!(json.get("password").is_some(), "password should be revealed");
        assert_eq!(json["name"], "Alice");
    }

    #[test]
    fn hidden_override_hides_extra_field() {
        let m = SimpleModel { id: 1, name: "Alice".into(), password: "secret".into() };
        let json = SerializeOverride::hidden(&m, &["name"]).serialize();
        assert!(json.get("password").is_none(), "password still hidden");
        assert!(json.get("name").is_none(), "name also hidden");
        assert_eq!(json["id"], 1);
    }

    #[test]
    fn default_hides_password() {
        let m = SimpleModel { id: 1, name: "Bob".into(), password: "pw".into() };
        // No override — base hidden applies
        let json = SerializeOverride::hidden(&m, &[]).serialize();
        assert!(json.get("password").is_none(), "default hidden works");
    }
}
