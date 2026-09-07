//! ProjectStore port — the file-system abstraction for the editor application.
//!
//! The trait and error types live in `editor_model::ports` (the model layer).
//! This module re-exports them. The concrete OPFS implementation is owned
//! by `editor_storage_web`; target-specific composition (WASM, native) wires
//! the implementation into `EditorSession`. See ADR-0057 and H1.2.

pub use editor_model::ports::{ProjectStore, StoreEntry, StoreError};
