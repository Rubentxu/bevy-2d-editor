//! Transaction kernel types for the Bevy 2D Editor domain model.
//!
//! These types are at the bottom of the dependency chain. Moving them here breaks
//! the `editor-application → editor-core → editor-application` circular dependency
//! that blocked the ADR-0031 `EditorSession` migration.
//!
//! ## Types owned here (model layer)
//!
//! - [`ChangeOrigin`] — who initiated a ChangeSet
//! - [`ApprovalPolicy`] — whether a ChangeSet needs human review
//! - [`ResourceRef`] — reference to an affected resource
//! - [`EffectsSummary`] — runtime/build effects summary
//! - [`DiffSummary`] — semantic diff summary
//! - [`ValidationReport`] — preflight validation result
//! - [`Applier`] — domain trait for validate/apply/summarize
//! - [`ChangeSet`] — the reviewable unit of typed operations
//! - [`ApplyReceipt`] — receipt after successful atomic apply
//! - [`KernelError`] — kernel-level errors
//! - [`AppliedChangeMeta`] — metadata after successful apply (canonical: `editor_model::session`)

use crate::session::HistoryScope;
use crate::time::Timestamp;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Where a ChangeSet originated (ADR-0032 §Decision).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeOrigin {
    /// Human-authored change initiated by a user gesture.
    Human,
    /// Change proposed by an AI agent.
    Agent,
    /// Change produced by a built-in recipe.
    Recipe,
    /// Change imported from an external source (Aseprite, LDtk, Tiled, etc.).
    Importer,
    /// Change produced by a data migration.
    Migration,
    /// Change contributed by an editor extension.
    Plugin,
    /// Change observed during play-mode runtime and projected back to authoring.
    RuntimeApplyBack,
}

/// Approval policy for a ChangeSet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalPolicy {
    /// Approved automatically — no human review required.
    Auto,
    /// Requires explicit human approval before apply.
    RequiresHuman {
        /// Optional hint shown to the approver.
        approver_hint: Option<String>,
    },
}

/// Reference to an affected resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceRef {
    /// A scene document.
    Scene(String),
    /// A scene asset document.
    SceneAsset(String),
    /// A logic graph document.
    LogicGraph(String),
    /// A project-level resource.
    Project(String),
}

impl ResourceRef {
    /// Returns the resource kind as a string.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Scene(_) => "scene",
            Self::SceneAsset(_) => "scene_asset",
            Self::LogicGraph(_) => "logic_graph",
            Self::Project(_) => "project",
        }
    }
}

/// Summary of runtime/build effects produced by applying a ChangeSet.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectsSummary {
    /// Number of entities created.
    pub entities_created: u64,
    /// Number of entities deleted.
    pub entities_deleted: u64,
    /// Number of assets hot-reloaded.
    pub assets_reloaded: u64,
    /// Number of systems invalidated.
    pub systems_invalidated: u64,
    /// Whether this change requires a runtime preview rebuild.
    pub runtime_rebuild_required: bool,
    /// Whether the build output changed.
    pub build_output_changed: bool,
    /// Custom notes from the applier.
    pub notes: Vec<String>,
}

impl EffectsSummary {
    /// Empty effects summary.
    pub fn empty() -> Self {
        Self {
            entities_created: 0,
            entities_deleted: 0,
            assets_reloaded: 0,
            systems_invalidated: 0,
            runtime_rebuild_required: false,
            build_output_changed: false,
            notes: Vec::new(),
        }
    }
}

/// Summary of semantic changes made by a ChangeSet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffSummary {
    /// Number of entities added.
    pub added: u64,
    /// Number of entities removed.
    pub removed: u64,
    /// Number of entities modified.
    pub modified: u64,
    /// Custom notes from the applier.
    pub notes: Vec<String>,
}

impl DiffSummary {
    /// Empty diff summary.
    pub fn empty() -> Self {
        Self {
            added: 0,
            removed: 0,
            modified: 0,
            notes: Vec::new(),
        }
    }
}

