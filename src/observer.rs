//! Model observers — lifecycle hooks for create, update, delete, restore.
//!
//! Implement [`ModelObserver`] to receive callbacks when model records change.
//! Register observers with [`ObserverRegistry`].
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::observer::{ModelObserver, ObserverRegistry};
//!
//! pub struct AuditObserver;
//!
//! impl ModelObserver<User> for AuditObserver {
//!     fn created(&self, model: &User) {
//!         println!("User {} created", model.id);
//!     }
//!     fn deleted(&self, model: &User) {
//!         println!("User {} deleted", model.id);
//!     }
//! }
//!
//! // Register at startup
//! ObserverRegistry::observe::<User>(AuditObserver);
//! ```

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ── ModelObserver trait ─────────────────────────────────────────────────────

/// Lifecycle callbacks for a model type `M`.
///
/// All methods have empty default implementations — override only what you need.
pub trait ModelObserver<M>: Send + Sync + 'static {
    fn creating(&self, _model: &M) {}
    fn created(&self, _model: &M) {}
    fn updating(&self, _model: &M) {}
    fn updated(&self, _model: &M) {}
    fn saving(&self, _model: &M) {}
    fn saved(&self, _model: &M) {}
    fn deleting(&self, _model: &M) {}
    fn deleted(&self, _model: &M) {}
    fn restoring(&self, _model: &M) {}
    fn restored(&self, _model: &M) {}
}

// ── Type-erased observer wrapper ────────────────────────────────────────────

type BoxAny = Box<dyn std::any::Any + Send + Sync>;

/// Internal type-erased observer entry.
struct ObserverEntry {
    /// The observer cast to `Box<dyn Any>`.
    inner: BoxAny,
    /// Dispatch functions for each lifecycle event.
    creating: fn(&BoxAny, &dyn std::any::Any),
    created:  fn(&BoxAny, &dyn std::any::Any),
    updating: fn(&BoxAny, &dyn std::any::Any),
    updated:  fn(&BoxAny, &dyn std::any::Any),
    saving:   fn(&BoxAny, &dyn std::any::Any),
    saved:    fn(&BoxAny, &dyn std::any::Any),
    deleting: fn(&BoxAny, &dyn std::any::Any),
    deleted:  fn(&BoxAny, &dyn std::any::Any),
    restoring:fn(&BoxAny, &dyn std::any::Any),
    restored: fn(&BoxAny, &dyn std::any::Any),
}

impl ObserverEntry {
    fn new<M: 'static, O: ModelObserver<M>>(obs: O) -> Self {
        fn dispatch<M: 'static, O: ModelObserver<M>>(
            inner: &BoxAny,
            model: &dyn std::any::Any,
            f: fn(&O, &M),
        ) {
            if let (Some(o), Some(m)) = (
                inner.downcast_ref::<O>(),
                model.downcast_ref::<M>(),
            ) {
                f(o, m);
            }
        }

        Self {
            inner: Box::new(obs),
            creating: |i, m| dispatch::<M, O>(i, m, O::creating),
            created:  |i, m| dispatch::<M, O>(i, m, O::created),
            updating: |i, m| dispatch::<M, O>(i, m, O::updating),
            updated:  |i, m| dispatch::<M, O>(i, m, O::updated),
            saving:   |i, m| dispatch::<M, O>(i, m, O::saving),
            saved:    |i, m| dispatch::<M, O>(i, m, O::saved),
            deleting: |i, m| dispatch::<M, O>(i, m, O::deleting),
            deleted:  |i, m| dispatch::<M, O>(i, m, O::deleted),
            restoring:|i, m| dispatch::<M, O>(i, m, O::restoring),
            restored: |i, m| dispatch::<M, O>(i, m, O::restored),
        }
    }
}

// ── Registry ────────────────────────────────────────────────────────────────

static REGISTRY: std::sync::OnceLock<RwLock<HashMap<TypeId, Vec<ObserverEntry>>>> =
    std::sync::OnceLock::new();

fn registry() -> &'static RwLock<HashMap<TypeId, Vec<ObserverEntry>>> {
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Global observer registry.
pub struct ObserverRegistry;

impl ObserverRegistry {
    /// Register an observer for model type `M`.
    pub fn observe<M: 'static, O: ModelObserver<M>>(observer: O) {
        let entry = ObserverEntry::new::<M, O>(observer);
        let mut reg = registry().write().unwrap();
        reg.entry(TypeId::of::<M>()).or_default().push(entry);
    }

    /// Dispatch a lifecycle event for model `M`.
    ///
    /// `event` selects which method to call on all registered observers.
    pub fn dispatch<M: 'static>(model: &M, event: ObserverEvent) {
        let reg = registry().read().unwrap();
        if let Some(observers) = reg.get(&TypeId::of::<M>()) {
            let model_any: &dyn std::any::Any = model;
            for obs in observers {
                match event {
                    ObserverEvent::Creating  => (obs.creating)(&obs.inner, model_any),
                    ObserverEvent::Created   => (obs.created)(&obs.inner, model_any),
                    ObserverEvent::Updating  => (obs.updating)(&obs.inner, model_any),
                    ObserverEvent::Updated   => (obs.updated)(&obs.inner, model_any),
                    ObserverEvent::Saving    => (obs.saving)(&obs.inner, model_any),
                    ObserverEvent::Saved     => (obs.saved)(&obs.inner, model_any),
                    ObserverEvent::Deleting  => (obs.deleting)(&obs.inner, model_any),
                    ObserverEvent::Deleted   => (obs.deleted)(&obs.inner, model_any),
                    ObserverEvent::Restoring => (obs.restoring)(&obs.inner, model_any),
                    ObserverEvent::Restored  => (obs.restored)(&obs.inner, model_any),
                }
            }
        }
    }
}

/// Which lifecycle event to dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObserverEvent {
    Creating, Created,
    Updating, Updated,
    Saving,   Saved,
    Deleting, Deleted,
    Restoring, Restored,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct DummyModel { pub id: i64 }

    struct TrackingObserver {
        events: Arc<Mutex<Vec<String>>>,
    }

    impl ModelObserver<DummyModel> for TrackingObserver {
        fn created(&self, model: &DummyModel) {
            self.events.lock().unwrap().push(format!("created:{}", model.id));
        }
        fn deleted(&self, model: &DummyModel) {
            self.events.lock().unwrap().push(format!("deleted:{}", model.id));
        }
    }

    #[test]
    fn observer_created_event_dispatched() {
        let events = Arc::new(Mutex::new(Vec::<String>::new()));
        ObserverRegistry::observe::<DummyModel, _>(TrackingObserver { events: events.clone() });
        let m = DummyModel { id: 42 };
        ObserverRegistry::dispatch(&m, ObserverEvent::Created);
        let log = events.lock().unwrap();
        assert!(log.iter().any(|e| e == "created:42"));
    }

    #[test]
    fn observer_noop_for_unregistered_event() {
        let m = DummyModel { id: 99 };
        // Should not panic even with no observers for this event type
        ObserverRegistry::dispatch(&m, ObserverEvent::Updating);
    }
}
