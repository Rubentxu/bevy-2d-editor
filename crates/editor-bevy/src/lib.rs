use bevy::prelude::Entity as BevyEntity;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

pub mod actuator_bus;
pub mod asset_command;
pub mod asset_files;
pub mod asset_state;
pub mod auto_layer;
mod bevy_anchor;
pub mod bevy_logic_binding;
pub mod bsn_codegen;
pub mod bsn_export;
pub mod bsn_import;
pub mod bsn_ir;
mod code_export;
pub mod command;
pub mod document;
mod dynamic_scene;
pub mod hot_reload_state;
pub mod importer;
pub mod instance_projection;
mod lock_utils;
pub mod logic_command;
pub mod logic_dispatch;
pub mod logic_evaluator;
pub mod logic_graph;
pub mod logic_recipes;
mod logic_state;
pub mod logic_validation;
pub mod operation_log;
mod persistence;
pub mod preview_inspector;
pub mod preview_runtime;
pub mod processor;
pub mod scene_asset;
pub use editor_model::scene_asset_catalog::{
    CatalogError, CatalogWarning, SceneAssetCatalog, SceneAssetCatalogEntry, mint_asset_id,
    normalize_logical_path,
};
pub mod scene_instance;
pub mod scene_instance_overrides;
pub mod scene_session;
pub mod scene_state;
mod scenes;
pub mod schema;
pub mod source_files;
mod state;
pub mod tile_layer;
pub mod tileset;
pub mod time;
pub mod transaction_bridge;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
mod wasm_auto_layer;
mod wasm_bsn;
mod wasm_export;
pub mod wasm_hot_reload;
mod wasm_layer;
mod wasm_preview;
mod wasm_recipes;
mod wasm_scene_instance;
mod wasm_tile;

// ─────────────────────────────────────────────────────────────────────────────
// ADR References (documentation only — no code changes here)
// ─────────────────────────────────────────────────────────────────────────────
//
// ## ADRs integrated in this crate
//
// - [ADR-0005](../../adr/0005-scene-asset-bsn-aligned-reusable-scene-model.md):
//   Scene Asset identity (`asset_id` + `logical_path`), roles, versioning.
// - [ADR-0006](../../adr/0006-authoring-first-roadmap-after-bsn-migration.md):
//   editor-owned source of truth; `.bsn` write-back deferred.
// - [ADR-0007](../../adr/0007-separate-asset-command-surface.md):
//   separate `AssetCommand` surface for authoring mutations.
// - [ADR-0008](../../adr/0008-path-based-scene-asset-opfs-layout.md):
//   `assets/<logical_path>.asset.json` path layout; catalog in `ProjectMetadata`.

// ─────────────────────────────────────────────────────────────────────────────
// Dual Dispatch Gate — TransactionKernel adoption (ADR-0049)
// ─────────────────────────────────────────────────────────────────────────────

/// Runtime flag controlling whether `dispatch_command` routes through the
/// `TransactionKernel` or the legacy direct path.
///
/// - `true` (default when `dispatch-via-kernel` feature is enabled): route through kernel.
/// - `false`: use the v0.88 legacy path via `scene_session::apply_command`.
///
/// This flag is set via `set_dispatch_mode_wasm()` and can be flipped at runtime
/// for testing and rollback purposes.
static DISPATCH_VIA_KERNEL: AtomicBool = AtomicBool::new(cfg!(feature = "dispatch-via-kernel"));

/// Returns `true` if the current dispatch mode routes through the TransactionKernel.
pub fn is_dispatch_via_kernel() -> bool {
    DISPATCH_VIA_KERNEL.load(Ordering::SeqCst)
}

/// Set the dispatch mode. Use `"kernel"` to enable kernel routing or `"legacy"` to
/// revert to the v0.88 path.
///
/// This is the WASM-callable entry point exposed to the frontend.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_dispatch_mode_wasm(mode: &str) -> Result<(), JsValue> {
    match mode {
        "kernel" => {
            DISPATCH_VIA_KERNEL.store(true, Ordering::SeqCst);
            Ok(())
        }
        "legacy" => {
            DISPATCH_VIA_KERNEL.store(false, Ordering::SeqCst);
            Ok(())
        }
        _ => Err(JsValue::from_str(&format!(
            "Unknown dispatch mode: {mode}. Expected 'kernel' or 'legacy'."
        ))),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Validation Issue — unified issue type for Validation Center
// ─────────────────────────────────────────────────────────────────────────────

/// Unified validation issue surfaced by the Validation Center.
/// Aggregates CatalogWarning, OverrideIssue, ExportWarning, and other
/// project-wide issues into a single typed structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Unique identifier for this issue (stable across polls).
    pub id: String,
    /// Error = blocks save/export. Warning = non-fatal. Info = advisory.
    pub severity: ValidationSeverity,
    /// Which subsystem generated this issue.
    pub category: ValidationCategory,
    /// Machine-readable issue code (e.g. "orphaned_index", "missing_entity").
    pub code: String,
    /// Human-readable description.
    pub message: String,
    /// StableId of the affected entity, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_entity_id: Option<String>,
    /// asset_id of the affected asset, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_asset_id: Option<String>,
    /// scene_id of the affected scene, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_scene_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationCategory {
    Catalog,
    Override,
    Export,
    Schema,
    Dirty,
    Logic,
    /// Issues produced during external source import (ADR-0041).
    Import,
}

pub use asset_command::{AssetCommand, AssetCommandError, AssetOperationLog};
pub use asset_files::{AssetFile, AssetFileId, AssetFileKind, RESOURCE_DIR};
pub use auto_layer::{
    AutoLayer, AutoLayerId, AutoRule, Pattern3x3, PatternCell, is_auto_layer_stale, regenerate,
};
pub use bevy_anchor::anchor_str_to_bevy_anchor;
pub use bsn_export::{
    BevyBsnExporter, BsnExportError, BsnExporter, EditorCoreBsnExporter, export_to_bsn_text,
    export_to_bsn_text_with_warnings,
};
pub use bsn_import::{BsnImportError, parse_bsn_text, scene_asset_from_bsn_ir};
pub use bsn_ir::{
    BsnIr, BsnIrNode, BsnIrRelationship, BsnPatch, BsnPatchOp, bsn_ir_from_scene_asset,
};
pub use code_export::{CodeGenResult, export_rust_source};
pub use command::{Command, CommandEnvelope, CommandError, CommandMetadata, CommandResult};
pub use document::{ComponentInstance, Entity, SceneDocument, StableId};
pub use dynamic_scene::{
    DynamicSceneExport, EXPORT_VERSION, EntityExport, ExportError, ExportWarning,
    anchor_str_to_normalized_offset, export_dynamic_scene, is_known_anchor_str,
};
pub use instance_projection::{PreviewEntity, project_instances, root_local_ids};
pub use logic_command::{LogicCommand, LogicCommandError, LogicOperationLog};
pub use logic_evaluator::{
    LogicNodeRegistry, NodeDescriptor, NodeEvaluator, ParamSpec, PortSpec, PortValue,
    PortValueType, global_node_registry,
};
pub use logic_graph::{
    LogicEdge, LogicGraphAsset, LogicInstance, LogicNode, LogicNodeRole, NodeId, NodeTypeId,
    PortId, count_logic_bindings, editor_logic_binding_component, find_dangling_edge_nodes,
    find_duplicate_node_id,
};
pub use logic_recipes::{is_builtin_recipe, list_builtin_recipes, seed_builtin_recipes};
pub use logic_validation::{LogicValidationIssue, LogicValidationIssueCode, validate_logic_graph};
pub use operation_log::{LogEntry, OperationLog, OperationLogError};
pub use persistence::{
    ASSETS_DIR, AssetPathError, PROJECT_FILE, ProjectMetadata, SCENES_DIR, SCHEMAS_DIR,
    TILESETS_DIR, asset_path, tileset_path, validate_logical_path,
};
pub use preview_inspector::{PreviewMappingEntry, PreviewMetrics, PreviewProvenance};
// §6: CausalityEdge + RebuildCause re-exported from editor-model for preview_inspector.rs.
pub use editor_model::RebuildCause;
pub use editor_model::causality::{CausalityEdge, CausalityEdgeKind};
pub use preview_runtime::in_play_mode;
pub use scene_asset::{
    AssetReference, ExposedProperty, LayerId, LevelLayer, LocalId, RelationshipKind, RoleWarning,
    SceneAssetDocument, SceneAssetEntity, SceneAssetMetadata, SceneAssetRelationship,
    SceneAssetRole, SceneInstanceLayer, SceneInstanceLayerKind, validate_role,
};
pub use scene_instance::{
    ComponentOverride, ComponentOverrideStatus, SceneInstance,
    component_override_status_after_field_rename,
};
pub use scene_instance_overrides::{OverrideIssue, ResyncReport};
pub use scenes::{SceneInfo, SceneRegistry, SwitchResult};
pub use schema::{ApplyBackPolicy, ApplyBackScope, ComponentTypeId};
pub use source_files::{SOURCES_DIR, SourceFile, SourceFileId};
pub use tile_layer::{TileLayer, TileLayerId};
pub use tileset::{
    AsepriteFrame, AsepriteMetadata, AsepriteSlice, AsepriteTag, TileCoord, TileGrid, TileRef,
    TilesetAsset, TilesetId, TilesetManager, TilesetMetadata,
};
pub use wasm_hot_reload::{
    force_reload_wasm, hot_reload_asset_wasm, hot_reload_bus_depth_for_tests,
    hot_reload_source_wasm,
};
/// Marker component for entities spawned from SceneDocument.
/// These are despawned and respawned when the document is mutated
/// (preview world rebuild strategy — matches Hito 0 decision 23).
#[derive(Component)]
pub struct SceneEntity;

/// Marker component for entities that are projected from a Scene Instance.
/// Carries the instance_id and local_id of the source entity.
/// Used for selection routing and despawn-all cleanup.
#[derive(Component)]
pub struct SceneInstanceChild {
    pub instance_id: crate::document::StableId,
    pub local_id: crate::scene_asset::LocalId,
}

/// Resource holding the current SceneDocument and a dirty flag
/// that signals `rebuild_preview_world` to respawn entities.
#[derive(Resource, Clone)]
pub struct SceneDocumentState {
    pub document: SceneDocument,
    pub dirty: bool,
}

impl SceneDocumentState {
    pub fn new(document: SceneDocument) -> Self {
        Self {
            document,
            dirty: true, // initial spawn
        }
    }
}

/// Resource exposing operation log metadata for UI hooks (undo/redo buttons).
/// Updated by `sync_log_state` after every apply/undo/redo.
#[derive(Resource, Clone, Default)]
pub struct OperationLogState {
    pub size: usize,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Play-mode state resource. Edit = editor commands + rebuild active;
/// Playing = commands paused, logic dispatch + actuators run free.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayMode {
    #[default]
    Edit,
    Playing,
}

/// Snapshot of placed-entity Transforms captured on Play entry.
/// Stores Bevy Entity → Transform. Entity IDs are stable within a play
/// session (rebuild_preview_world is gated off during play, so no despawn).
#[derive(Resource, Default)]
pub struct TransformSnapshot {
    pub transforms: std::collections::HashMap<bevy::prelude::Entity, Transform>,
}

// HIGH-1 god-module phase 1: re-exports from state module.
// The actual thread-local declarations + with_* helpers now live in
// crates/editor-core/src/state.rs. Re-exported here so existing
// callers in lib.rs continue to work without modification.
use crate::state::{
    ASSET_BODY_CACHE, ASSET_OPERATION_LOG, DIRTY_FLAG, HOT_RELOAD_BUS, HotReloadRequest,
    LOGIC_OPERATION_LOG, PLAY_MODE_REQUEST, PlayModeRequest, RESYNC_REPORTS, SCENE_ASSET_DOC,
    SCENE_REGISTRY, VALIDATION_ISSUES, clear_asset_catalog_warnings, get_asset_catalog_warnings,
    mark_dirty, with_asset_body_cache, with_asset_body_cache_mut, with_asset_catalog,
    with_asset_catalog_mut, with_asset_doc, with_asset_doc_and_log_mut, with_asset_doc_mut,
    with_asset_log, with_asset_log_mut, with_logic_graph, with_logic_graph_catalog,
    with_logic_graph_catalog_mut, with_logic_graph_mut, with_logic_log, with_logic_log_mut,
    with_registry, with_registry_mut,
};

/// Mutably access the asset body cache from integration tests.
pub fn with_asset_body_cache_mut_for_tests<F, R>(f: F) -> R
where
    F: FnOnce(&mut BTreeMap<String, SceneAssetDocument>) -> R,
{
    with_asset_body_cache_mut(f)
}

/// Clear the cross-system dirty flag from integration tests.
pub fn clear_dirty_for_tests() {
    DIRTY_FLAG.with(|dirty| *dirty.borrow_mut() = false);
}

/// Read the cross-system dirty flag from integration tests.
pub fn is_dirty_for_tests() -> bool {
    DIRTY_FLAG.with(|dirty| *dirty.borrow())
}

const CMD_MOVE_SPRITE: u16 = 1;
const EVT_SPRITE_POSITION: u16 = 1;
const EVT_FPS: u16 = 2;

const BUS_CAPACITY: usize = 65536;

/// Default scene JSON matching the original spike: green sprite at origin.
const DEFAULT_SCENE_JSON: &str = r#"{
    "version": "0.1",
    "scene_id": "default",
    "name": "Default Scene",
    "entities": [
        {
            "id": "spike-sprite-01",
            "name": "Green Sprite",
            "components": [
                {
                    "type_id": "editor.Name",
                    "values": {"name": "Green Sprite"}
                },
                {
                    "type_id": "editor.Transform2D",
                    "values": {
                        "translation": {"x": 0.0, "y": 0.0},
                        "rotation": 0.0,
                        "scale": {"x": 1.0, "y": 1.0}
                    }
                },
                {
                    "type_id": "editor.Sprite2D",
                    "values": {
                        "asset": "",
                        "color": {"r": 0.3, "g": 0.8, "b": 0.5, "a": 1.0},
                        "anchor": "Center"
                    }
                }
            ]
        }
    ]
}"#;

thread_local! {
    pub(crate) static COMMAND_BUS: RefCell<Option<LinearBus>> = const { RefCell::new(None) };
    pub(crate) static EVENT_BUS: RefCell<Option<LinearBus>> = const { RefCell::new(None) };
    pub(crate) static SCENE_DOC: RefCell<Option<SceneDocument>> = const { RefCell::new(None) };
    pub(crate) static OPERATION_LOG: RefCell<OperationLog> = const { RefCell::new(OperationLog::new_const()) };
}

struct LinearBus {
    buffer: Box<[u8]>,
}

impl LinearBus {
    fn new() -> Self {
        let mut buffer = vec![0u8; BUS_CAPACITY].into_boxed_slice();
        Self::set_write_offset(&mut buffer, 8);
        Self { buffer }
    }

    fn ptr(&self) -> u32 {
        self.buffer.as_ptr() as u32
    }

    fn len(&self) -> u32 {
        self.buffer.len() as u32
    }

    fn get_write_offset(buf: &[u8]) -> usize {
        u32::from_le_bytes(buf[0..4].try_into().unwrap()) as usize
    }

