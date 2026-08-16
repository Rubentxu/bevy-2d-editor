//! Session types for the Bevy 2D Editor domain model.
//!
//! These types are at the bottom of the dependency chain (`editor-model` has no
//! dependencies on `editor-core` or `editor-application`). Moving them here
//! breaks the `editor-application → editor-core → editor-application` circular
//! dependency that blocked the ADR-0031 `EditorSession` migration.

pub use crate::time::Timestamp;

/// Metadata about the most recently applied change within a [`HistoryScope`].
///
/// Stored by [`TransactionKernel::apply_atomic`] after each successful apply
/// so the UI can display provenance and the scope revision stays consistent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedChangeMeta {
    /// Change set ID that was applied.
    pub change_id: String,
    /// Where the change originated.
    pub origin: crate::transaction::ChangeOrigin,
    /// Actor who authored the change.
    pub actor: String,
    /// Timestamp when the change was applied.
    pub applied_at: Timestamp,
}

/// Explicit operation-history scope for one document or domain.
///
/// ADR-0031 rule: "operation histories are scoped explicitly" — each document
/// (scene, scene asset, logic graph) has its own `HistoryScope` that survives
/// document deactivation and is only reset on explicit "forget history" actions.
///
/// [`TransactionKernel::apply_atomic`]: super::transaction::TransactionKernel::apply_atomic
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryScope {
    revision: u64,
    /// Metadata about the most recently applied change, if any.
    last_change: Option<AppliedChangeMeta>,
}

impl HistoryScope {
    /// Construct a new history scope with revision 0 and no prior change.
    pub fn new() -> Self {
        Self {
            revision: 0,
            last_change: None,
        }
    }

    /// Returns the current revision number.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the next revision number and increments the stored value.
    pub fn next_revision(&mut self) -> u64 {
        let next = self.revision + 1;
        self.revision = next;
        next
    }

    /// Returns metadata about the most recently applied change, if any.
    pub fn last_change(&self) -> Option<&AppliedChangeMeta> {
        self.last_change.as_ref()
    }

    /// Record metadata about an applied change.
    ///
    /// Called by [`TransactionKernel::apply_atomic`] after a successful apply
    /// to record provenance.
    pub fn record_applied(&mut self, meta: AppliedChangeMeta) {
        self.last_change = Some(meta);
    }
}

impl Default for HistoryScope {
    fn default() -> Self {
        Self::new()
    }
}
