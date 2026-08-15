//! Transaction Kernel and ChangeSet (ADR-0032).
//!
//! Provides the shared mechanics for composing typed semantic operations
//! across domains without creating a universal command abstraction.
//!
//! ## Core concepts
//!
//! - [`ChangeSet`] groups typed operations with origin, actor, rationale,
//!   affected resources, approval policy, and rollback metadata.
//! - [`TransactionKernel`] applies a [`ChangeSet`] atomically, handling
//!   preflight validation, rollback on failure, and approval gating.
//! - The kernel never interprets operations — domain-specific [`Applier`]
//!   implementations handle validation, application, and inverse generation.
//!
//! ## Non-goals (ADR-0032)
//!
//! - Not event sourcing.
//! - Not a generic `Command<T>` abstraction erasing domain language.
//! - Not a database transaction engine.

use std::fmt::Debug;

use editor_model::time::Timestamp;

/// Where a ChangeSet came from (ADR-0032 §Decision).
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
    /// Requires human approval before application.
    RequiresHuman {
        /// Hint shown to the approver (e.g. "Scene will be modified").
        approver_hint: Option<String>,
    },
}

impl ApprovalPolicy {
    /// Returns `true` if this policy requires human approval.
    pub fn requires_approval(&self) -> bool {
        matches!(self, Self::RequiresHuman { .. })
    }
}

/// Reference to an affected resource, e.g. `("scene", "scenes/main.json")`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRef {
    /// Kind of resource, such as `"scene"`, `"scene_asset"`, `"logic_graph"`.
    pub kind: String,
    /// Logical path within the project store.
    pub path: String,
}

/// Runtime/build effects summary (domains fill; kernel surfaces to reviewers).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectsSummary {
    /// Whether the Bevy runtime preview world must be rebuilt.
    pub runtime_rebuild_required: bool,
    /// Whether a build output (WASM, assets) changed.
    pub build_output_changed: bool,
    /// Freeform notes for reviewers.
    pub notes: Vec<String>,
}

impl EffectsSummary {
    /// Empty effects summary — no runtime or build impact.
    pub fn empty() -> Self {
        Self {
            runtime_rebuild_required: false,
            build_output_changed: false,
            notes: Vec::new(),
        }
    }
}

/// Semantic diff summary (counts + freeform notes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffSummary {
    /// Number of added items.
    pub added: u64,
    /// Number of removed items.
    pub removed: u64,
    /// Number of modified items.
    pub modified: u64,
    /// Freeform notes for reviewers.
    pub notes: Vec<String>,
}

impl DiffSummary {
    /// Empty diff summary — no changes.
    pub fn empty() -> Self {
        Self {
            added: 0,
            removed: 0,
            modified: 0,
            notes: Vec::new(),
        }
    }
}

/// Preflight validation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    /// Critical errors that block application.
    pub errors: Vec<String>,
    /// Warnings that do not block application (kept for audit trail).
    pub warnings: Vec<String>,
}

impl ValidationReport {
    /// Returns `true` if there are no errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Build a report with a single error message.
    pub fn single_error(msg: impl Into<String>) -> Self {
        Self {
            errors: vec![msg.into()],
            warnings: Vec::new(),
        }
    }

    /// Build a report with a single error and the rest collected from an iterator.
    pub fn from_iter<I, S>(errors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let errors: Vec<String> = errors.into_iter().map(Into::into).collect();
        Self {
            errors,
            warnings: Vec::new(),
        }
    }
}

/// Domain-supplied applier.
///
/// The kernel never interprets operations (ADR-0032 non-goal: no universal
/// command abstraction). Each domain provides an `Applier` that knows how to
/// validate, apply, and summarize its own operation type.
pub trait Applier {
    /// The domain's operation type.
    type Operation: Clone + Debug + PartialEq;
    /// The domain's document type.
    type Document: Clone + Debug + PartialEq;
    /// Error type returned by preflight or apply; must be `Display`.
    type Error: Debug + std::fmt::Display;

