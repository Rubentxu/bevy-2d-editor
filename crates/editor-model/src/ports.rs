//! ProjectStore port — the file-system abstraction for the editor.
//!
//! `editor_application` provides the concrete implementation
//! (`OpfsProjectStore` for WASM, `InMemoryProjectStore` for tests).
//!
//! ## Architecture
//!
//! `PROJECT_STORE` lives here (not in `editor_core`) to break the circular
//! dependency: both `editor_application` and `editor_core` can access it via
//! `editor_model::ports` without depending on each other.

use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;

/// A single entry in the project store (file + metadata).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreEntry {
    /// The file path.
    pub path: String,
    /// File size in bytes.
    pub size: u64,
    /// Last modified timestamp in milliseconds since epoch.
    pub modified_ms: u64,
}

/// Errors that can occur when operating on the project store.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Entry not found.
    #[error("entry not found: {0}")]
    NotFound(String),
    /// IO error.
    #[error("io error: {0}")]
    Io(String),
    /// Atomic write failed and was rolled back.
    #[error("atomic write failed and was rolled back: {0}")]
    AtomicRollback(String),
    /// A lock was poisoned by a panicked thread.
    #[error("lock poisoned")]
    LockPoisoned,
}

/// ProjectStore — the file-system abstraction for the editor application.
///
/// In v1 this is SYNCHRONOUS per ADR-0048. Async migration deferred to v0.88.
pub trait ProjectStore: Send + Sync {
    /// List all entries under the given prefix.
    fn list(&self, prefix: &str) -> Result<Vec<StoreEntry>, StoreError>;

    /// Read the full contents of a file.
    fn read(&self, path: &str) -> Result<Vec<u8>, StoreError>;

    /// Write contents to a file.
    ///
    /// If `atomic` is true, the write should be atomic (all-or-nothing).
    fn write(&self, path: &str, bytes: &[u8], atomic: bool) -> Result<(), StoreError>;

    /// Delete a file.
    fn delete(&self, path: &str) -> Result<(), StoreError>;

    /// Check if a file exists.
    fn exists(&self, path: &str) -> Result<bool, StoreError>;

    /// Flush all pending operations (e.g., durable writes to OPFS).
    fn flush(&self) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + '_>>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Project store registry
// ─────────────────────────────────────────────────────────────────────────────

thread_local! {
    /// The global project store — set once at WASM startup via [`register_project_store`].
    static PROJECT_STORE: std::cell::RefCell<Option<Arc<dyn ProjectStore>>> =
        const { std::cell::RefCell::new(None) };
}

/// Register the project store (call once at WASM startup).
///
/// Takes ownership of the Arc — the store stays alive as long as either
/// the caller keeps its Arc clone alive OR this registration is held.
/// The canonical use: caller passes its Arc clone, keeps that clone alive
/// for the session, and this registration holds the shared ownership.
pub fn register_project_store(store: Arc<dyn ProjectStore>) {
    PROJECT_STORE.with(|cell| {
        *cell.borrow_mut() = Some(store);
    });
}

/// Get a clone of the registered project store, or `None` if not yet registered.
pub fn with_project_store() -> Option<Arc<dyn ProjectStore>> {
    PROJECT_STORE
        .try_with(|cell| cell.borrow().clone())
        .ok()
        .flatten()
}
