//! Adapter registry for `editor-bevy`.
//!
//! The `EditorAdapter` trait lives in `editor-model` (ADR-0030: bevy-free, wasm-free).
//! This module holds the runtime registry (OnceLock + init + accessor) that was moved
//! out of `editor-model` per ARCH-030.
//!
//! `init_registry` is called once from the test binary; `all_adapters()` is called
//! by the adapter contract tests and by any runtime code that needs to iterate
//! over registered adapters.

use std::sync::OnceLock;

/// Global adapter registry, initialized exactly once via [`init_registry`].
///
/// `OnceLock` makes the registry cross-thread safe (unlike the S1
/// `thread_local!` which silently no-op'd on secondary threads).
static ADAPTERS: OnceLock<Vec<Box<dyn editor_model::adapter::EditorAdapter + Send + Sync>>> =
    OnceLock::new();

/// Returns a shared reference to the globally registered adapters.
///
/// Returns an empty slice if [`init_registry`] has not been called yet.
pub fn all_adapters() -> &'static [Box<dyn editor_model::adapter::EditorAdapter + Send + Sync>] {
    ADAPTERS.get().map(Vec::as_slice).unwrap_or(&[])
}

/// Initializes the global adapter registry.
///
/// Takes ownership of the adapter `Vec`. This is the cross-crate seam that
/// avoids an `editor-model → editor-bevy` dep (ADR-0030). Callers construct
/// the `Vec` of concrete impls and hand it over exactly once.
///
/// # Panics
///
/// Panics if called more than once (single-shot by design — double
/// initialization indicates a wiring bug).
pub fn init_registry(adapters: Vec<Box<dyn editor_model::adapter::EditorAdapter + Send + Sync>>) {
    match ADAPTERS.set(adapters) {
        Ok(()) => {}
        Err(_) => panic!("init_registry must be called exactly once"),
    }
}
