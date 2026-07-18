use bevy::prelude::Entity as BevyEntity;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

pub mod asset_command;
mod bevy_anchor;
pub mod bsn_ir;
pub mod bsn_codegen;
mod code_export;
pub mod command;
pub mod document;
mod asset_state;
mod dynamic_scene;
mod hot_reload_state;
mod logic_state;
mod operation_log;
mod persistence;
mod scene_state;
mod state;
mod wasm_bsn;
mod wasm_hot_reload;
mod wasm_preview;
pub mod auto_layer;
pub mod bsn_export;
pub mod bsn_import;
pub mod preview_inspector;
pub mod processor;
pub mod scene_asset;
pub mod scene_asset_catalog;
pub mod scene_instance;
pub mod scene_instance_overrides;
pub mod instance_projection;
mod scenes;
pub mod schema;
pub mod tileset;
pub mod tile_layer;
pub mod logic_graph;
pub mod logic_evaluator;
pub mod logic_validation;
pub mod logic_command;
pub mod logic_recipes;
pub mod logic_dispatch;
pub mod actuator_bus;
pub mod source_files;
pub mod asset_files;

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
}

pub use bevy_anchor::anchor_str_to_bevy_anchor;
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
pub use operation_log::{LogEntry, OperationLog, OperationLogError};
pub use persistence::{asset_path, validate_logical_path, AssetPathError, PROJECT_FILE, ProjectMetadata, SCENES_DIR, SCHEMAS_DIR, ASSETS_DIR, TILESETS_DIR, tileset_path};
pub use asset_command::{AssetCommand, AssetCommandError, AssetOperationLog};
pub use logic_command::{LogicCommand, LogicCommandError, LogicOperationLog};
pub use logic_recipes::{is_builtin_recipe, list_builtin_recipes, seed_builtin_recipes};
pub use scene_asset::{
    AssetReference, ExposedProperty, LayerId, LevelLayer, LocalId, RelationshipKind, RoleWarning,
    SceneAssetDocument, SceneAssetEntity, SceneAssetMetadata, SceneAssetRelationship,
    SceneAssetRole, SceneInstanceLayer, SceneInstanceLayerKind, validate_role,
};
pub use logic_graph::{
    count_logic_bindings, editor_logic_binding_component, find_dangling_edge_nodes,
    find_duplicate_node_id, LogicEdge, LogicGraphAsset, LogicInstance, LogicNode,
    LogicNodeRole, NodeId, NodeTypeId, PortId,
};
pub use logic_evaluator::{
    global_node_registry, NodeDescriptor, NodeEvaluator, PortSpec, PortValue, PortValueType,
    ParamSpec, LogicNodeRegistry,
};
pub use logic_validation::{
    validate_logic_graph, LogicValidationIssue, LogicValidationIssueCode,
};
pub use auto_layer::{
    AutoLayer, AutoLayerId, AutoRule, Pattern3x3, PatternCell, is_auto_layer_stale, regenerate,
};
pub use scene_asset_catalog::{
    CatalogError, CatalogWarning, SceneAssetCatalog, SceneAssetCatalogEntry, mint_asset_id,
};
pub use bsn_export::{
    BevyBsnExporter, BsnExportError, BsnExporter, EditorCoreBsnExporter, export_to_bsn_text,
    export_to_bsn_text_with_warnings,
};
pub use bsn_import::{BsnImportError, parse_bsn_text, scene_asset_from_bsn_ir};
pub use preview_inspector::{
    PreviewMappingEntry, PreviewMetrics, PreviewProvenance,
};
pub use scene_instance::{
    ComponentOverride, ComponentOverrideStatus, SceneInstance,
    component_override_status_after_field_rename,
};
pub use scene_instance_overrides::{OverrideIssue, ResyncReport};
pub use instance_projection::{root_local_ids, PreviewEntity, project_instances};
pub use source_files::{SourceFile, SourceFileId, SOURCES_DIR};
pub use asset_files::{AssetFile, AssetFileId, AssetFileKind, RESOURCE_DIR};
pub use schema::ComponentTypeId;
pub use scenes::{SceneInfo, SceneRegistry, SwitchResult};
pub use tileset::{
    AsepriteFrame, AsepriteMetadata, AsepriteSlice, AsepriteTag, TileCoord, TileGrid, TileRef,
    TilesetAsset, TilesetId, TilesetManager, TilesetMetadata,
};
pub use tile_layer::{TileLayer, TileLayerId};
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
    HotReloadRequest, PlayModeRequest,
    DIRTY_FLAG, SCENE_REGISTRY, SCENE_ASSET_CATALOG, SCENE_ASSET_DOC,
    SCENE_ASSET_CATALOG_WARNINGS, ASSET_OPERATION_LOG, ASSET_BODY_CACHE,
    RESYNC_REPORTS, VALIDATION_ISSUES, LOGIC_GRAPH_DOC, LOGIC_OPERATION_LOG,
    HOT_RELOAD_BUS, PLAY_MODE_REQUEST,
    with_registry, with_registry_mut, with_asset_catalog, with_asset_catalog_mut,
    with_asset_doc, with_asset_doc_mut, get_asset_catalog_warnings,
    clear_asset_catalog_warnings, with_asset_log, with_asset_log_mut,
    with_asset_body_cache, with_asset_body_cache_mut, mark_dirty,
    with_logic_graph, with_logic_graph_mut, with_logic_log, with_logic_log_mut,
};


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
    static COMMAND_BUS: RefCell<Option<LinearBus>> = const { RefCell::new(None) };
    static EVENT_BUS: RefCell<Option<LinearBus>> = const { RefCell::new(None) };
    static SCENE_DOC: RefCell<Option<SceneDocument>> = const { RefCell::new(None) };
    static OPERATION_LOG: RefCell<OperationLog> = const { RefCell::new(OperationLog::new_const()) };
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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = onFrameEnd)]
    fn on_frame_end();
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
    SCENE_DOC.with(|s| *s.borrow_mut() = Some(doc));
    web_sys::console::log_1(&"[editor-core] Scene document loaded".into());
    Ok(())
}