    /// Preflight-validate one op against the CURRENT document state.
    ///
    /// Called sequentially during `validate`; op N sees the document state
    /// AFTER ops 1..N-1 have been applied.
    fn preflight(&self, doc: &Self::Document, op: &Self::Operation) -> Result<(), Self::Error>;

    /// Apply one op, returning its inverse operation for rollback.
    ///
    /// The inverse MUST be valid for undo — applying the inverse to the
    /// post-apply document state restores the pre-apply state.
    fn apply(
        &self,
        doc: &mut Self::Document,
        op: &Self::Operation,
    ) -> Result<Self::Operation, Self::Error>;

    /// Summarize effects and diff for the whole set AFTER successful application.
    ///
    /// Called once after all ops have been applied.
    fn summarize(
        &self,
        doc: &Self::Document,
        ops: &[Self::Operation],
    ) -> (EffectsSummary, DiffSummary);
}

/// A reviewable group of typed semantic operations (ADR-0032).
///
/// Carries provenance, rationale, and approval metadata alongside the
/// actual operation list. This is the unit that reviewers see.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSet<O> {
    /// Unique identifier for this change set.
    pub id: String,
    /// Where this change set originated.
    pub origin: ChangeOrigin,
    /// Who/what authored this change (e.g. user ID, agent name, recipe ID).
    pub actor: String,
    /// Human-readable rationale for this change.
    pub rationale: String,
    /// Resources affected by this change set.
    pub affected_resources: Vec<ResourceRef>,
    /// Approval policy for this change set.
    pub approval: ApprovalPolicy,
    /// Whether this change set has been approved (for `RequiresHuman` policy).
    approved: bool,
    /// The typed operations to apply.
    pub ops: Vec<O>,
}

