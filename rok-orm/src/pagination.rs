//! Pagination and aggregation helpers for rok-orm.
//!
//! # Pagination
//!
//! ```rust,ignore
//! use rok_orm::{Model, PgModel};
//!
//! #[derive(Model, sqlx::FromRow)]
//! pub struct Post { pub id: i64, pub title: String }
//!
//! let page: Page<Post> = Post::paginate(&pool, 1, 20).await?;
//!
//! println!("Total: {} pages", page.total_pages);
//! println!("Current: {}", page.current_page);
//! println!("Has next: {}", page.has_next);
//! println!("Has prev: {}", page.has_prev);
//! ```
//!
//! # Aggregations
//!
//! ```rust,ignore
//! let total: i64 = Order::count(&pool).await?;
//! let revenue: f64 = Order::sum("total", &pool).await?;
//! let avg_age: f64 = User::avg("age", &pool).await?;
//! ```

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub per_page: i64,
    pub current_page: i64,
    pub last_page: i64,
}

impl<T> Page<T> {
    pub fn new(data: Vec<T>, total: i64, per_page: i64, current_page: i64) -> Self {
        let last_page = if per_page > 0 {
            (total as f64 / per_page as f64).ceil() as i64
        } else {
            0
        };

        Self {
            data,
            total,
            per_page,
            current_page,
            last_page,
        }
    }

    pub fn has_next(&self) -> bool {
        self.current_page < self.last_page
    }

    pub fn has_prev(&self) -> bool {
        self.current_page > 1
    }

    pub fn total_pages(&self) -> i64 {
        self.last_page
    }

    pub fn from_offset(data: Vec<T>, total: i64, per_page: i64, offset: usize) -> Self {
        let current_page = if per_page > 0 {
            ((offset as i64) / per_page) + 1
        } else {
            1
        };
        Self::new(data, total, per_page, current_page)
    }
}

pub struct PaginationOptions {
    pub page: i64,
    pub per_page: i64,
}

impl PaginationOptions {
    pub fn new(page: i64, per_page: i64) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, 100),
        }
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }
}

pub fn calculate_pages(total: i64, per_page: i64) -> i64 {
    if per_page <= 0 {
        return 0;
    }
    (total as f64 / per_page as f64).ceil() as i64
}
