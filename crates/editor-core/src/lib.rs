use bevy::prelude::Entity as BevyEntity;
use bevy::prelude::*;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

mod bevy_anchor;
pub mod bsn_ir;
pub mod bsn_codegen;
mod code_export;
mod command;
mod document;
mod dynamic_scene;
mod operation_log;
mod persistence;
mod processor;
pub mod scene_asset;
pub mod scene_asset_catalog;
pub mod scene_instance;
pub mod scene_instance_overrides;
mod scenes;
mod schema;
mod template;

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
pub use persistence::{ENTITIES_DIR, PROJECT_FILE, ProjectMetadata, SCENES_DIR, SCHEMAS_DIR};
pub use scene_asset::{
    AssetReference, ExposedProperty, LocalId, RelationshipKind, RoleWarning, SceneAssetDocument,
    SceneAssetEntity, SceneAssetMetadata, SceneAssetRelationship, SceneAssetRole, validate_role,
};
pub use scene_asset_catalog::{
    CatalogError, CatalogWarning, SceneAssetCatalog, SceneAssetCatalogEntry, mint_asset_id,
};
pub use scene_instance::{
    OverridePatch, OverrideStatus, SceneInstance, patch_status_after_field_rename,
};
pub use scenes::{SceneInfo, SceneRegistry, SwitchResult};
pub use template::{EntityTemplate, TemplateEntity, TemplateError};

/// Marker component for entities spawned from SceneDocument.
/// These are despawned and respawned when the document is mutated
/// (preview world rebuild strategy — matches Hito 0 decision 23).
#[derive(Component)]
pub struct SceneEntity;

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

/// Cross-system dirty flag set by `dispatch_command` and read by
/// `rebuild_preview_world`. Visible across the WASM→Bevy boundary
/// because both run on the same thread (single-threaded WASM).
thread_local! {
    static DIRTY_FLAG: RefCell<bool> = const { RefCell::new(false) };
    static SCENE_REGISTRY: RefCell<Option<SceneRegistry>> = const { RefCell::new(None) };
}

/// Get an immutable borrowed reference to the SceneRegistry, initializing if needed.
fn with_registry<F, R>(f: F) -> R
where
    F: FnOnce(&SceneRegistry) -> R,
{
    SCENE_REGISTRY.with(|cell| {
        let mut_ref = &mut *cell.borrow_mut();
        if mut_ref.is_none() {
            *mut_ref = Some(SceneRegistry::default());
        }
        f(mut_ref.as_ref().unwrap())
    })
}

/// Get a mutable borrowed reference to the SceneRegistry, initializing if needed.
fn with_registry_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut SceneRegistry) -> R,
{
    SCENE_REGISTRY.with(|cell| {
        let mut_ref = &mut *cell.borrow_mut();
        if mut_ref.is_none() {
            *mut_ref = Some(SceneRegistry::default());
        }
        f(mut_ref.as_mut().unwrap())
    })
}

fn mark_dirty() {
    DIRTY_FLAG.with(|d| *d.borrow_mut() = true);
    with_registry_mut(|r| r.mark_current_dirty());
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

#[wasm_bindgen]
pub fn create_buses() {
    console_error_panic_hook::set_once();
    COMMAND_BUS.with(|b| *b.borrow_mut() = Some(LinearBus::new()));
    EVENT_BUS.with(|b| *b.borrow_mut() = Some(LinearBus::new()));
    web_sys::console::log_1(&"[editor-core] Buses created".into());
}

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
        .add_systems(Update, process_commands)
        .add_systems(Update, rebuild_preview_world.after(process_commands))
        .add_systems(Update, sync_log_state.after(rebuild_preview_world))
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
                    web_sys::console::error_1(
                        &format!("[editor-core] Failed to parse default scene: {}", e).into(),
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
    mark_dirty();
}

/// Respawns scene entities when the SceneDocumentState dirty flag is set.
/// Triggered by `dispatch_command` setting DIRTY_FLAG via mark_dirty().
///
/// Design decision 3: Syncs SceneDocumentState.document from SCENE_DOC on every
/// dirty tick. This fixes both multi-scene switching AND pre-existing preview
/// staleness where entity edits never reached the canvas.
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
    // Spawn new entities from the document
    for entity in state.document.entities.iter() {
        spawn_entity(&mut commands, entity);
    }
    state.dirty = false;
    DIRTY_FLAG.with(|d| *d.borrow_mut() = false);
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

/// Helper: await a JS Promise and return its resolved JsValue.
async fn js_await(promise: js_sys::Promise) -> Result<JsValue, JsValue> {
    let fut = JsFuture::from(promise);
    fut.await
        .map_err(|e| JsValue::from_str(&format!("JS promise rejected: {:?}", e)))
}

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
}

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

