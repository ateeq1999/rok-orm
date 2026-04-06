# Phase 11: Model Casting & Serialization

> **Target version:** v0.6.0
> **Status:** 🔜 Planned

---

## Goal

Models should transparently handle type conversion between database storage and Rust types, and give fine-grained control over what gets serialized to JSON.

---

## 11.1 Attribute Casting

### API

```rust
#[derive(Model, sqlx::FromRow)]
#[model(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,

    #[model(cast = "json")]
    pub permissions: Vec<String>,          // stored as TEXT/JSONB, decoded on read

    #[model(cast = "datetime")]
    pub last_login: Option<DateTime<Utc>>, // stored as TEXT in SQLite, decoded on read

    #[model(cast = "bool")]
    pub active: bool,                      // stored as INTEGER 0/1 in SQLite

    #[model(cast = "csv")]
    pub tags: Vec<String>,                 // stored as "rust,orm,async", split on read

    #[model(cast = "encrypted")]           // user supplies an Encryptor impl
    pub secret_token: String,
}
```

### Cast Implementations

| Cast | DB type stored as | Rust decode | Rust encode |
|------|------------------|-------------|-------------|
| `json` | TEXT / JSONB | `serde_json::from_str()` | `serde_json::to_string()` |
| `datetime` | TEXT / TIMESTAMPTZ | `DateTime<Utc>::parse_from_rfc3339()` | `.to_rfc3339()` |
| `bool` | INTEGER | `val != 0` | `if val { 1 } else { 0 }` |
| `csv` | TEXT | `s.split(',').collect()` | `v.join(",")` |
| `encrypted` | TEXT | `Encryptor::decrypt(val)` | `Encryptor::encrypt(val)` |

### Tasks

- [ ] Define `Cast` enum: `Json`, `DateTime`, `Bool`, `Csv`, `Encrypted`
- [ ] Add `#[model(cast = "...")]` field attribute to macro parser
- [ ] Generate a `post_process(row: &mut Self)` method called after `from_row`
- [ ] For each cast field, decode from raw `SqlValue` to typed Rust value in `post_process`
- [ ] Generate `pre_encode(data: &mut Vec<(&str, SqlValue)>)` called before INSERT/UPDATE
- [ ] For each cast field in data, encode Rust value → `SqlValue` via cast logic
- [ ] Define `Encryptor` trait: `encrypt(plaintext: &str) -> String`, `decrypt(ciphertext: &str) -> OrmResult<String>`
- [ ] Add `Model::set_encryptor(impl Encryptor + Send + Sync + 'static)` global registration
- [ ] Tests: all 5 cast types encode/decode correctly, round-trip through DB

---

## 11.2 Model Serialization Control

### API

```rust
#[derive(Model, sqlx::FromRow, Serialize)]
#[model(
    table   = "users",
    hidden  = ["password", "remember_token"],
    visible = ["id", "name", "email"],     // whitelist alternative to hidden
    appends = ["full_name", "avatar_url"], // computed fields added to serialization
)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String,         // hidden
    pub remember_token: String,   // hidden
}

// Implement computed fields
impl UserAppends for User {
    fn full_name(&self) -> serde_json::Value {
        format!("{} Doe", self.name).into()
    }
    fn avatar_url(&self) -> serde_json::Value {
        format!("https://cdn.example.com/{}.jpg", self.id).into()
    }
}

// Serialize — respects hidden + appends
let json = serde_json::to_string(&user.serialize())?;
// → {"id":1,"name":"Alice","email":"alice@example.com","full_name":"Alice Doe","avatar_url":"..."}

// Temporarily reveal a hidden field
let json = serde_json::to_string(&user.make_visible(&["password"]).serialize())?;

// Temporarily hide a visible field
let json = serde_json::to_string(&user.make_hidden(&["email"]).serialize())?;
```

### Tasks

- [ ] Add `hidden() -> &'static [&'static str]` to `Model` trait
- [ ] Add `visible() -> &'static [&'static str]` to `Model` trait
- [ ] Generate from `#[model(hidden = [...])]` / `#[model(visible = [...])]`
- [ ] Add `appends() -> &'static [&'static str]` to `Model` trait
- [ ] Generate `{Model}Appends` trait with one method per appended field returning `serde_json::Value`
- [ ] Add `Model::serialize(&self) -> serde_json::Value` — builds JSON from all non-hidden fields + appends
- [ ] Add `make_visible(&self, fields: &[&str]) -> SerializeOverride<Self>` wrapper
- [ ] Add `make_hidden(&self, fields: &[&str]) -> SerializeOverride<Self>` wrapper
- [ ] `SerializeOverride<T>` wraps the model + override sets, implements `Serialize`
- [ ] Tests: hidden fields absent, visible whitelist works, appends present, make_visible/hidden overrides

---

## 11.3 Accessors and Mutators

### API

```rust
#[derive(Model, sqlx::FromRow)]
pub struct User {
    pub id: i64,

    #[model(accessor)]   // mark for accessor generation
    pub name: String,

    #[model(accessor)]
    pub email: String,
}

// Implement the generated trait
impl UserAccessors for User {
    // Called when reading name (optional — default returns self.name)
    fn get_name(&self) -> String {
        format!("{}!", self.name)  // append !
    }

    // Called when encoding name for DB write
    fn set_name(val: SqlValue) -> SqlValue {
        match val {
            SqlValue::Text(s) => SqlValue::Text(s.trim().to_string()),
            other => other,
        }
    }

    fn set_email(val: SqlValue) -> SqlValue {
        match val {
            SqlValue::Text(s) => SqlValue::Text(s.to_lowercase()),
            other => other,
        }
    }
}

// get_* is called by .serialize() and .get_attribute("name")
let name = user.get_attribute("name");  // → "Alice!"
```

### Tasks

- [ ] Add `#[model(accessor)]` field attribute to macro parser
- [ ] Generate `{Model}Accessors` trait with:
  - `fn get_{field}(&self) -> {FieldType}` (default: return `self.{field}.clone()`)
  - `fn set_{field}(val: SqlValue) -> SqlValue` (default: return `val`)
- [ ] Add `Model::get_attribute(&self, col: &str) -> SqlValue` — calls the appropriate `get_*` method
- [ ] Apply `set_*` mutators in `pre_encode` before INSERT/UPDATE
- [ ] `serialize()` calls `get_*` for each accessor-marked field
- [ ] Tests: get accessor transforms value, set accessor transforms before insert, default (no accessor impl) passthrough

---

## Acceptance Criteria for Phase 11

- [ ] All 3 sub-sections implemented
- [ ] Casting round-trips through PG + SQLite (read/write)
- [ ] `serialize()` tested against JSON output
- [ ] Accessor transforms applied in both read and write paths
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Phase file tasks all checked off