/// Apply a typed command to the SceneDocument, mutating it and producing
/// an inverse command for undo. Returns the inverse as JSON.
///
/// The command envelope (command + metadata) is parsed from JSON. On success,
/// the dirty flag is set so `rebuild_preview_world` respawns Bevy entities.
/// The command is also recorded in the operation log for undo/redo.
#[wasm_bindgen]
pub fn dispatch_command(json: &str) -> Result<String, JsValue> {
    let envelope: CommandEnvelope = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Invalid command JSON: {}", e)))?;

    let result_json = SCENE_DOC.with(|s| {
        let mut doc_ref = s.borrow_mut();
        let doc = doc_ref
            .as_mut()
            .ok_or_else(|| JsValue::from_str("No scene loaded — call load_scene_json first"))?;

        let inverse = processor::apply(doc, &envelope.command)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Record in operation log (consumes inverse)
        OPERATION_LOG.with(|l| {
            l.borrow_mut().record(&envelope, inverse.clone());
        });

        let result = CommandResult {
            inverse,
            snapshot: doc.clone(),
        };
        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    })?;

    mark_dirty();
    Ok(result_json)
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
    let asset = with_asset_body_cache(|cache| {
        cache.get(&entry.logical_path).cloned()
    }).ok_or_else(|| JsValue::from_str(&format!("Asset not in cache: {}. Call load_project first.", entry.logical_path)))?;

    // Step 3: Check single-root gate
    let roots = root_local_ids(&asset);
    if roots.is_empty() {
        return Err(JsValue::from_str("Empty asset: cannot place instance with zero entities"));
    }
    if roots.len() > 1 {
        return Err(JsValue::from_str(&format!(
            "Multiple roots: asset has {} root entities, expected 1",
            roots.len()
        )));
    }

    // Step 4: Mint fresh instance_id
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let instance_id = crate::document::StableId::new(format!("inst_{:x}", now));

    // Step 5: Mint id_map entries with `inst_{iid}_{lid}` pattern
    let id_map: std::collections::BTreeMap<crate::scene_asset::LocalId, crate::document::StableId> = asset
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

    let envelope_json = serde_json::to_string(&envelope)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize envelope: {}", e)))?;

    // Use dispatch_command to apply
    let result_json = dispatch_command(&envelope_json)?;

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

    let envelope_json = serde_json::to_string(&envelope)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize envelope: {}", e)))?;

    dispatch_command(&envelope_json)
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
    let new_entry = with_asset_catalog(|cat| cat.get(new_asset_id).cloned())
        .ok_or_else(|| JsValue::from_str(&format!("Asset not found in catalog: {}", new_asset_id)))?;

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

    let envelope_json = serde_json::to_string(&envelope)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize envelope: {}", e)))?;

    dispatch_command(&envelope_json)
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

/// Get `instance_components` for a given placed `instance_id`.
///
/// Returns a JSON array of `ComponentInstance` objects, or `null` if no
/// instance with that id is loaded. Useful for the Scene Instance Layer
/// authoring UI to surface placement-time components (e.g. the
/// `editor.Transform2D` translation created by `place_scene_instance`).
#[wasm_bindgen]
pub fn get_instance_components_wasm(instance_id: &str) -> JsValue {
    let stable_id = crate::document::StableId::new(instance_id);
    SCENE_DOC.with(|s| {
        let doc_ref = s.borrow();
        match doc_ref.as_ref() {
            None => JsValue::NULL,
            Some(doc) => match doc.instances.get(&stable_id) {
                None => JsValue::NULL,
                Some(instance) => match serde_json::to_string(&instance.instance_components) {
                    Ok(json) => JsValue::from_str(&json),
                    Err(_) => JsValue::NULL,
                },
            },
        }
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Override / Resync WASM surface
// ─────────────────────────────────────────────────────────────────────────────

/// Validate a SceneInstance's overrides against an asset document.
/// Returns a JSON array of OverrideIssue objects.
#[wasm_bindgen]
pub fn validate_overrides_wasm(
    instance_json: &str,
    asset_json: &str,
) -> Result<String, JsValue> {
    let instance: SceneInstance = serde_json::from_str(instance_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid instance JSON: {}", e)))?;
    let asset: SceneAssetDocument = serde_json::from_str(asset_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid asset JSON: {}", e)))?;

    let issues = crate::scene_instance_overrides::validate_overrides(&asset, &instance);
    serde_json::to_string(&issues)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize issues: {}", e)))
}

/// Compute effective values: read-only merge of asset + active overrides.
/// Returns a JSON ResolvedScene object.
#[wasm_bindgen]
pub fn effective_values_wasm(
    instance_json: &str,
    asset_json: &str,
) -> Result<String, JsValue> {
    let instance: SceneInstance = serde_json::from_str(instance_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid instance JSON: {}", e)))?;
    let asset: SceneAssetDocument = serde_json::from_str(asset_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid asset JSON: {}", e)))?;

    let mut counter = 0u32;
    let mut mint = || {
        counter += 1;
        crate::document::StableId::new(format!("sid_{}", counter))
    };

    let resolved = crate::scene_instance_overrides::effective_values(&asset, &instance, &mut mint)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_json::to_string(&resolved)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize resolved scene: {}", e)))
}

/// Try to rebind an orphaned ComponentOverride to a new asset.
/// Returns the matching LocalId as JSON string, or null if no match.
#[wasm_bindgen]
pub fn try_rebind_wasm(
    orphaned_override_json: &str,
    asset_json: &str,
) -> Result<String, JsValue> {
    let patch: ComponentOverride = serde_json::from_str(orphaned_override_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid component override JSON: {}", e)))?;
    let asset: SceneAssetDocument = serde_json::from_str(asset_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid asset JSON: {}", e)))?;

    match crate::scene_instance_overrides::try_rebind(&asset, &patch) {
        Some(local_id) => serde_json::to_string(&local_id)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize local_id: {}", e))),
        None => Ok("null".to_string()),
    }
}

/// Drain and return all accumulated resync reports from the last load/resync.
/// Returns JSON array of [stable_id, ResyncReport] tuples.
/// Clears the internal reports cache after draining.
#[wasm_bindgen]
pub fn get_resync_reports() -> Result<String, JsValue> {
    let reports = RESYNC_REPORTS.with(|r| {
        let mut reports = r.borrow_mut();
        let result = reports.clone();
        reports.clear();
        result
    });

    // Serialize as a JSON array of [stable_id, ResyncReport] tuples
    let mut as_arrays: Vec<serde_json::Value> = Vec::with_capacity(reports.len());
    for (stable_id, report) in reports {
        let report_obj = serde_json::json!({
            "active": report.active,
            "orphaned": report.orphaned,
            "stale": report.stale,
            "conflict": report.conflict,
            "rebound": report.rebound,
        });
        as_arrays.push(serde_json::json!([stable_id.as_str(), report_obj]));
    }

    serde_json::to_string(&as_arrays)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize reports: {}", e)))
}

// ─────────────────────────────────────────────────────────────────────────────
// Override Mutation WASM surface
// ─────────────────────────────────────────────────────────────────────────────

/// Get per-field override status index for a SceneInstance.
/// Returns a JSON array of FieldOverrideEntry objects.
#[wasm_bindgen]
pub fn override_field_status_wasm(instance_json: &str) -> Result<String, JsValue> {
    let instance: SceneInstance = serde_json::from_str(instance_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid instance JSON: {}", e)))?;

    let index = crate::scene_instance_overrides::field_override_index(&instance);
    serde_json::to_string(&index)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize index: {}", e)))
}

/// Upsert a component override on a Scene Instance.
///
/// Dispatches `Command::UpsertOverride` through the shared OperationLog.
/// Returns the `CommandResult` JSON on success.
#[wasm_bindgen]
pub fn upsert_override_wasm(
    instance_id: &str,
    local_id: &str,
    type_id: &str,
    field_path_json: &str,
    value_json: &str,
) -> Result<String, JsValue> {
    let field_path: Vec<String> = serde_json::from_str(field_path_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid field_path JSON: {}", e)))?;
    let value: serde_json::Value = serde_json::from_str(value_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid value JSON: {}", e)))?;

    let command = Command::UpsertOverride {
        instance_id: crate::document::StableId::new(instance_id.to_string()),
        target_local_id: crate::scene_asset::LocalId::new(local_id.to_string()),
        component_type_id: ComponentTypeId::new(type_id.to_string()),
        field_path,
        value,
    };

    let envelope = CommandEnvelope {
        command,
        metadata: CommandMetadata::now("user"),
    };

    let envelope_json = serde_json::to_string(&envelope)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize envelope: {}", e)))?;

    dispatch_command(&envelope_json)
}

/// Revert a component override on a Scene Instance.
///
/// Dispatches `Command::RevertOverride` through the shared OperationLog.
/// Returns the `CommandResult` JSON on success.
#[wasm_bindgen]
pub fn revert_override_wasm(
    instance_id: &str,
    local_id: &str,
    type_id: &str,
    field_path_json: &str,
) -> Result<String, JsValue> {
    let field_path: Vec<String> = serde_json::from_str(field_path_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid field_path JSON: {}", e)))?;

    let command = Command::RevertOverride {
        instance_id: crate::document::StableId::new(instance_id.to_string()),
        target_local_id: crate::scene_asset::LocalId::new(local_id.to_string()),
        component_type_id: ComponentTypeId::new(type_id.to_string()),
        field_path,
    };

    let envelope = CommandEnvelope {
        command,
        metadata: CommandMetadata::now("user"),
    };

    let envelope_json = serde_json::to_string(&envelope)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize envelope: {}", e)))?;

    dispatch_command(&envelope_json)
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
    let current_doc_opt = SCENE_DOC.with(|s| s.borrow().clone());
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
    with_logic_graph(|doc_opt| {
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
    let entry = with_asset_catalog(|cat| {
        cat.get(asset_id).cloned()
    }).ok_or_else(|| JsValue::from_str(&format!("Asset not found: {}", asset_id)))?;

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

/// Live preview metrics (fps, frame time in ms, total rebuild count).
/// Read-only; updated by the Bevy `emit_events` and `rebuild_preview_world`
/// systems.
#[wasm_bindgen]
pub fn get_preview_metrics_wasm() -> Result<String, JsValue> {
    let m = crate::preview_inspector::get_metrics();
    serde_json::to_string(&m)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize metrics: {}", e)))
}

/// Live preview mapping list. Each entry is editor-owned (`StableId`,
/// `LocalId`, `AssetReference`); no Bevy Entity IDs leak to JS.
#[wasm_bindgen]
pub fn get_preview_mapping_wasm() -> Result<String, JsValue> {
    let m = crate::preview_inspector::get_mapping();
    serde_json::to_string(&m)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize mapping: {}", e)))
}

/// Per-instance provenance detail. Returns `null` if the `stable_id` is not
/// currently projected.
#[wasm_bindgen]
pub fn get_preview_provenance_wasm(stable_id: &str) -> JsValue {
    match crate::preview_inspector::get_provenance(stable_id) {
        Some(p) => match serde_json::to_string(&p) {
            Ok(json) => JsValue::from_str(&json),
            Err(_) => JsValue::NULL,
        },
        None => JsValue::NULL,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Scene Instance Layer WASM surface
// ─────────────────────────────────────────────────────────────────────────────

/// Helper: parse `asset_json` as `SceneAssetDocument`. Returns error JsValue on failure.
fn parse_asset_doc(asset_json: &str) -> Result<SceneAssetDocument, JsValue> {
    serde_json::from_str(asset_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid asset JSON: {}", e)))
}

/// List Scene Instance Layers of a Scene Asset document.
///
/// Returns a JSON array of `{ id, name, kind, order, instances_count }`,
/// omitting the `instances` vector for brevity at this level.
#[wasm_bindgen]
pub fn list_scene_instance_layers_wasm(asset_json: &str) -> Result<String, JsValue> {
    let doc: SceneAssetDocument = parse_asset_doc(asset_json)?;

    let mut out: Vec<serde_json::Value> = Vec::with_capacity(doc.layers.len());
    for layer in &doc.layers {
        match layer {
            LevelLayer::SceneInstance(scene_layer) => {
                out.push(serde_json::json!({
                    "id": scene_layer.id.as_str(),
                    "name": scene_layer.name,
                    "kind": scene_layer.kind,
                    "order": scene_layer.order,
                    "instances_count": scene_layer.instances.len(),
                }));
            }
            LevelLayer::Tile(_) | LevelLayer::Auto(_) => {
                // Tile and Auto layers are handled separately in their respective APIs
            }
        }
    }
    serde_json::to_string(&out)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize layers: {}", e)))
}

/// Create a new Scene Instance Layer in the asset document and return the
/// updated asset JSON. Rejects unknown `kind` values.
#[wasm_bindgen]
pub fn create_scene_instance_layer_wasm(
    asset_json: &str,
    name: &str,
    kind: &str,
) -> Result<String, JsValue> {
    let mut doc: SceneAssetDocument = parse_asset_doc(asset_json)?;

    // Parse kind
    let parsed_kind: SceneInstanceLayerKind = match kind {
        "actors" => SceneInstanceLayerKind::Actors,
        "props" => SceneInstanceLayerKind::Props,
        "spawns" => SceneInstanceLayerKind::Spawns,
        "triggers" => SceneInstanceLayerKind::Triggers,
        "collision" => SceneInstanceLayerKind::Collision,
        "custom" => SceneInstanceLayerKind::Custom,
        other => {
            return Err(JsValue::from_str(&format!(
                "Unknown layer kind '{}'. Allowed: actors, props, spawns, triggers, collision, custom",
                other
            )))
        }
    };

    // Generate a stable layer id.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let new_id = LayerId::new(format!("lyr_{:x}", now));

    // Compute next order = max(order) + 1, falling back to 0.
    let next_order = doc
        .layers
        .iter()
        .filter_map(|l| match l {
            LevelLayer::SceneInstance(s) => Some(s.order),
            LevelLayer::Tile(_) | LevelLayer::Auto(_) => None,
        })
        .max()
        .map(|o| o + 1)
        .unwrap_or(0);

    doc.layers.push(LevelLayer::SceneInstance(SceneInstanceLayer {
        id: new_id,
        name: name.to_string(),
        kind: parsed_kind,
        order: next_order,
        instances: Vec::new(),
    }));

    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize asset: {}", e)))
}

/// Delete a Scene Instance Layer by id and return the updated asset JSON.
/// If the layer id is unknown, the asset is returned unchanged.
#[wasm_bindgen]
pub fn delete_scene_instance_layer_wasm(
    asset_json: &str,
    layer_id: &str,
) -> Result<String, JsValue> {
    let mut doc: SceneAssetDocument = parse_asset_doc(asset_json)?;
    let before = doc.layers.len();
    doc.layers
        .retain(|l| match l { LevelLayer::SceneInstance(s) => s.id.as_str() != layer_id, _ => true });
    if doc.layers.len() == before {
        // Unknown id is a no-op; return current asset.
        // Doc comment in spec: "Delete unknown layer is a no-op".
    }
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize asset: {}", e)))
}

/// Replace the open Scene Asset document in the backend with the given JSON.
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
    let snapshot_json = SCENE_DOC.with(|s_doc| {
        OPERATION_LOG.with(|s_log| {
            let mut log = s_log.borrow_mut();
            let mut doc_ref = s_doc.borrow_mut();
            let doc = doc_ref
                .as_mut()
                .ok_or_else(|| JsValue::from_str("No scene loaded — call load_scene_json first"))?;
            let snapshot = log
                .undo(doc)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            serde_json::to_string(&snapshot)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize snapshot: {}", e)))
        })
    })?;
    mark_dirty();
    Ok(snapshot_json)
}

/// Redo the next operation. Returns the new document snapshot as JSON.
#[wasm_bindgen]
pub fn redo() -> Result<String, JsValue> {
    let snapshot_json = SCENE_DOC.with(|s_doc| {
        OPERATION_LOG.with(|s_log| {
            let mut log = s_log.borrow_mut();
            let mut doc_ref = s_doc.borrow_mut();
            let doc = doc_ref
                .as_mut()
                .ok_or_else(|| JsValue::from_str("No scene loaded — call load_scene_json first"))?;
            let snapshot = log
                .redo(doc)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            serde_json::to_string(&snapshot)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize snapshot: {}", e)))
        })
    })?;
    mark_dirty();
    Ok(snapshot_json)
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start_engine(canvas_id: &str) {
    let canvas_selector = format!("#{}", canvas_id);
    web_sys::console::log_1(
        &format!(
            "[editor-core] Starting Bevy with canvas: {}",
            canvas_selector
        )
        .into(),
    );

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some(canvas_selector),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        // process_play_mode_request runs every frame to handle WASM enter/exit requests
        .add_systems(Update, process_play_mode_request.before(process_commands))
        // process_hot_reload_requests drains the HOT_RELOAD_BUS each frame before rebuild
        .add_systems(Update, process_hot_reload_requests.before(rebuild_preview_world))
        // Editor-only systems gated during play mode
        .add_systems(Update, process_commands.run_if(in_edit_mode))
        .add_systems(Update, rebuild_preview_world.run_if(in_edit_mode).after(process_commands))
        .add_systems(Update, sync_log_state.run_if(in_edit_mode).after(rebuild_preview_world))
        // Play-mode sensor systems — run before logic evaluation
        .add_systems(Update, logic_evaluator::update_keyboard_state.run_if(in_play_mode).before(logic_dispatch::logic_evaluation_system))
        // Logic dispatch runs only in play mode
        .add_systems(Update, logic_dispatch::logic_evaluation_system.run_if(in_play_mode).after(sync_log_state))
        .add_systems(Update, actuator_bus::apply_actuator_outputs.run_if(in_play_mode).after(logic_dispatch::logic_evaluation_system))
        .add_systems(Last, emit_events)
        .run();

    web_sys::console::log_1(&"[editor-core] Bevy app.run() returned".into());
}

#[wasm_bindgen]
pub fn get_command_bus_ptr() -> u32 {
    COMMAND_BUS.with(|b| b.borrow().as_ref().unwrap().ptr())
}

#[wasm_bindgen]
pub fn get_command_bus_len() -> u32 {
    COMMAND_BUS.with(|b| b.borrow().as_ref().unwrap().len())
}

#[wasm_bindgen]
pub fn get_event_bus_ptr() -> u32 {
    EVENT_BUS.with(|b| b.borrow().as_ref().unwrap().ptr())
}

#[wasm_bindgen]
pub fn get_event_bus_len() -> u32 {
    EVENT_BUS.with(|b| b.borrow().as_ref().unwrap().len())
}

/// Enter play mode — schedules a snapshot of all entity Transforms and
/// switches to Playing. Consumed by process_play_mode_request next frame.
#[wasm_bindgen]
pub fn enter_play_mode() {
    PLAY_MODE_REQUEST.with(|r| *r.borrow_mut() = Some(PlayModeRequest::Enter));
}

/// Exit play mode — restores entity Transforms from the snapshot and
/// switches back to Edit. Consumed by process_play_mode_request next frame.
#[wasm_bindgen]
pub fn exit_play_mode() {
    PLAY_MODE_REQUEST.with(|r| *r.borrow_mut() = Some(PlayModeRequest::Exit));
}


/// Returns the current state of DIRTY_FLAG for tests.
pub fn is_dirty_for_tests() -> bool {
    DIRTY_FLAG.with(|d| *d.borrow())
}

/// Clears the DIRTY_FLAG for tests (run before each test to ensure clean state).
pub fn clear_dirty_for_tests() {
    DIRTY_FLAG.with(|d| *d.borrow_mut() = false);
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Try to load scene from thread-local SCENE_DOC, otherwise use default
    let doc = SCENE_DOC.with(|s| s.borrow().clone());
    let scene = match doc {
        Some(doc) => doc,
        None => {
            // Deserialize default scene
            match serde_json::from_str(DEFAULT_SCENE_JSON) {
                Ok(doc) => doc,
                Err(e) => {
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::error_1(
                        &format!("[editor-core] Failed to parse default scene: {}", e).into(),
                    );
                    #[cfg(not(target_arch = "wasm32"))]
                    eprintln!(
                        "[editor-core] Failed to parse default scene: {}",
                        e
                    );
                    return;
                }
            }
        }
    };

    // Insert SceneDocumentState resource
    commands.insert_resource(SceneDocumentState::new(scene));
    // Insert OperationLogState resource (UI hooks read this)
    commands.insert_resource(OperationLogState::default());
    // Insert PlayMode resource (defaults to Edit)
    commands.insert_resource(PlayMode::default());
    // Insert TransformSnapshot
    commands.insert_resource(TransformSnapshot::default());
    mark_dirty();
}

/// Handles play mode enter/exit requests from WASM, snapshot/restore transforms.
/// Runs BEFORE process_commands so Enter transitions are committed before editor
/// commands are processed, and Exit restores before rebuild_preview_world.
fn process_play_mode_request(
    mut play_mode: ResMut<PlayMode>,
    mut snapshot: ResMut<TransformSnapshot>,
    scene_transforms: Query<(bevy::prelude::Entity, &Transform), With<SceneEntity>>,
    mut scene_transforms_mut: Query<(bevy::prelude::Entity, &mut Transform), With<SceneEntity>>,
) {
    let request = PLAY_MODE_REQUEST.with(|r| (*r.borrow()).clone());

    match request {
        Some(PlayModeRequest::Enter) => {
            // Snapshot all placed entity transforms
            snapshot.transforms.clear();
            for (entity, transform) in scene_transforms.iter() {
                snapshot.transforms.insert(entity, *transform);
            }
            *play_mode = PlayMode::Playing;
            PLAY_MODE_REQUEST.with(|r| *r.borrow_mut() = None);
        }
        Some(PlayModeRequest::Exit) => {
            // Restore transforms from snapshot
            for (entity, mut transform) in scene_transforms_mut.iter_mut() {
                if let Some(saved) = snapshot.transforms.get(&entity) {
                    *transform = *saved;
                }
            }
            *play_mode = PlayMode::Edit;
            PLAY_MODE_REQUEST.with(|r| *r.borrow_mut() = None);
        }
        None => {}
    }
}

/// Drains the HOT_RELOAD_BUS, de-duplicates by (variant, key), and dispatches:
/// - Asset{asset_id}  → ASSET_BODY_CACHE.remove(&asset_id) + mark_dirty()
/// - ForceReloadAll   → clear all caches + LOGIC_GRAPH_DOC=None + mark_dirty()
///
/// Runs in Update before rebuild_preview_world so stale data is purged
/// before the next preview render.
pub fn process_hot_reload_requests() {
    use std::collections::HashSet;

    // Collect and clear bus atomically
    let requests: Vec<HotReloadRequest> = HOT_RELOAD_BUS.with(|bus| {
        let mut v = bus.borrow_mut();
        std::mem::take(&mut *v)
    });

    if requests.is_empty() {
        return;
    }

    // De-duplicate by (variant discriminant, key string)
    // Use (u8, String) — variant index + inner identifier
    let mut seen: HashSet<(u8, String)> = HashSet::new();
    let deduped: Vec<HotReloadRequest> = requests
        .into_iter()
        .filter(|req| {
            let key = match req {
                HotReloadRequest::Source { file_id } => (0u8, file_id.clone()),
                HotReloadRequest::Asset { asset_id } => (1u8, asset_id.clone()),
                HotReloadRequest::ForceReloadAll => (2u8, String::new()),
            };
            seen.insert(key)
        })
        .collect();

    for req in deduped {
        match req {
            HotReloadRequest::Source { file_id } => {
                source_files::invalidate_cache(&file_id);
            }
            HotReloadRequest::Asset { asset_id } => {
                with_asset_body_cache_mut(|c| {
                    c.remove(&asset_id);
                });
                mark_dirty();
            }
            HotReloadRequest::ForceReloadAll => {
                source_files::clear_cache();
                with_asset_body_cache_mut(|c| {
                    c.clear();
                });
                with_logic_graph_mut(|doc| *doc = None);
                mark_dirty();
            }
        }
    }
}

/// Respawns scene entities when the SceneDocumentState dirty flag is set.
/// Triggered by `dispatch_command` setting DIRTY_FLAG via mark_dirty().
///
/// Design decision 3: Syncs SceneDocumentState.document from SCENE_DOC on every
/// dirty tick. This fixes both multi-scene switching AND pre-existing preview
/// staleness where entity edits never reached the canvas.
///
/// Design decision 7: Also projects Scene Instances via `project_instances`
/// and spawns them with `SceneEntity` + `SceneInstanceChild` tags.
fn rebuild_preview_world(
    mut commands: Commands,
    mut state: ResMut<SceneDocumentState>,
    scene_entities: Query<BevyEntity, With<SceneEntity>>,
) {
    // Check both the resource dirty flag and the cross-thread flag
    let external_dirty = DIRTY_FLAG.with(|d| *d.borrow());
    if !state.dirty && !external_dirty {
        return;
    }

    // Sync document from SCENE_DOC thread_local (value-swap source)
    // This ensures the preview reflects the currently active scene after a switch
    let current_doc = SCENE_DOC.with(|s| s.borrow().clone());
    if let Some(doc) = current_doc {
        state.document = doc;
    }

    // Despawn existing scene entities (Camera2d survives)
    for entity in scene_entities.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn authored entities from the document
    for entity in state.document.entities.iter() {
        spawn_entity(&mut commands, entity);
    }

    // Project and spawn Scene Instances
    let resolver = |asset_ref: &crate::scene_asset::AssetReference| -> Option<crate::scene_asset::SceneAssetDocument> {
        with_asset_body_cache(|cache| cache.get(asset_ref.as_str()).cloned())
    };
    let projected = project_instances(&state.document, &resolver);

    for preview in &projected {
        spawn_preview_entity(&mut commands, preview);
    }

    // runtime-preview-inspector: push mapping + provenance + bump rebuild_count
    push_preview_inspector_state(&state.document, &projected);

    state.dirty = false;
    DIRTY_FLAG.with(|d| *d.borrow_mut() = false);
}

/// Update the runtime preview inspector thread-locals after a rebuild.
/// `projected` is the list returned by `project_instances` for the same doc.
fn push_preview_inspector_state(
    doc: &SceneDocument,
    projected: &[crate::instance_projection::PreviewEntity],
) {
    use std::collections::BTreeMap;
    use crate::preview_inspector::{
        PreviewMappingEntry, PreviewProvenance, set_mapping, set_provenance,
    };

    let mut mapping: Vec<PreviewMappingEntry> = Vec::new();
    let mut provenance: BTreeMap<StableId, PreviewProvenance> = BTreeMap::new();

    // Build per-instance mapping/provenance from doc.instances + projected.
    for instance in doc.instances.values() {
        let projected_for_instance: Vec<&crate::instance_projection::PreviewEntity> = projected
            .iter()
            .filter(|p| p.stable_id == instance.instance_id)
            .collect();
        if projected_for_instance.is_empty() {
            continue;
        }
        // For the listing we keep one entry per (instance, root local_id) pair.
        // We expose the asset_ref of the instance plus the local_id of the
        // first projected root for context.
        for preview in projected_for_instance {
            mapping.push(PreviewMappingEntry {
                stable_id: preview.stable_id.clone(),
                local_id: preview.local_id.clone(),
                asset_ref: instance.asset_ref.clone(),
                component_count: preview.component_values.len(),
            });
            provenance.insert(
                preview.stable_id.clone(),
                PreviewProvenance {
                    stable_id: preview.stable_id.clone(),
                    local_id: preview.local_id.clone(),
                    asset_ref: instance.asset_ref.clone(),
                    components: preview
                        .component_values
                        .iter()
                        .map(|c| c.type_id.clone())
                        .collect(),
                    is_from_instance: true,
                },
            );
        }
    }

    set_mapping(mapping);
    set_provenance(provenance);
    crate::preview_inspector::increment_rebuild_count();
}

fn spawn_entity(commands: &mut Commands, entity: &Entity) {
    use bevy::prelude::Name as BevyName;
    use bevy::sprite::Anchor;

    let mut name: Option<BevyName> = None;
    let mut transform: Option<Transform> = None;
    let mut sprite: Option<Sprite> = None;
    let mut anchor_str: Option<String> = None;

    for component in &entity.components {
        match component.type_id.as_str() {
            "editor.Name" => {
                if let Some(name_val) = component.values.get("name") {
                    if let Some(name_str) = name_val.as_str() {
                        name = Some(BevyName::new(name_str.to_string()));
                    }
                }
            }
            "editor.Transform2D" => {
                let translation = component
                    .values
                    .get("translation")
                    .and_then(|v| v.get("x").zip(v.get("y")))
                    .map(|(x, y)| {
                        Vec3::new(
                            x.as_f64().unwrap_or(0.0) as f32,
                            y.as_f64().unwrap_or(0.0) as f32,
                            0.0,
                        )
                    })
                    .unwrap_or(Vec3::ZERO);

                let rotation = component
                    .values
                    .get("rotation")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;

                let scale = component
                    .values
                    .get("scale")
                    .and_then(|v| v.get("x").zip(v.get("y")))
                    .map(|(x, y)| {
                        Vec3::new(
                            x.as_f64().unwrap_or(1.0) as f32,
                            y.as_f64().unwrap_or(1.0) as f32,
                            1.0,
                        )
                    })
                    .unwrap_or(Vec3::new(1.0, 1.0, 1.0));

                transform = Some(
                    Transform::from_translation(translation)
                        .with_rotation(Quat::from_rotation_z(rotation))
                        .with_scale(scale),
                );
            }
            "editor.Sprite2D" => {
                let color = component
                    .values
                    .get("color")
                    .and_then(|v| {
                        let r = v.get("r").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
                        let g = v.get("g").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
                        let b = v.get("b").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
                        let a = v.get("a").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
                        Some(Color::srgba(r, g, b, a))
                    })
                    .unwrap_or(Color::WHITE);

                sprite = Some(Sprite {
                    color,
                    custom_size: Some(Vec2::splat(100.0)),
                    ..default()
                });

                // Read anchor string. Missing → silent Center default.
                // We track the raw string so we can warn on invalid values after the
                // mapping; the Bevy Anchor Component itself is inserted after Sprite
                // (so it overrides the `#[require(Anchor)]` auto-insert).
                if let Some(s) = component.values.get("anchor").and_then(|v| v.as_str()) {
                    anchor_str = Some(s.to_string());
                }
            }
            // Skip editorial-only components: editor.Visible, editor.Locked
            _ => {}
        }
    }

    // Build and spawn the entity
    let mut cmd = commands.spawn_empty();
    cmd.insert(SceneEntity);

    if let Some(n) = name {
        cmd.insert(n);
    }
    if let Some(t) = transform {
        cmd.insert(t);
    }
    if let Some(s) = sprite {
        // Insert Sprite first — Bevy's `#[require(Anchor)]` auto-inserts
        // `Anchor::default()` (= Anchor::CENTER) at this point.
        cmd.insert(s);

        // Insert our Anchor AFTER Sprite so it overrides the auto-required default.
        let raw_anchor = anchor_str.as_deref().unwrap_or("Center");
        if !is_known_anchor_str(raw_anchor) {
            // Use web-sys console.warn directly so the message reaches the browser
            // devtools / Playwright console listeners (Bevy's warn! goes to the
            // logger plugin, which is not configured to forward to the browser
            // console in this WASM build).
            #[cfg(target_arch = "wasm32")]
            {
                let msg = format!(
                    "[editor-core] Sprite2D anchor '{}' on entity {} is not recognized; using Center",
                    raw_anchor, entity.id
                );
                web_sys::console::warn_1(&JsValue::from_str(&msg));
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                eprintln!(
                    "[editor-core] Sprite2D anchor '{}' on entity {} is not recognized; using Center",
                    raw_anchor, entity.id
                );
            }
        }
        let bevy_anchor = anchor_str_to_bevy_anchor(raw_anchor);
        cmd.insert(Anchor::from(bevy_anchor.0));
    }
}

/// Spawn a projected entity from a Scene Instance.
///
/// This is similar to `spawn_entity` but uses the `PreviewEntity` structure
/// which carries the stable_id from the instance's id_map and the local_id
/// from the source asset. The entity is tagged with `SceneInstanceChild`
/// so it can be identified and despawned separately from authored entities.
fn spawn_preview_entity(commands: &mut Commands, preview: &PreviewEntity) {
    use bevy::prelude::Name as BevyName;
    use bevy::sprite::Anchor;

    let mut name: Option<BevyName> = None;
    let mut transform: Option<Transform> = None;
    let mut sprite: Option<Sprite> = None;
    let mut anchor_str: Option<String> = None;
    let mut logic_binding: Option<crate::logic_graph::LogicBinding> = None;

    for component in &preview.component_values {
        match component.type_id.as_str() {
            "editor.Name" => {
                if let Some(name_val) = component.values.get("name") {
                    if let Some(name_str) = name_val.as_str() {
                        name = Some(BevyName::new(name_str.to_string()));
                    }
                }
            }
            "editor.Transform2D" => {
                let translation = component
                    .values
                    .get("translation")
                    .and_then(|v| v.get("x").zip(v.get("y")))
                    .map(|(x, y)| {
                        Vec3::new(
                            x.as_f64().unwrap_or(0.0) as f32,
                            y.as_f64().unwrap_or(0.0) as f32,
                            0.0,
                        )
                    })
                    .unwrap_or(Vec3::ZERO);

                let rotation = component
                    .values
                    .get("rotation")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;

                let scale = component
                    .values
                    .get("scale")
                    .and_then(|v| v.get("x").zip(v.get("y")))
                    .map(|(x, y)| {
                        Vec3::new(
                            x.as_f64().unwrap_or(1.0) as f32,
                            y.as_f64().unwrap_or(1.0) as f32,
                            1.0,
                        )
                    })
                    .unwrap_or(Vec3::new(1.0, 1.0, 1.0));

                transform = Some(
                    Transform::from_translation(translation)
                        .with_rotation(Quat::from_rotation_z(rotation))
                        .with_scale(scale),
                );
            }
            "editor.Sprite2D" => {
                let color = component
                    .values
                    .get("color")
                    .and_then(|v| {
                        let r = v.get("r").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
                        let g = v.get("g").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
                        let b = v.get("b").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
                        let a = v.get("a").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
                        Some(Color::srgba(r, g, b, a))
                    })
                    .unwrap_or(Color::WHITE);

                sprite = Some(Sprite {
                    color,
                    custom_size: Some(Vec2::splat(100.0)),
                    ..default()
                });

                if let Some(s) = component.values.get("anchor").and_then(|v| v.as_str()) {
                    anchor_str = Some(s.to_string());
                }
            }
            // LogicBinding: deserialize asset_id and version for the dispatch scheduler
            "editor.LogicBinding" => {
                let asset_id = component
                    .values
                    .get("asset_id")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_default();
                let version = component
                    .values
                    .get("version")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                logic_binding = Some(crate::logic_graph::LogicBinding { asset_id, version });
            }
            // Skip editorial-only components
            _ => {}
        }
    }

    // Build and spawn the entity with SceneInstanceChild tag
    let mut cmd = commands.spawn_empty();
    cmd.insert(SceneEntity);
    cmd.insert(SceneInstanceChild {
        instance_id: preview.stable_id.clone(),
        local_id: preview.local_id.clone(),
    });

    if let Some(n) = name {
        cmd.insert(n);
    }
    if let Some(t) = transform {
        cmd.insert(t);
    }
    if let Some(s) = sprite {
        cmd.insert(s);
        let raw_anchor = anchor_str.as_deref().unwrap_or("Center");
        let bevy_anchor = anchor_str_to_bevy_anchor(raw_anchor);
        cmd.insert(Anchor::from(bevy_anchor.0));
    }
    if let Some(lb) = logic_binding {
        cmd.insert(lb);
    }
}

fn process_commands(mut sprites: Query<&mut Transform, With<Sprite>>) {
    let cmds = COMMAND_BUS.with(|b| {
        b.borrow_mut()
            .as_mut()
            .map(|bus| bus.drain())
            .unwrap_or_default()
    });

    if let Ok(mut transform) = sprites.single_mut() {
        for (cmd_type, payload) in cmds {
            if cmd_type == CMD_MOVE_SPRITE && payload.len() >= 8 {
                let x = f32::from_le_bytes(payload[0..4].try_into().unwrap());
                let y = f32::from_le_bytes(payload[4..8].try_into().unwrap());
                transform.translation.x = x;
                transform.translation.y = y;
            }
        }
    }
}

fn emit_events(
    sprites: Query<&Transform, With<Sprite>>,
    time: Res<Time>,
    mut fps_accum: Local<f32>,
    mut frame_count: Local<u32>,
) {
    EVENT_BUS.with(|b| {
        if let Some(bus) = b.borrow_mut().as_mut() {
            bus.reset();

            if let Ok(transform) = sprites.single() {
                let mut payload = [0u8; 8];
                payload[0..4].copy_from_slice(&transform.translation.x.to_le_bytes());
                payload[4..8].copy_from_slice(&transform.translation.y.to_le_bytes());
                bus.write(EVT_SPRITE_POSITION, &payload);
            }

            *fps_accum += time.delta_secs();
            *frame_count += 1;
            if *fps_accum >= 0.5 {
                let fps = *frame_count as f32 / *fps_accum;
                let mut payload = [0u8; 4];
                payload.copy_from_slice(&fps.to_le_bytes());
                bus.write(EVT_FPS, &payload);
                // runtime-preview-inspector: snapshot live metrics for the JS inspector.
                let frame_time_ms = (*fps_accum * 1000.0) / (*frame_count as f32).max(1.0);
                crate::preview_inspector::set_metrics(crate::preview_inspector::PreviewMetrics {
                    fps,
                    frame_time_ms,
                    rebuild_count: crate::preview_inspector::get_metrics().rebuild_count,
                });
                *fps_accum = 0.0;
                *frame_count = 0;
            }
        }
    });

    on_frame_end();
}

/// Sync the OperationLogState Resource from the thread_local! OperationLog.
/// UI hooks (future change) read this resource to enable/disable undo/redo buttons.
fn sync_log_state(mut log_state: ResMut<OperationLogState>) {
    OPERATION_LOG.with(|l| {
        let log = l.borrow();
        log_state.size = log.get_log_size();
        log_state.can_undo = log.can_undo();
        log_state.can_redo = log.can_redo();
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// OPFS Persistence — wasm_bindgen externs + high-level functions
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
/// Helper: await a JS Promise and return its resolved JsValue.
async fn js_await(promise: js_sys::Promise) -> Result<JsValue, JsValue> {
    let fut = JsFuture::from(promise);
    fut.await
        .map_err(|e| JsValue::from_str(&format!("JS promise rejected: {:?}", e)))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    /// JS-side: `window.opfs_save_file(path, contents) -> Promise<{ok, error?}>`
    #[wasm_bindgen(js_namespace = window, js_name = opfs_save_file)]
    pub fn opfs_save_file_raw(path: &str, contents: &str) -> js_sys::Promise;

    /// JS-side: `window.opfs_load_file(path) -> Promise<{ok, value?, error?}>`
    #[wasm_bindgen(js_namespace = window, js_name = opfs_load_file)]
    pub fn opfs_load_file_raw(path: &str) -> js_sys::Promise;

    /// JS-side: `window.opfs_list_files(path) -> Promise<{ok, value?, error?}>`
    #[wasm_bindgen(js_namespace = window, js_name = opfs_list_files)]
    pub fn opfs_list_files_raw(path: &str) -> js_sys::Promise;

    /// JS-side: `window.opfs_exists(path) -> Promise<boolean>`
    #[wasm_bindgen(js_namespace = window, js_name = opfs_exists)]
    pub fn opfs_exists_raw(path: &str) -> js_sys::Promise;

    /// JS-side: `window.opfs_delete_file(path) -> Promise<{ok, error?}>`
    #[wasm_bindgen(js_namespace = window, js_name = opfs_delete_file)]
    pub fn opfs_delete_file_raw(path: &str) -> js_sys::Promise;

    /// JS-side: `window.opfs_save_binary(path, Uint8Array) -> Promise<{ok, error?}>`
    #[wasm_bindgen(js_namespace = window, js_name = opfs_save_binary)]
    pub fn opfs_save_binary_raw(path: &str, contents: &js_sys::Uint8Array) -> js_sys::Promise;

    /// JS-side: `window.opfs_load_binary(path) -> Promise<{ok, value: Uint8Array, error?}>`
    #[wasm_bindgen(js_namespace = window, js_name = opfs_load_binary)]
    pub fn opfs_load_binary_raw(path: &str) -> js_sys::Promise;
}

#[cfg(target_arch = "wasm32")]
async fn js_save_file(path: &str, contents: &str) -> Result<(), String> {
    let promise = opfs_save_file_raw(path, contents);
    let result = js_await(promise).await.map_err(|e| format!("{:?}", e))?;
    let val: serde_json::Value = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Bad bridge response: {}", e))?;
    if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        Ok(())
    } else {
        Err(val
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error")
            .to_string())
    }
}

#[cfg(target_arch = "wasm32")]
async fn js_load_file(path: &str) -> Result<String, String> {
    let promise = opfs_load_file_raw(path);
    let result = js_await(promise).await.map_err(|e| format!("{:?}", e))?;
    let val: serde_json::Value = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Bad bridge response: {}", e))?;
    if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        val.get("value")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Missing value in bridge response".to_string())
    } else {
        Err(val
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error")
            .to_string())
    }
}

#[cfg(target_arch = "wasm32")]
async fn js_exists(path: &str) -> bool {
    let promise = opfs_exists_raw(path);
    match js_await(promise).await {
        Ok(v) => v.as_bool().unwrap_or(false),
        Err(_) => false,
    }
}

#[cfg(target_arch = "wasm32")]
async fn js_list_files(path: &str) -> Result<Vec<String>, String> {
    let promise = opfs_list_files_raw(path);
    let result = js_await(promise).await.map_err(|e| format!("{:?}", e))?;
    let val: serde_json::Value = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Bad bridge response: {}", e))?;
    if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        let arr = val
            .get("value")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "Missing value array".to_string())?;
        Ok(arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect())
    } else {
        Err(val
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error")
            .to_string())
    }
}

#[cfg(target_arch = "wasm32")]
async fn js_delete_file(path: &str) -> Result<(), String> {
    let promise = opfs_delete_file_raw(path);
    let result = js_await(promise).await.map_err(|e| format!("{:?}", e))?;
    let val: serde_json::Value = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Bad bridge response: {}", e))?;
    if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        Ok(())
    } else {
        Err(val
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error")
            .to_string())
    }
}

/// Save binary bytes to OPFS at the given path.
/// Returns `Ok(())` on success.
#[cfg(target_arch = "wasm32")]
async fn js_save_binary(path: &str, contents: &[u8]) -> Result<(), String> {
    let js_bytes = js_sys::Uint8Array::new_with_length(contents.len() as u32);
    js_bytes.copy_from(contents);
    let promise = opfs_save_binary_raw(path, &js_bytes);
    let result = js_await(promise).await.map_err(|e| format!("{:?}", e))?;
    let val: serde_json::Value = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Bad bridge response: {}", e))?;
    if val.get("ok").and_then(|v| v.as_bool()) == Some(true) {
        Ok(())
    } else {
        Err(val
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error")
            .to_string())
    }
}

/// Load binary bytes from OPFS at the given path.
/// Returns `Ok(Vec<u8>)` on success.
#[cfg(target_arch = "wasm32")]
async fn js_load_binary(path: &str) -> Result<Vec<u8>, String> {
    let promise = opfs_load_binary_raw(path);
    let result = js_await(promise).await.map_err(|e| format!("{:?}", e))?;
    // Binary response: {ok: true, value: Uint8Array} — must extract bytes manually
    // Cannot use serde_wasm_bindgen for Vec<u8> from Uint8Array.
    let obj = js_sys::Object::from(result);
    // MED-13: Reflect::get returns Result; propagate via ? instead of unwrap so
    // a JS exception (e.g., getter throws on a malformed object) becomes an
    // Err to the caller instead of a panic that would unwind into the WASM
    // bridge and poison subsequent calls.
    let ok = js_sys::Reflect::get(&obj, &"ok".into())
        .map_err(|e| format!("Reflect::get('ok') failed: {:?}", e))?;
    if ok.as_bool() == Some(true) {
        let value = js_sys::Reflect::get(&obj, &"value".into())
            .map_err(|e| format!("Reflect::get('value') failed: {:?}", e))?;
        let bytes: Vec<u8> = js_sys::Uint8Array::new(&value).to_vec();
        Ok(bytes)
    } else {
        let err = js_sys::Reflect::get(&obj, &"error".into())
            .map_err(|e| format!("Reflect::get('error') failed: {:?}", e))?;
        Err(err
            .as_string()
            .unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[cfg(target_arch = "wasm32")]
async fn update_project_metadata(scene_name: &str) -> Result<(), String> {
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
            serde_wasm_bindgen::to_value(&response)
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => {
            let response = serde_json::json!({ "ok": false, "error": e });
            serde_wasm_bindgen::to_value(&response)
                .map_err(|e| JsValue::from_str(&e.to_string()))
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
    use crate::asset_files::{is_supported_mime, RESOURCE_DIR};

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
            serde_wasm_bindgen::to_value(&response)
                .map_err(|e| JsValue::from_str(&e.to_string()))
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
            serde_wasm_bindgen::to_value(&response)
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => {
            let response = serde_json::json!({ "ok": false, "error": e });
            serde_wasm_bindgen::to_value(&response)
                .map_err(|e| JsValue::from_str(&e.to_string()))
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
    SCENE_DOC.with(|s| match s.borrow().as_ref() {
        Some(doc) => match serde_json::to_string(doc) {
            Ok(json) => JsValue::from_str(&json),
            Err(_) => JsValue::NULL,
        },
        None => JsValue::NULL,
    })
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

/// Export a SceneDocument JSON string to a Bevy-compatible runtime scene
/// representation (Hito 0 §9.5). Returns a JSON string with shape:
///
/// ```json
/// {
///   "json": "<DynamicSceneExport JSON>",
///   "warnings": [
///     { "entity_stable_id": "ent_01", "component_type_id": "editor.Sprite2D", "message": "..." }
///   ]
/// }
/// ```
///
/// Use `JSON.parse(returnedString)` on the JS side to get the object. We use
/// `JsValue::from_str` (not `serde_wasm_bindgen::to_value`) because the export
/// contains nested `serde_json::Value` fields that `to_value` mangles to `{}`.
///
/// Returns a JsValue error (thrown as exception on the JS side) if the input
/// is not valid SceneDocument JSON.
#[wasm_bindgen]
pub fn export_dynamic_scene_wasm(doc_json: &str) -> Result<JsValue, JsValue> {
    let doc: SceneDocument = serde_json::from_str(doc_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    let export = dynamic_scene::export_dynamic_scene(&doc)
        .map_err(|e| JsValue::from_str(&format!("Export error: {}", e)))?;

    // Marshal the response as `{ json: String, warnings: ExportWarning[] }`.
    // We re-use the JSON string approach for the inner DynamicSceneExport
    // because it contains nested serde_json::Value inside BTreeMap values.
    let export_json = serde_json::to_string(&export)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;

    let response = serde_json::json!({
        "json": export_json,
        "warnings": export.warnings,
    });

    let response_str = serde_json::to_string(&response)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;

    Ok(JsValue::from_str(&response_str))
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
        Some(schema) => Ok(serde_json::to_string(&schema.source_location)
            .unwrap_or_else(|_| "null".to_string())),
        None => Ok("null".to_string()),
    }
}

/// Find all entity stable IDs in the current scene that have a component of the given type.
#[wasm_bindgen]
pub fn find_entities_by_type(type_id: &str) -> Result<String, JsValue> {
    let matching: Vec<String> = SCENE_DOC
        .with(|s| {
            let doc_ref = s.borrow();
            match doc_ref.as_ref() {
                Some(doc) => doc
                    .entities
                    .iter()
                    .filter(|e| e.components.iter().any(|c| c.type_id == type_id))
                    .map(|e| e.id.as_str().to_string())
                    .collect(),
                None => Vec::new(),
            }
        });
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
                scene_id: format!(
                    "scratch-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_nanos())
                        .unwrap_or(0)
                ),
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
        SCENE_DOC.with(|s| *s.borrow_mut() = Some(new_doc));
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
        SCENE_DOC.with(|s| *s.borrow_mut() = Some(doc));
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
    let entries: Vec<crate::scene_asset_catalog::SceneAssetCatalogEntry> =
        SCENE_ASSET_CATALOG.with(|cell| {
            match &*cell.borrow() {
                Some(cat) => cat.list_all().into_iter().cloned().collect(),
                None => Vec::new(),
            }
        });

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
                            scene_id: format!(
                                "loaded-{}",
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_nanos())
                                    .unwrap_or(0)
                            ),
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
                    SCENE_DOC.with(|s| *s.borrow_mut() = Some(doc));
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
            SCENE_ASSET_CATALOG_WARNINGS.with(|cell| {
                cell.borrow_mut().push(warning);
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
            SCENE_ASSET_CATALOG_WARNINGS.with(|cell| {
                cell.borrow_mut().push(warning);
            });
        }
    }
    // Store the rebuilt catalog in the thread-local holder
    SCENE_ASSET_CATALOG.with(|cell| {
        *cell.borrow_mut() = Some(catalog);
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
    let doc_json = SCENE_DOC.with(|s| {
        let doc_ref = s.borrow();
        let doc = doc_ref
            .as_ref()
            .ok_or_else(|| JsValue::from_str("No scene loaded — call load_scene_json first"))?;
        serde_json::to_string(doc).map_err(|e| JsValue::from_str(&e.to_string()))
    })?;

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
        let asset_id = match with_asset_catalog(|cat| cat.resolve_path(instance.asset_ref.as_str()).map(|s| s.to_string())) {
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

    SCENE_DOC.with(|s| *s.borrow_mut() = Some(doc));
    mark_dirty();
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

    let result_json = with_asset_doc_mut(|doc_opt| {
        let doc = doc_opt
            .as_mut()
            .ok_or_else(|| JsValue::from_str("No asset open — call open_scene_asset first"))?;

        let inverse = asset_command::apply(doc, &cmd)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Record in asset operation log
        with_asset_log_mut(|log| {
            log.record(&cmd, inverse.clone());
        });

        serde_json::to_string(&inverse)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize inverse: {}", e)))
    })?;

    Ok(result_json)
}

/// Undo the last asset command. Returns the inverse command JSON.
#[wasm_bindgen]
pub fn undo_asset() -> Result<String, JsValue> {
    let inverse_json = with_asset_doc_mut(|doc_opt| {
        with_asset_log_mut(|log| {
            let doc = doc_opt
                .as_mut()
                .ok_or_else(|| JsValue::from_str("No asset open"))?;
            log.undo(doc)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            serde_json::to_string(&())
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
        })
    })?;
    Ok(inverse_json)
}

/// Redo the next asset command.
#[wasm_bindgen]
pub fn redo_asset() -> Result<String, JsValue> {
    let result_json = with_asset_doc_mut(|doc_opt| {
        with_asset_log_mut(|log| {
            let doc = doc_opt
                .as_mut()
                .ok_or_else(|| JsValue::from_str("No asset open"))?;
            log.redo(doc)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            serde_json::to_string(&())
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
        })
    })?;
    Ok(result_json)
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

    let result_json = with_logic_graph_mut(|doc_opt| {
        let doc = doc_opt
            .as_mut()
            .ok_or_else(|| JsValue::from_str("No logic graph open — call create_logic_graph_asset first"))?;

        let inverse = logic_command::apply(doc, &cmd)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Record in logic operation log
        with_logic_log_mut(|log| {
            log.record(&cmd, inverse.clone());
        });

        serde_json::to_string(&inverse)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize inverse: {}", e)))
    })?;

    Ok(result_json)
}

/// List all built-in immutable recipes with metadata.
/// Returns JSON array of { asset_id, name, description, node_count }.
#[wasm_bindgen]
pub fn list_builtin_recipes_wasm() -> Result<JsValue, JsValue> {
    let recipes = logic_recipes::list_builtin_recipes();
    serde_wasm_bindgen::to_value(&recipes)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize recipes: {}", e)))
}

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
    with_logic_graph(|doc_opt| {
        let doc = doc_opt
            .as_ref()
            .ok_or_else(|| JsValue::from_str("No logic graph open"))?;
        serde_json::to_string(doc)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
    })
}

/// Create a new empty LogicGraphAsset and set it as the active graph.
#[wasm_bindgen]
pub fn create_logic_graph_asset(asset_id: &str, logical_path: &str) -> Result<String, JsValue> {
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

    // Clear the operation log for the new graph
    with_logic_log_mut(|log| {
        log.clear();
    });

    Ok(json)
}

/// List all logic graph assets (placeholder — returns empty for now).
/// Tracked for OPFS persistence wiring (post-Hito 4 Order 5; Hito 4 Order 5
/// shipped data-only hot-reload but did NOT introduce OPFS persistence for
/// logic graphs themselves — that requires a separate cycle).
#[wasm_bindgen]
pub fn list_logic_graph_assets() -> Result<String, JsValue> {
    // Placeholder: returns empty until OPFS persistence is wired (see docstring above).
    serde_json::to_string(&Vec::<LogicGraphAsset>::new())
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

    let normalized_path = scene_asset_catalog::normalize_logical_path(name);
    let asset_id = scene_asset_catalog::mint_asset_id();

    // Check for duplicate path
    let duplicate = with_asset_catalog(|cat| {
        cat.resolve_path(&normalized_path).is_some()
    });
    if duplicate {
        return Err(JsValue::from_str(&format!(
            "Duplicate logical path: {}",
            normalized_path
        )));
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let entry = scene_asset_catalog::SceneAssetCatalogEntry {
        asset_id: asset_id.clone(),
        logical_path: normalized_path.clone(),
        role,
        current_version: 1,
        tags: vec![],
        created_at: now,
        updated_at: now,
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

    let doc_json = serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Write body file first
    js_save_file(&persistence::asset_path(&normalized_path), &doc_json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Register in catalog
    with_asset_catalog_mut(|cat| {
        cat.register(entry.clone())
    }).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Update project.json
    update_project_metadata_for_asset(&entry, "create").await?;

    serde_json::to_string(&entry)
        .map_err(|e| JsValue::from_str(&e.to_string()))
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
    let entry: scene_asset_catalog::SceneAssetCatalogEntry = serde_json::from_str(&entry_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse created entry: {}", e)))?;

    // Override the imported doc's ids to match the created asset
    doc.asset_id = entry.asset_id.clone();
    doc.logical_path = entry.logical_path.clone();
    doc.role = entry.role.clone();
    doc.version = entry.current_version;

    // Write the imported body to OPFS (overwriting the empty one created above)
    let doc_json = serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_save_file(&persistence::asset_path(&entry.logical_path), &doc_json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Return the entry (catalog already updated by create_scene_asset)
    serde_json::to_string(&entry)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Rename a Scene Asset (moves the file and updates catalog).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn rename_scene_asset(asset_id: &str, new_path: &str) -> Result<String, JsValue> {
    let new_path_normalized = scene_asset_catalog::normalize_logical_path(new_path);

    // Get old entry
    let old_entry = with_asset_catalog(|cat| {
        cat.get(asset_id).cloned()
    }).ok_or_else(|| JsValue::from_str(&format!("Asset not found: {}", asset_id)))?;

    let old_path = &old_entry.logical_path;

    // Check for duplicate new path
    if old_path != &new_path_normalized {
        let duplicate = with_asset_catalog(|cat| {
            cat.resolve_path(&new_path_normalized).is_some()
        });
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
        let _ = cat.unregister(asset_id).map_err(|e| JsValue::from_str(&e.to_string()));
        let mut new_entry = old_entry.clone();
        new_entry.logical_path = new_path_normalized.clone();
        new_entry.current_version += 1;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        new_entry.updated_at = now;
        cat.register(new_entry.clone()).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok::<_, JsValue>(new_entry)
    })?;

    // Update project.json
    update_project_metadata_for_asset(&new_entry, "rename").await?;

    // Invalidate ASSET_BODY_CACHE by old_path (D4)
    with_asset_body_cache_mut(|cache| {
        cache.remove(old_path);
    });

    serde_json::to_string(&new_entry)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Duplicate a Scene Asset.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn duplicate_scene_asset(asset_id: &str) -> Result<String, JsValue> {
    // Get source entry
    let source_entry = with_asset_catalog(|cat| {
        cat.get(asset_id).cloned()
    }).ok_or_else(|| JsValue::from_str(&format!("Asset not found: {}", asset_id)))?;

    let source_path = &source_entry.logical_path;

    // Read source body
    let body = js_load_file(&persistence::asset_path(source_path))
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Mint new id
    let new_id = scene_asset_catalog::mint_asset_id();
    let new_path = derive_duplicate_path(&source_entry.logical_path);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let new_entry = scene_asset_catalog::SceneAssetCatalogEntry {
        asset_id: new_id.clone(),
        logical_path: new_path.clone(),
        role: source_entry.role,
        current_version: 1,
        tags: vec![],
        created_at: now,
        updated_at: now,
    };

    // Write new body file
    js_save_file(&persistence::asset_path(&new_path), &body)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Register in catalog
    with_asset_catalog_mut(|cat| {
        cat.register(new_entry.clone())
    }).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Update project.json
    update_project_metadata_for_asset(&new_entry, "duplicate").await?;

    serde_json::to_string(&new_entry)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Delete a Scene Asset.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn delete_scene_asset(asset_id: &str) -> Result<(), JsValue> {
    // Get entry
    let entry = with_asset_catalog(|cat| {
        cat.get(asset_id).cloned()
    }).ok_or_else(|| JsValue::from_str(&format!("Asset not found: {}", asset_id)))?;

    let path = entry.logical_path.clone();

    // Delete body file
    js_delete_file(&persistence::asset_path(&path))
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Unregister from catalog
    with_asset_catalog_mut(|cat| {
        cat.unregister(asset_id)
    }).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Update project.json
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
    let entries: Vec<scene_asset_catalog::SceneAssetCatalogEntry> = with_asset_catalog(|cat| {
        match role_filter {
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
        }
    });
    serde_json::to_string(&entries)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Open a Scene Asset by asset_id into SCENE_ASSET_DOC thread-local.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn open_scene_asset(asset_id: &str) -> Result<String, JsValue> {
    // Get entry
    let entry = with_asset_catalog(|cat| {
        cat.get(asset_id).cloned()
    }).ok_or_else(|| JsValue::from_str(&format!("Asset not found: {}", asset_id)))?;

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

    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&e.to_string()))
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
    with_asset_doc(|doc_opt| {
        match doc_opt {
            Some(doc) => serde_json::to_string(doc)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            None => Err(JsValue::from_str("No asset open")),
        }
    })
}

/// Get the Scene Asset Catalog as JSON.
#[wasm_bindgen]
pub fn get_scene_asset_catalog_json() -> Result<String, JsValue> {
    let entries = with_asset_catalog(|cat| {
        cat.list_all().into_iter().cloned().collect::<Vec<_>>()
    });
    serde_json::to_string(&entries)
        .map_err(|e| JsValue::from_str(&e.to_string()))
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
        let doc_json = serde_json::to_string(doc)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok::<_, JsValue>((asset_id, path, doc_json))
    })?;

    // Step 1: Write body file first
    js_save_file(&persistence::asset_path(&path), &doc_json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    // Step 2: Bump version in catalog
    let new_version = with_asset_catalog_mut(|cat| {
        let current = cat.get(&asset_id)
            .ok_or_else(|| JsValue::from_str(&format!("Asset not found: {}", asset_id)))?
            .current_version;
        let new_ver = current + 1;
        cat.update_version(&asset_id, new_ver)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok::<_, JsValue>(new_ver)
    })?;

    // Step 3: Write project.json
    let entries = with_asset_catalog(|cat| {
        cat.list_all().into_iter().cloned().collect::<Vec<_>>()
    });
    let mut project = load_project_metadata().await?;
    project.scene_assets = entries;
    let project_json = serde_json::to_string(&project)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
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
    js_delete_file(&path).await.map_err(|e| JsValue::from_str(&e))?;
    Ok(())
}

/// List all tilesets in the `tilesets/` directory.
/// Returns a JSON array of TilesetMetadata objects.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn list_tilesets() -> Result<String, JsValue> {
    let dir = persistence::TILESETS_DIR;
    let files = js_list_files(dir).await
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

/// Paint a tile onto a TileLayer.
///
/// HIGH-10: routes through the AssetCommand surface so undo/redo captures
/// the previous TileRef (if any) via the inverse `EraseTile`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn paint_tile(
    asset_ref: &str,
    layer_id: &str,
    x: i32,
    y: i32,
    tileset_id: &str,
    local_index: u32,
) -> Result<JsValue, JsValue> {
    // Load the SceneAssetDocument from cache
    let mut doc_opt: Option<SceneAssetDocument> = None;
    with_asset_body_cache(|cache| {
        doc_opt = cache.get(asset_ref).cloned();
    });

    let mut doc = doc_opt
        .ok_or_else(|| JsValue::from_str("Scene asset not found"))?;

    // Apply PaintTile through the command surface (captures inverse)
    let cmd = AssetCommand::PaintTile {
        layer_id: TileLayerId::from(layer_id),
        x,
        y,
        old_tile: None,
        tileset_id: tileset_id.to_string(),
        local_index,
    };
    let inverse = asset_command::apply(&mut doc, &cmd)
        .map_err(|e| JsValue::from_str(&format!("paint_tile failed: {}", e)))?;
    // Record in the asset operation log so undo restores the previous tile.
    with_asset_log_mut(|log| {
        log.record(&cmd, inverse.clone());
    });
    let _ = inverse;

    // Update the cache with modified document
    with_asset_body_cache_mut(|cache| {
        cache.insert(asset_ref.to_string(), doc.clone());
    });

    // Sync to SCENE_ASSET_DOC so save_scene_asset persists the change
    let doc_json = serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
    set_asset_document_wasm(&doc_json)?;

    Ok(JsValue::NULL)
}

/// Erase a tile from a TileLayer.
///
/// HIGH-10: routes through the AssetCommand surface so undo/redo captures
/// the erased TileRef via the inverse `PaintTile`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn erase_tile(
    asset_ref: &str,
    layer_id: &str,
    x: i32,
    y: i32,
) -> Result<JsValue, JsValue> {
    // Load the SceneAssetDocument from cache
    let mut doc_opt: Option<SceneAssetDocument> = None;
    with_asset_body_cache(|cache| {
        doc_opt = cache.get(asset_ref).cloned();
    });

    let mut doc = doc_opt
        .ok_or_else(|| JsValue::from_str("Scene asset not found"))?;

    // Apply EraseTile through the command surface (captures inverse)
    let cmd = AssetCommand::EraseTile {
        layer_id: TileLayerId::from(layer_id),
        x,
        y,
        erased_tile: None,
    };
    let inverse = asset_command::apply(&mut doc, &cmd)
        .map_err(|e| JsValue::from_str(&format!("erase_tile failed: {}", e)))?;
    // Record in the asset operation log so undo restores the erased tile.
    with_asset_log_mut(|log| {
        log.record(&cmd, inverse.clone());
    });
    let _ = inverse;

    // Update the cache with modified document
    with_asset_body_cache_mut(|cache| {
        cache.insert(asset_ref.to_string(), doc.clone());
    });

    // Sync to SCENE_ASSET_DOC so save_scene_asset persists the change
    let doc_json = serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
    set_asset_document_wasm(&doc_json)?;

    Ok(JsValue::NULL)
}

// ─────────────────────────────────────────────────────────────────────────────
// AutoLayer WASM surface (auto-layer-generation PR2)
// ─────────────────────────────────────────────────────────────────────────────

/// Check if an AutoLayer's cached grid is stale — i.e., whether the source
/// TileLayer has been modified since the cache was last built.
///
/// Returns `true` if stale, `false` if the cache is up-to-date.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn is_auto_layer_stale_wasm(asset_ref: &str, layer_id: &str) -> Result<bool, JsValue> {
    use crate::scene_asset::LevelLayer;

    // Load from asset_body_cache
    let doc = with_asset_body_cache(|cache| {
        cache.get(asset_ref).cloned()
    }).ok_or_else(|| JsValue::from_str("Scene asset not found"))?;

    // Find the AutoLayer
    let auto_layer = doc.layers
        .iter()
        .find(|l| matches!(l, LevelLayer::Auto(al) if al.id.as_str() == layer_id))
        .ok_or_else(|| JsValue::from_str("AutoLayer not found"))?;

    let LevelLayer::Auto(al) = auto_layer else {
        return Err(JsValue::from_str("Layer is not an AutoLayer"));
    };

    // Find the source TileLayer
    let source_tl = doc.layers
        .iter()
        .find(|l| matches!(l, LevelLayer::Tile(tl) if tl.id.as_str() == al.source_layer_id.as_str()))
        .ok_or_else(|| JsValue::from_str("Source TileLayer not found"))?;

    let LevelLayer::Tile(tl) = source_tl else {
        return Err(JsValue::from_str("Source layer is not a TileLayer"));
    };

    Ok(al.source_generation != tl.generation)
}

/// Regenerate an AutoLayer's cached tile grid from its source TileLayer.
///
/// Routes through `dispatch_asset_command` so the operation is recorded in the
/// asset operation log for undo/redo.
///
/// Returns the updated SceneAssetDocument JSON on success.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn regenerate_auto_layer_wasm(
    asset_ref: &str,
    layer_id: &str,
) -> Result<String, JsValue> {
    use crate::asset_command::AssetCommand;
    use crate::scene_asset::LevelLayer;

    // Load the doc from cache to find the AutoLayer
    let doc_for_layer = with_asset_body_cache(|cache| {
        cache.get(asset_ref).cloned()
    }).ok_or_else(|| JsValue::from_str("Scene asset not found"))?;

    // Find AutoLayer and capture old cached/source_generation for the command
    let (old_cached, old_source_generation) = match doc_for_layer.layers.iter().find(|l| matches!(l, LevelLayer::Auto(al) if al.id.as_str() == layer_id)) {
        Some(LevelLayer::Auto(al)) => (al.cached.clone(), al.source_generation),
        _ => return Err(JsValue::from_str("AutoLayer not found")),
    };

    // Build the RegenerateAutoLayer command
    let cmd = AssetCommand::RegenerateAutoLayer {
        layer_id: crate::scene_asset::LayerId::new(layer_id.to_string()),
        old_cached,
        old_source_generation,
    };
    let cmd_json = serde_json::to_string(&cmd)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize command: {}", e)))?;

    // Route through dispatch_asset_command for operation log recording
    dispatch_asset_command(&cmd_json)?;

    // Fetch the updated doc and sync to asset_body_cache and SCENE_ASSET_DOC
    let updated_doc = with_asset_doc(|doc_opt| {
        doc_opt.clone()
    }).ok_or_else(|| JsValue::from_str("No asset open — asset doc was not set"))?;

    // Update asset_body_cache
    with_asset_body_cache_mut(|cache| {
        cache.insert(asset_ref.to_string(), updated_doc.clone());
    });

    // Sync to SCENE_ASSET_DOC via set_asset_document_wasm
    let updated_json = serde_json::to_string(&updated_doc)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
    set_asset_document_wasm(&updated_json)?;

    Ok(updated_json)
}

/// Add an AutoRule to an AutoLayer.
///
/// MED-8: routes through the AssetCommand surface so undo/redo captures
/// the inverse (RemoveAutoRule at the appended index).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn add_auto_rule_wasm(
    asset_ref: &str,
    layer_id: &str,
    rule_json: &str,
) -> Result<String, JsValue> {
    use crate::auto_layer::AutoRule;

    let rule: AutoRule = serde_json::from_str(rule_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid rule JSON: {}", e)))?;

    let mut doc = with_asset_body_cache(|cache| cache.get(asset_ref).cloned())
        .ok_or_else(|| JsValue::from_str("Scene asset not found"))?;

    let cmd = AssetCommand::AddAutoRule {
        layer_id: AutoLayerId::from(layer_id),
        rule: rule.clone(),
    };
    let inverse = asset_command::apply(&mut doc, &cmd)
        .map_err(|e| JsValue::from_str(&format!("add_auto_rule failed: {}", e)))?;
    // Record in the asset operation log so undo removes the added rule.
    with_asset_log_mut(|log| {
        log.record(&cmd, inverse);
    });

    with_asset_body_cache_mut(|cache| {
        cache.insert(asset_ref.to_string(), doc.clone());
    });
    let doc_json = serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
    set_asset_document_wasm(&doc_json)?;
    Ok(doc_json)
}

/// Update an AutoRule in an AutoLayer at the given index.
///
/// MED-8: routes through the AssetCommand surface.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn update_auto_rule_wasm(
    asset_ref: &str,
    layer_id: &str,
    rule_index: usize,
    rule_json: &str,
) -> Result<String, JsValue> {
    use crate::auto_layer::AutoRule;

    let new_rule: AutoRule = serde_json::from_str(rule_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid rule JSON: {}", e)))?;

    let mut doc = with_asset_body_cache(|cache| cache.get(asset_ref).cloned())
        .ok_or_else(|| JsValue::from_str("Scene asset not found"))?;

    let cmd = AssetCommand::UpdateAutoRule {
        layer_id: AutoLayerId::from(layer_id),
        index: rule_index,
        old_rule: None,
        new_rule: new_rule.clone(),
    };
    let inverse = asset_command::apply(&mut doc, &cmd)
        .map_err(|e| JsValue::from_str(&format!("update_auto_rule failed: {}", e)))?;
    // Record in the asset operation log so undo restores the old rule.
    with_asset_log_mut(|log| {
        log.record(&cmd, inverse);
    });

    with_asset_body_cache_mut(|cache| {
        cache.insert(asset_ref.to_string(), doc.clone());
    });
    let doc_json = serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
    set_asset_document_wasm(&doc_json)?;
    Ok(doc_json)
}

/// Remove an AutoRule from an AutoLayer at the given index.
///
/// MED-8: routes through the AssetCommand surface.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn remove_auto_rule_wasm(
    asset_ref: &str,
    layer_id: &str,
    rule_index: usize,
) -> Result<String, JsValue> {
    let mut doc = with_asset_body_cache(|cache| cache.get(asset_ref).cloned())
        .ok_or_else(|| JsValue::from_str("Scene asset not found"))?;

    let cmd = AssetCommand::RemoveAutoRule {
        layer_id: AutoLayerId::from(layer_id),
        index: rule_index,
        removed_rule: None,
    };
    let inverse = asset_command::apply(&mut doc, &cmd)
        .map_err(|e| JsValue::from_str(&format!("remove_auto_rule failed: {}", e)))?;
    // Record in the asset operation log so undo restores the removed rule.
    with_asset_log_mut(|log| {
        log.record(&cmd, inverse);
    });

    with_asset_body_cache_mut(|cache| {
        cache.insert(asset_ref.to_string(), doc.clone());
    });
    let doc_json = serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
    set_asset_document_wasm(&doc_json)?;
    Ok(doc_json)
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper functions
// ─────────────────────────────────────────────────────────────────────────────

/// Derive a unique path for duplication (appends `_2`, `_3`, etc. if collision).
fn derive_duplicate_path(original: &str) -> String {
    let base = format!("{}_2", original);
    // Check if collision
    let exists = with_asset_catalog(|cat| {
        cat.resolve_path(&base).is_some()
    });
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
    entry: &scene_asset_catalog::SceneAssetCatalogEntry,
    _operation: &str,
) -> Result<(), JsValue> {
    let mut project = load_project_metadata().await?;

    // Find and replace or add entry
    if let Some(existing) = project.scene_assets.iter_mut().find(|e| e.asset_id == entry.asset_id) {
        *existing = entry.clone();
    } else {
        project.scene_assets.push(entry.clone());
    }

    let json = serde_json::to_string(&project)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_save_file(persistence::PROJECT_FILE, &json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    Ok(())
}

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

    /// Test helper: set LOGIC_GRAPH_DOC for testing.
    #[cfg(test)]
    pub(crate) fn set_logic_graph_for_test(asset: Option<LogicGraphAsset>) {
        LOGIC_GRAPH_DOC.with(|cell| {
            *cell.borrow_mut() = asset;
        });
    }

    /// Test helper: clear LOGIC_GRAPH_DOC after each test.
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

        assert!(!logic_issues.is_empty(), "expected logic issues for cycle, got none");
        let cycle_issue = logic_issues
            .iter()
            .find(|i| i.code == "cycle");
        assert!(cycle_issue.is_some(), "expected code='cycle', got: {:?}", logic_issues);
        assert!(cycle_issue.unwrap().affected_asset_id.is_some());
        assert_eq!(cycle_issue.unwrap().affected_asset_id.as_deref(), Some("cycle_test"));

        clear_logic_graph();
    }

    // Test 2: LOGIC_GRAPH_DOC is None → no logic issues
    #[test]
    fn wasm_validation_no_logic_issues_when_no_graph() {
        clear_logic_graph();

        let json = get_validation_issues_wasm().unwrap();
        let issues: Vec<ValidationIssue> = serde_json::from_str(&json).unwrap();

        let logic_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.category == ValidationCategory::Logic)
            .collect();

        assert!(logic_issues.is_empty(), "expected no logic issues when no graph, got: {:?}", logic_issues);
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

        assert!(logic_issues.is_empty(), "expected no logic issues for clean graph, got: {:?}", logic_issues);

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
                    name: "Enemy".to_string(),
                    parent: None,
                    components: vec![
                        ComponentInstance {
                            type_id: "game.EnemyAI".to_string(),
                            values: serde_json::json!({}),
                        },
                    ],
                },
                Entity {
                    id: StableId::new("ent_ally"),
                    name: "Ally".to_string(),
                    parent: None,
                    components: vec![
                        ComponentInstance {
                            type_id: "game.PlayerHealth".to_string(),
                            values: serde_json::json!({}),
                        },
                    ],
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

