#![deny(missing_docs)]
#![warn(rust_2018_idioms)]
//! Editor application services.
//!
//! See ADR-0031 (EditorSession), ADR-0033 (ProjectStore), ADR-0048 (sync v1).

pub mod adapters;
pub mod ports;
pub mod session;
pub mod transaction;

pub use adapters::in_memory::InMemoryProjectStore;
pub use adapters::opfs::OpfsProjectStore;
pub use ports::project_store::{ProjectStore, StoreEntry, StoreError};
pub use session::{CacheEntry, DocumentSelection, EditorSession, HistoryScope};
// Re-export transaction kernel types for ergonomic use.
pub use transaction::{
    Applier, ApprovalPolicy, ChangeOrigin, ChangeSet, DiffSummary, EffectsSummary, KernelError,
    ResourceRef, TransactionKernel, ValidationReport,
};