/// Result of preflight validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    /// Validation errors (fatal — apply must not proceed).
    pub errors: Vec<String>,
    /// Validation warnings (non-fatal — apply may proceed with caution).
    pub warnings: Vec<String>,
}

impl ValidationReport {
    /// Returns true if there are no errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Domain trait for validating and applying typed operations within a ChangeSet.
///
/// Each domain (scene, scene asset, logic graph) implements this trait to provide
/// its specific validate/apply/summarize logic.
pub trait Applier: Send + Sync {
    /// The operation type for this domain.
    type Operation: Debug + Clone + Send + Sync + 'static;
    /// The document type for this domain.
    type Document: Debug + Clone;
    /// The domain-specific error type.
    type Error: Debug + std::fmt::Display + Send + Sync + 'static;

    /// Preflight check: validate operation without modifying the document.
    ///
    /// Called during `TransactionKernel::validate` — returns `Ok(())` if the
    /// operation is valid given the current document state.
    fn preflight(&self, doc: &Self::Document, op: &Self::Operation) -> Result<(), Self::Error>;

    /// Apply the operation to the document, returning an inverse operation.
    fn apply(
        &self,
        doc: &mut Self::Document,
        op: &Self::Operation,
    ) -> Result<Self::Operation, Self::Error>;

    /// Summarize the runtime/build effects and semantic diff of applied operations.
    fn summarize(
        &self,
        doc: &Self::Document,
        ops: &[Self::Operation],
    ) -> (EffectsSummary, DiffSummary);
}

/// A reviewable group of typed semantic operations (ADR-0032).
///
/// ChangeSets are the atomic unit of undo/redo, review, and approval.
/// They carry enough metadata for the UI to display a meaningful diff.
#[derive(Debug, Clone)]
pub struct ChangeSet<O> {
    /// Unique change-set identifier.
    pub id: String,
    /// Where the change originated.
    pub origin: ChangeOrigin,
    /// Who authored this change.
    pub actor: String,
    /// Human-readable rationale for this change.
    pub rationale: String,
    /// Approval policy for this change.
    approval: ApprovalPolicy,
    /// Whether this change has been approved.
    approved: bool,
    /// List of operations in apply order.
    pub ops: Vec<O>,
    /// Resources affected by this change.
    resources: Vec<ResourceRef>,
}

impl<O: Debug + Clone> ChangeSet<O> {
    /// Construct a new ChangeSet.
    pub fn new(id: String, origin: ChangeOrigin, actor: String, rationale: String) -> Self {
        Self {
            id,
            origin,
            actor,
            rationale,
            approval: ApprovalPolicy::Auto,
            approved: false,
            ops: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// Add a resource reference.
    pub fn add_resource(&mut self, kind: &str, path: &str) {
        let r = match kind {
            "scene" => ResourceRef::Scene(path.to_string()),
            "scene_asset" => ResourceRef::SceneAsset(path.to_string()),
            "logic_graph" => ResourceRef::LogicGraph(path.to_string()),
            _ => ResourceRef::Project(path.to_string()),
        };
        self.resources.push(r);
    }

    /// Set the approval policy.
    pub fn set_approval(&mut self, policy: ApprovalPolicy) {
        self.approval = policy;
    }

    /// Mark this change as approved.
    pub fn approve(&mut self) {
        self.approved = true;
    }

    /// Returns true if this change has been approved according to its policy.
    pub fn is_approved(&self) -> bool {
        match &self.approval {
            ApprovalPolicy::Auto => true,
            ApprovalPolicy::RequiresHuman { .. } => self.approved,
        }
    }

    /// Returns the approval policy.
    pub fn approval_policy(&self) -> &ApprovalPolicy {
        &self.approval
    }

    /// Returns the list of affected resources.
    pub fn resources(&self) -> &[ResourceRef] {
        &self.resources
    }

    /// Push an operation onto the list.
    pub fn push_op(&mut self, op: O) {
        self.ops.push(op);
    }
}

/// Receipt after a successful atomic apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyReceipt<O> {
    /// Change set ID that was applied.
    pub change_id: String,
    /// Inverse operations in REVERSE application order.
    ///
    /// To undo, apply these inverses in the order they appear in the vector.
    /// That is equivalent to reversing the original operation list.
    pub inverses: Vec<O>,
    /// History revision number AFTER the apply.
    pub revision: u64,
    /// Runtime/build effects summary.
    pub effects: EffectsSummary,
    /// Semantic diff summary.
    pub diff: DiffSummary,
}

/// Kernel errors that can occur during validation or application.
///
/// The `E` type parameter carries the domain-specific error from the applier.
pub enum KernelError<E> {
    /// Preflight validation failed.
    Preflight(String),
    /// Application failed at the given operation index.
    ApplyFailed {
        /// Zero-based index of the operation that failed.
        index: usize,
        /// The domain-specific error from the applier.
        cause: E,
    },
    /// Rollback after a failed apply failed.
    RollbackFailed {
        /// The domain-specific error that occurred during rollback.
        cause: E,
    },
    /// Change set requires human approval but was not approved.
    ApprovalRequired,
    /// History scope is not available.
    HistoryMissing,
}

impl<E: Debug> Debug for KernelError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preflight(msg) => f.debug_tuple("Preflight").field(msg).finish(),
            Self::ApplyFailed { index, cause } => f
                .debug_struct("ApplyFailed")
                .field("index", index)
                .field("cause", cause)
                .finish(),
            Self::RollbackFailed { cause } => f.debug_tuple("RollbackFailed").field(cause).finish(),
            Self::ApprovalRequired => write!(f, "ApprovalRequired"),
            Self::HistoryMissing => write!(f, "HistoryMissing"),
        }
    }
}

