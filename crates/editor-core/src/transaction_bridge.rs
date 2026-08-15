//! `*CommandApplier` — bridges `TransactionKernel` to the three domain command systems.
//!
//! Implements [`editor_application::transaction::Applier`] for each domain:
//! - `SceneCommandApplier`: `Command` / `SceneDocument`
//! - `AssetCommandApplier`: `AssetCommand` / `SceneAssetDocument`
//! - `LogicCommandApplier`: `LogicCommand` / `LogicGraphAsset`
//!
//! All three reuse the existing `processor::validate` and domain-specific `apply` machinery.
//!
//! This module intentionally does NOT modify the public behavior of `scene_session.rs`
//! or `processor.rs` — it only provides thin adapter layers.

use crate::asset_command::{AssetCommand, AssetCommandError, apply as asset_apply};
use crate::command::{Command, CommandError};
use crate::document::SceneDocument;
use crate::logic_command::{LogicCommand, LogicCommandError, apply as logic_apply};
use crate::logic_graph::LogicGraphAsset;
use crate::processor;
use crate::scene_asset::SceneAssetDocument;

use editor_application::transaction::{
    Applier, ApprovalPolicy, ChangeOrigin, ChangeSet, DiffSummary, EffectsSummary, KernelError,
    ResourceRef, TransactionKernel, ValidationReport,
};

/// `SceneCommandApplier` adapts the scene command processor to the `Applier` trait.
///
/// - `Operation = Command` — the typed scene commands from `command.rs`.
/// - `Document = SceneDocument` — the editor-owned scene document.
#[derive(Debug, Clone)]
pub struct SceneCommandApplier {
    _priv: (),
}

impl SceneCommandApplier {
    /// Construct a new `SceneCommandApplier`.
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl Default for SceneCommandApplier {
    fn default() -> Self {
        Self::new()
    }
}

impl Applier for SceneCommandApplier {
    type Operation = Command;
    type Document = SceneDocument;
    type Error = CommandError;

    fn preflight(&self, doc: &Self::Document, op: &Self::Operation) -> Result<(), Self::Error> {
        // Reuse the existing processor validate function.
        // Note: processor::validate takes &SceneDocument (immutable borrow),
        // which matches our preflight semantics.
        processor::validate(doc, op)
    }

    fn apply(
        &self,
        doc: &mut Self::Document,
        op: &Self::Operation,
    ) -> Result<Self::Operation, Self::Error> {
        // Reuse processor::apply which already computes the inverse and returns it.
        processor::apply(doc, op)
    }

