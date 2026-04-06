//! Model lifecycle hooks/events for rok-orm.
//!
//! Implement the `ModelHooks` trait on your model to intercept and modify
//! model lifecycle events.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::hooks::ModelHooks;
//!
//! impl ModelHooks for User {
//!     async fn before_create(&mut self) -> Result<(), HookError> {
//!         self.email = self.email.to_lowercase();
//!         Ok(())
//!     }
//!
//!     async fn after_create(&self) {
//!         tracing::info!("User {} created", self.id);
//!     }
//!
//!     async fn before_update(&mut self) -> Result<(), HookError> {
//!         self.updated_at = chrono::Utc::now().to_rfc3339();
//!         Ok(())
//!     }
//! }
//! ```

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HookError {
    #[error("Hook validation failed: {0}")]
    Validation(String),
    
    #[error("Hook constraint violation: {0}")]
    Constraint(String),
    
    #[error("Hook error: {0}")]
    Other(String),
}

impl From<&str> for HookError {
    fn from(s: &str) -> Self {
        HookError::Other(s.to_string())
    }
}

impl From<String> for HookError {
    fn from(s: String) -> Self {
        HookError::Other(s)
    }
}

#[async_trait::async_trait]
pub trait ModelHooks: Sized + Send + Sync {
    async fn before_create(&mut self) -> Result<(), HookError> {
        Ok(())
    }

    async fn after_create(&self) {}

    async fn before_update(&mut self) -> Result<(), HookError> {
        Ok(())
    }

    async fn after_update(&self) {}

    async fn before_delete(&self) -> Result<(), HookError> {
        Ok(())
    }

    async fn after_delete(&self) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookType {
    BeforeCreate,
    AfterCreate,
    BeforeUpdate,
    AfterUpdate,
    BeforeDelete,
    AfterDelete,
}

pub struct HookExecutor<M: ModelHooks> {
    _marker: std::marker::PhantomData<M>,
}

impl<M: ModelHooks> HookExecutor<M> {
    pub async fn run_before_create(model: &mut M) -> Result<(), HookError> {
        model.before_create().await
    }

    pub async fn run_after_create(model: &M) {
        model.after_create().await;
    }

    pub async fn run_before_update(model: &mut M) -> Result<(), HookError> {
        model.before_update().await
    }

    pub async fn run_after_update(model: &M) {
        model.after_update().await;
    }

    pub async fn run_before_delete(model: &M) -> Result<(), HookError> {
        model.before_delete().await
    }

    pub async fn run_after_delete(model: &M) {
        model.after_delete().await;
    }
}
