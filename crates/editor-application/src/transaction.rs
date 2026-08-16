//! Transaction Kernel and ChangeSet (ADR-0032).
//!
//! All shared types (`ChangeOrigin`, `ChangeSet`, `Applier`, `ApprovalPolicy`,
//! `EffectsSummary`, `DiffSummary`, `ValidationReport`, `ResourceRef`,
//! `ApplyReceipt`, `KernelError`, `TransactionKernel`) live in
//! `editor_model::transaction` — the model layer. They are re-exported here for
//! ergonomic use by application-layer code.
//!
//! This module also provides `ChangeSetSummary` (used only by editor-application).
//!
//! ## Non-goals (ADR-0032)
//!
//! - Not event sourcing.
//! - Not a generic `Command<T>` abstraction erasing domain language.
//! - Not a database transaction engine.

// Re-export all shared types from editor_model (the model layer).
pub use editor_model::transaction::{AppliedChangeMeta, ApplyReceipt};
pub use editor_model::transaction::{
    Applier, ApprovalPolicy, ChangeOrigin, ChangeSet, DiffSummary, EffectsSummary, KernelError,
    ResourceRef, TransactionKernel, ValidationReport,
};

// ChangeSetSummary is application-only (used by RecentChangeSetsBuffer) — lives here.
use serde::{Deserialize, Serialize};

/// A query-friendly summary of a recently applied change set.
///
/// Returned by [`OperationLog::recent_change_sets_for`](crate::operation_log::OperationLog::recent_change_sets_for).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeSetSummary {
    /// Where the change originated.
    pub origin: String,
    /// Who authored this change.
    pub actor: String,
    /// Timestamp when the change was applied (Unix milliseconds).
    pub applied_at_ms: u64,
    /// Number of operations in this entry that touched the queried stable ID.
    pub ops_touched: usize,
}
