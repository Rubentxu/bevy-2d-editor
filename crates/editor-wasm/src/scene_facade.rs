//! Scene-facing WASM facade.
//!
//! Per spec §6.5 (editor-wasm-facade), this module exposes scene-facing
//! adapters behind a stable facade. Scene operations flow through the
//! ChangeSet mechanism (ADR-0039); this facade organizes those exports
//! into a cohesive scene-oriented API surface.
//!
//! All functions in this module delegate to existing #[wasm_bindgen] exports
//! in lib.rs. This facade is NOT a stub — it provides scene-oriented
//! organization of the actual ChangeSet-based API.
//!
//! Scope: scene-facing operations (create, update, delete entities, layers,
//! scene assets, overrides). Logic-bricks and importers keep their current
//! crates per OQ-3.

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

// These re-exports match the actual #[wasm_bindgen] exports in lib.rs
// at the crate root. The lib name is "editor_application" per Cargo.toml.
#[cfg(target_arch = "wasm32")]
use crate::submit_pending_change_set;

#[cfg(target_arch = "wasm32")]
use crate::get_pending_change_sets;

#[cfg(target_arch = "wasm32")]
use crate::approve_change_set;

#[cfg(target_arch = "wasm32")]
use crate::approve_selected_ops;

#[cfg(target_arch = "wasm32")]
use crate::reject_change_set;

#[cfg(target_arch = "wasm32")]
use crate::get_change_set_summaries;

/// Submit a scene command as a ChangeSet.
///
/// This is the primary entry point for scene mutations. Commands include:
/// - Entity CRUD (CreateEntity, DeleteEntity, RenameEntity)
/// - Component CRUD (AddComponent, SetComponentField, RemoveComponent)
/// - Layer operations (CreateLayer, DeleteLayer, ReorderLayer)
/// - Scene Asset operations (InstantiateSceneAsset, BindOverride)
/// - Transform operations (Transform2D, Transform3D)
///
/// The envelope JSON shape:
/// ```json
/// {
///   "id": "local:12345",
///   "origin": "Human",
///   "actor": "user:local",
///   "rationale": "Create player spawn point",
///   "ops": [{ "command": { "type": "CreateEntity", ... }, "metadata": {} }]
/// }
/// ```
#[wasm_bindgen]
pub fn scene_submit_change_set(json: &str) -> Result<String, JsValue> {
    #[cfg(target_arch = "wasm32")]
    return submit_pending_change_set(json);

    #[cfg(not(target_arch = "wasm32"))]
    Err(JsValue::from_str(
        "scene_submit_change_set is only available in WASM",
    ))
}

/// Get all pending scene ChangeSets awaiting approval.
#[wasm_bindgen]
pub fn scene_get_pending() -> Result<JsValue, JsValue> {
    #[cfg(target_arch = "wasm32")]
    return get_pending_change_sets();

    #[cfg(not(target_arch = "wasm32"))]
    Err(JsValue::from_str(
        "scene_get_pending is only available in WASM",
    ))
}

/// Approve and apply all operations in a scene ChangeSet.
#[wasm_bindgen]
pub fn scene_approve(change_id: &str) -> Result<String, JsValue> {
    #[cfg(target_arch = "wasm32")]
    return approve_change_set(change_id);

    #[cfg(not(target_arch = "wasm32"))]
    Err(JsValue::from_str("scene_approve is only available in WASM"))
}

/// Selectively approve specific operations within a ChangeSet.
#[wasm_bindgen]
pub fn scene_approve_ops(change_id: &str, indices_json: &str) -> Result<String, JsValue> {
    #[cfg(target_arch = "wasm32")]
    return approve_selected_ops(change_id, indices_json);

    #[cfg(not(target_arch = "wasm32"))]
    Err(JsValue::from_str(
        "scene_approve_ops is only available in WASM",
    ))
}

/// Reject and discard a pending scene ChangeSet.
#[wasm_bindgen]
pub fn scene_reject(change_id: &str) -> Result<(), JsValue> {
    #[cfg(target_arch = "wasm32")]
    return reject_change_set(change_id);

    #[cfg(not(target_arch = "wasm32"))]
    Err(JsValue::from_str("scene_reject is only available in WASM"))
}

/// Get summaries of recent scene ChangeSets (history).
#[wasm_bindgen]
pub fn scene_change_set_summaries() -> Result<JsValue, JsValue> {
    #[cfg(target_arch = "wasm32")]
    return get_change_set_summaries();

    #[cfg(not(target_arch = "wasm32"))]
    Err(JsValue::from_str(
        "scene_change_set_summaries is only available in WASM",
    ))
}
