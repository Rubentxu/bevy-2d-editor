//! Bevy runtime systems and `start_engine` entry point.
//!
//! HIGH-1 god-module phase 4: extracted from `lib.rs` to keep the crate root
//! focused on public API + module wiring. Contains:
//!
//! - `start_engine` — the WASM entry point that builds and runs the Bevy `App`
//! - `setup` — `Startup` system that initializes default resources
//! - `process_play_mode_request` — handles Enter/Exit Play transitions
//! - `process_hot_reload_requests` — drains the hot-reload bus each frame
//! - `rebuild_preview_world` — respawns scene entities when DIRTY_FLAG is set
//! - `process_commands` — drains the command bus and applies the legacy
//!   sprite-move command (kept for backward compatibility with the JS host)
//! - `sync_log_state` — mirrors thread-local `OPERATION_LOG` into the
//!   `OperationLogState` resource for UI hooks
//! - `emit_events` — publishes Sprite position + FPS on the event bus
//! - `in_play_mode` / `in_edit_mode` — `RunIf` helpers gating systems
//!
//! Plus the `spawn_entity` and `spawn_preview_entity` helpers used by
//! `rebuild_preview_world`.

use bevy::prelude::*;
use wasm_bindgen::prelude::*;

use crate::actuator_bus;
use crate::bevy_anchor::anchor_str_to_bevy_anchor;
use crate::bevy_logic_binding::LogicBinding;
use crate::document::{Entity, SceneDocument, StableId};
use crate::dynamic_scene::is_known_anchor_str;
use crate::instance_projection::{PreviewEntity, project_instances};
use crate::logic_dispatch;
use crate::logic_evaluator;
use crate::state::{
    DIRTY_FLAG, HOT_RELOAD_BUS, HotReloadRequest, PLAY_MODE_REQUEST, PlayModeRequest, mark_dirty,
    with_asset_body_cache_mut, with_logic_graph_mut,
};
use crate::{
    BevyEntity, OPERATION_LOG, OperationLogState, PlayMode, SceneDocumentState, SceneEntity,
    SceneInstanceChild, TransformSnapshot, source_files,
};

// ─────────────────────────────────────────────────────────────────────────────
// EditorComponent — JSON representation of component data (ADR-0042)
// ─────────────────────────────────────────────────────────────────────────────

use crate::document::ComponentInstance;

/// A Bevy component that stores the full JSON representation of a component's
/// field values. This is inserted on every scene entity so that
/// `process_play_mode_request` can capture tunable baselines at PlayModeEnter
/// without needing to re-run project_instances.
#[derive(Component, Clone)]
pub struct EditorComponent(pub ComponentInstance);

/// Stores the last-computed tunable baselines as a JSON string.
// v0.90 PR3: TUNABLE_BASELINES thread_local removed. The canonical owner is
// `EditorSession.tunable_baselines`, written/read via
// `editor_model::ports::with_session_mut` from `capture_tunable_baselines_internal`
// below. The WASM export `get_tunable_baselines_wasm` lives in
// `editor-application::wasm` (reads from the session). No more dual-write.

/// Capture tunable baselines synchronously from `SCENE_DOC`.
///
/// This function reads the current scene document and uses `project_instances`
/// to derive baseline values WITHOUT needing a Bevy world / query. It can be
/// called from `enter_play_mode` in `wasm.rs` before setting the
/// `PlayModeRequest`, ensuring baselines are available immediately.
///
/// Returns a JSON string: `BTreeMap<String, serde_json::Value>` keyed by
/// `stable_id`.
pub fn capture_baselines_from_scene_doc() -> String {
    use crate::instance_projection::project_instances;
    use crate::state::with_asset_body_cache;
    use std::collections::BTreeMap;

    let doc = crate::SCENE_DOC.with(|s| s.borrow().clone());
    let doc = match doc {
        Some(d) => d,
        None => return String::new(),
    };

    let resolver = |asset_ref: &crate::scene_asset::AssetReference| -> Option<crate::scene_asset::SceneAssetDocument> {
        with_asset_body_cache(|cache| cache.get(asset_ref.as_str()).cloned())
    };

    let projected = project_instances(&doc, &resolver);
    let mut baselines: BTreeMap<String, serde_json::Value> = BTreeMap::new();

    for preview in projected {
        let mut merged = serde_json::Map::new();
        for comp in &preview.component_values {
            if let Some(obj) = comp.values.as_object() {
                let nested: serde_json::Map<String, serde_json::Value> =
                    obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                merged.insert(comp.type_id.clone(), serde_json::Value::Object(nested));
            }
        }
        baselines.insert(
            preview.stable_id.as_str().to_string(),
            serde_json::Value::Object(merged),
        );
    }

    serde_json::to_string(&baselines).unwrap_or_default()
}

