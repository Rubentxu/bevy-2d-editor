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
pub use editor_model::session::ChangeSetSummary;
pub use editor_model::transaction::{AppliedChangeMeta, ApplyReceipt};
pub use editor_model::transaction::{
    Applier, ApprovalPolicy, ChangeOrigin, ChangeSet, DiffSummary, EffectsSummary, KernelError,
    ResourceRef, TransactionKernel, ValidationReport,
};
