//! ProjectStore + EditorSession ports — the cross-crate abstractions for the editor.
//!
//! `editor_application` provides the concrete implementations
//! (`OpfsProjectStore` + `EditorSession` for WASM, `InMemoryProjectStore` for tests).
//!
//! ## Architecture
//!
//! `PROJECT_STORE` and `EDITOR_SESSION` live here (not in `editor_core`) to break
//! the circular dependency: both `editor_application` and `editor_core` can access
//! them via `editor_model::ports` without depending on each other.

use crate::importer::{ImporterDescriptor, ImporterError, ImporterHandle, ImporterInput, ParseOutput};
use crate::session_port::EditorSessionPort;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

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

// ─────────────────────────────────────────────────────────────────────────────
// EditorSession registry (v0.90 PR1)
// ─────────────────────────────────────────────────────────────────────────────
//
// Same pattern as `PROJECT_STORE` above. `editor_application::wasm::init_project_store`
// registers the session once at WASM startup; `editor-core` (Bevy systems) can
// then read/write the session through the `EditorSessionPort` trait without
// importing `editor-application`. The registry holds `Arc<Mutex<dyn
// EditorSessionPort>>` (trait object) so the concrete session type stays in
// `editor-application`.

thread_local! {
    /// The global `EditorSession` — set once at WASM startup via
    /// [`register_editor_session`]. Same ownership semantics as `PROJECT_STORE`.
    static EDITOR_SESSION: std::cell::RefCell<Option<Arc<Mutex<dyn EditorSessionPort>>>> =
        const { std::cell::RefCell::new(None) };
}

/// Register the editor session (call once at WASM startup).
///
/// Takes ownership of the `Arc<Mutex<dyn EditorSessionPort>>`. The session stays
/// alive as long as either the caller keeps its `Arc` clone alive OR this
/// registration is held.
pub fn register_editor_session(session: Arc<Mutex<dyn EditorSessionPort>>) {
    EDITOR_SESSION.with(|cell| {
        *cell.borrow_mut() = Some(session);
    });
}

/// Run a closure with mutable access to the global `EditorSession`.
///
/// Returns `None` if the session is not yet initialized (callers should treat
/// this as a no-op and continue). The closure's return value is passed through.
///
/// Locks are released as soon as the closure returns. Callers MUST drop any
/// reference returned by the closure before calling another function that takes
/// the session lock.
pub fn with_session_mut<R, F: FnOnce(&mut dyn EditorSessionPort) -> R>(f: F) -> Option<R> {
    EDITOR_SESSION
        .try_with(|cell| {
            cell.borrow()
                .as_ref()
                .and_then(|arc| arc.lock().ok().map(|mut g| f(&mut *g)))
        })
        .ok()
        .flatten()
}

/// Read-only counterpart to [`with_session_mut`].
pub fn with_session<R, F: FnOnce(&dyn EditorSessionPort) -> R>(f: F) -> Option<R> {
    EDITOR_SESSION
        .try_with(|cell| {
            cell.borrow()
                .as_ref()
                .and_then(|arc| arc.lock().ok().map(|g| f(&*g)))
        })
        .ok()
        .flatten()
}

// ─────────────────────────────────────────────────────────────────────────────
// Extension registry (v0.92 — ADR-0040)
// ─────────────────────────────────────────────────────────────────────────────
//
// Mirrors the `PROJECT_STORE` / `EDITOR_SESSION` pattern exactly. The concrete
// `ExtensionRegistry` lives in `editor_application::extension`; this port trait
// and thread_local allow `editor_core` (Bevy systems) to check extension
// permissions without importing `editor_application`.

use crate::extension::{
    ExtensionError, ExtensionHandle, ExtensionManifest, ExtensionSummary,
};

/// Port trait for the extension registry.
///
/// Object-safe (`dyn ExtensionRegistryPort` is valid) so it can be held behind
/// an `Arc<Mutex<dyn ExtensionRegistryPort>>` on `EditorSession`.
pub trait ExtensionRegistryPort: Send + Sync {
    /// Register an extension manifest.
    ///
    /// Returns `Ok(handle)` on success. Returns `Err(ExtensionError::DuplicateId)`
    /// if the ID is already registered.
    fn register(&mut self, manifest: ExtensionManifest) -> Result<ExtensionHandle, ExtensionError>;

    /// Unregister an extension by ID.
    ///
    /// Returns `Ok(())` on success. Returns `Err(ExtensionError::NotFound)` if
    /// the ID is not registered.
    fn unregister(&mut self, id: &str) -> Result<(), ExtensionError>;

    /// List all registered extensions as lightweight summaries.
    fn list(&self) -> Vec<ExtensionSummary>;

    /// Get a registered extension's full manifest by ID.
    fn get(&self, id: &str) -> Option<ExtensionManifest>;
}

