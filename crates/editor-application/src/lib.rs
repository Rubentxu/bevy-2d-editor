#![deny(missing_docs)]
#![warn(rust_2018_idioms)]
//! Editor application services.
//!
//! See ADR-0031 (EditorSession), ADR-0033 (ProjectStore), ADR-0048 (sync v1).

pub mod adapters;
pub mod ports;

pub use adapters::in_memory::InMemoryProjectStore;
pub use ports::project_store::{ProjectStore, StoreEntry, StoreError};
