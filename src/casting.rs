//! Attribute casting system for Phase 11.1.
//!
//! Provides:
//! - The [`Encryptor`] trait for `cast = "encrypted"` fields.
//! - A global [`set_encryptor`] / [`encrypt`] / [`decrypt`] API.
//!
//! # Supported casts
//!
//! | `cast = "…"` | DB type  | `to_fields()` encoding           | `post_process()` decoding          |
//! |--------------|----------|----------------------------------|------------------------------------|
//! | `json`       | TEXT/JSONB | `serde_json::to_string()`      | `serde_json::from_str()`           |
//! | `bool`       | INTEGER  | `if val { 1 } else { 0 }`       | `val != 0` (via `sqlx::FromRow`)   |
//! | `datetime`   | TEXT     | `.to_rfc3339()`                  | RFC3339 parse (via `sqlx::FromRow`)|
//! | `csv`        | TEXT     | `vec.join(",")`                  | `s.split(',').collect()`           |
//! | `encrypted`  | TEXT     | `encrypt(val)`                   | `decrypt(val)` in `post_process()` |

use std::sync::OnceLock;

// ── Encryptor trait ───────────────────────────────────────────────────────────

/// Implement this trait and call [`set_encryptor`] to enable `cast = "encrypted"` fields.
pub trait Encryptor: Send + Sync {
    /// Encrypt `plaintext` and return the ciphertext stored in the DB.
    fn encrypt(&self, plaintext: &str) -> String;
    /// Decrypt `ciphertext` from the DB, returning the original plaintext.
    fn decrypt(&self, ciphertext: &str) -> Result<String, String>;
}

// ── Global registry ───────────────────────────────────────────────────────────

static ENCRYPTOR: OnceLock<Box<dyn Encryptor>> = OnceLock::new();

/// Register the global [`Encryptor`] implementation.
///
/// Must be called before any model with `cast = "encrypted"` fields is written.
/// Calling this more than once is a no-op (the first registration wins).
pub fn set_encryptor(enc: Box<dyn Encryptor>) {
    let _ = ENCRYPTOR.set(enc);
}

/// Encrypt `plaintext` using the registered [`Encryptor`].
///
/// If no encryptor has been registered, returns `plaintext` unchanged.
/// This is called automatically in the generated `to_fields()` for
/// `cast = "encrypted"` fields.
pub fn encrypt(plaintext: &str) -> String {
    ENCRYPTOR
        .get()
        .map(|e| e.encrypt(plaintext))
        .unwrap_or_else(|| plaintext.to_string())
}

/// Decrypt `ciphertext` using the registered [`Encryptor`].
///
/// Returns `Err` if no encryptor has been registered, or if decryption fails.
pub fn decrypt(ciphertext: &str) -> Result<String, String> {
    ENCRYPTOR
        .get()
        .ok_or_else(|| "No encryptor registered — call set_encryptor() first".to_string())
        .and_then(|e| e.decrypt(ciphertext))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Rot13;
    impl Encryptor for Rot13 {
        fn encrypt(&self, plaintext: &str) -> String {
            plaintext.chars().map(|c| match c {
                'a'..='m' | 'A'..='M' => (c as u8 + 13) as char,
                'n'..='z' | 'N'..='Z' => (c as u8 - 13) as char,
                _ => c,
            }).collect()
        }
        fn decrypt(&self, ciphertext: &str) -> Result<String, String> {
            Ok(self.encrypt(ciphertext)) // ROT13 is its own inverse
        }
    }

    #[test]
    fn encryptor_trait_works() {
        let e = Rot13;
        let cipher = e.encrypt("hello");
        assert_eq!(cipher, "uryyb");
        let plain = e.decrypt(&cipher).unwrap();
        assert_eq!(plain, "hello");
    }

    #[test]
    fn encrypt_without_registration_returns_plaintext() {
        // ENCRYPTOR may or may not be set at this point in the test suite.
        // Just verify the function doesn't panic.
        let _ = encrypt("test");
    }
}