impl<O: Clone + Debug + PartialEq> ChangeSet<O> {
    /// Construct a new change set with the minimal required fields.
    ///
    /// The operation list starts empty and is populated via the `ops` field.
    /// Approval defaults to `Auto`.
    pub fn new(
        id: impl Into<String>,
        origin: ChangeOrigin,
        actor: impl Into<String>,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            origin,
            actor: actor.into(),
            rationale: rationale.into(),
            affected_resources: Vec::new(),
            approval: ApprovalPolicy::Auto,
            approved: true, // Auto-approved by default
            ops: Vec::new(),
        }
    }

    /// Add a resource to the affected resources list.
    pub fn add_resource(&mut self, kind: impl Into<String>, path: impl Into<String>) {
        self.affected_resources.push(ResourceRef {
            kind: kind.into(),
            path: path.into(),
        });
    }

    /// Set the approval policy.
    pub fn set_approval(&mut self, policy: ApprovalPolicy) {
        self.approval = policy.clone();
        self.approved = !policy.requires_approval();
    }

    /// Mark this change set as approved (used for `RequiresHuman` policy).
    ///
    /// Returns `true` if the change set was previously unapproved.
    pub fn approve(&mut self) -> bool {
        if !self.approved {
            self.approved = true;
            true
        } else {
            false
        }
    }

    /// Returns `true` if this change set has been approved.
    pub fn is_approved(&self) -> bool {
        self.approved
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

impl<E: Debug> std::fmt::Debug for KernelError<E> {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedChangeMeta {
    /// Change set ID that was applied.
    pub change_id: String,
    /// Where the change originated.
    pub origin: ChangeOrigin,
    /// Actor who authored the change.
    pub actor: String,
    /// Timestamp when the change was applied.
    pub applied_at: Timestamp,
}

/// The transaction kernel owns the common apply/rollback mechanics.
///
/// It is stateless except for holding the domain-specific [`Applier`]
/// (stored behind an `Arc` so `TransactionKernel` remains cheaply cloneable).
///
/// # Type parameters
///
/// - `A`: the domain-specific [`Applier`] implementation.
/// - `D`: the document type (must match `A::Document`).
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
    ///
    /// # Arguments
    ///
    /// - `cs`: the change set to validate.
    /// - `doc`: the current document state.
    pub fn validate(&self, cs: &ChangeSet<A::Operation>, doc: &A::Document) -> ValidationReport {
        let mut errors = Vec::new();
        // Clone doc and apply ops in order to simulate the final state.
        // Each preflight sees post-op-N-1 semantics (sequential apply semantics).
        let mut simulated_doc = doc.clone();
        for (i, op) in cs.ops.iter().enumerate() {
            match self.applier.preflight(&simulated_doc, op) {
                Ok(()) => {
                    // Apply op to simulated doc for next iteration's preflight
                    match self.applier.apply(&mut simulated_doc, op) {
                        Ok(inverse) => {
                            // Op applied successfully in simulation; inverse goes to
                            // simulated_doc by definition since apply returned it as
                            // the result of mutating doc.
                            // We need to re-apply the inverse to simulated_doc to keep
                            // it in sync for the next iteration.
                            // But we don't have the inverse to apply back...
                            // Actually, the apply function mutates doc IN PLACE and returns
                            // the inverse. So simulated_doc IS post-op after apply returns.
                            // The inverse is for undo purposes; we don't need to apply it
                            // to simulated_doc for preflight purposes.
                            let _ = inverse; // silence unused warning
                        }
                        Err(e) => {
                            errors.push(format!("op {i}: {e}"));
                            break;
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("op {i}: {e}"));
                    break;
                }
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
    ///    - On rollback failure: return `Err(RollbackFailed)` (catastrophic — doc may be partially rolled back).
    /// 4. On success: call `history.next_revision()`, build receipt, record applied change metadata.
    ///
    /// # Arguments
    ///
    /// - `cs`: the change set to apply.
    /// - `doc`: the current document (mutated in place on success).
    /// - `history`: the history scope (bumped revision + recorded `last_change`).
    pub fn apply_atomic(
        &self,
        cs: &ChangeSet<A::Operation>,
        doc: &mut A::Document,
        history: &mut super::session::HistoryScope,
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
                Ok(()) => {
                    // Apply to simulated doc for next iteration's preflight
                    match self.applier.apply(&mut simulated_doc, op) {
                        Ok(_inverse) => {
                            // simulated_doc is now post-op-N; continue to next
                        }
                        Err(e) => {
                            return Err(KernelError::Preflight(format!("op {i}: {e}")));
                        }
                    }
                }
                Err(e) => {
                    return Err(KernelError::Preflight(format!("op {i}: {e}")));
                }
            }
        }

        // Step 3: Apply for real, collecting inverses
        let mut inverses = Vec::with_capacity(cs.ops.len());
        let mut applied_doc = doc.clone();

        for (i, op) in cs.ops.iter().enumerate() {
            match self.applier.apply(&mut applied_doc, op) {
                Ok(inverse) => {
                    inverses.push(inverse);
                }
                Err(e) => {
                    // Rollback: apply inverses in reverse order
                    let mut first_rollback_err = None;
                    for inverse in inverses.into_iter().rev() {
                        if let Err(rb_err) = self.applier.apply(&mut applied_doc, &inverse) {
                            first_rollback_err = Some(rb_err);
                            break;
                        }
                    }
                    if let Some(rb_err) = first_rollback_err {
                        return Err(KernelError::RollbackFailed { cause: rb_err });
                    }
                    return Err(KernelError::ApplyFailed { index: i, cause: e });
                }
            }
        }

        // Success: update doc and history
        *doc = applied_doc;
        let revision = history.next_revision();

        // Record applied change metadata
        let meta = AppliedChangeMeta {
            change_id: cs.id.clone(),
            origin: cs.origin.clone(),
            actor: cs.actor.clone(),
            applied_at: Timestamp(0), // Will be set by caller via Clock
        };
        history.record_applied(meta);

        // Build receipt
        // inverses collected in application order [op0_inv, op1_inv];
        // reverse to get reverse application order [op1_inv, op0_inv].
        inverses.reverse();
        let (effects, diff) = self.applier.summarize(doc, &cs.ops);

        Ok(ApplyReceipt {
            change_id: cs.id.clone(),
            inverses,
            revision,
            effects,
            diff,
        })
    }
}

// Provide a blanket Arc-wrapped constructor for convenience
impl<A: Applier> TransactionKernel<A> {
    /// Wrap this kernel in an `Arc` (ergonomic for sharing across threads).
    pub fn into_arc(self) -> std::sync::Arc<Self> {
        std::sync::Arc::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal applier for testing — implements CreateEntity semantics.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum TestOp {
        CreateEntity { id: String, name: String },
        SetName { id: String, new_name: String },
        DeleteEntity { id: String },
    }

    impl std::fmt::Display for TestOp {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::CreateEntity { id, name } => write!(f, "CreateEntity({id}, {name})"),
                Self::SetName { id, new_name } => write!(f, "SetName({id}, {new_name})"),
                Self::DeleteEntity { id } => write!(f, "DeleteEntity({id})"),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct TestDoc {
        pub entities: Vec<TestEntity>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TestEntity {
        pub id: String,
        pub name: String,
    }

    #[derive(Debug)]
    pub struct TestApplier;

    impl Applier for TestApplier {
        type Operation = TestOp;
        type Document = TestDoc;
        type Error = String;

        fn preflight(&self, doc: &Self::Document, op: &Self::Operation) -> Result<(), Self::Error> {
            match op {
                TestOp::CreateEntity { id, .. } => {
                    if doc.entities.iter().any(|e| &e.id == id) {
                        Err(format!("duplicate id: {id}"))
                    } else {
                        Ok(())
                    }
                }
                TestOp::SetName { id, .. } => {
                    if !doc.entities.iter().any(|e| &e.id == id) {
                        Err(format!("entity not found: {id}"))
                    } else {
                        Ok(())
                    }
                }
                TestOp::DeleteEntity { id } => {
                    if !doc.entities.iter().any(|e| &e.id == id) {
                        Err(format!("entity not found: {id}"))
                    } else {
                        Ok(())
                    }
                }
            }
        }

        fn apply(
            &self,
            doc: &mut Self::Document,
            op: &Self::Operation,
        ) -> Result<Self::Operation, Self::Error> {
            match op.clone() {
                TestOp::CreateEntity { id, name } => {
                    doc.entities.push(TestEntity {
                        id: id.clone(),
                        name: name.clone(),
                    });
                    // TRUE inverse: creating is undone by deleting.
                    Ok(TestOp::DeleteEntity { id })
                }
                TestOp::SetName { id, new_name } => {
                    let entity = doc.entities.iter_mut().find(|e| e.id == *id).unwrap();
                    let old_name = entity.name.clone();
                    entity.name = new_name;
                    Ok(TestOp::SetName {
                        id,
                        new_name: old_name,
                    })
                }
                TestOp::DeleteEntity { id } => {
                    let entity = doc.entities.iter().find(|e| e.id == *id).unwrap();
                    let name = entity.name.clone();
                    doc.entities.retain(|e| e.id != *id);
                    Ok(TestOp::CreateEntity { id, name })
                }
            }
        }

        fn summarize(
            &self,
            _doc: &Self::Document,
            ops: &[Self::Operation],
        ) -> (EffectsSummary, DiffSummary) {
            let added = ops
                .iter()
                .filter(|o| matches!(o, TestOp::CreateEntity { .. }))
                .count() as u64;
            let modified = ops
                .iter()
                .filter(|o| matches!(o, TestOp::SetName { .. }))
                .count() as u64;
            (
                EffectsSummary::empty(),
                DiffSummary {
                    added,
                    removed: 0,
                    modified,
                    notes: Vec::new(),
                },
            )
        }
    }

    /// Applier whose preflight ALWAYS succeeds and whose `apply` fails on a
    /// chosen INVOCATION COUNT (not on a specific op). The kernel's preflight
    /// is a dry-run simulation, so any op that deterministically fails is
    /// classified `Preflight` and the rollback branch stays unreached. A
    /// non-deterministic applier (fails only on the Nth apply invocation —
    /// e.g. I/O hiccup, time-dependent logic) is what exercises the real
    /// rollback path.
    struct FlakyApplier {
        fail_at_invocation: usize,
        invocations: std::cell::Cell<usize>,
    }

    impl FlakyApplier {
        /// Build an applier that fails the `fail_at_invocation`-th apply call
        /// (1-based) and succeeds everywhere else.
        fn failing_at(fail_at_invocation: usize) -> Self {
            Self {
                fail_at_invocation,
                invocations: std::cell::Cell::new(0),
            }
        }
    }

    impl Applier for FlakyApplier {
        type Operation = TestOp;
        type Document = TestDoc;
        type Error = String;

        fn preflight(
            &self,
            _doc: &Self::Document,
            _op: &Self::Operation,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn apply(
            &self,
            doc: &mut Self::Document,
            op: &Self::Operation,
        ) -> Result<Self::Operation, Self::Error> {
            let n = self.invocations.get() + 1;
            self.invocations.set(n);
            if n == self.fail_at_invocation {
                return Err(format!("injected apply-time failure (invocation {n})"));
            }
            TestApplier.apply(doc, op)
        }

        fn summarize(
            &self,
            doc: &Self::Document,
            ops: &[Self::Operation],
        ) -> (EffectsSummary, DiffSummary) {
            TestApplier.summarize(doc, ops)
        }
    }

    fn make_history() -> super::super::session::HistoryScope {
        super::super::session::HistoryScope::new()
    }

    // Test 1: Simple 2-op ChangeSet applies atomically, revision bumped
    #[test]
    fn test_atomic_apply_bumps_revision() {
        let kernel = TransactionKernel::new(TestApplier);
        let mut doc = TestDoc {
            entities: Vec::new(),
        };
        let mut history = make_history();

        let mut cs = ChangeSet::new("cs-1", ChangeOrigin::Human, "test", "create then rename");
        cs.push_op(TestOp::CreateEntity {
            id: "ent-1".into(),
            name: "Alice".into(),
        });
        cs.push_op(TestOp::SetName {
            id: "ent-1".into(),
            new_name: "Bob".into(),
        });

        let receipt = kernel.apply_atomic(&cs, &mut doc, &mut history).unwrap();

        assert_eq!(receipt.change_id, "cs-1");
        assert_eq!(receipt.revision, 1);
        assert_eq!(doc.entities.len(), 1);
        assert_eq!(doc.entities[0].name, "Bob");
        // 2 inverses in reverse order
        assert_eq!(receipt.inverses.len(), 2);
    }

    // Test 2: Preflight failure → document untouched (nothing was applied).
    #[test]
    fn test_failed_op_rolls_back() {
        let kernel = TransactionKernel::new(TestApplier);
        let mut doc = TestDoc {
            entities: vec![TestEntity {
                id: "ent-1".into(),
                name: "Original".into(),
            }],
        };
        let original_doc = doc.clone();
        let mut history = make_history();

        let mut cs = ChangeSet::new("cs-2", ChangeOrigin::Human, "test", "duplicate id");
        cs.push_op(TestOp::CreateEntity {
            id: "new-ent".into(),
            name: "NewEntity".into(),
        });
        // Same id → preflight fails before apply
        cs.push_op(TestOp::CreateEntity {
            id: "new-ent".into(),
            name: "DuplicateEntity".into(),
        });

        let err: KernelError<String> = kernel
            .apply_atomic(&cs, &mut doc, &mut history)
            .unwrap_err();

        // Preflight fails at op 1 (duplicate id)
        assert!(matches!(err, KernelError::Preflight(_)));
        // Document should be back to original (nothing was applied)
        assert_eq!(doc.entities.len(), original_doc.entities.len());
        assert_eq!(doc.entities[0].name, original_doc.entities[0].name);
        assert_eq!(doc.entities[0].id, original_doc.entities[0].id);
        // Revision not bumped on failure
        assert_eq!(history.revision(), 0);
    }

    // Test 2b: TRUE apply-time rollback. The kernel preflights via dry-run
    // simulation, so deterministic failures surface as `Preflight`. To reach
    // the rollback branch we need apply to diverge between the simulated and
    // the real pass: FlakyApplier fails on invocation #4 — the kernel's apply
    // sequence is simulate-op0 (#1), simulate-op1 (#2), real-op0 (#3),
    // real-op1 (#4 → FAIL). Op 0 already applied with a collected inverse, so
    // the kernel must apply that inverse in reverse and restore the document.
    #[test]
    fn test_apply_time_failure_rolls_back_applied_ops() {
        let kernel = TransactionKernel::new(FlakyApplier::failing_at(4));
        let mut doc = TestDoc {
            entities: Vec::new(),
        };
        let mut history = make_history();

        let mut cs = ChangeSet::new("cs-2b", ChangeOrigin::Human, "test", "apply-time failure");
        // Op 0: applies successfully in both passes; inverse = DeleteEntity("ent-a").
        cs.push_op(TestOp::CreateEntity {
            id: "ent-a".into(),
            name: "A".into(),
        });
        // Op 1: passes the simulated preflight, fails on the real apply.
        cs.push_op(TestOp::SetName {
            id: "ent-a".into(),
            new_name: "Boom".into(),
        });

        let err: KernelError<String> = kernel
            .apply_atomic(&cs, &mut doc, &mut history)
            .unwrap_err();

        match &err {
            KernelError::ApplyFailed { index, .. } => assert_eq!(*index, 1),
            other => panic!("expected ApplyFailed, got {other:?}"),
        }
        // Rollback restored the document to the pre-change state: the entity
        // created by op 0 was removed by its inverse during rollback.
        assert!(doc.entities.is_empty(), "rollback must undo op 0");
        // History untouched — the change never committed.
        assert_eq!(history.revision(), 0);
        assert!(history.last_change().is_none());
    }

    // Test 3: RequiresHuman policy without approval → ApprovalRequired
    #[test]
    fn test_requires_human_unapproved_fails() {
        let kernel = TransactionKernel::new(TestApplier);
        let mut doc = TestDoc {
            entities: Vec::new(),
        };
        let mut history = make_history();

        let mut cs = ChangeSet::new("cs-3", ChangeOrigin::Agent, "ai-agent", "agent proposal");
        cs.set_approval(ApprovalPolicy::RequiresHuman {
            approver_hint: None,
        });
        cs.push_op(TestOp::CreateEntity {
            id: "ent-x".into(),
            name: "X".into(),
        });

        let err: KernelError<String> = kernel
            .apply_atomic(&cs, &mut doc, &mut history)
            .unwrap_err();
        assert!(matches!(err, KernelError::ApprovalRequired));
        // Document untouched
        assert!(doc.entities.is_empty());
        assert_eq!(history.revision(), 0);
    }

    // Test 4: RequiresHuman policy with prior approval → succeeds
    #[test]
    fn test_requires_human_approved_succeeds() {
        let kernel = TransactionKernel::new(TestApplier);
        let mut doc = TestDoc {
            entities: Vec::new(),
        };
        let mut history = make_history();

        let mut cs = ChangeSet::new("cs-4", ChangeOrigin::Agent, "ai-agent", "agent proposal");
        cs.set_approval(ApprovalPolicy::RequiresHuman {
            approver_hint: None,
        });
        cs.approve(); // Explicit approval
        cs.push_op(TestOp::CreateEntity {
            id: "ent-y".into(),
            name: "Y".into(),
        });

        let receipt = kernel.apply_atomic(&cs, &mut doc, &mut history).unwrap();

        assert_eq!(receipt.change_id, "cs-4");
        assert_eq!(doc.entities.len(), 1);
        assert_eq!(history.revision(), 1);
    }

    // Test 5: Validate returns errors for duplicate id
    #[test]
    fn test_validate_catches_duplicate() {
        let kernel = TransactionKernel::new(TestApplier);
        let doc = TestDoc {
            entities: vec![TestEntity {
                id: "ent-1".into(),
                name: "Existing".into(),
            }],
        };

        let mut cs = ChangeSet::new("cs-5", ChangeOrigin::Human, "test", "duplicate id");
        cs.push_op(TestOp::CreateEntity {
            id: "ent-1".into(),
            name: "Duplicate".into(),
        });

        let report = kernel.validate(&cs, &doc);
        assert!(!report.is_ok());
        assert!(report.errors[0].contains("duplicate id"));
    }
}
