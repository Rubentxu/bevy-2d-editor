use bevy::prelude::*;
use bevy::prelude::Entity as BevyEntity;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

mod command;
mod document;
mod operation_log;
mod processor;
mod schema;

pub use command::{Command, CommandEnvelope, CommandError, CommandMetadata, CommandResult};
pub use document::{SceneDocument, Entity, ComponentInstance, StableId};
pub use operation_log::{LogEntry, OperationLog, OperationLogError};

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
}

fn mark_dirty() {
    DIRTY_FLAG.with(|d| *d.borrow_mut() = true);
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
            let payload_len = u16::from_le_bytes(self.buffer[pos + 2..pos + 4].try_into().unwrap()) as usize;
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
        self.buffer[write_offset..write_offset + 2]
            .copy_from_slice(&event_type.to_le_bytes());
        self.buffer[write_offset + 2..write_offset + 4]
            .copy_from_slice(&(payload.len() as u16).to_le_bytes());
        self.buffer[write_offset + 4..write_offset + 4 + payload.len()]
            .copy_from_slice(payload);
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
    web_sys::console::log_1(&format!("[editor-core] Starting Bevy with canvas: {}", canvas_selector).into());

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
                    web_sys::console::error_1(&format!("[editor-core] Failed to parse default scene: {}", e).into());
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

    let mut name: Option<BevyName> = None;
    let mut transform: Option<Transform> = None;
    let mut sprite: Option<Sprite> = None;

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
                    .map(|(x, y)| Vec3::new(x.as_f64().unwrap_or(0.0) as f32, y.as_f64().unwrap_or(0.0) as f32, 0.0))
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
                    .map(|(x, y)| Vec3::new(x.as_f64().unwrap_or(1.0) as f32, y.as_f64().unwrap_or(1.0) as f32, 1.0))
                    .unwrap_or(Vec3::new(1.0, 1.0, 1.0));

                transform = Some(Transform::from_translation(translation).with_rotation(Quat::from_rotation_z(rotation)).with_scale(scale));
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
        cmd.insert(s);
    }
}

fn process_commands(mut sprites: Query<&mut Transform, With<Sprite>>) {
    let cmds = COMMAND_BUS.with(|b| {
        b.borrow_mut().as_mut().map(|bus| bus.drain()).unwrap_or_default()
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