    fn set_write_offset(buf: &mut [u8], offset: usize) {
        buf[0..4].copy_from_slice(&(offset as u32).to_le_bytes());
    }

    fn drain(&mut self) -> Vec<(u16, Vec<u8>)> {
        let end = Self::get_write_offset(&self.buffer);
        Self::set_write_offset(&mut self.buffer, 8);
        let mut result = Vec::new();
        let mut pos = 8;
        while pos + 4 <= end && pos + 4 <= self.buffer.len() {
            let cmd_type = u16::from_le_bytes(self.buffer[pos..pos + 2].try_into().unwrap());
            let payload_len =
                u16::from_le_bytes(self.buffer[pos + 2..pos + 4].try_into().unwrap()) as usize;
            if pos + 4 + payload_len > self.buffer.len() {
                break;
            }
            let payload = self.buffer[pos + 4..pos + 4 + payload_len].to_vec();
            result.push((cmd_type, payload));
            pos += 4 + payload_len;
        }
        result
    }

    fn reset(&mut self) {
        Self::set_write_offset(&mut self.buffer, 8);
    }

    fn write(&mut self, event_type: u16, payload: &[u8]) -> bool {
        let write_offset = Self::get_write_offset(&self.buffer);
        let slot_size = 4 + payload.len();
        if write_offset + slot_size > self.buffer.len() {
            return false;
        }
        self.buffer[write_offset..write_offset + 2].copy_from_slice(&event_type.to_le_bytes());
        self.buffer[write_offset + 2..write_offset + 4]
            .copy_from_slice(&(payload.len() as u16).to_le_bytes());
        self.buffer[write_offset + 4..write_offset + 4 + payload.len()].copy_from_slice(payload);
        Self::set_write_offset(&mut self.buffer, write_offset + slot_size);
        true
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn create_buses() {
    console_error_panic_hook::set_once();
    COMMAND_BUS.with(|b| *b.borrow_mut() = Some(LinearBus::new()));
    EVENT_BUS.with(|b| *b.borrow_mut() = Some(LinearBus::new()));
    web_sys::console::log_1(&"[editor-core] Buses created".into());
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn load_scene_json(json: &str) -> Result<(), JsValue> {
    let doc: SceneDocument = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse scene JSON: {}", e)))?;
    scene_session::replace_active_doc(doc);
    web_sys::console::log_1(&"[editor-core] Scene document loaded".into());
    Ok(())
}

/// Load a scene by name from OPFS and set it as the active SceneDocument.
///
/// Mirrors `save_scene(name)`: round-trip pair for OPFS persistence.
/// Returns the loaded SceneDocument JSON string so callers can re-parse
/// it without an extra read. Returns Err if the file does not exist or
/// cannot be parsed.
///
/// Used by tests via the engine-bridge (`window.load_scene`).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn load_scene_by_name(name: &str) -> Result<String, JsValue> {
    let path = persistence::scene_path(name);
    let json = js_load_file(&path)
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to load scene '{}': {}", name, e)))?;
    let doc: SceneDocument = serde_json::from_str(&json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse scene JSON: {}", e)))?;
    scene_session::replace_active_doc(doc);
    web_sys::console::log_1(&format!("[editor-core] Scene '{}' loaded from {}", name, path).into());
    Ok(json)
}

/// Apply a typed command to the SceneDocument, mutating it and producing
/// an inverse command for undo. Returns the inverse as JSON.
///
/// The command envelope (command + metadata) is parsed from JSON. On success,
/// the dirty flag is set so `rebuild_preview_world` respawns Bevy entities.
/// The command is also recorded in the operation log for undo/redo.
/// Apply a typed command envelope to the active SceneDocument and
/// record the inverse in the operation log. Returns the inverse plus
/// a post-apply snapshot, all serialised as JSON.
///
/// `dispatch_command` is the primary editor mutation seam. Every
/// edit flows through here, so the operation log can drive undo/redo
/// and the dirty flag can schedule `rebuild_preview_world`.
///
/// Wave D3: the underlying state transitions live in
/// `scene_session::apply_command`. This WASM binding is now a thin
/// adapter that parses JSON, delegates, and serialises the result.
/// Apply a typed command envelope to the active SceneDocument and
/// record the inverse in the operation log. Returns the inverse plus
/// a post-apply snapshot, all serialised as JSON.
///
/// `dispatch_command` is the primary editor mutation seam. Every
/// edit flows through here, so the operation log can drive undo/redo
/// and the dirty flag can schedule `rebuild_preview_world`.
///
/// Wave D3: the underlying state transitions live in
/// `scene_session::apply_command`. This function delegates to that
/// module for the actual apply + log + dirty-flag sequence.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn dispatch_command(json: &str) -> Result<String, JsValue> {
    let envelope: CommandEnvelope = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Invalid command JSON: {}", e)))?;

    // v0.91 PR5: always route through TransactionKernel. The legacy v0.88
    // path (via `is_dispatch_via_kernel() == false`) is removed; ADR-0032
    // established the kernel as the single dispatch path. The `dispatch_*_legacy`
    // functions are also removed below.
    dispatch_command_via_kernel(envelope).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Kernel path: route a command envelope through SceneTransactionKernel.
///
/// This is the internal dispatch function used by both the legacy WASM entry point
/// and by `editor_application::wasm` for ChangeWorkbench approval.
#[cfg(target_arch = "wasm32")]
pub fn dispatch_command_via_kernel(
    envelope: CommandEnvelope,
) -> Result<String, editor_protocol::DispatchError> {
    use crate::transaction_bridge::scene_transaction_kernel;
    use editor_model::session::HistoryScope;
    use editor_model::transaction::{Applier, ChangeOrigin, ChangeSet};
    use editor_protocol::DispatchError;

    // Determine ChangeOrigin from authorship metadata.
    // ADR-0040: "extension:<id>" prefix indicates Plugin origin.
    let origin = match envelope.metadata.authorship.as_str() {
        "user" => ChangeOrigin::Human,
        s if s.starts_with("agent:") => ChangeOrigin::Agent,
        "system" => ChangeOrigin::Migration,
        s if s.starts_with("extension:") => ChangeOrigin::Plugin,
        _ => ChangeOrigin::Human,
    };

    // Wrap single command in a ChangeSet.
    let mut cs = ChangeSet::new(
        format!("cmd-{}", envelope.metadata.timestamp),
        origin.clone(),
        envelope.metadata.authorship.clone(),
        envelope.metadata.rationale.clone().unwrap_or_default(),
    );
    cs.add_resource("scene", "scenes/current.json");
    cs.push_op(envelope.command.clone());

    // ADR-0040 v0.92: For Plugin-origin commands, verify the extension is registered.
    // This is a lightweight check: just confirm the extension ID is in the registry.
    // The full permission check (covering resource-specific scopes) happens in
    // editor_application::transaction::transaction_kernel_check_plugin_permission at
    // ChangeSet approval time. This here check ensures unknown extensions can't
    // even dispatch commands through the kernel path.
    if matches!(origin, ChangeOrigin::Plugin) {
        let ext_id = envelope
            .metadata
            .authorship
            .strip_prefix("extension:")
            .unwrap_or(&envelope.metadata.authorship);
        if let Some(registry) = editor_model::ports::with_extension_registry() {
            if let Ok(guard) = registry.lock() {
                if guard.get(ext_id).is_none() {
                    return Err(DispatchError::PermissionDenied(format!(
                        "extension '{}' is not registered",
                        ext_id
                    )));
                }
            }
        }
        // If registry not initialized, allow dispatch (fail-open for dev scenarios).
    }

    // Get mutable access to the scene doc and operation log.
    let (inverse, snapshot) = SCENE_DOC.with(|cell| {
        let mut doc_ref = cell.borrow_mut();
        let doc = doc_ref
            .as_mut()
            .ok_or_else(|| DispatchError::ExecutionFailed("No active scene".to_string()))?;

        // Create a temporary HistoryScope for the kernel call.
        // Note: The kernel updates HistoryScope but we also record in OperationLog
        // for undo/redo compatibility. The HistoryScope is not persisted.
        let mut history = HistoryScope::new();

        let kernel = scene_transaction_kernel();
        let receipt = kernel
            .apply_atomic(&cs, doc, &mut history)
            .map_err(|e| DispatchError::KernelError(e.to_string()))?;

        // Extract the inverse (kernel returns inverses in reverse order).
        let inverse =
            receipt
                .inverses
                .into_iter()
                .next()
                .unwrap_or_else(|| Command::CreateEntity {
                    id: StableId::new("__no_inverse__"),
                    name: String::new(),
                    components: vec![],
                });

        // Record in OperationLog for undo/redo (byte-equality with legacy path).
        // Use record_with_provenance to pass origin, actor, and change_id from the ChangeSet.
        OPERATION_LOG.with(|l| {
            l.borrow_mut().record_with_provenance(
                &envelope,
                inverse.clone(),
                format!("{:?}", origin),
                envelope.metadata.authorship.clone(),
                Some(cs.id.clone()),
            );
        });

        // Return the inverse and post-apply snapshot.
        Ok::<(Command, SceneDocument), DispatchError>((inverse, doc.clone()))
    })?;

    scene_state::mark_dirty();

    let result_json = serde_json::to_string(&CommandResult { inverse, snapshot }).map_err(|e| {
        DispatchError::ExecutionFailed(format!("Failed to serialize result: {}", e))
    })?;

    Ok(result_json)
}

/// Legacy v0.88 path: direct dispatch through scene_session::apply_command.
/// v0.91 PR5: removed. ADR-0032 established the kernel as the single
/// dispatch path; the legacy fallback is no longer reachable.
/// Internal helper that wraps `scene_session::apply_command` and
/// returns the post-apply snapshot directly. Used by
/// `place_scene_instance` and `replace_scene_instance_asset` so they
/// do not have to round-trip through JSON.
pub(crate) fn apply_envelope_internal(envelope: &CommandEnvelope) -> Result<SceneDocument, String> {
    let result = scene_session::apply_command(envelope)
        .map_err(|e| format!("apply_envelope_internal: {e}"))?;
    Ok(result.snapshot)
}

/// Place a Scene Asset as a new Scene Instance in the active SceneDocument.
///
/// Design D5: `place_scene_instance(asset_id, translation_json?)`
/// - Resolves asset via catalog + cache
/// - Checks single-root gate via `root_local_ids`
/// - Mints fresh `instance_id` and `id_map` with `inst_` prefix
/// - Creates an `editor.Transform2D` ComponentInstance in `instance_components`
///   when `translation_json` is provided (placement-time data, not asset patch)
/// - Dispatches `Command::PlaceInstance` through the shared OperationLog
///
/// Returns the `CommandResult` JSON on success.
#[wasm_bindgen]
pub fn place_scene_instance(
    asset_id: &str,
    translation_json: Option<String>,
) -> Result<String, JsValue> {
    use crate::instance_projection::root_local_ids;

    // Step 1: Look up catalog entry
    let entry = with_asset_catalog(|cat| cat.get(asset_id).cloned())
        .ok_or_else(|| JsValue::from_str(&format!("Asset not found in catalog: {}", asset_id)))?;

    // Step 2: Look up asset body in cache (keyed by logical_path)
    let asset = with_asset_body_cache(|cache| cache.get(&entry.logical_path).cloned()).ok_or_else(
        || {
            JsValue::from_str(&format!(
                "Asset not in cache: {}. Call load_project first.",
                entry.logical_path
            ))
        },
    )?;

    // Step 3: Check single-root gate
    let roots = root_local_ids(&asset);
    if roots.is_empty() {
        return Err(JsValue::from_str(
            "Empty asset: cannot place instance with zero entities",
        ));
    }
    if roots.len() > 1 {
        return Err(JsValue::from_str(&format!(
            "Multiple roots: asset has {} root entities, expected 1",
            roots.len()
        )));
    }

    // Step 4: Mint fresh instance_id
    let now = crate::time::now_nanos();
    let instance_id = crate::document::StableId::new(format!("inst_{:x}", now));

    // Step 5: Mint id_map entries with `inst_{iid}_{lid}` pattern
    let id_map: std::collections::BTreeMap<crate::scene_asset::LocalId, crate::document::StableId> =
        asset
            .entities
            .iter()
            .map(|e| {
                let stable_id = crate::document::StableId::new(format!(
                    "{}_{}",
                    instance_id.as_str(),
                    e.local_id.as_str()
                ));
                (e.local_id.clone(), stable_id)
            })
            .collect();

    // Step 6: Build instance_components from translation_json (placement-time data).
    // Per level-design-layers-research design, placement is NOT a ComponentOverride
    // against the asset's components; it is owned by the placed occurrence.
    let mut instance_components: Vec<crate::document::ComponentInstance> = Vec::new();
    if let Some(trans_json) = translation_json {
        let translation: serde_json::Value = serde_json::from_str(&trans_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid translation JSON: {}", e)))?;

        // Validate translation shape early to surface bad input before dispatch.
        if !translation.is_object() {
            return Err(JsValue::from_str(
                "Invalid translation JSON: expected an object with `translation` or full Transform2D fields",
            ));
        }

        instance_components.push(crate::document::ComponentInstance {
            type_id: "editor.Transform2D".to_string(),
            values: serde_json::json!({
                "translation": translation,
            }),
        });
    }

    // Step 7: Build PlaceInstance command
    let command = Command::PlaceInstance {
        instance_id,
        asset_ref: crate::scene_asset::AssetReference::new(entry.logical_path.clone()),
        asset_version: entry.current_version,
        id_map,
        instance_components,
        component_overrides: Vec::new(),
        orphaned_component_overrides: Vec::new(),
    };

    // Step 8: Wrap in envelope and dispatch
    let envelope = CommandEnvelope {
        command,
        metadata: CommandMetadata::now("user"),
    };

    // Apply through the scene_session module directly to avoid a
    // JSON round-trip on the hot path. The dirty flag is set inside
    // apply_envelope_internal via the scene_session::mark_dirty call.
    let snapshot = apply_envelope_internal(&envelope)
        .map_err(|e| JsValue::from_str(&format!("place_scene_instance: {e}")))?;
    let result_json = serde_json::to_string(&CommandResult {
        inverse: Command::Noop {},
        snapshot,
    })
    .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))?;

    Ok(result_json)
}

/// Remove a Scene Instance from the active SceneDocument.
///
/// Returns the `CommandResult` JSON on success.
#[wasm_bindgen]
pub fn remove_scene_instance(instance_id: &str) -> Result<String, JsValue> {
    let stable_id = crate::document::StableId::new(instance_id);

    let command = Command::RemoveInstance {
        instance_id: stable_id,
    };

    let envelope = CommandEnvelope {
        command,
        metadata: CommandMetadata::now("user"),
    };

    let snapshot = apply_envelope_internal(&envelope)
        .map_err(|e| JsValue::from_str(&format!("remove_scene_instance: {e}")))?;
    let result_json = serde_json::to_string(&CommandResult {
        inverse: Command::Noop {},
        snapshot,
    })
    .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))?;
    Ok(result_json)
}