// ─────────────────────────────────────────────────────────────────────────────
// Bus / event constants and default scene payload
// ─────────────────────────────────────────────────────────────────────────────

pub(crate) const CMD_MOVE_SPRITE: u16 = 1;
pub(crate) const EVT_SPRITE_POSITION: u16 = 1;
pub(crate) const EVT_FPS: u16 = 2;

const DEFAULT_SCENE_JSON: &str = r#"{
  "scene_id": "default",
  "version": "1",
  "name": "Default Scene",
  "entities": [],
  "instances": {}
}"#;

// ─────────────────────────────────────────────────────────────────────────────
// WASM binding for JS-side frame-end notification (kept here because emit_events
// is the only caller and lives in this module).
// ─────────────────────────────────────────────────────────────────────────────

#[wasm_bindgen]
unsafe extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = onFrameEnd)]
    fn on_frame_end();
}

// ─────────────────────────────────────────────────────────────────────────────
// RunIf helpers
// ─────────────────────────────────────────────────────────────────────────────

/// RunIf helper — returns true when PlayMode is Playing.
pub fn in_play_mode(mode: Res<PlayMode>) -> bool {
    *mode == PlayMode::Playing
}

/// RunIf helper — returns true when PlayMode is Edit.
fn in_edit_mode(mode: Res<PlayMode>) -> bool {
    *mode == PlayMode::Edit
}

// ─────────────────────────────────────────────────────────────────────────────
// start_engine — WASM entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Entry point invoked from JS after the WASM module finishes loading.
/// Builds the Bevy `App`, wires the plugin stack and all update systems,
/// then calls `.run()` which blocks until the canvas is closed.
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
        // BUG-2 (Bevy 0.19 B0001) fix: every system pair that could alias on
        // `Transform` / `Sprite` is connected by an explicit `.before()` /
        // `.after()` chain OR by a `Without<SceneEntity>` disjoint filter on
        // the editor/legacy systems. `apply_actuator_outputs` (play mode) is
        // chained after `process_play_mode_request` + `process_commands` so
        // Bevy 0.19's stricter conflict detector sees the systems as ordered
        // (never running in parallel) and accepts the Transform/Sprite
        // mutable overlap with the scene-entity ParamSet below.
        .add_systems(Update, process_play_mode_request)
        // process_hot_reload_requests drains the HOT_RELOAD_BUS each frame before rebuild
        .add_systems(
            Update,
            process_hot_reload_requests.before(rebuild_preview_world),
        )
        // Editor-only systems gated during play mode. `Without<SceneEntity>`
        // keeps these queries provably disjoint from play-mode systems that
        // write to scene-entity Transforms/Sprites (process_play_mode_request
        // ParamSet, apply_actuator_outputs).
        .add_systems(
            Update,
            process_commands
                .run_if(in_edit_mode)
                .before(actuator_bus::apply_actuator_outputs),
        )
        .add_systems(
            Update,
            rebuild_preview_world
                .run_if(in_edit_mode)
                .after(process_commands),
        )
        .add_systems(
            Update,
            sync_log_state
                .run_if(in_edit_mode)
                .after(rebuild_preview_world),
        )
        // Play-mode sensor systems — run before logic evaluation
        .add_systems(
            Update,
            logic_evaluator::update_keyboard_state
                .run_if(in_play_mode)
                .before(logic_dispatch::logic_evaluation_system),
        )
        // Logic dispatch runs only in play mode
        .add_systems(
            Update,
            logic_dispatch::logic_evaluation_system
                .run_if(in_play_mode)
                .after(sync_log_state),
        )
        // apply_actuator_outputs (play mode) is explicitly ordered AFTER
        // process_play_mode_request and process_commands so Bevy 0.19 treats
        // the &mut Transform overlap with those systems as sequential rather
        // than parallel. `before(emit_events)` makes the Update→Last ordering
        // explicit for the same reason.
        .add_systems(
            Update,
            actuator_bus::apply_actuator_outputs
                .run_if(in_play_mode)
                .after(logic_dispatch::logic_evaluation_system)
                .after(process_play_mode_request)
                .before(emit_events),
        )
        .add_systems(Last, emit_events)
        .run();

    web_sys::console::log_1(&"[editor-core] Bevy app.run() returned".into());
}