async fn js_exists(path: &str) -> bool {
    let promise = opfs_exists_raw(path);
    match js_await(promise).await {
        Ok(v) => v.as_bool().unwrap_or(false),
        Err(_) => false,
    }
}

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
#[wasm_bindgen]
pub async fn project_exists() -> bool {
    js_exists(PROJECT_FILE).await
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

/// Helper: update project.json's schemas list (add or remove a type_id).
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
// Entity Templates — wasm_bindgen surface
// ─────────────────────────────────────────────────────────────────────────────

/// Helper: update project.json's templates list (add or remove a template_id).
async fn update_project_templates(template_id: &str, add: bool) -> Result<(), String> {
    let mut project = if js_exists(PROJECT_FILE).await {
        match js_load_file(PROJECT_FILE).await {
            Ok(json_str) => serde_json::from_str::<ProjectMetadata>(&json_str).unwrap_or_default(),
            Err(_) => ProjectMetadata::default(),
        }
    } else {
        ProjectMetadata::default()
    };

    if add {
        if !project.templates.contains(&template_id.to_string()) {
            project.templates.push(template_id.to_string());
        }
    } else {
        project.templates.retain(|t| t != template_id);
    }

    let json = serde_json::to_string(&project).map_err(|e| e.to_string())?;
    js_save_file(PROJECT_FILE, &json).await
}

/// Save an EntityTemplate to OPFS at `entities/<template_id>.template.json`.
#[wasm_bindgen]
pub async fn save_template(template_id: &str, template_json: &str) -> Result<(), JsValue> {
    let template: EntityTemplate = serde_json::from_str(template_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    template::validate(&template).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let path = persistence::template_path(template_id);
    let json = serde_json::to_string(&template).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_save_file(&path, &json)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    update_project_templates(template_id, true)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    template::cache_template(template);
    Ok(())
}

/// Load an EntityTemplate from OPFS and cache in memory.
#[wasm_bindgen]
pub async fn load_template(template_id: &str) -> Result<(), JsValue> {
    let path = persistence::template_path(template_id);
    let json_str = js_load_file(&path)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    let template: EntityTemplate = serde_json::from_str(&json_str)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    template::validate(&template).map_err(|e| JsValue::from_str(&e.to_string()))?;
    template::cache_template(template);
    Ok(())
}

/// List all template IDs in OPFS.
#[wasm_bindgen]
pub async fn list_templates() -> Result<JsValue, JsValue> {
    let files = js_list_files(persistence::ENTITIES_DIR)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    let ids: Vec<String> = files
        .into_iter()
        .filter(|f| f.ends_with(".template.json"))
        .map(|f| f.trim_end_matches(".template.json").to_string())
        .collect();
    serde_wasm_bindgen::to_value(&ids).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Delete an EntityTemplate from OPFS and clear from cache.
#[wasm_bindgen]
pub async fn delete_template(template_id: &str) -> Result<(), JsValue> {
    let path = persistence::template_path(template_id);
    let promise = opfs_delete_file_raw(&path);
    js_await(promise)
        .await
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    template::remove_cached_template(template_id);
    update_project_templates(template_id, false)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    Ok(())
}

/// Check if a template is loaded in the in-memory cache.
#[wasm_bindgen]
pub fn is_template_loaded(template_id: &str) -> bool {
    template::get_cached_template(template_id).is_some()
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

/// Load complete project: project.json + schemas + templates + all scenes (atomic).
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

    // Load all templates into cache
    for template_id in &project.templates {
        load_template(template_id).await.map_err(|e| {
            JsValue::from_str(&format!(
                "Failed to load template {}: {:?}",
                template_id,
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

    Ok(())
}

/// Save the current SceneDocument to OPFS at `scenes/<name>.scene.json`.
/// Also clears the `is_dirty` flag on the current scene entry.
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

/// Load a SceneDocument from OPFS into the current SCENE_DOC thread_local.
/// Note: For multi-scene, prefer `scene_switch` which handles the full
/// value-swap through the registry.
#[wasm_bindgen]
pub async fn load_scene(name: &str) -> Result<(), JsValue> {
    let path = persistence::scene_path(name);
    let json_str = js_load_file(&path)
        .await
        .map_err(|e| JsValue::from_str(&e))?;

    let doc: SceneDocument = serde_json::from_str(&json_str)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    SCENE_DOC.with(|s| *s.borrow_mut() = Some(doc));
    mark_dirty();
    Ok(())
}
