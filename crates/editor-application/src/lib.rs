#![deny(missing_docs)]
#![warn(rust_2018_idioms)]
//! Editor application services.
//!
//! See ADR-0031 (EditorSession), ADR-0033 (ProjectStore), ADR-0048 (sync v1).

pub mod adapters;
pub mod ports;
pub mod runtime_delta;
pub mod session;
pub mod transaction;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use adapters::in_memory::InMemoryProjectStore;
pub use adapters::opfs::OpfsProjectStore;
pub use ports::project_store::{ProjectStore, StoreEntry, StoreError};
pub use runtime_delta::{ApplyBackPolicy, RuntimeDelta};
pub use session::{CacheEntry, DocumentSelection, EditorSession};
// Re-export session and transaction types from editor_model (the model layer).
// This allows editor-core to import these types without a circular dependency.
pub use editor_model::session::HistoryScope;
pub use editor_model::transaction::{
    Applier, ApprovalPolicy, ChangeOrigin, ChangeSet, DiffSummary, EffectsSummary, ResourceRef,
    ValidationReport,
};
// editor-application's transaction module provides TransactionKernel and KernelError
// (stateful, impl-specific) but re-exports the trait/types from editor_model.
pub use transaction::{ChangeSetSummary, KernelError, TransactionKernel};
