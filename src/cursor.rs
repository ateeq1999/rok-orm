//! Cursor-based pagination for stable, efficient traversal of large result sets.
//!
//! # Opaque cursor tokens
//!
//! `CursorResult::next_token()` encodes the cursor as a base64 JSON string
//! that is safe to expose in APIs. `CursorPage::from_token()` decodes it.
//!
//! ```rust
//! use rok_orm::cursor::{CursorPage, CursorResult};
//!
//! // Encode
//! let result: CursorResult<i64> = CursorResult {
//!     data: vec![1, 2, 3],
//!     next_cursor: Some(42),
//!     has_more: true,
//! };
//! let token = result.next_token(); // Some("eyJpZCI6NDJ9")
//!
//! // Decode
//! let page = CursorPage::from_token(token.as_deref(), 20).unwrap();
//! assert_eq!(page.after, Some(42));
//! ```

// ── Minimal base64 (URL-safe, no padding) ────────────────────────────────────

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn b64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity((input.len() * 4).div_ceil(3));
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        let take = chunk.len() + 1;
        for i in (0..4).take(take) {
            out.push(B64[((n >> (18 - i * 6)) & 0x3f) as usize] as char);
        }
    }
    out
}

fn b64_decode(input: &str) -> Option<Vec<u8>> {
    let mut table = [0xffu8; 256];
    for (i, &c) in B64.iter().enumerate() { table[c as usize] = i as u8; }
    let bytes: Vec<u8> = input.bytes().collect();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    for chunk in bytes.chunks(4) {
        let v: Vec<u8> = chunk.iter().map(|&b| table[b as usize]).collect();
        if v.contains(&0xff) { return None; }
        let n = ((v[0] as u32) << 18) | ((v[1] as u32) << 12)
              | ((v.get(2).copied().unwrap_or(0) as u32) << 6)
              | (v.get(3).copied().unwrap_or(0) as u32);
        out.push((n >> 16) as u8);
        if chunk.len() > 2 { out.push((n >> 8) as u8); }
        if chunk.len() > 3 { out.push(n as u8); }
    }
    Some(out)
}

/// Encode cursor id → opaque base64 JSON token.
pub fn encode_cursor(id: i64) -> String {
    b64_encode(format!("{{\"id\":{id}}}").as_bytes())
}

/// Decode opaque base64 JSON token → cursor id.
pub fn decode_cursor(token: &str) -> Option<i64> {
    let bytes = b64_decode(token)?;
    let s = std::str::from_utf8(&bytes).ok()?;
    // Parse {"id": <n>} — find value after "id":
    let after_id = s.find("\"id\":").map(|p| &s[p + 5..])?;
    let trimmed = after_id.trim_matches(|c: char| c.is_ascii_whitespace());
    let end = trimmed.find(|c: char| !c.is_ascii_digit() && c != '-').unwrap_or(trimmed.len());
    trimmed[..end].parse().ok()
}

// ── CursorPage ────────────────────────────────────────────────────────────────

/// Input parameters for a cursor-paginated query.
#[derive(Debug, Clone)]
pub struct CursorPage {
    /// The cursor from the previous page's `next_cursor`. `None` for the first page.
    pub after: Option<i64>,
    /// Maximum number of records per page.
    pub limit: usize,
}

impl CursorPage {
    /// Create a page cursor for the first page.
    pub fn first(limit: usize) -> Self { Self { after: None, limit } }

    /// Create a page cursor from a known raw cursor value.
    pub fn after(cursor: i64, limit: usize) -> Self { Self { after: Some(cursor), limit } }

    /// Decode an opaque token produced by [`CursorResult::next_token`].
    /// Returns `None` if the token is invalid.
    pub fn from_token(token: Option<&str>, limit: usize) -> Option<Self> {
        match token {
            None => Some(Self::first(limit)),
            Some(t) => Some(Self { after: Some(decode_cursor(t)?), limit }),
        }
    }
}

/// The result of a cursor-paginated query.
#[derive(Debug, Clone)]
pub struct CursorResult<T> {
    /// The records on this page.
    pub data: Vec<T>,
    /// Cursor value to pass as `CursorPage::after` for the next page.
    /// `None` if this is the last page.
    pub next_cursor: Option<i64>,
    /// Whether more records exist after this page.
    pub has_more: bool,
}

impl<T> CursorResult<T> {
    /// Build a `CursorResult` from a batch that may contain one extra record.
    ///
    /// Pass `limit + 1` rows; if `rows.len() > limit`, trims the last row and
    /// sets `has_more = true`. `get_id` extracts the cursor value from a row.
    pub fn from_rows(mut rows: Vec<T>, limit: usize, get_id: impl Fn(&T) -> i64) -> Self {
        let has_more = rows.len() > limit;
        if has_more { rows.truncate(limit); }
        let next_cursor = if has_more { rows.last().map(&get_id) } else { None };
        Self { data: rows, next_cursor, has_more }
    }

    /// Encode `next_cursor` as an opaque base64 token for use in API responses.
    /// Pass the returned string to [`CursorPage::from_token`] to continue pagination.
    pub fn next_token(&self) -> Option<String> {
        self.next_cursor.map(encode_cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_result_has_more_true_when_extra_row() {
        let rows = vec![1i64, 2, 3, 4, 5, 6];
        let result = CursorResult::from_rows(rows, 5, |r| *r);
        assert!(result.has_more);
        assert_eq!(result.data.len(), 5);
        assert_eq!(result.next_cursor, Some(5));
    }

    #[test]
    fn cursor_result_has_more_false_when_exact() {
        let rows = vec![1i64, 2, 3];
        let result = CursorResult::from_rows(rows, 5, |r| *r);
        assert!(!result.has_more);
        assert_eq!(result.next_cursor, None);
    }

    #[test]
    fn cursor_page_first() {
        let p = CursorPage::first(10);
        assert!(p.after.is_none());
        assert_eq!(p.limit, 10);
    }

    #[test]
    fn cursor_page_after() {
        let p = CursorPage::after(42, 20);
        assert_eq!(p.after, Some(42));
        assert_eq!(p.limit, 20);
    }

    #[test]
    fn encode_decode_roundtrip() {
        for id in [0i64, 1, 42, -1, i64::MAX / 2] {
            let token = encode_cursor(id);
            assert_eq!(decode_cursor(&token), Some(id));
        }
    }

    #[test]
    fn encode_cursor_is_base64_json() {
        let token = encode_cursor(99);
        // Must be decodable base64 whose JSON contains "id":99
        let bytes = b64_decode(&token).unwrap();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("\"id\":99"));
    }

    #[test]
    fn next_token_roundtrip_via_from_token() {
        let result: CursorResult<i64> = CursorResult {
            data: vec![1, 2, 3],
            next_cursor: Some(77),
            has_more: true,
        };
        let token = result.next_token().unwrap();
        let page = CursorPage::from_token(Some(&token), 10).unwrap();
        assert_eq!(page.after, Some(77));
        assert_eq!(page.limit, 10);
    }

    #[test]
    fn from_token_none_gives_first_page() {
        let page = CursorPage::from_token(None, 5).unwrap();
        assert!(page.after.is_none());
        assert_eq!(page.limit, 5);
    }

    #[test]
    fn decode_cursor_invalid_returns_none() {
        assert!(decode_cursor("not-valid-json-base64!!!").is_none());
    }
}
