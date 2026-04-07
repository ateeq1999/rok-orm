//! Cursor-based pagination for stable, efficient traversal of large result sets.
//!
//! Unlike offset pagination, cursor pagination doesn't skip rows when records
//! are added or deleted mid-traversal.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::cursor::{CursorPage, CursorResult};
//!
//! // First page
//! let page = CursorPage { after: None, limit: 20 };
//! let result: CursorResult<Post> = Post::query()
//!     .order_by_desc("id")
//!     .cursor_paginate(&pool, page)
//!     .await?;
//!
//! // Next page
//! let page = CursorPage { after: result.next_cursor, limit: 20 };
//! let result = Post::query()
//!     .order_by_desc("id")
//!     .cursor_paginate(&pool, page)
//!     .await?;
//!
//! println!("has_more: {}", result.has_more);
//! ```

/// Input parameters for a cursor-paginated query.
#[derive(Debug, Clone)]
pub struct CursorPage {
    /// The cursor from the previous page's `next_cursor`. `None` for the first page.
    ///
    /// For simple ID-based cursors this is the last `id` seen.
    pub after: Option<i64>,
    /// Maximum number of records per page.
    pub limit: usize,
}

impl CursorPage {
    /// Create a page cursor for the first page.
    pub fn first(limit: usize) -> Self {
        Self { after: None, limit }
    }

    /// Create a page cursor for continuation from a known cursor value.
    pub fn after(cursor: i64, limit: usize) -> Self {
        Self { after: Some(cursor), limit }
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
        if has_more {
            rows.truncate(limit);
        }
        let next_cursor = if has_more {
            rows.last().map(&get_id)
        } else {
            None
        };
        Self { data: rows, next_cursor, has_more }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_result_has_more_true_when_extra_row() {
        let rows = vec![1i64, 2, 3, 4, 5, 6]; // limit=5, 6 rows
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
        assert_eq!(result.data.len(), 3);
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
}