/// Replace the asset of an existing Scene Instance.
///
/// Dispatches `Command::ReplaceInstanceAsset` which runs resync to reclassify
/// overrides. Returns the `CommandResult` JSON on success.
#[wasm_bindgen]
pub fn replace_scene_instance_asset(
    instance_id: &str,
    new_asset_id: &str,
) -> Result<String, JsValue> {
    // Look up new asset in catalog
    let new_entry = with_asset_catalog(|cat| cat.get(new_asset_id).cloned()).ok_or_else(|| {
        JsValue::from_str(&format!("Asset not found in catalog: {}", new_asset_id))
    })?;

    let stable_id = crate::document::StableId::new(instance_id);

    let command = Command::ReplaceInstanceAsset {
        instance_id: stable_id,
        new_asset_ref: crate::scene_asset::AssetReference::new(new_entry.logical_path.clone()),
        new_asset_version: new_entry.current_version,
        captured_old: None, // Processor fills this in
    };

    let envelope = CommandEnvelope {
        command,
        metadata: CommandMetadata::now("user"),
    };

    let snapshot = apply_envelope_internal(&envelope)
        .map_err(|e| JsValue::from_str(&format!("replace_scene_instance_asset: {e}")))?;
    let result_json = serde_json::to_string(&CommandResult {
        inverse: Command::Noop {},
        snapshot,
    })
    .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))?;
    Ok(result_json)
}

/// Get all Scene Instances from the active SceneDocument as JSON.
///
/// Returns the `instances` BTreeMap serialized as JSON.
#[wasm_bindgen]
pub fn get_scene_instances() -> Result<String, JsValue> {
    SCENE_DOC.with(|s| {
        let doc_ref = s.borrow();
        let doc = doc_ref
            .as_ref()
            .ok_or_else(|| JsValue::from_str("No scene loaded — call load_scene_json first"))?;

        serde_json::to_string(&doc.instances)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize instances: {}", e)))
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Validation Center WASM surface
// ─────────────────────────────────────────────────────────────────────────────

/// Collect all project-wide validation issues.
///
/// This is a synchronous function that aggregates:
/// - Catalog warnings (from SCENE_ASSET_CATALOG_WARNINGS)
/// - Export warnings (by re-exporting the current scene document)
///
/// Override issues are NOT collected here because validating overrides requires
/// loading asset bodies from OPFS (async). Override validation is handled
/// separately on the TypeScript side via validateOverrides().
///
/// Returns a JSON array of ValidationIssue objects.
fn serialize_logic_code(code: &LogicValidationIssueCode) -> String {
    match code {
        LogicValidationIssueCode::DuplicateNodeId => "duplicate-node-id".to_string(),
        LogicValidationIssueCode::DanglingEdgeEndpoint => "dangling-edge-endpoint".to_string(),
        LogicValidationIssueCode::InvalidPortType => "invalid-port-type".to_string(),
        LogicValidationIssueCode::Cycle => "cycle".to_string(),
        LogicValidationIssueCode::DanglingControllerRef => "dangling-controller-ref".to_string(),
    }
}

#[wasm_bindgen]
pub fn get_validation_issues_wasm() -> Result<String, JsValue> {
    let mut issues: Vec<ValidationIssue> = Vec::new();
    let mut next_id: u32 = 0;
    let mut mint_id = || {
        next_id += 1;
        format!("vi_{}", next_id)
    };

    // 1. Catalog warnings
    let catalog_warnings = get_asset_catalog_warnings();
    for cw in catalog_warnings {
        issues.push(ValidationIssue {
            id: mint_id(),
            severity: ValidationSeverity::Warning,
            category: ValidationCategory::Catalog,
            code: cw.code,
            message: cw.message,
            affected_entity_id: None,
            affected_asset_id: cw.asset_id,
            affected_scene_id: None,
        });
    }

    // 2. Export warnings from the current scene document
    let current_doc_opt = scene_session::snapshot_active_doc();
    if let Some(doc) = current_doc_opt {
        match dynamic_scene::export_dynamic_scene(&doc) {
            Ok(export) => {
                for ew in export.warnings {
                    issues.push(ValidationIssue {
                        id: mint_id(),
                        severity: ValidationSeverity::Warning,
                        category: ValidationCategory::Export,
                        code: ew
                            .component_type_id
                            .as_deref()
                            .unwrap_or("export_warning")
                            .to_string(),
                        message: ew.message,
                        affected_entity_id: ew.entity_stable_id,
                        affected_asset_id: None,
                        affected_scene_id: Some(doc.scene_id.clone()),
                    });
                }
            }
            Err(_) => {
                // Export failed — skip export warnings for this poll
            }
        }
    }

    // 3. Logic graph validation
    with_logic_graph_mut(|doc_opt| {
        if let Some(asset) = doc_opt {
            let logic_issues = validate_logic_graph(asset, global_node_registry());
            for li in logic_issues {
                issues.push(ValidationIssue {
                    id: mint_id(),
                    severity: ValidationSeverity::Error,
                    category: ValidationCategory::Logic,
                    code: serialize_logic_code(&li.code),
                    message: li.message,
                    affected_entity_id: None,
                    affected_asset_id: Some(asset.asset_id.clone()),
                    affected_scene_id: None,
                });
            }
        }
    });

    // 4. Dirty scene issues — not available synchronously in WASM.
    //    The frontend manages dirty state per-scene. Dirty issues are handled
    //    by the TypeScript ValidationCenter service separately.

    serde_json::to_string(&issues)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize issues: {}", e)))
}

// ─────────────────────────────────────────────────────────────────────────────
// BSN file export WASM surface
// ─────────────────────────────────────────────────────────────────────────────

/// Export a `SceneAsset` by `asset_id` to `.bsn` text without changing the
/// currently-open document in the editor. Returns raw `.bsn` text or an error.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn export_asset_to_bsn_wasm(asset_id: &str) -> Result<String, JsValue> {
    use crate::persistence;

    // 1. Get catalog entry to resolve logical_path
    let entry = with_asset_catalog(|cat| cat.get(asset_id).cloned())
        .ok_or_else(|| JsValue::from_str(&format!("Asset not found: {}", asset_id)))?;

    // 2. Load body JSON from OPFS
    let body_json = js_load_file(&persistence::asset_path(&entry.logical_path))
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // 3. Parse and export
    let doc: SceneAssetDocument = serde_json::from_str(&body_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    crate::bsn_export::export_to_bsn_text(&doc)
        .map_err(|e| JsValue::from_str(&format!("BSN export error: {}", e)))
}

// ─────────────────────────────────────────────────────────────────────────────
// Runtime Preview Inspector WASM surface
// ─────────────────────────────────────────────────────────────────────────────
//
// HIGH-1 phase 3a moved get_preview_metrics_wasm, get_preview_mapping_wasm,
// and get_preview_provenance_wasm to crates/editor-core/src/wasm_preview.rs.
// Only get_preview_provenance_wasm remains here for backward compat (delete
// in a follow-up if it can be removed without breaking other consumers).
//
// NOTE: HIGH-1 phase 4 found two bugs in the previous phase 3a/3b/3d extractions:
//   1. in_edit_mode / in_play_mode RunIf helpers were silently dropped
//      during phase 1's state.rs extraction. The host build didn't catch
//      this because start_engine is cfg(target_arch = "wasm32")-gated.
//   2. Three of the moved functions (get_preview_metrics_wasm,
//      get_preview_mapping_wasm, get_preview_provenance_wasm) were NOT
//      deleted from lib.rs, causing duplicate-symbol errors when
//      building for wasm32. This commit fixes both classes of bug.

// ─────────────────────────────────────────────────────────────────────────────
// Scene Instance Layer WASM surface
// ─────────────────────────────────────────────────────────────────────────────

/// Helper: parse `asset_json` as `SceneAssetDocument`. Returns error JsValue on failure.
fn parse_asset_doc(asset_json: &str) -> Result<SceneAssetDocument, JsValue> {
    serde_json::from_str(asset_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid asset JSON: {}", e)))
}

///
/// Used by the scene-instance-layer slice to commit layer mutations back to the
/// in-memory doc so subsequent saves (save_scene_asset) persist them.
#[wasm_bindgen]
pub fn set_asset_document_wasm(asset_json: &str) -> Result<(), JsValue> {
    let doc: SceneAssetDocument = parse_asset_doc(asset_json)?;
    SCENE_ASSET_DOC.with(|s| {
        *s.borrow_mut() = Some(doc);
    });
    // Bump version so downstream resync sees a change if same logical content.
    // (Optional — semantic versioning is out of scope for this slice.)
    Ok(())
}

/// Undo the last operation. Returns the new document snapshot as JSON.
#[wasm_bindgen]
pub fn undo() -> Result<String, JsValue> {
    let snapshot = scene_session::undo()
        .ok_or_else(|| JsValue::from_str("No scene loaded — call load_scene_json first"))?;
    serde_json::to_string(&snapshot)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize snapshot: {}", e)))
}

/// Redo the next operation. Returns the new document snapshot as JSON.
#[wasm_bindgen]
pub fn redo() -> Result<String, JsValue> {
    let snapshot = scene_session::redo()
        .ok_or_else(|| JsValue::from_str("No scene loaded — call load_scene_json first"))?;
    serde_json::to_string(&snapshot)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize snapshot: {}", e)))
}

/// Returns operation log metadata as JSON.
/// Useful for UI to enable/disable undo/redo buttons.
#[wasm_bindgen]
pub fn get_log_state() -> String {
    OPERATION_LOG.with(|l| {
        let log = l.borrow();
        serde_json::json!({
            "size": log.get_log_size(),
            "can_undo": log.can_undo(),
            "can_redo": log.can_redo(),
            "cursor": log.get_cursor(),
        })
        .to_string()
    })
}

// HIGH-1 god-module phase 4: Bevy runtime systems extracted to
// `crate::preview_runtime`. Re-export the WASM-facing symbols here so the
// public API of `editor-core` is unchanged.
pub use preview_runtime::process_hot_reload_requests;
#[cfg(target_arch = "wasm32")]
pub use preview_runtime::start_engine;

// ─────────────────────────────────────────────────────────────────────────────
// Bus pointer accessors (used by engine-bridge.ts to build DataView over shared
// memory). These are wasm32-only exports that the JS bridge calls via
// `wasm.get_command_bus_ptr()` / `wasm.get_event_bus_ptr()`. They live here
// (not in `preview_runtime.rs`) because they are 1-line thunks that must
// remain reachable from the public API surface.
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the pointer (offset into WebAssembly.Memory) of the command bus
/// LinearBus buffer. Used by the JS engine-bridge to build a DataView for
/// polling commands written by the host (move-sprite, etc.).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_command_bus_ptr() -> u32 {
    COMMAND_BUS.with(|b| b.borrow().as_ref().unwrap().ptr())
}

/// Returns the byte length of the command bus LinearBus buffer.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_command_bus_len() -> u32 {
    COMMAND_BUS.with(|b| b.borrow().as_ref().unwrap().len())
}

/// Returns the pointer of the event bus LinearBus buffer (where the Bevy
/// runtime writes sprite positions, FPS, etc.).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_event_bus_ptr() -> u32 {
    EVENT_BUS.with(|b| b.borrow().as_ref().unwrap().ptr())
}

/// Returns the byte length of the event bus LinearBus buffer.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_event_bus_len() -> u32 {
    EVENT_BUS.with(|b| b.borrow().as_ref().unwrap().len())
}

// ─────────────────────────────────────────────────────────────────────────────
// OPFS Persistence — dynamic ProjectStore via editor_model registry (ADR-0031).
//
// ## Architecture
//
// `PROJECT_STORE` lives in `editor_model::ports` (shared by both crates).
// `editor_application::wasm::init_project_store()` creates OpfsProjectStore,
// hydrates it, and calls `editor_model::ports::register_project_store()`.
// `editor_core`'s js_* wrappers call `editor_model::ports::with_project_store()`.
//
// This breaks the `editor_core → editor_application` compile-time dependency
// for OPFS, while keeping the WASM composition root pattern intact.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
use editor_model::ports::{ProjectStore, with_project_store};
#[cfg(target_arch = "wasm32")]
use std::future::Future;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

/// Save a text file to OPFS and flush (durability-preserving).
#[cfg(target_arch = "wasm32")]
pub(crate) async fn js_save_file(path: &str, contents: &str) -> Result<(), String> {
    let store = with_project_store().ok_or_else(|| "project store not initialized")?;
    store
        .write(path, contents.as_bytes(), false)
        .map_err(|e| e.to_string())?;
    store.flush().await.map_err(|e| e.to_string())
}

/// Load a text file from OPFS (mirror read).
#[cfg(target_arch = "wasm32")]
pub(crate) async fn js_load_file(path: &str) -> Result<String, String> {
    let bytes = with_project_store()
        .ok_or_else(|| "project store not initialized")?
        .read(path)
        .map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| format!("Not UTF-8: {}", e))
}

/// Check if a file exists in OPFS (mirror read).
#[cfg(target_arch = "wasm32")]
pub(crate) async fn js_exists(path: &str) -> bool {
    with_project_store()
        .map(|s| s.exists(path).unwrap_or(false))
        .unwrap_or(false)
}

/// List files under a prefix in OPFS (mirror read).
#[cfg(target_arch = "wasm32")]
pub(crate) async fn js_list_files(path: &str) -> Result<Vec<String>, String> {
    with_project_store()
        .ok_or_else(|| "project store not initialized")?
        .list(path)
        .map(|entries| entries.into_iter().map(|e| e.path).collect())
        .map_err(|e| e.to_string())
}

/// Delete a file from OPFS and flush (durability-preserving).
#[cfg(target_arch = "wasm32")]
pub(crate) async fn js_delete_file(path: &str) -> Result<(), String> {
    let store = with_project_store().ok_or_else(|| "project store not initialized")?;
    store.delete(path).map_err(|e| e.to_string())?;
    store.flush().await.map_err(|e| e.to_string())
}

/// Save binary bytes to OPFS and flush (durability-preserving).
#[cfg(target_arch = "wasm32")]
pub(crate) async fn js_save_binary(path: &str, contents: &[u8]) -> Result<(), String> {
    let store = with_project_store().ok_or_else(|| "project store not initialized")?;
    store
        .write(path, contents, false)
        .map_err(|e| e.to_string())?;
    store.flush().await.map_err(|e| e.to_string())
}

/// Load binary bytes from OPFS (mirror read).
#[cfg(target_arch = "wasm32")]
pub(crate) async fn js_load_binary(path: &str) -> Result<Vec<u8>, String> {
    with_project_store()
        .ok_or_else(|| "project store not initialized")?
        .read(path)
        .map_err(|e| e.to_string())
}

/// List all scene names from `project.json`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn list_scenes() -> Result<JsValue, JsValue> {
    if !js_exists(PROJECT_FILE).await {
        return serde_wasm_bindgen::to_value(&Vec::<String>::new())
            .map_err(|e| JsValue::from_str(&e.to_string()));
    }
    let json_str = js_load_file(PROJECT_FILE)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    let project: ProjectMetadata = serde_json::from_str(&json_str)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    serde_wasm_bindgen::to_value(&project.scenes).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Check if `project.json` exists in OPFS.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn project_exists() -> bool {
    js_exists(PROJECT_FILE).await
}

/// Internal helper: await a JS Promise and return its resolved JsValue.
/// Used only by update_project_metadata (which needs direct OPFS access for now).
#[cfg(target_arch = "wasm32")]
async fn js_await(promise: js_sys::Promise) -> Result<JsValue, JsValue> {
    let fut = JsFuture::from(promise);
    fut.await
        .map_err(|e| JsValue::from_str(&format!("JS promise rejected: {:?}", e)))
}