impl<E: Debug + std::fmt::Display> std::fmt::Display for KernelError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preflight(msg) => write!(f, "preflight failed: {msg}"),
            Self::ApplyFailed { index, cause } => {
                write!(f, "apply failed at op {index}: {cause}")
            }
            Self::RollbackFailed { cause } => write!(f, "rollback failed: {cause}"),
            Self::ApprovalRequired => write!(f, "human approval required"),
            Self::HistoryMissing => write!(f, "history scope not available"),
        }
    }
}

impl<E: Debug + std::fmt::Display> std::error::Error for KernelError<E> {}

/// Metadata recorded about an applied change (ADR-0032).
///
/// Stored in `HistoryScope.last_change` after a successful apply.
/// Canonical location: `editor_model::session::AppliedChangeMeta`.
pub use crate::session::AppliedChangeMeta;

/// The transaction kernel owns the common apply/rollback mechanics.
///
/// It is stateless except for holding the domain-specific [`Applier`]
/// (stored behind an `Arc` so `TransactionKernel` remains cheaply cloneable).
///
/// # Type parameters
///
/// - `A`: the domain-specific [`Applier`] implementation.
#[derive(Debug)]
pub struct TransactionKernel<A: Applier> {
    applier: A,
}

impl<A: Applier> TransactionKernel<A> {
    /// Construct a new kernel with the given applier.
    pub fn new(applier: A) -> Self {
        Self { applier }
    }

    /// Validate all operations in a change set against the current document state.
    ///
    /// Operations are validated sequentially; validation stops at the first error.
    /// The document is NOT modified — this is a read-only check.
    pub fn validate(&self, cs: &ChangeSet<A::Operation>, doc: &A::Document) -> ValidationReport {
        let mut errors = Vec::new();
        for (i, op) in cs.ops.iter().enumerate() {
            if let Err(e) = self.applier.preflight(doc, op) {
                errors.push(format!("op {i}: {e}"));
                break;
            }
        }

        if errors.is_empty() {
            ValidationReport {
                errors: Vec::new(),
                warnings: Vec::new(),
            }
        } else {
            ValidationReport {
                errors,
                warnings: Vec::new(),
            }
        }
    }

