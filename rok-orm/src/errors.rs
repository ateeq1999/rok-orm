//! Structured error types for rok-orm.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::errors::{OrmError, OrmResult};
//!
//! async fn get_user(id: i64) -> OrmResult<User> {
//!     User::find_by_pk(&pool, id)
//!         .await
//!         .map_err(OrmError::Database)?;
//!     Ok(user)
//! }
//!
//! match User::find_by_pk(&pool, id).await {
//!     Ok(user) => user,
//!     Err(OrmError::NotFound { model, .. }) => {
//!         println!("{} not found", model);
//!     }
//!     Err(e) => panic!("Database error: {}", e),
//! }
//! ```

use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrmError {
    #[error("Record not found: {model}::{pk}={id}")]
    NotFound {
        model: String,
        pk: String,
        id: String,
    },

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Constraint violation: {0}")]
    Constraint(String),

    #[error("Transaction failed: {0}")]
    Transaction(String),

    #[error("Hook failed: {0}")]
    Hook(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("{0}")]
    Other(String),
}

impl OrmError {
    pub fn not_found(
        model: impl Into<String>,
        pk: impl Into<String>,
        id: impl Into<String>,
    ) -> Self {
        Self::NotFound {
            model: model.into(),
            pk: pk.into(),
            id: id.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn constraint(message: impl Into<String>) -> Self {
        Self::Constraint(message.into())
    }

    pub fn transaction(message: impl Into<String>) -> Self {
        Self::Transaction(message.into())
    }

    pub fn hook(message: impl Into<String>) -> Self {
        Self::Hook(message.into())
    }

    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation(_))
    }

    pub fn is_constraint(&self) -> bool {
        matches!(self, Self::Constraint(_))
    }

    #[cfg(any(feature = "postgres", feature = "sqlite"))]
    pub fn from_sqlx_error(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound {
                model: "Unknown".to_string(),
                pk: "id".to_string(),
                id: "?".to_string(),
            },
            sqlx::Error::ConstraintViolation(msg) => Self::Constraint(msg),
            _ => Self::Database(err.to_string()),
        }
    }
}

pub type OrmResult<T> = Result<T, OrmError>;

pub trait IntoOrmResult<T> {
    fn into_orm_result(self) -> OrmResult<T>;
}

#[cfg(any(feature = "postgres", feature = "sqlite"))]
impl<T> IntoOrmResult<T> for Result<T, sqlx::Error> {
    fn into_orm_result(self) -> OrmResult<T> {
        self.map_err(OrmError::from_sqlx_error)
    }
}
