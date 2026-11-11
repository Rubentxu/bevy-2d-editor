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

use editor_model::command::Command;

// ─────────────────────────────────────────────────────────────────────────────
// Extension permission checking (ADR-0040 — v0.92)
// ─────────────────────────────────────────────────────────────────────────────

/// Check whether a Plugin-origin ChangeSet has the required permissions.
///
/// Returns `Ok(())` if the extension is allowed to proceed. Returns
/// `Err(KernelError::PermissionDenied)` if:
/// - The extension ID cannot be parsed from the actor string
/// - The extension is not registered
/// - The manifest does not grant the required permission for the affected resources
///
/// This is a no-op for non-Plugin origins (Human, Agent, Recipe, Importer,
/// Migration, RuntimeApplyBack).
///
/// # Arguments
///
/// * `cs` — The ChangeSet to check
pub fn transaction_kernel_check_plugin_permission<O>(
    cs: &ChangeSet<O>,
) -> Result<(), KernelError<std::convert::Infallible>>
where
    O: std::fmt::Debug + Clone,
{
    // Fast path: only check Plugin origins
    if !matches!(cs.origin, ChangeOrigin::Plugin) {
        return Ok(());
    }

    // Extract extension ID from actor string ("extension:<id>" convention).
    let extension_id =
        cs.actor
            .strip_prefix("extension:")
            .ok_or_else(|| KernelError::PermissionDenied {
                extension: cs.actor.clone(),
                area: "unknown".to_string(),
                scope_needed: "extension: prefix required".to_string(),
                scope_granted: "none".to_string(),
            })?;

    // Look up the extension manifest from the global registry.
    let registry = editor_model::ports::with_extension_registry().ok_or_else(|| {
        KernelError::PermissionDenied {
            extension: extension_id.to_string(),
            area: "extension registry".to_string(),
            scope_needed: "registry available".to_string(),
            scope_granted: "not initialized".to_string(),
        }
    })?;

    let registry_guard = registry.lock().map_err(|_| KernelError::PermissionDenied {
        extension: extension_id.to_string(),
        area: "extension registry".to_string(),
        scope_needed: "lock acquisition".to_string(),
        scope_granted: "lock poisoned".to_string(),
    })?;

    let manifest =
        registry_guard
            .get(extension_id)
            .ok_or_else(|| KernelError::PermissionDenied {
                extension: extension_id.to_string(),
                area: "extension manifest".to_string(),
                scope_needed: "registered".to_string(),
                scope_granted: "not found".to_string(),
            })?;

    // Check permissions for each resource affected by the ChangeSet.
    // For each resource, we check if the manifest grants Read/Write/Propose
    // permission for the appropriate area.
    for resource in (*cs.resources()).iter() {
        let (area_str, scope_needed_str) = match resource {
            ResourceRef::Scene(_) => {
                // Scene mutations are Commands area with Propose scope
                // (commands are always Propose in v0.92)
                ("Commands", "Propose")
            }
            ResourceRef::SceneAsset(_) => {
                // Asset mutations are AssetProcessors area
                ("AssetProcessors", "Write")
            }
            ResourceRef::LogicGraph(_) => {
                // Logic graph mutations are Commands area
                ("Commands", "Propose")
            }
            ResourceRef::Project(_) => {
                // Project-level resources use Project area
                ("Project", "Read")
            }
        };

        // Map area string to PermissionArea enum
        let area = match area_str {
            "Commands" => editor_model::extension::PermissionArea::Commands,
            "AssetProcessors" => editor_model::extension::PermissionArea::AssetProcessors,
            "Project" => editor_model::extension::PermissionArea::Project,
            other => {
                return Err(KernelError::PermissionDenied {
                    extension: extension_id.to_string(),
                    area: other.to_string(),
                    scope_needed: scope_needed_str.to_string(),
                    scope_granted: "n/a".to_string(),
                });
            }
        };

        // Map scope string to PermissionScope enum
        let scope_needed = match scope_needed_str {
            "Read" => editor_model::extension::PermissionScope::Read,
            "Write" => editor_model::extension::PermissionScope::Write,
            "Propose" => editor_model::extension::PermissionScope::Propose,
            other => {
                return Err(KernelError::PermissionDenied {
                    extension: extension_id.to_string(),
                    area: area_str.to_string(),
                    scope_needed: other.to_string(),
                    scope_granted: "n/a".to_string(),
                });
            }
        };

        // Check: manifest grants scope_needed or higher
        let granted = manifest.has_permission(&area, &scope_needed);
        if !granted {
            // Also check if they have a broader scope (Write covers Read/Propose, etc.)
            let broader_scope = match scope_needed {
                editor_model::extension::PermissionScope::Read => {
                    Some(editor_model::extension::PermissionScope::Write)
                }
                editor_model::extension::PermissionScope::Propose => {
                    Some(editor_model::extension::PermissionScope::Write)
                }
                editor_model::extension::PermissionScope::Write => None,
                editor_model::extension::PermissionScope::Subscribe => None,
                _ => None, // #[non_exhaustive] catch-all
            };

            let granted_broader = broader_scope
                .and_then(|s| Some(manifest.has_permission(&area, &s)))
                .unwrap_or(false);

            if !granted_broader {
                return Err(KernelError::PermissionDenied {
                    extension: extension_id.to_string(),
                    area: area_str.to_string(),
                    scope_needed: scope_needed_str.to_string(),
                    scope_granted: "none".to_string(),
                });
            }
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Importer permission checking (ADR-0040 step 3 + ADR-0041 — v0.93)
// ─────────────────────────────────────────────────────────────────────────────

/// Check whether an Importer-origin ChangeSet comes from a registered importer.
///
/// Per decision #1 (ADR-0041): "ANY `Capability::Importers` can import — no separate
/// Extension registration required." The importer declares `Capability::Importers` at
/// registration time and is checked here against the `ImporterRegistryPort`.
///
/// Returns `Ok(())` if the importer is registered. Returns
/// `Err(KernelError::PermissionDenied)` if:
/// - The importer ID cannot be parsed from the actor string (`importer:<id>`)
/// - The importer is not found in the registry
///
/// This is a no-op for non-Importer origins (Human, Agent, Recipe, Plugin,
/// Migration, RuntimeApplyBack).
///
/// # Arguments
///
/// * `cs` — The ChangeSet to check
pub fn transaction_kernel_check_importer_permission<O>(
    cs: &ChangeSet<O>,
) -> Result<(), KernelError<std::convert::Infallible>>
where
    O: std::fmt::Debug + Clone,
{
    // Fast path: only check Importer origins
    if !matches!(cs.origin, ChangeOrigin::Importer) {
        return Ok(());
    }

    // Extract importer ID from actor string ("importer:<id>" convention).
    let importer_id =
        cs.actor
            .strip_prefix("importer:")
            .ok_or_else(|| KernelError::PermissionDenied {
                extension: cs.actor.clone(),
                area: "unknown".to_string(),
                scope_needed: "importer: prefix required".to_string(),
                scope_granted: "none".to_string(),
            })?;

    // Look up the importer from the global registry.
    let registry = editor_model::ports::with_importer_registry().ok_or_else(|| {
        KernelError::PermissionDenied {
            extension: importer_id.to_string(),
            area: "importer registry".to_string(),
            scope_needed: "registry available".to_string(),
            scope_granted: "not initialized".to_string(),
        }
    })?;

    let registry_guard = registry.lock().map_err(|_| KernelError::PermissionDenied {
        extension: importer_id.to_string(),
        area: "importer registry".to_string(),
        scope_needed: "lock acquisition".to_string(),
        scope_granted: "lock poisoned".to_string(),
    })?;

    // Check: importer must be registered (is_registered returns true for known IDs)
    let _importer = registry_guard
        .is_registered(importer_id)
        .then_some(())
        .ok_or_else(|| KernelError::PermissionDenied {
            extension: importer_id.to_string(),
            area: "importer registry".to_string(),
            scope_needed: "registered".to_string(),
            scope_granted: "not found".to_string(),
        })?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Resource derivation from real ops (H0.02 — ADR-0040)
// ─────────────────────────────────────────────────────────────────────────────

/// Placeholder scene identity for resources derived from pending ops.
///
/// ADR-0040 manifests gate permission by resource *kind* (the kernel maps
/// kind → area/scope and never inspects the payload), so scene ops derived
/// from a pending ChangeSet all refer to the scene document the ChangeWorkbench
/// operates on. The payload is deliberately a placeholder — wiring it to real
/// scene paths would be a policy change, not a derivation fix.
pub const SCENE_DOC_PLACEHOLDER: &str = "active-scene";

/// Error deriving affected resources for permission gating (H0.02).
///
/// Not a `CommandError`: this is an application-layer permission policy
/// verdict, not an apply-path contract.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResourceDerivationError {
    /// A command kind without a sanctioned permission mapping was proposed
    /// through the plugin approval flow (fail-closed). New `Command` variants
    /// must extend the derivation before plugins can propose them.
    #[error("command kind has no sanctioned permission mapping (H0.02): {kind}")]
    UnmappedCommandKind {
        /// Serde-ish tag of the offending `Command` variant.
        kind: &'static str,
    },
}

/// Derive the [`ResourceRef`] set affected by the given commands.
///
/// Used to build the resource list of a pending ChangeSet so the plugin
/// permission check runs against the real approved operations (H0.02 AC#2)
/// instead of an empty temporary ChangeSet, which bypassed the gate.
///
/// Fail-closed: a command kind without a sanctioned mapping is an error.
pub fn derive_resources_from_commands(
    commands: &[Command],
) -> Result<Vec<ResourceRef>, ResourceDerivationError> {
    fn scene_resource() -> ResourceRef {
        ResourceRef::Scene(SCENE_DOC_PLACEHOLDER.to_string())
    }

    fn visit(cmd: &Command, out: &mut Vec<ResourceRef>) -> Result<(), ResourceDerivationError> {
        match cmd {
            // No mutation — nothing to gate.
            Command::Noop {} => Ok(()),
            // Scene document mutations.
            Command::CreateEntity { .. }
            | Command::DeleteEntity { .. }
            | Command::AddComponent { .. }
            | Command::RemoveComponent { .. }
            | Command::SetComponentField { .. }
            | Command::SetComponentFieldOnMultiple { .. }
            | Command::ReparentEntity { .. }
            | Command::RenameEntity { .. }
            | Command::PlaceInstance { .. }
            | Command::RemoveInstance { .. }
            | Command::ReplaceInstanceAsset { .. }
            | Command::UpsertOverride { .. }
            | Command::RevertOverride { .. } => {
                let r = scene_resource();
                if !out.contains(&r) {
                    out.push(r);
                }
                Ok(())
            }
            // Batch mutates whatever its children mutate.
            Command::Batch { commands, .. } => {
                for c in commands {
                    visit(c, out)?;
                }
                Ok(())
            }
            // No sanctioned mapping yet: fail closed rather than letting an
            // unknown mutation surface skip the permission gate.
            Command::CreateSceneComponent { .. } => {
                Err(ResourceDerivationError::UnmappedCommandKind {
                    kind: "CreateSceneComponent",
                })
            }
            Command::UpdateSceneComponentFields { .. } => {
                Err(ResourceDerivationError::UnmappedCommandKind {
                    kind: "UpdateSceneComponentFields",
                })
            }
            Command::BindSceneToSchema { .. } => {
                Err(ResourceDerivationError::UnmappedCommandKind {
                    kind: "BindSceneToSchema",
                })
            }
            Command::World(_) => {
                Err(ResourceDerivationError::UnmappedCommandKind { kind: "World" })
            }
        }
    }

    let mut resources = Vec::new();
    for c in commands {
        visit(c, &mut resources)?;
    }
    Ok(resources)
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_model::ports::{ExtensionRegistryPort, register_extension_registry};
    use std::sync::{Arc, Mutex};

    fn make_test_registry() -> Arc<Mutex<dyn ExtensionRegistryPort>> {
        let registry = crate::extension::ExtensionRegistry::with_builtins();
        Arc::new(Mutex::new(registry)) as Arc<Mutex<dyn ExtensionRegistryPort>>
    }

    #[test]
    fn plugin_permission_check_no_op_for_human_origin() {
        let mut cs: ChangeSet<String> = ChangeSet::new(
            "cs1".into(),
            ChangeOrigin::Human,
            "user".into(),
            "test".into(),
        );
        cs.add_resource("scene", "test.json");
        assert!(transaction_kernel_check_plugin_permission(&cs).is_ok());
    }

    #[test]
    fn plugin_permission_check_no_op_for_agent_origin() {
        let mut cs: ChangeSet<String> = ChangeSet::new(
            "cs2".into(),
            ChangeOrigin::Agent,
            "agent:code-writer".into(),
            "test".into(),
        );
        cs.add_resource("scene", "test.json");
        assert!(transaction_kernel_check_plugin_permission(&cs).is_ok());
    }

    #[test]
    fn plugin_permission_denied_for_unknown_extension() {
        let mut cs: ChangeSet<String> = ChangeSet::new(
            "cs3".into(),
            ChangeOrigin::Plugin,
            "extension:com.unknown.extension".into(),
            "test".into(),
        );
        cs.add_resource("scene", "test.json");

        // Register empty registry (no unknown extension)
        let registry: Arc<Mutex<dyn ExtensionRegistryPort>> =
            Arc::new(Mutex::new(crate::extension::ExtensionRegistry::empty()));
        let _ = register_extension_registry(Arc::clone(&registry));

        let err = transaction_kernel_check_plugin_permission(&cs).unwrap_err();
        assert!(matches!(err, KernelError::PermissionDenied { .. }));
    }

    #[test]
    fn plugin_permission_allowed_for_builtin_with_correct_permission() {
        let mut cs: ChangeSet<String> = ChangeSet::new(
            "cs4".into(),
            ChangeOrigin::Plugin,
            "extension:builtin.logic-bricks.controllers".into(),
            "test".into(),
        );
        cs.add_resource("scene", "test.json");

        // Register the built-in registry
        let registry: Arc<Mutex<dyn ExtensionRegistryPort>> = make_test_registry();
        let _ = register_extension_registry(Arc::clone(&registry));

        assert!(transaction_kernel_check_plugin_permission(&cs).is_ok());
    }

    // ─── Importer permission tests (v0.93) ───────────────────────────────────────

    #[test]
    fn importer_permission_check_no_op_for_human_origin() {
        let mut cs: ChangeSet<String> = ChangeSet::new(
            "cs5".into(),
            ChangeOrigin::Human,
            "user".into(),
            "test".into(),
        );
        cs.add_resource("scene", "test.json");
        assert!(transaction_kernel_check_importer_permission(&cs).is_ok());
    }

    #[test]
    fn importer_permission_check_no_op_for_plugin_origin() {
        let mut cs: ChangeSet<String> = ChangeSet::new(
            "cs6".into(),
            ChangeOrigin::Plugin,
            "extension:builtin.test".into(),
            "test".into(),
        );
        cs.add_resource("scene", "test.json");
        assert!(transaction_kernel_check_importer_permission(&cs).is_ok());
    }

    #[test]
    fn importer_permission_denied_for_unknown_importer() {
        use editor_model::ports::{ImporterRegistryPort, register_importer_registry};

        let mut cs: ChangeSet<String> = ChangeSet::new(
            "cs7".into(),
            ChangeOrigin::Importer,
            "importer:builtin.unknown".into(),
            "test".into(),
        );
        cs.add_resource("scene", "test.json");

        // Register empty importer registry
        let registry: Arc<Mutex<dyn ImporterRegistryPort>> = Arc::new(Mutex::new(
            crate::importer_registry::ImporterRegistry::empty(),
        ));
        let _ = register_importer_registry(Arc::clone(&registry));

        let err = transaction_kernel_check_importer_permission(&cs).unwrap_err();
        assert!(matches!(err, KernelError::PermissionDenied { .. }));
    }

    #[test]
    fn importer_permission_denied_for_missing_importer_prefix() {
        let mut cs: ChangeSet<String> = ChangeSet::new(
            "cs8".into(),
            ChangeOrigin::Importer,
            "builtin.aseprite".into(), // missing "importer:" prefix
            "test".into(),
        );
        cs.add_resource("scene", "test.json");
        let err = transaction_kernel_check_importer_permission(&cs).unwrap_err();
        assert!(matches!(err, KernelError::PermissionDenied { .. }));
    }

    // ─── Resource derivation from real ops (H0.02) ──────────────────────────────

    /// A representative scene-mutating command used across derivation tests.
    fn scene_write_command() -> Command {
        Command::SetComponentField {
            entity_id: editor_model::StableId::new("e1"),
            type_id: "Transform".into(),
            field_path: "translation.x".into(),
            value: serde_json::json!(1.0),
        }
    }

    #[test]
    fn derive_resources_maps_scene_mutations_to_scene_resource() {
        let resources =
            derive_resources_from_commands(&[scene_write_command()]).expect("scene ops must map");
        assert_eq!(
            resources,
            vec![ResourceRef::Scene(SCENE_DOC_PLACEHOLDER.into())]
        );
    }

    #[test]
    fn derive_resources_recurses_batch_and_skips_noop() {
        let cmds = vec![
            Command::Noop {},
            Command::Batch {
                label: "b".into(),
                commands: vec![scene_write_command(), scene_write_command()],
            },
        ];
        let resources = derive_resources_from_commands(&cmds).expect("batch must map");
        // One scene resource total: Noop mutates nothing, duplicates dedup.
        assert_eq!(
            resources,
            vec![ResourceRef::Scene(SCENE_DOC_PLACEHOLDER.into())]
        );
    }

    #[test]
    fn derive_resources_empty_slice_yields_no_resources() {
        // Pins the contract the WASM boundary relies on: no ops → no
        // resources (the empty-selection path is rejected upstream).
        let resources = derive_resources_from_commands(&[]).expect("empty slice must map");
        assert!(resources.is_empty());
    }

    #[test]
    fn derive_resources_fail_closed_for_unmapped_command_kinds() {
        // Plugin approval must not proceed when a command kind has no
        // sanctioned resource mapping yet (fail-closed per H0.02).
        let world = Command::World(editor_model::world_command::WorldCommand::WorldCreate {
            name: "w".into(),
            layout_policy: editor_model::world::LayoutPolicy::Free,
        });
        let err = derive_resources_from_commands(&[world]).unwrap_err();
        assert_eq!(
            err,
            ResourceDerivationError::UnmappedCommandKind { kind: "World" }
        );
    }

    #[test]
    fn derive_resources_fail_closed_through_batch_nesting() {
        // A fail-closed kind hidden inside a Batch must still be rejected —
        // recursion must not launder an unmapped mutation through a mapped
        // parent (goldfish review gap).
        let batch = Command::Batch {
            label: "b".into(),
            commands: vec![
                scene_write_command(),
                Command::World(editor_model::world_command::WorldCommand::WorldCreate {
                    name: "w".into(),
                    layout_policy: editor_model::world::LayoutPolicy::Free,
                }),
            ],
        };
        let err = derive_resources_from_commands(&[batch]).unwrap_err();
        assert_eq!(
            err,
            ResourceDerivationError::UnmappedCommandKind { kind: "World" }
        );
    }

    #[test]
    fn plugin_permission_denied_for_extension_without_commands_permission() {
        // H0.02 AC#1: a plugin WITHOUT Commands permission proposing a scene
        // write must be denied, with the check seeing the resources derived
        // from the real ops (not an empty temporary ChangeSet).
        let registry: Arc<Mutex<dyn ExtensionRegistryPort>> = make_test_registry();
        let _ = register_extension_registry(Arc::clone(&registry));

        let mut cs: ChangeSet<Command> = ChangeSet::new(
            "cs9".into(),
            ChangeOrigin::Plugin,
            "extension:builtin.logic-recipes".into(),
            "recipe proposes scene mutation".into(),
        );
        cs.ops.push(scene_write_command());
        for r in derive_resources_from_commands(&cs.ops).expect("scene op must map") {
            cs.add_resource_ref(r);
        }

        let err = transaction_kernel_check_plugin_permission(&cs).unwrap_err();
        match err {
            KernelError::PermissionDenied {
                area, scope_needed, ..
            } => {
                assert_eq!(area, "Commands");
                assert_eq!(scope_needed, "Propose");
            }
            other => panic!("expected PermissionDenied, got {other:?}"),
        }
    }

    #[test]
    fn plugin_permission_allowed_for_extension_with_commands_permission_and_real_ops() {
        // Positive control: same derived resources, extension WITH
        // Commands/Propose passes the check.
        let registry: Arc<Mutex<dyn ExtensionRegistryPort>> = make_test_registry();
        let _ = register_extension_registry(Arc::clone(&registry));

        let mut cs: ChangeSet<Command> = ChangeSet::new(
            "cs10".into(),
            ChangeOrigin::Plugin,
            "extension:builtin.logic-bricks.controllers".into(),
            "controller proposes scene mutation".into(),
        );
        cs.ops.push(scene_write_command());
        for r in derive_resources_from_commands(&cs.ops).expect("scene op must map") {
            cs.add_resource_ref(r);
        }

        assert!(transaction_kernel_check_plugin_permission(&cs).is_ok());
    }

    #[test]
    fn plugin_permission_check_rejection_is_pure() {
        // H0.02 AC#3 (kernel side): a denied check must be a pure verdict —
        // evaluating it repeatedly yields the same denial and mutates nothing.
        let registry: Arc<Mutex<dyn ExtensionRegistryPort>> = make_test_registry();
        let _ = register_extension_registry(Arc::clone(&registry));

        let mut cs: ChangeSet<Command> = ChangeSet::new(
            "cs11".into(),
            ChangeOrigin::Plugin,
            "extension:builtin.logic-recipes".into(),
            "repeat denial".into(),
        );
        cs.ops.push(scene_write_command());
        for r in derive_resources_from_commands(&cs.ops).expect("scene op must map") {
            cs.add_resource_ref(r);
        }

        for _ in 0..3 {
            let err = transaction_kernel_check_plugin_permission(&cs).unwrap_err();
            assert!(matches!(err, KernelError::PermissionDenied { .. }));
        }
        assert_eq!(cs.ops.len(), 1);
    }
}