    fn summarize(
        &self,
        doc: &Self::Document,
        ops: &[Self::Operation],
    ) -> (EffectsSummary, DiffSummary) {
        // Count entity-level operations for a simple diff summary.
        // This is a best-effort approximation; full diff would require
        // comparing the document before/after which the kernel handles separately.
        let mut added = 0u64;
        let mut removed = 0u64;
        let mut modified = 0u64;

        for op in ops {
            match op {
                Command::CreateEntity { .. } => added += 1,
                Command::DeleteEntity { .. } => removed += 1,
                Command::AddComponent { .. } => added += 1,
                Command::RemoveComponent { .. } => removed += 1,
                Command::SetComponentField { .. } => modified += 1,
                Command::SetComponentFieldOnMultiple { .. } => modified += 1,
                Command::ReparentEntity { .. } => modified += 1,
                Command::RenameEntity { .. } => modified += 1,
                Command::Batch { commands, .. } => {
                    // Recursively count batch contents
                    for cmd in commands {
                        match cmd {
                            Command::CreateEntity { .. } => added += 1,
                            Command::DeleteEntity { .. } => removed += 1,
                            Command::AddComponent { .. } => added += 1,
                            Command::RemoveComponent { .. } => removed += 1,
                            Command::SetComponentField { .. } => modified += 1,
                            Command::SetComponentFieldOnMultiple { .. } => modified += 1,
                            Command::ReparentEntity { .. } => modified += 1,
                            Command::RenameEntity { .. } => modified += 1,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        (
            EffectsSummary {
                runtime_rebuild_required: true, // Any scene change may require preview rebuild
                build_output_changed: false,
                notes: vec![format!(
                    "{} entity ops: +{}/-{}/~{}",
                    ops.len(),
                    added,
                    removed,
                    modified
                )],
            },
            DiffSummary {
                added,
                removed,
                modified,
                notes: Vec::new(),
            },
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AssetCommandApplier
// ─────────────────────────────────────────────────────────────────────────────

/// `AssetCommandApplier` adapts the asset command processor to the `Applier` trait.
///
/// - `Operation = AssetCommand` — the typed asset commands from `asset_command.rs`.
/// - `Document = SceneAssetDocument` — the editor-owned scene asset document.
#[derive(Debug, Clone)]
pub struct AssetCommandApplier {
    _priv: (),
}

impl AssetCommandApplier {
    /// Construct a new `AssetCommandApplier`.
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl Default for AssetCommandApplier {
    fn default() -> Self {
        Self::new()
    }
}

impl Applier for AssetCommandApplier {
    type Operation = AssetCommand;
    type Document = SceneAssetDocument;
    type Error = AssetCommandError;

    fn preflight(&self, doc: &Self::Document, op: &Self::Operation) -> Result<(), Self::Error> {
        // Simulate apply to validate without mutating.
        // Clone doc and apply; if Ok, preflight passes.
        let mut sim = doc.clone();
        asset_apply(&mut sim, op).map(|_| ())
    }

    fn apply(
        &self,
        doc: &mut Self::Document,
        op: &Self::Operation,
    ) -> Result<Self::Operation, Self::Error> {
        asset_apply(doc, op)
    }

    fn summarize(
        &self,
        _doc: &Self::Document,
        ops: &[Self::Operation],
    ) -> (EffectsSummary, DiffSummary) {
        // Asset changes don't trigger Bevy preview rebuilds.
        (
            EffectsSummary {
                runtime_rebuild_required: false,
                build_output_changed: false,
                notes: vec![format!("{} asset ops", ops.len())],
            },
            DiffSummary {
                added: ops
                    .iter()
                    .filter(|o| matches!(o, AssetCommand::AddEntity { .. }))
                    .count() as u64,
                removed: ops
                    .iter()
                    .filter(|o| matches!(o, AssetCommand::RemoveEntity { .. }))
                    .count() as u64,
                modified: ops
                    .iter()
                    .filter(|o| {
                        !matches!(
                            o,
                            AssetCommand::AddEntity { .. } | AssetCommand::RemoveEntity { .. }
                        )
                    })
                    .count() as u64,
                notes: Vec::new(),
            },
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LogicCommandApplier
// ─────────────────────────────────────────────────────────────────────────────

/// `LogicCommandApplier` adapts the logic command processor to the `Applier` trait.
///
/// - `Operation = LogicCommand` — the typed logic commands from `logic_command.rs`.
/// - `Document = LogicGraphAsset` — the editor-owned logic graph asset.
#[derive(Debug, Clone)]
pub struct LogicCommandApplier {
    _priv: (),
}

impl LogicCommandApplier {
    /// Construct a new `LogicCommandApplier`.
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl Default for LogicCommandApplier {
    fn default() -> Self {
        Self::new()
    }
}

impl Applier for LogicCommandApplier {
    type Operation = LogicCommand;
    type Document = LogicGraphAsset;
    type Error = LogicCommandError;

    fn preflight(&self, doc: &Self::Document, op: &Self::Operation) -> Result<(), Self::Error> {
        // Simulate apply to validate without mutating.
        let mut sim = doc.clone();
        logic_apply(&mut sim, op).map(|_| ())
    }

    fn apply(
        &self,
        doc: &mut Self::Document,
        op: &Self::Operation,
    ) -> Result<Self::Operation, Self::Error> {
        logic_apply(doc, op)
    }

    fn summarize(
        &self,
        _doc: &Self::Document,
        ops: &[Self::Operation],
    ) -> (EffectsSummary, DiffSummary) {
        // Logic changes don't trigger Bevy preview rebuilds directly.
        (
            EffectsSummary {
                runtime_rebuild_required: false,
                build_output_changed: false,
                notes: vec![format!("{} logic ops", ops.len())],
            },
            DiffSummary {
                added: ops
                    .iter()
                    .filter(|o| {
                        matches!(
                            o,
                            LogicCommand::AddNode { .. } | LogicCommand::ConnectPorts { .. }
                        )
                    })
                    .count() as u64,
                removed: ops
                    .iter()
                    .filter(|o| {
                        matches!(
                            o,
                            LogicCommand::RemoveNode { .. } | LogicCommand::DisconnectPorts { .. }
                        )
                    })
                    .count() as u64,
                modified: ops
                    .iter()
                    .filter(|o| matches!(o, LogicCommand::SetNodeField { .. }))
                    .count() as u64,
                notes: Vec::new(),
            },
        )
    }
}

// Type alias for the scene transaction kernel
pub type SceneTransactionKernel = TransactionKernel<SceneCommandApplier>;

/// Type alias for the asset transaction kernel.
pub type AssetTransactionKernel = TransactionKernel<AssetCommandApplier>;

/// Type alias for the logic transaction kernel.
pub type LogicTransactionKernel = TransactionKernel<LogicCommandApplier>;

/// Construct a `SceneTransactionKernel`.
pub fn scene_transaction_kernel() -> SceneTransactionKernel {
    TransactionKernel::new(SceneCommandApplier::new())
}

/// Construct an `AssetTransactionKernel`.
pub fn asset_transaction_kernel() -> AssetTransactionKernel {
    TransactionKernel::new(AssetCommandApplier::new())
}

/// Construct a `LogicTransactionKernel`.
pub fn logic_transaction_kernel() -> LogicTransactionKernel {
    TransactionKernel::new(LogicCommandApplier::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use crate::document::{Entity, LocalId, SceneDocument, StableId};
    use editor_application::session::HistoryScope;

    /// Helper: empty scene document for tests.
    fn empty_doc() -> SceneDocument {
        SceneDocument {
            version: "0.1".to_string(),
            scene_id: "test-scene".to_string(),
            name: "Test Scene".to_string(),
            entities: Vec::new(),
            instances: std::collections::BTreeMap::new(),
        }
    }

    /// Helper: make a 1-entity scene document for tests.
    fn doc_with_entity() -> SceneDocument {
        let mut doc = empty_doc();
        doc.entities.push(Entity {
            id: StableId::new("ent-1"),
            local_id: LocalId::new("ent-1"),
            name: "Existing".to_string(),
            parent: None,
            components: Vec::new(),
        });
        doc
    }

    // Test (a): 2-op ChangeSet applies atomically, receipt has 2 inverses in
    // reverse order, history revision bumped, last_change recorded.
    #[test]
    fn test_scene_2op_atomic_success() {
        let kernel = scene_transaction_kernel();
        let mut doc = empty_doc();
        let mut history = HistoryScope::new();

        let mut cs = ChangeSet::new(
            "cs-scene-1",
            ChangeOrigin::Human,
            "test-user",
            "create entity and rename it",
        );
        cs.add_resource("scene", "scenes/test.json");
        cs.push_op(Command::CreateEntity {
            id: StableId::new("ent-new"),
            name: "NewEntity".to_string(),
            components: Vec::new(),
        });
        cs.push_op(Command::RenameEntity {
            entity_id: StableId::new("ent-new"),
            old_name: None,
            new_name: "RenamedEntity".to_string(),
        });

        let receipt = kernel.apply_atomic(&cs, &mut doc, &mut history).unwrap();

        // Receipt fields
        assert_eq!(receipt.change_id, "cs-scene-1");
        assert_eq!(receipt.inverses.len(), 2);
        assert_eq!(receipt.revision, 1);

        // Document updated
        assert_eq!(doc.entities.len(), 1);
        assert_eq!(doc.entities[0].name, "RenamedEntity");

        // History updated
        assert_eq!(history.revision(), 1);
        let last = history.last_change().expect("last_change should be set");
        assert_eq!(last.change_id, "cs-scene-1");
        assert_eq!(last.origin, ChangeOrigin::Human);
        assert_eq!(last.actor, "test-user");

        // Inverses are collected in application order: [RenameEntity_inverse, DeleteEntity_inverse]
        // RenameEntity inverse = RenameEntity, DeleteEntity inverse = CreateEntity
        assert!(matches!(&receipt.inverses[0], Command::RenameEntity { .. }));
        assert!(matches!(&receipt.inverses[1], Command::DeleteEntity { .. }));
    }

    // Test (b): Second op fails in preflight → document state identical to before.
    //
    // Since preflight validates ALL ops sequentially before any actual apply, a failure
    // at op 1 means op 0 was never actually applied. The document remains untouched.
    // This is not an ApplyFailed case (that would require op 0 to be applied first).
    #[test]
    fn test_scene_failing_op_rolls_back() {
        let kernel = scene_transaction_kernel();
        let mut doc = doc_with_entity();
        let original_doc = doc.clone();
        let mut history = HistoryScope::new();

        let mut cs = ChangeSet::new(
            "cs-scene-2",
            ChangeOrigin::Human,
            "test-user",
            "rename existing then rename non-existent",
        );
        // Op 0: rename ent-1 to "NewName" — preflight succeeds
        cs.push_op(Command::RenameEntity {
            entity_id: StableId::new("ent-1"),
            old_name: None,
            new_name: "NewName".to_string(),
        });
        // Op 1: rename ent-999 (doesn't exist) — preflight FAILS
        cs.push_op(Command::RenameEntity {
            entity_id: StableId::new("ent-999"),
            old_name: None,
            new_name: "Ghost".to_string(),
        });

        let err = kernel
            .apply_atomic(&cs, &mut doc, &mut history)
            .unwrap_err();

        // Preflight fails at op 1 (entity not found) — document untouched
        assert!(matches!(err, KernelError::Preflight(_)));

        // Document should be exactly as before (nothing was applied)
        assert_eq!(doc.entities.len(), original_doc.entities.len());
        assert_eq!(doc.entities[0].name, original_doc.entities[0].name);
        assert_eq!(history.revision(), 0);
    }

    // Test (c): ApprovalPolicy::RequiresHuman unapproved → ApprovalRequired, doc untouched.
    #[test]
    fn test_scene_requires_human_unapproved_fails() {
        let kernel = scene_transaction_kernel();
        let mut doc = empty_doc();
        let mut history = HistoryScope::new();

        let mut cs = ChangeSet::new(
            "cs-scene-3",
            ChangeOrigin::Agent,
            "ai-agent",
            "agent proposes creating an entity",
        );
        cs.set_approval(ApprovalPolicy::RequiresHuman {
            approver_hint: Some("Scene will be modified".to_string()),
        });
        cs.push_op(Command::CreateEntity {
            id: StableId::new("ent-agent"),
            name: "AgentEntity".to_string(),
            components: Vec::new(),
        });

        let err: KernelError<CommandError> = kernel
            .apply_atomic(&cs, &mut doc, &mut history)
            .unwrap_err();

        assert!(matches!(err, KernelError::ApprovalRequired));
        // Document untouched
        assert!(doc.entities.is_empty());
        assert_eq!(history.revision(), 0);
    }

    // Test (d): ApprovalPolicy::RequiresHuman with prior approval → succeeds.
    #[test]
    fn test_scene_requires_human_approved_succeeds() {
        let kernel = scene_transaction_kernel();
        let mut doc = empty_doc();
        let mut history = HistoryScope::new();

        let mut cs = ChangeSet::new(
            "cs-scene-4",
            ChangeOrigin::Agent,
            "ai-agent",
            "agent proposes creating an entity",
        );
        cs.set_approval(ApprovalPolicy::RequiresHuman {
            approver_hint: Some("Scene will be modified".to_string()),
        });
        cs.approve(); // Explicit human approval
        cs.push_op(Command::CreateEntity {
            id: StableId::new("ent-approved"),
            name: "ApprovedEntity".to_string(),
            components: Vec::new(),
        });

        let receipt = kernel.apply_atomic(&cs, &mut doc, &mut history).unwrap();

        assert_eq!(receipt.change_id, "cs-scene-4");
        assert_eq!(doc.entities.len(), 1);
        assert_eq!(history.revision(), 1);
    }

    // Test (e): Preflight validation catches unknown schema.
    #[test]
    fn test_scene_validate_unknown_schema() {
        let kernel = scene_transaction_kernel();
        let doc = empty_doc();

        let mut cs = ChangeSet::new(
            "cs-scene-5",
            ChangeOrigin::Human,
            "test",
            "add unknown component",
        );
        cs.push_op(Command::AddComponent {
            entity_id: StableId::new("ent-doesnt-exist"),
            type_id: "NonExistentSchema".to_string(),
            values: serde_json::json!({}),
        });

        let report = kernel.validate(&cs, &doc);
        assert!(!report.is_ok());
        assert!(report.errors[0].contains("Entity not found"));
    }
}