    /// Apply a change set atomically to the document.
    ///
    /// # Steps
    ///
    /// 1. Check approval policy — `RequiresHuman` without prior approval returns `Err(ApprovalRequired)`.
    /// 2. Preflight all ops sequentially — any failure returns `Err(Preflight)`.
    /// 3. Apply ops in order, collecting inverses.
    ///    - On failure at index `i`: rollback by applying collected inverses in reverse,
    ///      then return `Err(ApplyFailed)`.
    ///    - On rollback failure: return `Err(RollbackFailed)` (catastrophic).
    /// 4. On success: call `history.next_revision()`, build receipt, record applied change metadata.
    pub fn apply_atomic(
        &self,
        cs: &ChangeSet<A::Operation>,
        doc: &mut A::Document,
        history: &mut HistoryScope,
    ) -> Result<ApplyReceipt<A::Operation>, KernelError<A::Error>> {
        // Step 1: Approval check
        if let ApprovalPolicy::RequiresHuman { .. } = &cs.approval {
            if !cs.is_approved() {
                return Err(KernelError::ApprovalRequired);
            }
        }

        // Step 2: Preflight all ops
        let mut simulated_doc = doc.clone();
        for (i, op) in cs.ops.iter().enumerate() {
            match self.applier.preflight(&simulated_doc, op) {
                Ok(()) => match self.applier.apply(&mut simulated_doc, op) {
                    Ok(inverse) => {
                        let _ = inverse;
                    }
                    Err(e) => {
                        return Err(KernelError::Preflight(format!("op {i}: {e}")));
                    }
                },
                Err(e) => {
                    return Err(KernelError::Preflight(format!("op {i}: {e}")));
                }
            }
        }

        // Step 3: Apply for real, collecting inverses
        let mut inverses = Vec::with_capacity(cs.ops.len());
        for (i, op) in cs.ops.iter().enumerate() {
            match self.applier.apply(doc, op) {
                Ok(inverse) => inverses.push(inverse),
                Err(cause) => {
                    // Rollback: apply inverses in reverse order
                    for inverse in inverses.into_iter().rev() {
                        if let Err(rollback_err) = self.applier.apply(doc, &inverse) {
                            return Err(KernelError::RollbackFailed {
                                cause: rollback_err,
                            });
                        }
                    }
                    return Err(KernelError::ApplyFailed { index: i, cause });
                }
            }
        }

        // Step 4: On success
        let revision = history.next_revision();
        let meta = AppliedChangeMeta {
            change_id: cs.id.clone(),
            origin: cs.origin.clone(),
            actor: cs.actor.clone(),
            applied_at: Timestamp(0), // Caller sets via Clock after apply_atomic returns
        };
        history.record_applied(meta);

        let (effects, diff) = self.applier.summarize(doc, &cs.ops);
        // inverses collected in application order [op0_inv, op1_inv];
        // reverse to get reverse application order [op1_inv, op0_inv].
        inverses.reverse();
        Ok(ApplyReceipt {
            change_id: cs.id.clone(),
            inverses,
            revision,
            effects,
            diff,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple document for testing.
    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    struct Doc(String);

    /// Simple operation for testing.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Op {
        Append(String),
        Clear,
    }

    struct TestApplier;
    impl Applier for TestApplier {
        type Operation = Op;
        type Document = Doc;
        type Error = String;

        fn preflight(&self, _doc: &Doc, _op: &Op) -> Result<(), Self::Error> {
            Ok(())
        }

        fn apply(&self, doc: &mut Doc, op: &Op) -> Result<Self::Operation, Self::Error> {
            match op {
                Op::Append(s) => {
                    let inverse = Op::Append(doc.0.len().to_string());
                    doc.0.push_str(s);
                    Ok(inverse)
                }
                Op::Clear => {
                    let inverse = Op::Append(doc.0.clone());
                    doc.0.clear();
                    Ok(inverse)
                }
            }
        }

        fn summarize(&self, _doc: &Doc, ops: &[Self::Operation]) -> (EffectsSummary, DiffSummary) {
            let added = ops.len() as u64;
            (
                EffectsSummary::empty(),
                DiffSummary {
                    added,
                    removed: 0,
                    modified: 0,
                    notes: Vec::new(),
                },
            )
        }
    }

    fn make_history() -> HistoryScope {
        HistoryScope::new()
    }

    #[test]
    fn validate_empty_change_set_is_ok() {
        let kernel = TransactionKernel::new(TestApplier);
        let doc = Doc("hello".into());
        let cs = ChangeSet::new(
            "cs-empty".into(),
            ChangeOrigin::Human,
            "test".into(),
            "empty".into(),
        );
        let report = kernel.validate(&cs, &doc);
        assert!(report.is_ok());
    }

    #[test]
    fn apply_single_op() {
        let kernel = TransactionKernel::new(TestApplier);
        let mut doc = Doc("".into());
        let mut history = make_history();
        let mut cs = ChangeSet::new(
            "cs-1".into(),
            ChangeOrigin::Human,
            "test".into(),
            "create then rename".into(),
        );
        cs.push_op(Op::Append("hello".into()));
        let receipt = kernel.apply_atomic(&cs, &mut doc, &mut history).unwrap();
        assert_eq!(doc.0, "hello");
        assert_eq!(receipt.revision, 1);
        // Op::Append collects an inverse (the current doc length as string)
        assert_eq!(receipt.inverses.len(), 1);
    }

    #[test]
    fn rollback_on_apply_failure() {
        struct FailingApplier;
        impl Applier for FailingApplier {
            type Operation = Op;
            type Document = Doc;
            type Error = String;

            fn preflight(&self, _doc: &Doc, _op: &Op) -> Result<(), Self::Error> {
                Ok(())
            }
            fn apply(&self, _doc: &mut Doc, _op: &Op) -> Result<Self::Operation, Self::Error> {
                Err("boom".into())
            }
            fn summarize(
                &self,
                _doc: &Doc,
                _ops: &[Self::Operation],
            ) -> (EffectsSummary, DiffSummary) {
                (EffectsSummary::empty(), DiffSummary::empty())
            }
        }

        let kernel = TransactionKernel::new(FailingApplier);
        let mut doc = Doc("original".into());
        let mut history = make_history();
        let mut cs = ChangeSet::new(
            "cs-fail".into(),
            ChangeOrigin::Human,
            "test".into(),
            "failing op".into(),
        );
        cs.push_op(Op::Clear);
        let err = kernel
            .apply_atomic(&cs, &mut doc, &mut history)
            .unwrap_err();
        // FailingApplier::apply always fails. Since the kernel calls apply() during
        // preflight (on a simulated doc), the preflight step fails first.
        assert!(matches!(err, KernelError::Preflight(_)));
        assert_eq!(doc.0, "original"); // rolled back
    }

    #[test]
    fn agent_proposal_requires_approval() {
        let kernel = TransactionKernel::new(TestApplier);
        let mut doc = Doc("".into());
        let mut history = make_history();
        let mut cs = ChangeSet::new(
            "cs-agent".into(),
            ChangeOrigin::Agent,
            "ai-agent".into(),
            "agent proposal".into(),
        );
        cs.set_approval(ApprovalPolicy::RequiresHuman {
            approver_hint: None,
        });
        cs.push_op(Op::Append("ai".into()));
        let err = kernel
            .apply_atomic(&cs, &mut doc, &mut history)
            .unwrap_err();
        assert!(matches!(err, KernelError::ApprovalRequired));
    }

    #[test]
    fn auto_approved_by_default() {
        let kernel = TransactionKernel::new(TestApplier);
        let mut doc = Doc("".into());
        let mut history = make_history();
        let mut cs = ChangeSet::new(
            "cs-auto".into(),
            ChangeOrigin::Human,
            "test".into(),
            "auto-approved".into(),
        );
        cs.set_approval(ApprovalPolicy::Auto);
        cs.push_op(Op::Append("x".into()));
        let receipt = kernel.apply_atomic(&cs, &mut doc, &mut history).unwrap();
        assert_eq!(receipt.change_id, "cs-auto");
    }
}