/// `window.opfs_*` externs — used by update_project_metadata for direct OPFS
/// access during the transition period (ADR-0031 WASM clause).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = opfs_save_file)]
    fn opfs_save_file_raw(path: &str, contents: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_load_file)]
    fn opfs_load_file_raw(path: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_exists)]
    fn opfs_exists_raw(path: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_delete_file)]
    fn opfs_delete_file_raw(path: &str) -> js_sys::Promise;
}

#[cfg(target_arch = "wasm32")]
async fn update_project_metadata(scene_name: &str) -> Result<(), String> {
    // Direct OPFS calls during transition — this function will also migrate
    // to PROJECT_STORE in a future PR.
    let project = if js_exists(PROJECT_FILE).await {
        match js_load_file(PROJECT_FILE).await {
            Ok(json_str) => serde_json::from_str::<ProjectMetadata>(&json_str).unwrap_or_default(),
            Err(_) => ProjectMetadata::default(),
        }
    } else {
        ProjectMetadata::default()
    };
    let mut project = project;
    if !project.scenes.contains(&scene_name.to_string()) {
        project.scenes.push(scene_name.to_string());
    }
    let json = serde_json::to_string(&project).map_err(|e| e.to_string())?;
    js_save_file(PROJECT_FILE, &json).await
}

// ─────────────────────────────────────────────────────────────────────────────
// Source Files WASM surface — CRUD for Rust source files in OPFS `sources/`
// ─────────────────────────────────────────────────────────────────────────────

/// List all source files in the `sources/` directory.
/// Returns `OpfsResult<Vec<SourceFile>>` with shape `{ok: true, value: [SourceFile, ...]}`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn list_source_files() -> Result<JsValue, JsValue> {
    use crate::source_files::{SourceFile, SourceFileId};

    // Lazy-create sources/ directory on first list (if no files exist, dir may not exist yet)
    let files = match js_list_files(SOURCES_DIR).await {
        Ok(names) => names,
        Err(_) => Vec::new(), // sources/ dir doesn't exist yet — return empty list
    };

    let sources: Vec<SourceFile> = files
        .iter()
        .filter(|name| name.ends_with(".rs"))
        .filter_map(|name| {
            // name is like "src/main.rs" → strip extension → "src/main"
            let path = name.strip_suffix(".rs")?;
            let file_name = std::path::Path::new(name)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(name);
            Some(SourceFile {
                id: SourceFileId::new(path.to_string()),
                path: path.to_string(),
                name: file_name.to_string(),
            })
        })
        .collect();

    let response = serde_json::json!({ "ok": true, "value": sources });
    serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Read a source file's content by id.
/// Returns `OpfsResult<String>` with shape `{ok: true, value: "<content>"}` on success,
/// or `{ok: false, error: "<message>"}` on failure.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn read_source_file(id: &str) -> Result<JsValue, JsValue> {
    let path = crate::source_files::source_path_from_id(id);
    match js_load_file(&path).await {
        Ok(content) => {
            let response = serde_json::json!({ "ok": true, "value": content });
            serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => {
            let response = serde_json::json!({ "ok": false, "error": e });
            serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
        }
    }
}

/// Write content to a source file by id.
/// Creates the file if it doesn't exist, overwrites if it does.
/// Returns `OpfsResult<()>` — empty value on success.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn write_source_file(id: &str, content: &str) -> Result<JsValue, JsValue> {
    let path = crate::source_files::source_path_from_id(id);
    // Lazy-create sources/ directory by creating the first file
    // js_save_file handles the file creation within the dir
    match js_save_file(&path, content).await {
        Ok(()) => {
            let response = serde_json::json!({ "ok": true });
            serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => {
            let response = serde_json::json!({ "ok": false, "error": e });
            serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
        }
    }
}

/// Create a new source file with the given path and name.
/// The `path` is the id (e.g., "src/main"), `name` is the display name (e.g., "main.rs").
/// Returns `OpfsResult<SourceFile>` with the created file's metadata.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn create_source_file(path: &str, name: &str) -> Result<JsValue, JsValue> {
    use crate::source_files::{SourceFile, SourceFileId};

    let id = path; // id IS the path for source files
    let full_path = crate::source_files::source_path_from_id(id);

    // Check if file already exists
    if js_exists(&full_path).await {
        let response = serde_json::json!({ "ok": false, "error": "File already exists" });
        return serde_wasm_bindgen::to_value(&response)
            .map_err(|e| JsValue::from_str(&e.to_string()));
    }

    // Create empty file
    match js_save_file(&full_path, "").await {
        Ok(()) => {
            let file = SourceFile {
                id: SourceFileId::new(id.to_string()),
                path: path.to_string(),
                name: name.to_string(),
            };
            let response = serde_json::json!({ "ok": true, "value": file });
            serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => {
            let response = serde_json::json!({ "ok": false, "error": e });
            serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
        }
    }
}

/// Delete a source file by id.
/// Returns `OpfsResult<()>` — empty value on success.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn delete_source_file(id: &str) -> Result<JsValue, JsValue> {
    let path = crate::source_files::source_path_from_id(id);
    let response = match js_delete_file(&path).await {
        Ok(()) => serde_json::json!({ "ok": true }),
        Err(e) => serde_json::json!({ "ok": false, "error": e }),
    };
    serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Asset Files WASM surface — CRUD for binary texture assets in OPFS `resources/`
// ─────────────────────────────────────────────────────────────────────────────

/// Metadata sidecar for binary assets (stored as JSON next to the binary file).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AssetMeta {
    mime_type: String,
    size_bytes: u64,
}

/// List all asset files in the `resources/` directory.
/// Returns `OpfsResult<Vec<AssetFile>>` with shape `{ok: true, value: [AssetFile, ...]}`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn list_asset_files() -> Result<JsValue, JsValue> {
    use crate::asset_files::RESOURCE_DIR;

    let files = match js_list_files(RESOURCE_DIR).await {
        Ok(names) => names,
        Err(_) => Vec::new(), // resources/ dir doesn't exist yet — return empty list
    };

    let mut assets: Vec<AssetFile> = Vec::new();
    for name in files.iter().filter(|name| !name.ends_with(".meta.json")) {
        let id = name.to_string();
        let path = name.to_string();
        let file_name = std::path::Path::new(name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(name);

        // Try to read metadata sidecar
        let meta_path = format!("{}/{}.meta.json", RESOURCE_DIR, name);
        if let Ok(meta_json) = js_load_file(&meta_path).await {
            if let Ok(meta) = serde_json::from_str::<AssetMeta>(&meta_json) {
                assets.push(AssetFile {
                    id: AssetFileId::new(id),
                    path,
                    name: file_name.to_string(),
                    kind: AssetFileKind::Texture, // Default kind; extensible later
                    mime_type: meta.mime_type,
                    size_bytes: meta.size_bytes,
                });
            }
        }
    }

    let response = serde_json::json!({ "ok": true, "value": assets });
    serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Import a new asset file: saves binary bytes to OPFS and creates metadata sidecar.
/// Returns `OpfsResult<AssetFile>` with the created file's metadata.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn import_asset_file(
    name: &str,
    mime_type: &str,
    bytes: js_sys::Uint8Array,
) -> Result<JsValue, JsValue> {
    use crate::asset_files::{RESOURCE_DIR, is_supported_mime};

    // Validate MIME type
    if !is_supported_mime(mime_type) {
        let response = serde_json::json!({
            "ok": false,
            "error": format!("Unsupported MIME type: {}", mime_type)
        });
        return serde_wasm_bindgen::to_value(&response)
            .map_err(|e| JsValue::from_str(&e.to_string()));
    }

    let id = name.to_string();
    let file_path = format!("{}/{}", RESOURCE_DIR, name);
    let meta_path = format!("{}/{}.meta.json", RESOURCE_DIR, name);

    let size_bytes = bytes.length() as u64;
    let contents = bytes.to_vec();

    // Save binary file
    match js_save_binary(&file_path, &contents).await {
        Ok(()) => {
            // Save metadata sidecar
            let meta = AssetMeta {
                mime_type: mime_type.to_string(),
                size_bytes,
            };
            let meta_json = serde_json::to_string(&meta)
                .map_err(|e| JsValue::from_str(&format!("Meta serialization error: {}", e)))?;

            match js_save_file(&meta_path, &meta_json).await {
                Ok(()) => {
                    let asset = AssetFile {
                        id: AssetFileId::new(id.clone()),
                        path: id,
                        name: name.to_string(),
                        kind: AssetFileKind::Texture,
                        mime_type: mime_type.to_string(),
                        size_bytes,
                    };
                    let response = serde_json::json!({ "ok": true, "value": asset });
                    serde_wasm_bindgen::to_value(&response)
                        .map_err(|e| JsValue::from_str(&e.to_string()))
                }
                Err(e) => {
                    // Rollback: delete the binary file on meta write failure
                    let _ = js_delete_file(&file_path).await;
                    let response = serde_json::json!({ "ok": false, "error": e });
                    serde_wasm_bindgen::to_value(&response)
                        .map_err(|e| JsValue::from_str(&e.to_string()))
                }
            }
        }
        Err(e) => {
            let response = serde_json::json!({ "ok": false, "error": e });
            serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
        }
    }
}

/// Read binary bytes of an asset file by id.
/// Returns `OpfsResult<Uint8Array>` with shape `{ok: true, value: Uint8Array}`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn read_asset_file_bytes(id: &str) -> Result<JsValue, JsValue> {
    use crate::asset_files::RESOURCE_DIR;

    let path = format!("{}/{}", RESOURCE_DIR, id);
    match js_load_binary(&path).await {
        Ok(bytes) => {
            let response = serde_json::json!({ "ok": true, "value": bytes });
            serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => {
            let response = serde_json::json!({ "ok": false, "error": e });
            serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
        }
    }
}

/// Delete an asset file by id (both binary and metadata sidecar).
/// Returns `OpfsResult<()>` — empty value on success.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn delete_asset_file(id: &str) -> Result<JsValue, JsValue> {
    use crate::asset_files::RESOURCE_DIR;

    let file_path = format!("{}/{}", RESOURCE_DIR, id);
    let meta_path = format!("{}/{}.meta.json", RESOURCE_DIR, id);

    // Delete both binary file and metadata sidecar
    let file_result = js_delete_file(&file_path).await;
    let meta_result = js_delete_file(&meta_path).await;

    let response = match (file_result, meta_result) {
        (Ok(()), Ok(())) => serde_json::json!({ "ok": true }),
        (Err(e), _) | (_, Err(e)) => serde_json::json!({ "ok": false, "error": e }),
    };
    serde_wasm_bindgen::to_value(&response).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get the current SceneDocument as JSON. Returns null if no scene loaded.
/// Read-only — does NOT mutate state, operation log, or dirty flag.
#[wasm_bindgen]
pub fn get_scene_snapshot() -> JsValue {
    let doc = scene_session::snapshot_active_doc();
    match doc.as_ref() {
        Some(doc) => match serde_json::to_string(doc) {
            Ok(json) => JsValue::from_str(&json),
            Err(_) => JsValue::NULL,
        },
        None => JsValue::NULL,
    }
}

