//! Application ports — abstract interfaces for external dependencies.

pub mod project_store;

pub use project_store::{ProjectStore, StoreEntry, StoreError};