// ─────────────────────────────────────────────────────────────────────────────
// Setup
// ─────────────────────────────────────────────────────────────────────────────

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Try to load scene from thread-local SCENE_DOC, otherwise use default
    let doc = crate::SCENE_DOC.with(|s| s.borrow().clone());
    let scene = match doc {
        Some(doc) => doc,
        None => match serde_json::from_str(DEFAULT_SCENE_JSON) {
            Ok(doc) => doc,
            Err(e) => {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::error_1(
                    &format!("[editor-core] Failed to parse default scene: {}", e).into(),
                );
                #[cfg(not(target_arch = "wasm32"))]
                eprintln!("[editor-core] Failed to parse default scene: {}", e);
                return;
            }
        },
    };

    // Insert SceneDocumentState resource
    commands.insert_resource(SceneDocumentState::new(scene.clone()));
    // Insert OperationLogState resource (UI hooks read this)
    commands.insert_resource(OperationLogState::default());
    // Insert PlayMode resource (defaults to Edit)
    commands.insert_resource(PlayMode::default());
    // Insert TransformSnapshot
    commands.insert_resource(TransformSnapshot::default());

    // Hito 5 (bevy-engine-hardening): also populate SCENE_DOC thread-local
    // so that `get_scene_snapshot()` (which reads from SCENE_DOC) returns
    // the same data as SceneDocumentState. Without this, the JS bridge
    // returns NULL on the very first call (before any load_scene_json).
    crate::SCENE_DOC.with(|s| *s.borrow_mut() = Some(scene));

    mark_dirty();
}

// ─────────────────────────────────────────────────────────────────────────────
// process_play_mode_request
// ─────────────────────────────────────────────────────────────────────────────