thread_local! {
    /// The global extension registry — set at WASM startup via
    /// [`register_extension_registry`]. Same ownership semantics as `PROJECT_STORE`.
    static EXTENSION_REGISTRY: std::cell::RefCell<Option<Arc<Mutex<dyn ExtensionRegistryPort>>>> =
        const { std::cell::RefCell::new(None) };
}

/// Register the extension registry (call once at WASM startup).
///
/// Takes ownership of the `Arc<Mutex<dyn ExtensionRegistryPort>>`. The registry
/// stays alive as long as either the caller keeps its `Arc` clone alive OR this
/// registration is held.
pub fn register_extension_registry(registry: Arc<Mutex<dyn ExtensionRegistryPort>>) {
    EXTENSION_REGISTRY.with(|cell| {
        *cell.borrow_mut() = Some(registry);
    });
}

/// Get a clone of the registered extension registry, or `None` if not yet registered.
pub fn with_extension_registry() -> Option<Arc<Mutex<dyn ExtensionRegistryPort>>> {
    EXTENSION_REGISTRY
        .try_with(|cell| cell.borrow().clone())
        .ok()
        .flatten()
}

// ─────────────────────────────────────────────────────────────────────────────
// Importer registry (v0.93 — ADR-0040 step 3 + ADR-0041)
// ─────────────────────────────────────────────────────────────────────────────
//
// Same pattern as `EXTENSION_REGISTRY`. The concrete `ImporterRegistry` lives in
// `editor_application::importer_registry`; this port trait and thread_local allow
// `editor_core` (Bevy systems doing import) and `editor-application` WASM exports
// to interact with the registry without a circular dependency.

use crate::external_source::ExternalSourceKind;

/// Port trait for the importer registry.
///
/// Object-safe (`dyn ImporterRegistryPort` is valid) so it can be held behind
/// an `Arc<Mutex<dyn ImporterRegistryPort>>` on `EditorSession`.
pub trait ImporterRegistryPort: Send + Sync {
    /// Register an importer with its descriptor and concrete implementation.
    ///
    /// Returns `Ok(handle)` on success. Returns `Err(ImporterError::DuplicateId)`
    /// if the ID is already registered.
    fn register(
        &mut self,
        descriptor: ImporterDescriptor,
        importer: std::sync::Arc<dyn crate::importer::Importer>,
    ) -> Result<ImporterHandle, ImporterError>;

    /// Unregister an importer by ID.
    ///
    /// Returns `Ok(())` on success. Returns `Err(ImporterError::NotFound)` if
    /// the ID is not registered.
    fn unregister(&mut self, id: &str) -> Result<(), ImporterError>;

    /// List all registered importers whose kind matches `kind`.
    fn list_by_kind(&self, kind: &ExternalSourceKind) -> Vec<ImporterDescriptor>;

    /// Dispatch parsing to the first registered importer matching `kind`.
    ///
    /// Returns `Err(ImporterError::NoImporterForKind)` if no importer is registered
    /// for the given kind.
    fn dispatch(
        &self,
        kind: &ExternalSourceKind,
        source: ImporterInput<'_>,
    ) -> Result<ParseOutput, ImporterError>;

    /// Get a registered importer by its ID.
    fn get(&self, id: &str) -> Option<std::sync::Arc<dyn crate::importer::Importer>>;

    /// Check whether a given importer ID is registered (any version).
    ///
    /// Used by the transaction kernel's permission gate — it only needs to
    /// verify the importer is known, not that it has an active implementation.
    fn is_registered(&self, id: &str) -> bool;
}

thread_local! {
    /// The global importer registry — set at WASM startup via
    /// [`register_importer_registry`]. Same ownership semantics as `PROJECT_STORE`.
    static IMPORTER_REGISTRY: std::cell::RefCell<Option<Arc<Mutex<dyn ImporterRegistryPort>>>> =
        const { std::cell::RefCell::new(None) };
}

/// Register the importer registry (call once at WASM startup).
///
/// Takes ownership of the `Arc<Mutex<dyn ImporterRegistryPort>>`. The registry
/// stays alive as long as either the caller keeps its `Arc` clone alive OR this
/// registration is held.
pub fn register_importer_registry(registry: Arc<Mutex<dyn ImporterRegistryPort>>) {
    IMPORTER_REGISTRY.with(|cell| {
        *cell.borrow_mut() = Some(registry);
    });
}

/// Get a clone of the registered importer registry, or `None` if not yet registered.
pub fn with_importer_registry() -> Option<Arc<Mutex<dyn ImporterRegistryPort>>> {
    IMPORTER_REGISTRY
        .try_with(|cell| cell.borrow().clone())
        .ok()
        .flatten()
}