/// Export a SceneDocument JSON string to runnable Bevy 0.19 Rust source code.
/// Returns a JSON object with shape:
/// `{ source: String, warnings: ExportWarning[] }`
///
/// Use `JSON.parse(returnedString)` on the JS side to get the object.
#[wasm_bindgen]
pub fn export_code(doc_json: &str) -> Result<JsValue, JsValue> {
    let doc: SceneDocument = serde_json::from_str(doc_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    let schemas = schema::combined_registry();
    let result = code_export::export_rust_source(&doc, &schemas);

    let response = serde_json::json!({
        "source": result.source,
        "warnings": result.warnings,
    });

    serde_wasm_bindgen::to_value(&response)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

// ─────────────────────────────────────────────────────────────────────────────
// Schema Registry Persistence — wasm_bindgen surface
// ─────────────────────────────────────────────────────────────────────────────

/// Helper: get a schema's JSON from the combined registry.
fn get_schema_json(type_id: &str) -> Result<String, JsValue> {
    let combined = schema::combined_registry();
    let schema = combined
        .get(type_id)
        .ok_or_else(|| JsValue::from_str(&format!("Schema not found: {}", type_id)))?;
    serde_json::to_string(schema).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get the source location for a component schema type_id.
/// Returns JSON string of SourceLocation or "null" if not found / not set.
#[wasm_bindgen]
pub fn find_source_location(type_id: &str) -> Result<String, JsValue> {
    let registry = schema::combined_registry();
    match registry.get(type_id) {
        Some(schema) => {
            Ok(serde_json::to_string(&schema.source_location)
                .unwrap_or_else(|_| "null".to_string()))
        }
        None => Ok("null".to_string()),
    }
}

/// Find all entity stable IDs in the current scene that have a component of the given type.
#[wasm_bindgen]
pub fn find_entities_by_type(type_id: &str) -> Result<String, JsValue> {
    let matching: Vec<String> = scene_session::snapshot_active_doc()
        .map(|doc| {
            doc.entities
                .iter()
                .filter(|e| e.components.iter().any(|c| c.type_id == type_id))
                .map(|e| e.id.as_str().to_string())
                .collect()
        })
        .unwrap_or_default();
    serde_json::to_string(&matching).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Helper: update project.json's schemas list (add or remove a type_id).
#[cfg(target_arch = "wasm32")]
async fn update_project_schemas(type_id: &str, add: bool) -> Result<(), String> {
    let mut project = if js_exists(PROJECT_FILE).await {
        match js_load_file(PROJECT_FILE).await {
            Ok(json_str) => serde_json::from_str::<ProjectMetadata>(&json_str).unwrap_or_default(),
            Err(_) => ProjectMetadata::default(),
        }
    } else {
        ProjectMetadata::default()
    };

    if add {
        if !project.schemas.contains(&type_id.to_string()) {
            project.schemas.push(type_id.to_string());
        }
    } else {
        project.schemas.retain(|s| s != type_id);
    }

    let json = serde_json::to_string(&project).map_err(|e| e.to_string())?;
    js_save_file(PROJECT_FILE, &json).await
}

/// Save a schema to OPFS at `schemas/<type_id>.schema.json`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn save_schema(type_id: &str) -> Result<String, JsValue> {
    let schema_json = get_schema_json(type_id)?;
    let path = persistence::schema_path(type_id);
    js_save_file(&path, &schema_json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    update_project_schemas(type_id, true)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    Ok(path)
}

/// Load a schema from OPFS and register it in the combined registry.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn load_schema(type_id: &str) -> Result<String, JsValue> {
    let path = persistence::schema_path(type_id);
    let json_str = js_load_file(&path)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    let schema: schema::ComponentSchema = serde_json::from_str(&json_str)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    schema::register_schema(schema).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(json_str)
}

/// Delete a schema from OPFS and unregister it (built-ins protected).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn delete_schema(type_id: &str) -> Result<(), JsValue> {
    if schema::is_builtin_type(type_id) {
        return Err(JsValue::from_str("Cannot delete built-in schema"));
    }
    let path = persistence::schema_path(type_id);
    let promise = opfs_delete_file_raw(&path);
    js_await(promise)
        .await
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    schema::unregister_schema(type_id).map_err(|e| JsValue::from_str(&e.to_string()))?;
    update_project_schemas(type_id, false)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    Ok(())
}

/// List all schemas (built-in + user).
#[wasm_bindgen]
pub fn list_schemas() -> Result<JsValue, JsValue> {
    let combined = schema::combined_registry();
    let type_ids: Vec<String> = combined.iter().map(|s| s.type_id.clone()).collect();
    serde_wasm_bindgen::to_value(&type_ids).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Register a schema from JSON (in-memory only, no OPFS save).
/// Built-in schemas (editor.*) are rejected.
#[wasm_bindgen]
pub fn register_schema_from_json(schema_json: &str) -> Result<(), JsValue> {
    let schema: schema::ComponentSchema = serde_json::from_str(schema_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    schema::register_schema(schema).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(())
}

/// Unregister a schema (built-ins protected, no OPFS touch).
#[wasm_bindgen]
pub fn unregister_schema(type_id: &str) -> Result<(), JsValue> {
    schema::unregister_schema(type_id).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Hito 4 Order 7 — SceneComponent authoring WASM exports
// ─────────────────────────────────────────────────────────────────────────────

/// Create a new SceneComponent schema (Bevy 0.19 `#[derive(SceneComponent)]`).
/// Accepts a full `ComponentSchema` JSON. The schema's `kind` field MUST be
/// `SceneComponent` and `bound_scene_asset_ref` MUST reference an existing
/// scene asset.
///
/// Returns the registered schema's `type_id` on success.
#[wasm_bindgen]
pub fn create_scene_component(schema_json: &str) -> Result<JsValue, JsValue> {
    let mut schema: schema::ComponentSchema = serde_json::from_str(schema_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    if schema.kind != schema::SchemaKind::SceneComponent {
        return Err(JsValue::from_str(
            "create_scene_component requires kind = SceneComponent",
        ));
    }
    // Resolve the bound_scene_asset_ref (must be present and non-empty)
    let asset_ref = match &schema.bound_scene_asset_ref {
        Some(s) if !s.is_empty() => s.clone(),
        _ => {
            return Err(JsValue::from_str(
                "create_scene_component requires non-empty bound_scene_asset_ref",
            ));
        }
    };
    // Store the type_id to return before moving the schema
    let type_id = schema.type_id.clone();
    // Register via the standard path (validates SceneComponent requires binding)
    schema::register_schema(schema).map_err(|e| JsValue::from_str(&e.to_string()))?;
    // Note: the actual scene asset is not stored via this call — the caller
    // is expected to have already created the scene asset via the existing
    // create_scene_asset WASM export. The binding here is a pointer to the
    // existing asset (no further validation needed at this level).
    Ok(JsValue::from_str(&type_id))
}

/// Bind an existing schema to a scene asset. Pass `scene_asset_id = None`
/// to clear the binding (downgrades SceneComponent → Simple).
#[wasm_bindgen]
pub fn bind_scene_to_schema(type_id: &str, scene_asset_id: Option<String>) -> Result<(), JsValue> {
    // Read the current schema from whichever registry holds it (built-in
    // or user), mutate it, and re-register via the user registry.
    // Built-in schemas CAN be bound (this is how the editor augments
    // built-ins with Scene Component metadata), but the resulting schema
    // is registered as a user override.
    let mut schema = if schema::is_builtin_type(type_id) {
        schema::global_registry()
            .get(type_id)
            .ok_or_else(|| JsValue::from_str(&format!("Schema not found: {}", type_id)))?
            .clone()
    } else {
        let user = schema::USER_SCHEMAS
            .with(|r| r.borrow().get(type_id).cloned())
            .ok_or_else(|| JsValue::from_str(&format!("Schema not found: {}", type_id)))?;
        user
    };
    match &scene_asset_id {
        Some(s) if !s.is_empty() => {
            schema.kind = schema::SchemaKind::SceneComponent;
            schema.bound_scene_asset_ref = Some(s.clone());
        }
        _ => {
            schema.kind = schema::SchemaKind::Simple;
            schema.bound_scene_asset_ref = None;
        }
    }
    schema::register_schema(schema).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(())
}

/// List all schemas with `kind = SceneComponent` as a JSON array.
#[wasm_bindgen]
pub fn list_scene_component_schemas() -> Result<JsValue, JsValue> {
    let schemas: Vec<&schema::ComponentSchema> = schema::global_registry()
        .iter()
        .filter(|s| s.kind == schema::SchemaKind::SceneComponent)
        .collect();
    let json = serde_json::to_string(&schemas)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;
    Ok(JsValue::from_str(&json))
}

/// Check if a type_id is a built-in.
#[wasm_bindgen]
pub fn is_builtin_type(type_id: &str) -> bool {
    schema::is_builtin_type(type_id)
}

/// Combined registry size (built-ins + user).
#[wasm_bindgen]
pub fn combined_registry_size() -> usize {
    schema::combined_registry().iter().count()
}

/// Return the full combined registry (built-ins + user) as a JSON array string.
/// Used by the AI-assisted editing frontend service to send schema context
/// to the Ollama/OpenAI proxy endpoint.
#[wasm_bindgen]
pub fn get_combined_schemas_json() -> String {
    let combined = schema::combined_registry();
    let schemas: Vec<&schema::ComponentSchema> = combined.iter().collect();
    serde_json::to_string(&schemas).unwrap_or_else(|_| "[]".to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Scene Registry — multi-scene WASM surface
// ─────────────────────────────────────────────────────────────────────────────

/// Create a new scene with the given name.
/// Returns the actual name used (may differ if name was duplicate).
#[wasm_bindgen]
pub fn scene_create(name: &str) -> Result<String, JsValue> {
    with_registry_mut(|r| r.create(name)).map_err(|e| e.to_js_value())
}

/// Probe a scene switch. Returns `SwitchResult` indicating whether
/// the switch happened directly or requires a dirty-prompt round-trip.
///
/// - If `switched: true`: the scene was switched immediately (source was clean).
/// - If `dirty_prompt_required: true`: frontend must show dialog, then call
///   `scene_switch_commit(target_id)` after user resolves Save/Discard.
#[wasm_bindgen]
pub fn scene_switch(id: &str) -> Result<JsValue, JsValue> {
    let result = with_registry(|r| r.switch(id)).map_err(|e| e.to_js_value())?;

    if result.switched {
        // Perform the value-swap: store current to registry, load target into thread_locals
        perform_scene_swap(&result.source_name, id);
    }

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Commit a scene switch after the user resolves the dirty prompt.
/// Call this ONLY after Save or Discard has cleared the source's dirty flag.
#[wasm_bindgen]
pub fn scene_switch_commit(id: &str) -> Result<(), JsValue> {
    // Get the current id before we overwrite it (clone for use after lock release)
    let old_id =
        with_registry(|r| r.current_id()).ok_or_else(|| JsValue::from_str("No current scene"))?;

    with_registry_mut(|r| r.commit_switch(id)).map_err(|e| e.to_js_value())?;

    // Perform value-swap (old_id is owned, so safe to use after lock release)
    perform_scene_swap(&old_id, id);
    Ok(())
}

/// Perform the actual value-swap between scenes.
/// Stores current SCENE_DOC/OPERATION_LOG to registry[old_id],
/// then loads registry[new_id] into the thread_locals.
fn perform_scene_swap(old_id: &str, new_id: &str) {
    // Store current scene back to registry
    let doc_opt = SCENE_DOC.with(|s| s.borrow().clone());
    let log = OPERATION_LOG.with(|l| l.borrow().clone());

    let (doc, log) = match doc_opt {
        Some(doc) => (doc, log),
        None => (
            crate::document::SceneDocument {
                version: "0.1".to_string(),
                scene_id: format!("scratch-{}", crate::time::now_nanos()),
                name: old_id.to_string(),
                entities: Vec::new(),
                instances: BTreeMap::new(),
            },
            crate::operation_log::OperationLog::new_const(),
        ),
    };

    with_registry_mut(|r| r.store_to(old_id, doc, log));

    // Load new scene from registry into thread_locals
    if let Some((new_doc, new_log)) = with_registry(|r| r.swap_in(new_id)) {
        scene_session::replace_active_doc(new_doc);
        OPERATION_LOG.with(|l| *l.borrow_mut() = new_log);
    }

    mark_dirty();
}

/// Delete a scene. Fails if it's the last remaining scene.
#[wasm_bindgen]
pub fn scene_delete(id: &str) -> Result<(), JsValue> {
    with_registry_mut(|r| r.delete(id)).map_err(|e| e.to_js_value())
}

/// Rename a scene. Returns the actual new name (may differ if duplicate).
#[wasm_bindgen]
pub fn scene_rename(id: &str, new_name: &str) -> Result<String, JsValue> {
    with_registry_mut(|r| r.rename(id, new_name)).map_err(|e| e.to_js_value())
}

/// List all scenes with extended metadata (id, name, isCurrent, isDirty).
#[wasm_bindgen]
pub fn list_scenes_extended() -> JsValue {
    let scenes = with_registry(|r| r.list());
    serde_wasm_bindgen::to_value(&scenes).unwrap_or_else(|_| JsValue::NULL)
}

/// Get the current scene ID.
#[wasm_bindgen]
pub fn get_current_scene_id() -> Option<String> {
    with_registry(|r| r.current_id())
}

/// List entities in a specific scene by scene ID.
/// Returns a vector of { stable_id, local_id, name } for each entity.
#[wasm_bindgen]
pub fn list_scene_entities(scene_id: &str) -> JsValue {
    let result: Vec<serde_json::Value> = with_registry(|r| {
        let entry = match r.get(scene_id) {
            Some(e) => e,
            None => return Vec::new(),
        };
        entry
            .scene
            .entities
            .iter()
            .map(|e| {
                serde_json::json!({
                    "stable_id": e.id.as_str(),
                    "local_id": e.local_id.as_str(),
                    "name": e.name,
                })
            })
            .collect()
    });
    serde_wasm_bindgen::to_value(&result).unwrap_or_else(|_| JsValue::NULL)
}

/// Discard unsaved changes in the current scene by reloading it from OPFS.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn discard_scene_changes(id: &str) -> Result<(), JsValue> {
    let path = persistence::scene_path(id);
    let json_str = js_load_file(&path)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    let doc: SceneDocument = serde_json::from_str(&json_str)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    // Store reloaded doc back to registry and to thread_locals if current
    let current_id = with_registry(|r| r.current_id());
    let log = OperationLog::new_const(); // Fresh log on discard

    with_registry_mut(|r| r.store_to(id, doc.clone(), log.clone()));

    if current_id.as_deref() == Some(id) {
        scene_session::replace_active_doc(doc);
        OPERATION_LOG.with(|l| *l.borrow_mut() = log);
    }

    with_registry_mut(|r| r.clear_current_dirty());
    mark_dirty();
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// load_project integration — populates SceneRegistry from OPFS
// ─────────────────────────────────────────────────────────────────────────────

/// Warm the ASSET_BODY_CACHE by loading all scene asset bodies from OPFS.
///
/// Called after `load_project` clears the cache. This ensures that subsequent
/// `place_scene_instance` calls find assets in cache without needing to load
/// them individually.
///
/// For each catalog entry, loads the body file and stores it keyed by
/// `logical_path` in the cache.
#[cfg(target_arch = "wasm32")]
async fn warm_asset_body_cache() {
    use crate::scene_asset::SceneAssetDocument;

    // Access the catalog that was just loaded
    let entries: Vec<editor_model::scene_asset_catalog::SceneAssetCatalogEntry> =
        crate::asset_state::with_asset_catalog(|cat| cat.list_all().into_iter().cloned().collect());

    for entry in entries {
        let path = &entry.logical_path;
        let body_exists = js_exists(&persistence::asset_path(path)).await;
        if !body_exists {
            continue; // Skip missing bodies (catalog warnings already emitted)
        }

        match js_load_file(&persistence::asset_path(path)).await {
            Ok(body_json) => {
                match serde_json::from_str::<SceneAssetDocument>(&body_json) {
                    Ok(doc) => {
                        with_asset_body_cache_mut(|cache| {
                            cache.insert(path.clone(), doc);
                        });
                    }
                    Err(_) => {
                        // Skip invalid JSON - catalog warnings already handle this
                    }
                }
            }
            Err(_) => {
                // Skip load failures
            }
        }
    }
}

/// Load complete project: project.json + schemas + all scenes (atomic).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn load_project() -> Result<(), JsValue> {
    if !js_exists(PROJECT_FILE).await {
        return Err(JsValue::from_str("project.json not found"));
    }
    let project_json = js_load_file(PROJECT_FILE)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    let project: ProjectMetadata = serde_json::from_str(&project_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    // Step A: Clear ASSET_BODY_CACHE (D4)
    with_asset_body_cache_mut(|cache| {
        cache.clear();
    });

    // Register all schemas first (so AddComponent validates against them)
    for schema_id in &project.schemas {
        load_schema(schema_id).await.map_err(|e| {
            JsValue::from_str(&format!(
                "Failed to load schema {}: {:?}",
                schema_id,
                e.as_string().unwrap_or_default()
            ))
        })?;
    }

    // Load all scenes into the registry
    let active = project.active_scene.clone();
    for scene_name in &project.scenes {
        let path = persistence::scene_path(scene_name);
        if js_exists(&path).await {
            match js_load_file(&path).await {
                Ok(json_str) => {
                    let doc: SceneDocument = serde_json::from_str(&json_str).unwrap_or_else(|_| {
                        crate::document::SceneDocument {
                            version: "0.1".to_string(),
                            scene_id: format!("loaded-{}", crate::time::now_nanos()),
                            name: scene_name.clone(),
                            entities: Vec::new(),
                            instances: BTreeMap::new(),
                        }
                    });
                    let log = OperationLog::new_const();
                    with_registry_mut(|r| r.load_scene(scene_name.clone(), doc, log));
                }
                Err(_) => {
                    // Skip scenes that fail to load; they'll be absent from registry
                }
            }
        }
    }

    // Set active scene (defaults to first if not specified)
    let active_id = active.or_else(|| project.scenes.first().cloned());
    with_registry_mut(|r| r.set_current(active_id.clone()));

    // Load the active scene into SCENE_DOC thread_local for preview
    if let Some(ref active_name) = active_id {
        let path = persistence::scene_path(active_name);
        if js_exists(&path).await {
            if let Ok(json_str) = js_load_file(&path).await {
                if let Ok(doc) = serde_json::from_str::<SceneDocument>(&json_str) {
                    scene_session::replace_active_doc(doc);
                }
            }
        }
    }

    // Rebuild Scene Asset catalog from project.scene_assets (ADR-0008 §Decision).
    // For each catalog entry, check if the body file exists. If missing, emit a
    // typed CatalogWarning (S16) and KEEP the entry — never silent delete.
    clear_asset_catalog_warnings();
    let mut catalog = SceneAssetCatalog::new();
    for entry in &project.scene_assets {
        let lp = &entry.logical_path;
        let body_exists = js_exists(&persistence::asset_path(lp)).await;
        if !body_exists {
            // Orphan: body file is missing. Emit typed warning and keep entry.
            let warning = CatalogWarning {
                code: "orphaned_index".to_string(),
                message: format!(
                    "asset '{}' (id={}) is listed in project.json but the body file is missing",
                    lp, entry.asset_id
                ),
                asset_id: Some(entry.asset_id.clone()),
                logical_path: Some(lp.clone()),
            };
            // Push warning to the session (v0.91 PR2).
            let warning_clone = warning.clone();
            let _ = editor_model::ports::with_session_mut(|sess| {
                sess.asset_state_mut(crate::asset_state::ACTIVE_ASSET_PATH)
                    .catalog_warnings
                    .push(warning_clone);
            });
        }
        // Register the entry (keep it regardless of orphan status)
        if let Err(e) = catalog.register(entry.clone()) {
            // If registration fails (e.g., duplicate), still keep the entry in warnings
            let warning = CatalogWarning {
                code: "catalog_error".to_string(),
                message: format!("failed to register asset '{}': {}", lp, e),
                asset_id: Some(entry.asset_id.clone()),
                logical_path: Some(lp.clone()),
            };
            // Push warning to the session (v0.91 PR2).
            let warning_clone = warning.clone();
            let _ = editor_model::ports::with_session_mut(|sess| {
                sess.asset_state_mut(crate::asset_state::ACTIVE_ASSET_PATH)
                    .catalog_warnings
                    .push(warning_clone);
            });
        }
    }
    // Store the rebuilt catalog in the session (v0.91 PR2).
    let catalog_clone = catalog.clone();
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(crate::asset_state::ACTIVE_ASSET_PATH)
            .catalog = Some(catalog_clone);
    });

    // Step D4: Warm ASSET_BODY_CACHE with all scene asset bodies
    warm_asset_body_cache().await;

    Ok(())
}

/// Save the current SceneDocument to OPFS at `scenes/<name>.scene.json`.
/// Also clears the `is_dirty` flag on the current scene entry.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn save_scene(name: &str) -> Result<String, JsValue> {
    let doc = scene_session::snapshot_active_doc()
        .ok_or_else(|| JsValue::from_str("No scene loaded — call load_scene_json first"))?;
    let doc_json =
        serde_json::to_string(&doc).map_err(|e| JsValue::from_str(&format!("serialize: {e}")))?;

    let path = persistence::scene_path(name);
    js_save_file(&path, &doc_json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    update_project_metadata(name)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Clear the dirty flag on the current scene (design decision 4)
    with_registry_mut(|r| r.clear_current_dirty());
    mark_dirty();
    Ok(path)
}

/// Resync Scene Instances on scene load.
///
/// Checks each instance's `asset_version_seen` against the current asset version.
/// If the asset has been bumped, calls `resync` to reclassify overrides.
///
/// Design S8/S9: Never silently delete overrides — orphaned patches are moved
/// to `orphaned_overrides` and a `ResyncReport` is returned for UI surfacing.
///
/// Returns a Vec of (instance_id, ResyncReport) for each instance that was
/// resynced. Empty Vec if no instances or no version mismatches.
fn resync_instances_on_load(
    doc: &mut SceneDocument,
) -> Vec<(crate::document::StableId, ResyncReport)> {
    use crate::scene_instance_overrides::resync;

    let mut reports = Vec::new();

    for (instance_id, instance) in doc.instances.iter_mut() {
        // Look up asset in catalog to get current version
        // First resolve the logical path to an asset_id, then look up the entry
        let asset_id = match with_asset_catalog(|cat| {
            cat.resolve_path(instance.asset_ref.as_str())
                .map(|s| s.to_string())
        }) {
            Some(id) => id,
            None => continue, // Unresolved path — skip
        };
        let entry = match with_asset_catalog(|cat| cat.get(&asset_id).cloned()) {
            Some(e) => e,
            None => continue, // Missing catalog entry — skip
        };

        // Check if version has changed
        if instance.asset_version_seen >= entry.current_version {
            continue; // No version bump
        }

        // Look up asset body in cache
        let asset = match with_asset_body_cache(|cache| cache.get(&entry.logical_path).cloned()) {
            Some(a) => a,
            None => continue, // Not in cache — skip (cache should be warm)
        };

        // Run resync
        let report = resync(&asset, instance, entry.current_version);
        reports.push((instance_id.clone(), report));
    }

    reports
}

#[cfg(target_arch = "wasm32")]
pub async fn load_scene(name: &str) -> Result<(), JsValue> {
    let path = persistence::scene_path(name);
    let json_str = js_load_file(&path)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    let mut doc: SceneDocument = serde_json::from_str(&json_str)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    // Run resync to catch any asset version bumps since last save
    let reports = resync_instances_on_load(&mut doc);
    // Store in thread-local for UI to drain via get_resync_reports()
    RESYNC_REPORTS.with(|r| *r.borrow_mut() = reports);

    scene_session::replace_active_doc(doc);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Scene Asset Authoring — WASM surface (ADR-0007, design §6)
// ─────────────────────────────────────────────────────────────────────────────

/// Apply an AssetCommand to the active SceneAssetDocument, mutating it and
/// producing an inverse command for undo. Returns the inverse as JSON.
///
/// Does NOT call mark_dirty() — asset changes don't affect the Bevy preview.
#[wasm_bindgen]
pub fn dispatch_asset_command(cmd_json: &str) -> Result<String, JsValue> {
    let cmd: AssetCommand = serde_json::from_str(cmd_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid command JSON: {}", e)))?;

    // v0.91 PR5: always route through AssetTransactionKernel. Legacy path removed.
    dispatch_asset_command_via_kernel(cmd)
}

/// Kernel path: route an asset command through AssetTransactionKernel.
fn dispatch_asset_command_via_kernel(cmd: AssetCommand) -> Result<String, JsValue> {
    use crate::transaction_bridge::asset_transaction_kernel;
    use editor_model::session::HistoryScope;
    use editor_model::transaction::{ChangeOrigin, ChangeSet};

    let timestamp = crate::time::now_nanos();

    // Wrap single command in a ChangeSet.
    let mut cs = ChangeSet::new(
        format!("asset-cmd-{}", timestamp),
        ChangeOrigin::Human,
        "user".to_string(),
        "asset edit".to_string(),
    );
    cs.add_resource("scene_asset", "assets/current.asset.json");
    cs.push_op(cmd.clone());

    // Get mutable access to the asset doc and log.
    let result_json = with_asset_doc_mut(|doc_opt| {
        let doc = doc_opt
            .as_mut()
            .ok_or_else(|| JsValue::from_str("No asset open — call open_scene_asset first"))?;

        let mut history = HistoryScope::new();
        let kernel = asset_transaction_kernel();

        let receipt = kernel
            .apply_atomic(&cs, doc, &mut history)
            .map_err(|e| JsValue::from_str(&format!("kernel apply failed: {}", e)))?;

        // Extract the inverse.
        let inverse = receipt.inverses.into_iter().next().unwrap_or_else(|| {
            // Should not happen for a well-formed apply, but handle gracefully.
            AssetCommand::RenameEntity {
                local_id: String::new(),
                old_name: None,
                new_name: String::new(),
            }
        });

        // Record in asset operation log for undo/redo.
        with_asset_log_mut(|log| {
            log.record(&cmd, inverse.clone());
        });

        serde_json::to_string(&inverse)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize inverse: {}", e)))
    })?;

    Ok(result_json)
}

/// Legacy v0.88 path: direct dispatch through asset_command::apply.
/// v0.91 PR5: removed. The kernel path is the only dispatch (ADR-0032).

/// Undo the last asset command. Returns the inverse command JSON.
#[wasm_bindgen]
pub fn undo_asset() -> Result<String, JsValue> {
    with_asset_doc_and_log_mut(|doc, log| {
        log.undo(doc)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        serde_json::to_string(&())
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
    })
    .map_err(|e| JsValue::from_str(e))?;
    Ok(String::new())
}

/// Redo the next asset command.
#[wasm_bindgen]
pub fn redo_asset() -> Result<String, JsValue> {
    with_asset_doc_and_log_mut(|doc, log| {
        log.redo(doc)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        serde_json::to_string(&())
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
    })
    .map_err(|e| JsValue::from_str(e))?;
    Ok(String::new())
}

/// Returns asset operation log metadata as JSON.
#[wasm_bindgen]
pub fn get_asset_log_state() -> String {
    with_asset_log(|log| {
        serde_json::json!({
            "size": log.get_log_size(),
            "can_undo": log.can_undo(),
            "can_redo": log.can_redo(),
            "cursor": log.get_cursor(),
            "dirty": log.is_dirty(),
        })
        .to_string()
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Logic Graph WASM surface
// ─────────────────────────────────────────────────────────────────────────────

/// Apply a LogicCommand to the active LogicGraphAsset, mutating it and
/// producing an inverse command for undo. Returns the inverse as JSON.
#[wasm_bindgen]
pub fn dispatch_logic_command(cmd_json: &str) -> Result<String, JsValue> {
    let cmd: LogicCommand = serde_json::from_str(cmd_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid command JSON: {}", e)))?;

    // v0.91 PR5: always route through LogicTransactionKernel. Legacy path removed.
    dispatch_logic_command_via_kernel(cmd)
}

/// Kernel path: route a logic command through LogicTransactionKernel.
fn dispatch_logic_command_via_kernel(cmd: LogicCommand) -> Result<String, JsValue> {
    use crate::transaction_bridge::logic_transaction_kernel;
    use editor_model::session::HistoryScope;
    use editor_model::transaction::{ChangeOrigin, ChangeSet};

    let timestamp = crate::time::now_nanos();

    // Wrap single command in a ChangeSet.
    let mut cs = ChangeSet::new(
        format!("logic-cmd-{}", timestamp),
        ChangeOrigin::Human,
        "user".to_string(),
        "logic edit".to_string(),
    );
    cs.add_resource("logic_graph", "logic/current.graph.json");
    cs.push_op(cmd.clone());

    // Get mutable access to the logic doc and log.
    let result_json = with_logic_graph_mut(|doc_opt| {
        let doc = doc_opt.as_mut().ok_or_else(|| {
            JsValue::from_str("No logic graph open — call create_logic_graph_asset first")
        })?;

        let mut history = HistoryScope::new();
        let kernel = logic_transaction_kernel();

        let receipt = kernel
            .apply_atomic(&cs, doc, &mut history)
            .map_err(|e| JsValue::from_str(&format!("kernel apply failed: {}", e)))?;

        // Extract the inverse.
        let inverse =
            receipt
                .inverses
                .into_iter()
                .next()
                .unwrap_or_else(|| LogicCommand::RemoveNode {
                    node_id: NodeId(String::new()),
                });

        // Record in logic operation log for undo/redo.
        with_logic_log_mut(|log| {
            log.record(&cmd, inverse.clone());
        });

        serde_json::to_string(&inverse)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize inverse: {}", e)))
    })?;

    Ok(result_json)
}

/// Legacy v0.88 path: direct dispatch through logic_command::apply.
/// v0.91 PR5: removed. The kernel path is the only dispatch (ADR-0032).

/// Undo the last logic command.
#[wasm_bindgen]
pub fn undo_logic() -> Result<String, JsValue> {
    let result_json = with_logic_graph_mut(|doc_opt| {
        with_logic_log_mut(|log| {
            let doc = doc_opt
                .as_mut()
                .ok_or_else(|| JsValue::from_str("No logic graph open"))?;
            log.undo(doc)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            serde_json::to_string(&())
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
        })
    })?;
    Ok(result_json)
}

/// Redo the next logic command.
#[wasm_bindgen]
pub fn redo_logic() -> Result<String, JsValue> {
    let result_json = with_logic_graph_mut(|doc_opt| {
        with_logic_log_mut(|log| {
            let doc = doc_opt
                .as_mut()
                .ok_or_else(|| JsValue::from_str("No logic graph open"))?;
            log.redo(doc)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            serde_json::to_string(&())
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
        })
    })?;
    Ok(result_json)
}

/// Returns logic operation log metadata as JSON.
#[wasm_bindgen]
pub fn get_logic_log_state() -> String {
    with_logic_log(|log| {
        serde_json::json!({
            "size": log.get_log_size(),
            "can_undo": log.can_undo(),
            "can_redo": log.can_redo(),
            "cursor": log.get_cursor(),
        })
        .to_string()
    })
}

/// Get the active LogicGraphAsset as JSON.
#[wasm_bindgen]
pub fn get_logic_graph() -> Result<String, JsValue> {
    with_logic_graph_mut(|doc_opt| {
        let doc = doc_opt
            .as_ref()
            .ok_or_else(|| JsValue::from_str("No logic graph open"))?;
        serde_json::to_string(doc)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
    })
}

/// Create a new empty LogicGraphAsset and set it as the active graph.
/// Saves the body to OPFS (catalog-first per ADR-0019) and registers in the
/// in-memory catalog.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn create_logic_graph_asset(
    asset_id: &str,
    logical_path: &str,
) -> Result<String, JsValue> {
    use crate::logic_graph::{LogicGraphAsset, LogicGraphCatalogEntry};

    let doc = LogicGraphAsset {
        asset_id: asset_id.to_string(),
        logical_path: logical_path.to_string(),
        version: 1,
        ..Default::default()
    };

    let now = crate::time::now_millis();

    // 1. Register in the in-memory catalog
    let entry = LogicGraphCatalogEntry {
        asset_id: asset_id.to_string(),
        logical_path: logical_path.to_string(),
        builtin: false,
        created_at: now,
        updated_at: now,
    };
    with_logic_graph_catalog_mut(|cat| {
        // Ignore duplicate errors — if it's already registered, just proceed
        let _ = cat.register(entry);
    });

    // 2. Save body to OPFS (catalog-first: body saved after catalog registration)
    if let Err(e) = crate::logic_graph::save_logic_graph_body(&doc).await {
        // Log but don't fail — the in-memory state is valid
        web_sys::console::error_1(
            &format!("[create_logic_graph_asset] OPFS save failed: {}", e).into(),
        );
    }

    // 3. Set as active graph
    with_logic_graph_mut(|doc_opt| {
        *doc_opt = Some(doc.clone());
    });

    // 4. Clear the operation log for the new graph
    with_logic_log_mut(|log| {
        log.clear();
    });

    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
}

/// Non-WASM stub for create_logic_graph_asset.
#[cfg(not(target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn create_logic_graph_asset(asset_id: &str, logical_path: &str) -> Result<String, JsValue> {
    use crate::logic_graph::LogicGraphAsset;
    let doc = LogicGraphAsset {
        asset_id: asset_id.to_string(),
        logical_path: logical_path.to_string(),
        version: 1,
        ..Default::default()
    };
    let json = serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))?;
    with_logic_graph_mut(|doc_opt| {
        *doc_opt = Some(doc);
    });
    with_logic_log_mut(|log| {
        log.clear();
    });
    Ok(json)
}

/// Open an existing LogicGraphAsset from OPFS by asset_id.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn open_logic_graph_asset(asset_id: &str) -> Result<String, JsValue> {
    // 1. Look up the catalog entry
    let entry = with_logic_graph_catalog(|cat| cat.get(asset_id).cloned())
        .ok_or_else(|| JsValue::from_str(&format!("Logic graph not found: {}", asset_id)))?;

    // 2. Load the body from OPFS
    let doc: crate::logic_graph::LogicGraphAsset =
        crate::logic_graph::load_logic_graph_body(&entry.logical_path)
            .await
            .map_err(|e| JsValue::from_str(&e))?;

    // 3. Set as active graph
    with_logic_graph_mut(|doc_opt| {
        *doc_opt = Some(doc.clone());
    });

    // 4. Clear the operation log
    with_logic_log_mut(|log| {
        log.clear();
    });

    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
}

/// Non-WASM stub for open_logic_graph_asset.
#[cfg(not(target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn open_logic_graph_asset(_asset_id: &str) -> Result<String, JsValue> {
    Err(JsValue::from_str(
        "open_logic_graph_asset not available on non-WASM target",
    ))
}

/// List all logic graph assets from the in-memory catalog.
/// Returns the catalog entries as JSON.
#[wasm_bindgen]
pub fn list_logic_graph_assets() -> Result<String, JsValue> {
    // Seed built-in recipes into the catalog so they appear in the listing.
    crate::logic_state::seed_builtin_recipes_to_catalog();
    let entries: Vec<crate::logic_graph::LogicGraphCatalogEntry> =
        with_logic_graph_catalog(|cat| cat.list_all().to_vec());
    serde_json::to_string(&entries)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
}

/// Get node descriptors from the global registry as JSON.
#[wasm_bindgen]
pub fn get_node_descriptors() -> Result<String, JsValue> {
    let registry = crate::logic_evaluator::global_node_registry();
    let descriptors: Vec<_> = registry.all_descriptors().values().cloned().collect();
    serde_json::to_string(&descriptors)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
}

// ─────────────────────────────────────────────────────────────────────────────
// Scene Asset Catalog WASM functions (design §6)
// ─────────────────────────────────────────────────────────────────────────────

/// Create a new Scene Asset with the given name and role.
/// Returns the new catalog entry as JSON.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn create_scene_asset(name: &str, role: &str) -> Result<String, JsValue> {
    use crate::scene_asset::SceneAssetRole;

    let role = match role {
        "actor" => SceneAssetRole::Actor,
        "fragment" => SceneAssetRole::Fragment,
        "screen" => SceneAssetRole::Screen,
        "level" => SceneAssetRole::Level,
        "ui" => SceneAssetRole::Ui,
        "effect" => SceneAssetRole::Effect,
        _ => return Err(JsValue::from_str(&format!("Unknown role: {}", role))),
    };

    let normalized_path = editor_model::scene_asset_catalog::normalize_logical_path(name);
    let asset_id = editor_model::scene_asset_catalog::mint_asset_id(
        &crate::time::JsSysClock::new(),
        &editor_model::scene_asset_catalog::random_hex_8(),
    );

    // Check for duplicate path
    let duplicate = with_asset_catalog(|cat| cat.resolve_path(&normalized_path).is_some());
    if duplicate {
        return Err(JsValue::from_str(&format!(
            "Duplicate logical path: {}",
            normalized_path
        )));
    }

    let now = crate::time::now_millis();

    let entry = editor_model::scene_asset_catalog::SceneAssetCatalogEntry {
        asset_id: asset_id.clone(),
        logical_path: normalized_path.clone(),
        role,
        current_version: 1,
        tags: vec![],
        created_at: now,
        updated_at: now,
        // ADR-0026: no authoring UI for `preview_resource` yet;
        // every newly-created asset defaults to None.
        preview_resource: None,
    };

    // Create empty document
    let doc = SceneAssetDocument {
        asset_id: asset_id.clone(),
        logical_path: normalized_path.clone(),
        role,
        version: 1,
        entities: vec![],
        relationships: vec![],
        exposed_properties: vec![],
        metadata: Default::default(),
        layers: vec![],
    };

    let doc_json = serde_json::to_string(&doc).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Write body file first
    js_save_file(&persistence::asset_path(&normalized_path), &doc_json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Register in catalog
    with_asset_catalog_mut(|cat| cat.register(entry.clone()))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Update project.json (awaited). On failure roll back the in-memory
    // registration so we don't publish a ghost. ADR-0019.
    if let Err(e) = update_project_metadata_for_asset(&entry, "create").await {
        with_asset_catalog_mut(|cat| {
            let _ = cat.unregister(&asset_id);
        });
        return Err(e);
    }

    serde_json::to_string(&entry).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Import a `.bsn` text file as a new Scene Asset.
///
/// This is the counterpart to `export_asset_to_bsn_wasm`: it parses the `.bsn`
/// text produced by `EditorCoreBsnExporter` and creates a new `SceneAssetDocument`
/// in the project.
///
/// The resulting document has `role = Fragment` (lossy round-trip semantics).
/// User can rename it after import.
///
/// Returns the JSON string of the new `SceneAssetCatalogEntry`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn import_bsn_asset_wasm(name: &str, bsn_text: &str) -> Result<String, JsValue> {
    // Parse and convert the BSN text
    let ir = crate::bsn_import::parse_bsn_text(bsn_text)
        .map_err(|e| JsValue::from_str(&format!("BSN parse error: {:?}", e)))?;
    let mut doc = crate::bsn_import::scene_asset_from_bsn_ir(ir);

    // Create an empty asset to get the asset_id and logical_path
    let entry_json = create_scene_asset(name, "fragment").await?;
    let entry: editor_model::scene_asset_catalog::SceneAssetCatalogEntry =
        serde_json::from_str(&entry_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse created entry: {}", e)))?;

    // Override the imported doc's ids to match the created asset
    doc.asset_id = entry.asset_id.clone();
    doc.logical_path = entry.logical_path.clone();
    doc.role = entry.role.clone();
    doc.version = entry.current_version;

    // Write the imported body to OPFS (overwriting the empty one created above)
    let doc_json = serde_json::to_string(&doc).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_save_file(&persistence::asset_path(&entry.logical_path), &doc_json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Return the entry (catalog already updated by create_scene_asset)
    serde_json::to_string(&entry).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Rename a Scene Asset (moves the file and updates catalog).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn rename_scene_asset(asset_id: &str, new_path: &str) -> Result<String, JsValue> {
    let new_path_normalized = editor_model::scene_asset_catalog::normalize_logical_path(new_path);

    // Get old entry
    let old_entry = with_asset_catalog(|cat| cat.get(asset_id).cloned())
        .ok_or_else(|| JsValue::from_str(&format!("Asset not found: {}", asset_id)))?;

    let old_path = &old_entry.logical_path;

    // Check for duplicate new path
    if old_path != &new_path_normalized {
        let duplicate = with_asset_catalog(|cat| cat.resolve_path(&new_path_normalized).is_some());
        if duplicate {
            return Err(JsValue::from_str(&format!(
                "Duplicate logical path: {}",
                new_path_normalized
            )));
        }
    }

    // Read old body
    let body = js_load_file(&persistence::asset_path(old_path))
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Write new body file
    js_save_file(&persistence::asset_path(&new_path_normalized), &body)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Delete old body file
    js_delete_file(&persistence::asset_path(old_path))
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Update catalog: unregister old, register new
    let new_entry = with_asset_catalog_mut(|cat| {
        let _ = cat
            .unregister(asset_id)
            .map_err(|e| JsValue::from_str(&e.to_string()));
        let mut new_entry = old_entry.clone();
        new_entry.logical_path = new_path_normalized.clone();
        new_entry.current_version += 1;
        let now = crate::time::now_millis();
        new_entry.updated_at = now;
        cat.register(new_entry.clone())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok::<_, JsValue>(new_entry)
    })?;

    // Update project.json (awaited). On failure restore the OLD entry under
    // its OLD logical_path so the next read does not see a half-rename.
    // ADR-0019.
    if let Err(e) = update_project_metadata_for_asset(&new_entry, "rename").await {
        with_asset_catalog_mut(|cat| {
            let _ = cat.unregister(asset_id);
            let mut restored = old_entry.clone();
            restored.current_version = restored.current_version.saturating_sub(1);
            let _ = cat.register(restored);
        });
        return Err(e);
    }

    // Invalidate ASSET_BODY_CACHE by old_path (D4)
    with_asset_body_cache_mut(|cache| {
        cache.remove(old_path);
    });

    serde_json::to_string(&new_entry).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Duplicate a Scene Asset.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn duplicate_scene_asset(asset_id: &str) -> Result<String, JsValue> {
    // Get source entry
    let source_entry = with_asset_catalog(|cat| cat.get(asset_id).cloned())
        .ok_or_else(|| JsValue::from_str(&format!("Asset not found: {}", asset_id)))?;

    let source_path = &source_entry.logical_path;

    // Read source body
    let body = js_load_file(&persistence::asset_path(source_path))
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Mint new id
    let new_id = editor_model::scene_asset_catalog::mint_asset_id(
        &crate::time::JsSysClock::new(),
        &editor_model::scene_asset_catalog::random_hex_8(),
    );
    let new_path = derive_duplicate_path(&source_entry.logical_path);

    let now = crate::time::now_millis();

    let new_entry = editor_model::scene_asset_catalog::SceneAssetCatalogEntry {
        asset_id: new_id.clone(),
        logical_path: new_path.clone(),
        role: source_entry.role,
        current_version: 1,
        tags: vec![],
        created_at: now,
        updated_at: now,
        // ADR-0026: duplicating an asset never inherits the source's
        // preview_resource — a duplicated asset has no preview until
        // an authoring flow (future cycle) sets one explicitly.
        preview_resource: None,
    };

    // Write new body file
    js_save_file(&persistence::asset_path(&new_path), &body)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Register in catalog
    with_asset_catalog_mut(|cat| cat.register(new_entry.clone()))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Update project.json (awaited). On failure roll back the duplicate's
    // in-memory registration; the source body file is untouched. ADR-0019.
    if let Err(e) = update_project_metadata_for_asset(&new_entry, "duplicate").await {
        with_asset_catalog_mut(|cat| {
            let _ = cat.unregister(&new_id);
        });
        return Err(e);
    }

    serde_json::to_string(&new_entry).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Delete a Scene Asset.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn delete_scene_asset(asset_id: &str) -> Result<(), JsValue> {
    // Get entry
    let entry = with_asset_catalog(|cat| cat.get(asset_id).cloned())
        .ok_or_else(|| JsValue::from_str(&format!("Asset not found: {}", asset_id)))?;

    let path = entry.logical_path.clone();

    // Delete body file
    js_delete_file(&persistence::asset_path(&path))
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Unregister from catalog
    with_asset_catalog_mut(|cat| cat.unregister(asset_id))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Update project.json — awaited so subsequent reads observe the removal.
    // Delete has no catalog rollback: the entry is intentionally gone. If the
    // metadata write fails, we propagate the error to the caller; the in-memory
    // catalog state matches the OPFS body state (both removed) which is the
    // correct partial state. See opfs-catalog-flake-fix ADR-0019.
    update_project_metadata_for_asset(&entry, "delete").await?;

    // Invalidate ASSET_BODY_CACHE by logical_path (D4)
    with_asset_body_cache_mut(|cache| {
        cache.remove(&path);
    });

    Ok(())
}

/// List all Scene Assets, optionally filtered by role.
#[wasm_bindgen]
pub fn list_scene_assets(role_filter: Option<String>) -> Result<String, JsValue> {
    let entries: Vec<editor_model::scene_asset_catalog::SceneAssetCatalogEntry> =
        with_asset_catalog(|cat| match role_filter {
            Some(role) => {
                use crate::scene_asset::SceneAssetRole;
                let r = match role.as_str() {
                    "actor" => SceneAssetRole::Actor,
                    "fragment" => SceneAssetRole::Fragment,
                    "screen" => SceneAssetRole::Screen,
                    "level" => SceneAssetRole::Level,
                    "ui" => SceneAssetRole::Ui,
                    "effect" => SceneAssetRole::Effect,
                    _ => return cat.list_all().into_iter().cloned().collect(),
                };
                cat.list_by_role(r).into_iter().cloned().collect()
            }
            None => cat.list_all().into_iter().cloned().collect(),
        });
    serde_json::to_string(&entries).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Open a Scene Asset by asset_id into SCENE_ASSET_DOC thread-local.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn open_scene_asset(asset_id: &str) -> Result<String, JsValue> {
    // Get entry
    let entry = with_asset_catalog(|cat| cat.get(asset_id).cloned())
        .ok_or_else(|| JsValue::from_str(&format!("Asset not found: {}", asset_id)))?;

    let path = &entry.logical_path;

    // Load body
    let body_json = js_load_file(&persistence::asset_path(path))
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    let doc: SceneAssetDocument = serde_json::from_str(&body_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    // Store in thread-local
    with_asset_doc_mut(|doc_opt| {
        *doc_opt = Some(doc.clone());
    });

    // Reset operation log
    with_asset_log_mut(|log| {
        log.clear();
    });

    serde_json::to_string(&doc).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Close the currently open Scene Asset (no-op if none open).
#[wasm_bindgen]
pub fn close_scene_asset() {
    with_asset_doc_mut(|doc_opt| {
        *doc_opt = None;
    });
    with_asset_log_mut(|log| {
        log.clear();
    });
}

/// Get the active SceneAssetDocument as JSON.
#[wasm_bindgen]
pub fn get_asset_document_json() -> Result<String, JsValue> {
    with_asset_doc(|doc_opt| match doc_opt {
        Some(doc) => serde_json::to_string(doc).map_err(|e| JsValue::from_str(&e.to_string())),
        None => Err(JsValue::from_str("No asset open")),
    })
}

/// Get the Scene Asset Catalog as JSON.
#[wasm_bindgen]
pub fn get_scene_asset_catalog_json() -> Result<String, JsValue> {
    let entries = with_asset_catalog(|cat| cat.list_all().into_iter().cloned().collect::<Vec<_>>());
    serde_json::to_string(&entries).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Save the active Scene Asset: body-first, then catalog update.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn save_scene_asset() -> Result<String, JsValue> {
    let (asset_id, path, doc_json) = with_asset_doc_mut(|doc_opt| {
        let doc = doc_opt
            .as_ref()
            .ok_or_else(|| JsValue::from_str("No asset open"))?;
        let asset_id = doc.asset_id.clone();
        let path = doc.logical_path.clone();
        let doc_json = serde_json::to_string(doc).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok::<_, JsValue>((asset_id, path, doc_json))
    })?;

    // Step 1: Write body file first
    js_save_file(&persistence::asset_path(&path), &doc_json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Step 2: Bump version in catalog
    let new_version = with_asset_catalog_mut(|cat| {
        let current = cat
            .get(&asset_id)
            .ok_or_else(|| JsValue::from_str(&format!("Asset not found: {}", asset_id)))?
            .current_version;
        let new_ver = current + 1;
        cat.update_version(&asset_id, new_ver)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok::<_, JsValue>(new_ver)
    })?;

    // Step 3: Write project.json
    let entries = with_asset_catalog(|cat| cat.list_all().into_iter().cloned().collect::<Vec<_>>());
    let mut project = load_project_metadata().await?;
    project.scene_assets = entries;
    let project_json =
        serde_json::to_string(&project).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_save_file(persistence::PROJECT_FILE, &project_json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Step 4: Clear dirty flag
    with_asset_log_mut(|log| {
        log.clear();
    });

    // Step 5: Invalidate ASSET_BODY_CACHE by logical_path (D4)
    with_asset_body_cache_mut(|cache| {
        cache.remove(&path);
    });

    Ok(format!("Saved {} v{}", path, new_version))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tileset Persistence (OPFS)
// ─────────────────────────────────────────────────────────────────────────────

/// Save a TilesetAsset to OPFS at `tilesets/<id>.tileset.json`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn save_tileset(tileset_json: &str) -> Result<String, JsValue> {
    let tileset: tileset::TilesetAsset = serde_json::from_str(tileset_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    let id = tileset.metadata.id.as_str();
    if id.is_empty() {
        return Err(JsValue::from_str("Tileset id cannot be empty"));
    }

    let path = persistence::tileset_path(id);
    let json = serde_json::to_string(&tileset)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

    js_save_file(&path, &json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    Ok(path)
}

/// Load a TilesetAsset from OPFS by tileset ID.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn load_tileset(id: &str) -> Result<String, JsValue> {
    if id.is_empty() {
        return Err(JsValue::from_str("Tileset id cannot be empty"));
    }

    let path = persistence::tileset_path(id);

    let json_str = js_load_file(&path)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    let tileset: tileset::TilesetAsset = serde_json::from_str(&json_str)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    serde_json::to_string(&tileset)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Delete a TilesetAsset from OPFS by tileset ID.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn delete_tileset(id: &str) -> Result<(), JsValue> {
    if id.is_empty() {
        return Err(JsValue::from_str("Tileset id cannot be empty"));
    }

    let path = persistence::tileset_path(id);
    js_delete_file(&path)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    Ok(())
}

/// List all tilesets in the `tilesets/` directory.
/// Returns a JSON array of TilesetMetadata objects.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn list_tilesets() -> Result<String, JsValue> {
    let dir = persistence::TILESETS_DIR;
    let files = js_list_files(dir)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    let mut tilesets: Vec<tileset::TilesetMetadata> = Vec::new();

    for file in files {
        // Only process .tileset.json files
        if !file.ends_with(".tileset.json") {
            continue;
        }
        let path = format!("{}/{}", dir, file);
        match js_load_file(&path).await {
            Ok(json_str) => {
                match serde_json::from_str::<tileset::TilesetAsset>(&json_str) {
                    Ok(tileset) => tilesets.push(tileset.metadata),
                    Err(_) => {
                        // Skip invalid tileset files
                    }
                }
            }
            Err(_) => {
                // Skip files that can't be read
            }
        }
    }

    serde_json::to_string(&tilesets)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────

/// Derive a unique path for duplication (appends `_2`, `_3`, etc. if collision).
fn derive_duplicate_path(original: &str) -> String {
    let base = format!("{}_2", original);
    // Check if collision
    let exists = with_asset_catalog(|cat| cat.resolve_path(&base).is_some());
    if exists {
        // Try _3, _4, etc.
        let mut counter = 3;
        loop {
            let candidate = format!("{}_{}", original, counter);
            if !with_asset_catalog(|cat| cat.resolve_path(&candidate).is_some()) {
                return candidate;
            }
            counter += 1;
            if counter > 100 {
                return base; // Fallback
            }
        }
    } else {
        base
    }
}

/// Update project.json with a modified scene_assets list.
#[cfg(target_arch = "wasm32")]
async fn update_project_metadata_for_asset(
    entry: &editor_model::scene_asset_catalog::SceneAssetCatalogEntry,
    _operation: &str,
) -> Result<(), JsValue> {
    let mut project = load_project_metadata().await?;

    // Find and replace or add entry
    if let Some(existing) = project
        .scene_assets
        .iter_mut()
        .find(|e| e.asset_id == entry.asset_id)
    {
        *existing = entry.clone();
    } else {
        project.scene_assets.push(entry.clone());
    }

    let json = serde_json::to_string(&project).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_save_file(persistence::PROJECT_FILE, &json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    Ok(())
}

/// Load project metadata from OPFS.

/// Load project metadata from OPFS.
#[cfg(target_arch = "wasm32")]
async fn load_project_metadata() -> Result<ProjectMetadata, JsValue> {
    if js_exists(persistence::PROJECT_FILE).await {
        let json_str = js_load_file(persistence::PROJECT_FILE)
            .await
            .map_err(|e| JsValue::from_str(&e))?;
        serde_json::from_str(&json_str)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))
    } else {
        Ok(ProjectMetadata::default())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Validation Center WASM boundary tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod validation_center_tests {
    use super::*;

    /// Test helper: set with_logic_graph for testing.
    #[cfg(test)]
    pub(crate) fn set_logic_graph_for_test(asset: Option<LogicGraphAsset>) {
        editor_model::ports::with_session_mut(|sess| {
            sess.logic_state_mut(crate::logic_state::ACTIVE_LOGIC_GRAPH_PATH)
                .graph_docs
                .insert(
                    "_active".to_string(),
                    asset.unwrap_or_else(|| {
                        // Empty graph fallback — tests should provide their own.
                        LogicGraphAsset::default()
                    }),
                );
        });
    }

    /// Test helper: clear with_logic_graph after each test.
    fn clear_logic_graph() {
        set_logic_graph_for_test(None);
    }

    // Test 1: active graph with a cycle → get_validation_issues_wasm returns Logic error
    #[test]
    fn wasm_validation_cycle_in_active_graph() {
        // Build a graph with a cycle: sensor.always -> controller.and -> controller.if
        // Then wire controller.if -> controller.and to create a cycle
        let node_sensor = LogicNode {
            node_id: NodeId::new("sensor"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.always"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_and = LogicNode {
            node_id: NodeId::new("and"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.and"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_if = LogicNode {
            node_id: NodeId::new("if"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.if"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };

        // sensor.tick -> and.a
        let edge1 = LogicEdge {
            from_node: NodeId::new("sensor"),
            from_port: PortId::new("tick"),
            to_node: NodeId::new("and"),
            to_port: PortId::new("a"),
        };
        // and.out -> if.condition
        let edge2 = LogicEdge {
            from_node: NodeId::new("and"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("if"),
            to_port: PortId::new("condition"),
        };
        // if.done -> and.b (creates cycle!)
        let edge3 = LogicEdge {
            from_node: NodeId::new("if"),
            from_port: PortId::new("done"),
            to_node: NodeId::new("and"),
            to_port: PortId::new("b"),
        };

        let asset = LogicGraphAsset {
            asset_id: "cycle_test".to_string(),
            logical_path: "logic/cycle".to_string(),
            version: 1,
            nodes: vec![node_sensor, node_and, node_if],
            edges: vec![edge1, edge2, edge3],
            ..Default::default()
        };

        set_logic_graph_for_test(Some(asset));

        let json = get_validation_issues_wasm().unwrap();
        let issues: Vec<ValidationIssue> = serde_json::from_str(&json).unwrap();

        let logic_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.category == ValidationCategory::Logic)
            .collect();

        assert!(
            !logic_issues.is_empty(),
            "expected logic issues for cycle, got none"
        );
        let cycle_issue = logic_issues.iter().find(|i| i.code == "cycle");
        assert!(
            cycle_issue.is_some(),
            "expected code='cycle', got: {:?}",
            logic_issues
        );
        assert!(cycle_issue.unwrap().affected_asset_id.is_some());
        assert_eq!(
            cycle_issue.unwrap().affected_asset_id.as_deref(),
            Some("cycle_test")
        );

        clear_logic_graph();
    }

    // Test 2: with_logic_graph is None → no logic issues
    #[test]
    fn wasm_validation_no_logic_issues_when_no_graph() {
        clear_logic_graph();

        let json = get_validation_issues_wasm().unwrap();
        let issues: Vec<ValidationIssue> = serde_json::from_str(&json).unwrap();

        let logic_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.category == ValidationCategory::Logic)
            .collect();

        assert!(
            logic_issues.is_empty(),
            "expected no logic issues when no graph, got: {:?}",
            logic_issues
        );
    }

    // Test 3: clean graph → no logic issues
    #[test]
    fn wasm_validation_clean_graph_no_logic_issues() {
        // sensor.always -> controller.and -> controller.if (no cycle)
        let node_sensor = LogicNode {
            node_id: NodeId::new("sensor"),
            role: LogicNodeRole::Sensor,
            node_type_id: NodeTypeId::new("sensor.always"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_and = LogicNode {
            node_id: NodeId::new("and"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.and"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };
        let node_if = LogicNode {
            node_id: NodeId::new("if"),
            role: LogicNodeRole::Controller,
            node_type_id: NodeTypeId::new("controller.if"),
            field_values: serde_json::json!({}),
            controller_id: None,
        };

        let edge1 = LogicEdge {
            from_node: NodeId::new("sensor"),
            from_port: PortId::new("tick"),
            to_node: NodeId::new("and"),
            to_port: PortId::new("a"),
        };
        let edge2 = LogicEdge {
            from_node: NodeId::new("and"),
            from_port: PortId::new("out"),
            to_node: NodeId::new("if"),
            to_port: PortId::new("condition"),
        };

        let asset = LogicGraphAsset {
            asset_id: "clean_test".to_string(),
            logical_path: "logic/clean".to_string(),
            version: 1,
            nodes: vec![node_sensor, node_and, node_if],
            edges: vec![edge1, edge2],
            ..Default::default()
        };

        set_logic_graph_for_test(Some(asset));

        let json = get_validation_issues_wasm().unwrap();
        let issues: Vec<ValidationIssue> = serde_json::from_str(&json).unwrap();

        let logic_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.category == ValidationCategory::Logic)
            .collect();

        assert!(
            logic_issues.is_empty(),
            "expected no logic issues for clean graph, got: {:?}",
            logic_issues
        );

        clear_logic_graph();
    }

    // Test: ValidationCategory::Logic serde round-trip
    #[test]
    fn validation_category_logic_serde() {
        let json = serde_json::to_string(&ValidationCategory::Logic).unwrap();
        assert_eq!(json, "\"logic\"");
        let parsed: ValidationCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ValidationCategory::Logic);
    }
}

/// Test helpers re-exported from editor-model for integration tests.
pub mod test_helpers {
    pub use editor_model::time::FakeClock;
}

// ===== Rust-source-integration tests =====
// Tests for find_source_location and find_entities_by_type WASM functions.
// These require wasm32 target to compile (wasm_bindgen JsValue dependency).

#[cfg(target_arch = "wasm32")]
#[cfg(test)]
mod rust_source_integration_tests {
    use super::*;

    /// Test helper: set SCENE_DOC for testing.
    fn set_scene_doc_for_test(doc: Option<SceneDocument>) {
        SCENE_DOC.with(|cell| {
            *cell.borrow_mut() = doc;
        });
    }

    /// Test helper: clear SCENE_DOC after each test.
    fn clear_scene_doc() {
        set_scene_doc_for_test(None);
    }

    // B.1: Rust unit test — find_entities_by_type
    #[test]
    fn find_entities_by_type_returns_matching_stable_ids() {
        use std::collections::BTreeMap;
        // Build a scene with entities that have components of specific type_ids
        let doc = SceneDocument {
            version: "0.1".to_string(),
            scene_id: "test_scene".to_string(),
            name: "Test Scene".to_string(),
            entities: vec![
                Entity {
                    id: StableId::new("ent_player"),
                    local_id: LocalId::new("ent_player"),
                    name: "Player".to_string(),
                    parent: None,
                    components: vec![
                        ComponentInstance {
                            type_id: "game.PlayerHealth".to_string(),
                            values: serde_json::json!({}),
                        },
                        ComponentInstance {
                            type_id: "game.Transform2D".to_string(),
                            values: serde_json::json!({}),
                        },
                    ],
                },
                Entity {
                    id: StableId::new("ent_enemy"),
                    local_id: LocalId::new("ent_enemy"),
                    name: "Enemy".to_string(),
                    parent: None,
                    components: vec![ComponentInstance {
                        type_id: "game.EnemyAI".to_string(),
                        values: serde_json::json!({}),
                    }],
                },
                Entity {
                    id: StableId::new("ent_ally"),
                    local_id: LocalId::new("ent_ally"),
                    name: "Ally".to_string(),
                    parent: None,
                    components: vec![ComponentInstance {
                        type_id: "game.PlayerHealth".to_string(),
                        values: serde_json::json!({}),
                    }],
                },
            ],
            instances: BTreeMap::new(),
        };

        set_scene_doc_for_test(Some(doc));

        // Call find_entities_by_type for game.PlayerHealth
        let result = find_entities_by_type("game.PlayerHealth").unwrap();
        let stable_ids: Vec<String> = serde_json::from_str(&result).unwrap();

        // Should return ent_player and ent_ally (both have PlayerHealth component)
        assert_eq!(stable_ids.len(), 2);
        assert!(stable_ids.contains(&"ent_player".to_string()));
        assert!(stable_ids.contains(&"ent_ally".to_string()));
        assert!(!stable_ids.contains(&"ent_enemy".to_string()));

        clear_scene_doc();
    }

    #[test]
    fn find_entities_by_type_returns_empty_for_unused_type() {
        clear_scene_doc();

        let result = find_entities_by_type("game.Unused").unwrap();
        let stable_ids: Vec<String> = serde_json::from_str(&result).unwrap();

        assert!(stable_ids.is_empty());
        clear_scene_doc();
    }

    #[test]
    fn find_entities_by_type_returns_empty_for_empty_scene() {
        use std::collections::BTreeMap;
        let doc = SceneDocument {
            version: "0.1".to_string(),
            scene_id: "empty_scene".to_string(),
            name: "Empty Scene".to_string(),
            entities: vec![],
            instances: BTreeMap::new(),
        };
        set_scene_doc_for_test(Some(doc));

        let result = find_entities_by_type("editor.Transform2D").unwrap();
        let stable_ids: Vec<String> = serde_json::from_str(&result).unwrap();

        assert!(stable_ids.is_empty());
        clear_scene_doc();
    }

    // B.2: Rust integration test — find_source_location
    #[test]
    fn find_source_location_returns_null_for_unknown_type() {
        // Ensure game.Unknown is not in the registry
        let result = find_source_location("game.Unknown_Type_12345").unwrap();
        assert_eq!(result, "null");
    }

    #[test]
    fn find_source_location_returns_null_for_type_without_source_location() {
        // Register a schema without source_location
        let schema = schema::ComponentSchema {
            type_id: "game.NoSource".to_string(),
            display_name: "NoSource".to_string(),
            fields: vec![],
            exports_to_bevy: true,
            source_location: None,
        };
        schema::register_schema(schema).unwrap();

        let result = find_source_location("game.NoSource").unwrap();
        assert_eq!(result, "null");

        let _ = schema::unregister_schema("game.NoSource");
    }

    #[test]
    fn find_source_location_returns_json_for_type_with_source_location() {
        // Register a schema with source_location
        let schema = schema::ComponentSchema {
            type_id: "game.HasSource".to_string(),
            display_name: "HasSource".to_string(),
            fields: vec![],
            exports_to_bevy: true,
            source_location: Some(schema::SourceLocation {
                file_id: "src/ecs/components.rs".to_string(),
                line: 42,
                column: 7,
            }),
        };
        schema::register_schema(schema).unwrap();

        let result = find_source_location("game.HasSource").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["file_id"], "src/ecs/components.rs");
        assert_eq!(parsed["line"], 42);
        assert_eq!(parsed["column"], 7);

        let _ = schema::unregister_schema("game.HasSource");
    }
}