/// Handles play mode enter/exit requests from WASM, snapshot/restore transforms.
/// Runs BEFORE process_commands so Enter transitions are committed before editor
/// commands are processed, and Exit restores before rebuild_preview_world.
fn process_play_mode_request(
    mut play_mode: ResMut<PlayMode>,
    mut snapshot: ResMut<TransformSnapshot>,
    // Hito 5 (bevy-engine-hardening): Bevy 0.19 enforces no-aliasing on
    // system parameters. Two queries filtered by the same `With<SceneEntity>`
    // are considered aliasing (the mutable one could invalidate the
    // immutable one) — even if the system only uses one at a time in
    // separate `match` arms. We use `ParamSet` to declare them as
    // mutually exclusive at runtime.
    mut scene_transforms: ParamSet<(
        Query<(bevy::prelude::Entity, &Transform), With<SceneEntity>>,
        Query<(bevy::prelude::Entity, &mut Transform), With<SceneEntity>>,
    )>,
    // ADR-0042: Query EditorComponent + SceneInstanceChild for tunable baseline capture.
    baseline_components: Query<(&EditorComponent, &SceneInstanceChild)>,
) {
    let request = PLAY_MODE_REQUEST.with(|r| (*r.borrow()).clone());

    match request {
        Some(PlayModeRequest::Enter) => {
            // Snapshot all placed entity transforms
            snapshot.transforms.clear();
            for (entity, transform) in scene_transforms.p0().iter() {
                snapshot.transforms.insert(entity, *transform);
            }
            // ADR-0042: Capture tunable baselines (component values at Enter time)
            capture_tunable_baselines_internal(&baseline_components);
            *play_mode = PlayMode::Playing;
            PLAY_MODE_REQUEST.with(|r| *r.borrow_mut() = None);
        }
        Some(PlayModeRequest::Exit) => {
            // v0.90 PR1: compute runtime deltas BEFORE restoring transforms,
            // so the comparison reads the post-play-mode values from
            // EditorComponent. The closure reads each instance's current
            // EditorComponent from the Bevy query.
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let _count = compute_runtime_deltas_internal(
                |instance_id| {
                    for (editor_comp, instance_child) in baseline_components.iter() {
                        if instance_child.instance_id.as_str() == instance_id {
                            return Some(editor_comp.0.clone());
                        }
                    }
                    None
                },
                now_ms,
            );

            // Restore transforms from snapshot
            for (entity, mut transform) in scene_transforms.p1().iter_mut() {
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

/// Internal helper — captures baselines from the given query (avoid generic in Bevy system).
///
/// v0.90 PR1: writes to `EditorSession.tunable_baselines` (canonical owner) via
/// the `EditorSessionPort` trait. The session is the single source of truth
/// for the apply-back pipeline. The `TUNABLE_BASELINES` thread_local is also
/// updated as a secondary read cache (kept for backward compat with the
/// `get_tunable_baselines_wasm` export; can be removed in v0.91).
fn capture_tunable_baselines_internal(
    editor_components: &Query<(&EditorComponent, &SceneInstanceChild)>,
) {
    use std::collections::BTreeMap;

    let mut baselines: BTreeMap<String, serde_json::Value> = BTreeMap::new();

    for (editor_comp, instance_child) in editor_components.iter() {
        let key = instance_child.instance_id.as_str().to_string();
        baselines.insert(key, editor_comp.0.values.clone());
    }

    // v0.90 PR3: write ONLY to the session (single source of truth).
    // The TUNABLE_BASELINES thread_local has been removed; the
    // session is the canonical owner (per ADR-0031).
    let _ = editor_model::ports::with_session_mut(|sess| {
        *sess.tunable_baselines_mut() = baselines;
    });
}

/// v0.91 PR1: derive `apply_back_eligible` from the schema's `ApplyBackPolicy`
/// (ADR-0050 / ADR-0042). A delta is eligible only when the schema's policy
/// is NOT `Never`. Schemas with `ExplicitOnly` or `Tunable` produce eligible
/// deltas; schemas with `Never` produce deltas that the ApplyBackPanel will
/// filter out (the "Never policy records no delta" scenario in spec §3).
///
/// Falls back to `true` (eligible) if the schema is not registered —
/// conservative default to avoid silently dropping deltas for unknown
/// component types.
fn is_eligible_for_apply_back(component_type_id: &str) -> bool {
    use crate::ApplyBackPolicy;
    crate::schema::global_registry()
        .get(component_type_id)
        .map(|schema| !matches!(schema.apply_back, ApplyBackPolicy::Never))
        .unwrap_or(true)
}

/// Recursive field-level diff. Emits one `RuntimeDelta` per leaf key whose
/// value differs from the current (or that is missing in current).
fn diff_recursive(
    instance_id: &str,
    component_type_id: &str,
    field_path: &str,
    baseline: &serde_json::Value,
    current: Option<&serde_json::Value>,
    now_ms: u64,
    out: &mut Vec<editor_model::RuntimeDelta>,
) {
    match (baseline, current) {
        // Both are objects → recurse into the keys.
        (serde_json::Value::Object(b), Some(serde_json::Value::Object(c))) => {
            for (k, v) in b {
                let nested_path = format!("{field_path}.{k}");
                let cur = c.get(k);
                diff_recursive(
                    instance_id,
                    component_type_id,
                    &nested_path,
                    v,
                    cur,
                    now_ms,
                    out,
                );
            }
            // Keys present in current but not in baseline → runtime-only changes,
            // not part of the apply-back delta stream (we report baseline→current
            // only). The user can still edit them in the editor.
        }
        // Baseline is an object, current is not (e.g. was removed) → one delta
        // for the whole path.
        (serde_json::Value::Object(_), Some(other)) => {
            out.push(editor_model::RuntimeDelta {
                instance_id: instance_id.to_string(),
                target_local_id: String::new(),
                component_type_id: component_type_id.to_string(),
                field_path: field_path.to_string(),
                baseline_value: baseline.clone(),
                runtime_value: other.clone(),
                captured_at_ms: now_ms,
                apply_back_eligible: is_eligible_for_apply_back(component_type_id),
            });
        }
        // Leaf values: compare and emit delta on difference.
        _ => {
            if Some(baseline) != current {
                out.push(editor_model::RuntimeDelta {
                    instance_id: instance_id.to_string(),
                    target_local_id: String::new(),
                    component_type_id: component_type_id.to_string(),
                    field_path: field_path.to_string(),
                    baseline_value: baseline.clone(),
                    runtime_value: current.cloned().unwrap_or(serde_json::Value::Null),
                    captured_at_ms: now_ms,
                    apply_back_eligible: is_eligible_for_apply_back(component_type_id),
                });
            }
        }
    }
}

/// Compute runtime deltas (v0.90 PR1) — pure function for testability.
///
/// Compares the baselines captured at `PlayModeEnter` (in
/// `EditorSession.tunable_baselines`) against the current `EditorComponent`
/// values from the Bevy query, and appends a `RuntimeDelta` to
/// `EditorSession.runtime_delta_buffer` for every instance whose values
/// changed.
///
/// The runtime getter `current_values` is a closure that maps an instance id
/// to its current `ComponentInstance` (typically queried from Bevy ECS). The
/// function does NOT depend on Bevy ECS directly — only the closure does —
/// which makes it testable from `crates/editor-application/tests/` without
/// spinning up a Bevy App.
///
/// The `apply_back_eligible` field on each delta is set to `true` for any
/// field whose baseline differs from the current value. (Field-level
/// diff granularity is a v0.91 follow-up; v0.90 PR1 emits one delta per
/// instance with `field_path = "*"`.)
///
/// Returns the number of deltas appended.
pub fn compute_runtime_deltas_internal<F>(current_values: F, now_ms: u64) -> usize
where
    F: Fn(&str) -> Option<crate::document::ComponentInstance>,
{
    use crate::document::ComponentInstance;

    // Snapshot the baselines out of the session (release the lock before
    // taking the runtime values).
    let baselines: std::collections::BTreeMap<String, serde_json::Value> =
        match editor_model::ports::with_session_mut(|sess| sess.tunable_baselines_mut().clone()) {
            Some(b) => b,
            None => return 0,
        };

    let mut deltas: Vec<editor_model::RuntimeDelta> = Vec::new();

    for (instance_id, baseline_value) in &baselines {
        let current = current_values(instance_id);
        let (current_obj, baseline_obj) = match (current, baseline_value.as_object()) {
            (Some(c), Some(b)) => (c.values, b.clone()),
            _ => continue,
        };

        // Field-level diff (recursive). Each component at the top level is
        // keyed by component_type_id. Nested fields are walked recursively:
        // e.g. {"editor.Transform2D": {"translation": {"x": 10.0, "y": 5.0}}}
        // emits one delta per leaf key (e.g. "translation.x", "translation.y")
        // when the value differs from the current.
        for (component_type_id, baseline_comp_value) in &baseline_obj {
            let baseline_comp = match baseline_comp_value.as_object() {
                Some(o) => o,
                None => continue,
            };
            let current_comp = current_obj
                .get(component_type_id)
                .and_then(|v| v.as_object());

            let mut leaf_deltas: Vec<editor_model::RuntimeDelta> = Vec::new();
            for (field_name, baseline_field_value) in baseline_comp {
                let current_field_value = current_comp.and_then(|c| c.get(field_name));
                diff_recursive(
                    instance_id,
                    component_type_id,
                    field_name,
                    baseline_field_value,
                    current_field_value,
                    now_ms,
                    &mut leaf_deltas,
                );
            }
            // If the component was present in baseline but missing in current,
            // emit one delta per top-level field with runtime_value = Null.
            if current_comp.is_none() {
                for (field_name, baseline_field_value) in baseline_comp {
                    leaf_deltas.push(editor_model::RuntimeDelta {
                        instance_id: instance_id.clone(),
                        target_local_id: String::new(),
                        component_type_id: component_type_id.clone(),
                        field_path: field_name.clone(),
                        baseline_value: baseline_field_value.clone(),
                        runtime_value: serde_json::Value::Null,
                        captured_at_ms: now_ms,
                        apply_back_eligible: is_eligible_for_apply_back(component_type_id),
                    });
                }
            }
            deltas.extend(leaf_deltas);
        }
        // Suppress unused-variable lint when ComponentInstance import is only
        // used in the closure signature above.
        let _ = std::marker::PhantomData::<ComponentInstance>;
    }

    let appended = deltas.len();
    if appended > 0 {
        let _ = editor_model::ports::with_session_mut(|sess| {
            let buffer = sess.runtime_delta_buffer_mut();
            for d in deltas {
                buffer.push_back(d);
            }
        });
    }
    appended
}

// ─────────────────────────────────────────────────────────────────────────────
// process_hot_reload_requests
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// rebuild_preview_world + helpers (push_preview_inspector_state, spawn_*)
// ─────────────────────────────────────────────────────────────────────────────

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
    let current_doc = crate::SCENE_DOC.with(|s| s.borrow().clone());
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
        crate::state::with_asset_body_cache(|cache| cache.get(asset_ref.as_str()).cloned())
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
fn push_preview_inspector_state(doc: &SceneDocument, projected: &[PreviewEntity]) {
    use crate::preview_inspector::{
        PreviewMappingEntry, PreviewProvenance, set_mapping, set_provenance,
    };
    use std::collections::BTreeMap;

    let mut mapping: Vec<PreviewMappingEntry> = Vec::new();
    let mut provenance: BTreeMap<StableId, PreviewProvenance> = BTreeMap::new();

    // Build per-instance mapping/provenance from doc.instances + projected.
    for instance in doc.instances.values() {
        let projected_for_instance: Vec<&PreviewEntity> = projected
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
                    causality_edges: Vec::new(), // §6: stamped by submit_actuator_output
                },
            );
        }
    }

    set_mapping(mapping);
    set_provenance(provenance);
    // §6: Apply any causality edges collected during logic evaluation.
    // Note: full BevyEntity→StableId mapping requires Query iteration;
    // edges stamped by submit_actuator_output are stored by entity bits
    // and converted using the scene_entity query in apply_actuator_outputs.
    crate::preview_inspector::apply_pending_causality_edges();
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
    let mut logic_binding: Option<LogicBinding> = None;

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
                logic_binding = Some(LogicBinding { asset_id, version });
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

    // ADR-0042: Store all component values as JSON so process_play_mode_request
    // can capture tunable baselines at PlayModeEnter without re-running project_instances.
    // Nested structure: { "editor.Transform2D": {"translation": {...}}, ... }
    let mut merged_values = serde_json::Map::new();
    for comp in &preview.component_values {
        if let Some(obj) = comp.values.as_object() {
            let nested: serde_json::Map<String, serde_json::Value> =
                obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            merged_values.insert(comp.type_id.clone(), serde_json::Value::Object(nested));
        }
    }
    let merged = serde_json::Value::Object(merged_values);
    cmd.insert(EditorComponent(ComponentInstance {
        type_id: String::new(), // unused — values are self-contained
        values: merged,
    }));

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

// ─────────────────────────────────────────────────────────────────────────────
// process_commands
// ─────────────────────────────────────────────────────────────────────────────

fn process_commands(
    // BUG-2 (Bevy 0.19 B0001) fix: `Without<SceneEntity>` makes this query
    // provably disjoint from scene-entity transforms, which are mutated by
    // `process_play_mode_request` (ParamSet) and `apply_actuator_outputs` in
    // play mode. The legacy JS sprite-move command targets the single
    // non-scene sprite that pre-dates the SceneInstance pipeline.
    mut sprites: Query<&mut Transform, (With<Sprite>, Without<SceneEntity>)>,
) {
    // §6 D7: Stamp the rebuild cause so rebuild_preview_world records it.
    crate::preview_inspector::record_rebuild_cause(crate::RebuildCause::UserEdit {
        command_id: "legacy_sprite_move".to_string(),
    });

    let cmds = crate::COMMAND_BUS.with(|b| {
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

// ─────────────────────────────────────────────────────────────────────────────
// emit_events
// ─────────────────────────────────────────────────────────────────────────────

fn emit_events(
    // BUG-2 (Bevy 0.19 B0001) fix: `Without<SceneEntity>` keeps this immutable
    // Transform read disjoint from the mutable Transform accesses in
    // `apply_actuator_outputs` and `process_play_mode_request` (ParamSet).
    // `single()` expects exactly one entity — the legacy sprite — not a scene
    // entity.
    sprites: Query<&Transform, (With<Sprite>, Without<SceneEntity>)>,
    time: Res<Time>,
    mut fps_accum: Local<f32>,
    mut frame_count: Local<u32>,
) {
    crate::EVENT_BUS.with(|b| {
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

// ─────────────────────────────────────────────────────────────────────────────
// sync_log_state
// ─────────────────────────────────────────────────────────────────────────────

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
// v0.91 PR1: poll_recent_change_sets system — populates
// `EditorSession.recent_change_sets` from the in-process `OPERATION_LOG`.
// Runs after `process_commands` so each apply shows up in the ApplyBackPanel's
// "Recent History" view on the next poll.
// ─────────────────────────────────────────────────────────────────────────────

/// Periodic poll: walks the operation log and pushes one
/// `ChangeSetSummary` per entry to the active scene's recent-change buffer.
///
/// v0.91 PR1 stop-gap: iterates the in-process `OPERATION_LOG` once per call,
/// pushes any entries not yet seen. Deduplication by `change_id` is deferred
/// (a future PR will add a `last_seen` cursor on the buffer). Currently the
/// poll is called from the Bevy system below; tests call it directly.
pub fn poll_recent_change_sets_inner() {
    use editor_model::ChangeSetSummary;
    let entries: Vec<crate::operation_log::LogEntry> =
        crate::OPERATION_LOG.with(|log| log.borrow().snapshot_entries());
    if entries.is_empty() {
        return;
    }
    // Use a synthetic scene path when no active document is selected.
    // The session-level `DocumentSelection::path` is not exposed via the
    // trait; v0.91+ follow-up will thread it through.
    let scene_path = "_default".to_string();

    let _ = editor_model::ports::with_session_mut(|sess| {
        for entry in &entries {
            let summary = ChangeSetSummary {
                origin: entry
                    .origin
                    .clone()
                    .unwrap_or_else(|| entry.metadata.authorship.clone()),
                actor: entry
                    .actor
                    .clone()
                    .unwrap_or_else(|| entry.metadata.authorship.clone()),
                applied_at_ms: entry.metadata.timestamp,
                ops_touched: 1, // coarse — OperationLog entries don't carry the
                                // full command list, so we use a placeholder.
                                // v0.91+ follow-up will compute the real count.
            };
            sess.push_recent_change_set(&scene_path, summary);
        }
    });
}

/// Bevy system: `poll_recent_change_sets`. Runs in `Update` after
/// `process_commands` so the buffer stays in sync with the in-process log.
pub fn poll_recent_change_sets_system() {
    poll_recent_change_sets_inner();
}
