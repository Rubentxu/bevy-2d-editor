//! ProjectStore port — the file-system abstraction for the editor application.
//!
//! The trait and error types live in `editor_model::ports` (the model layer).
//! This module re-exports them and provides the concrete `OpfsProjectStore` adapter.

pub use editor_model::ports::{ProjectStore, StoreEntry, StoreError};
