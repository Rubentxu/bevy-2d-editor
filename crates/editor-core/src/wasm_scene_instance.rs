//! HIGH-1 phase 3c: WASM exports for scene instance operations.
//!
//! Owns 7 functions covering the scene instance authoring surface:
//! - get_instance_components_wasm: read instance_components for a placed instance
//! - validate_overrides_wasm / effective_values_wasm / try_rebind_wasm: read-only
//!   validation/effective/rebind operations
//! - get_resync_reports: drain accumulated resync reports
//! - override_field_status_wasm / upsert_override_wasm / revert_override_wasm:
//!   override mutation through the shared OperationLog

use wasm_bindgen::prelude::*;

use crate::command::{Command, CommandEnvelope, CommandMetadata};
use crate::document::ComponentInstance;
use crate::scene_asset::{LocalId, SceneAssetDocument};
use crate::scene_instance::{ComponentOverride, SceneInstance};
use crate::schema::ComponentTypeId;

/// Get `instance_components` for a given placed `instance_id`.
///
/// Returns a JSON array of `ComponentInstance` objects, or `null` if no
/// instance with that id is loaded. Useful for the Scene Instance Layer
/// authoring UI to surface placement-time components (e.g. the
/// `editor.Transform2D` translation created by `place_scene_instance`).
#[wasm_bindgen]
pub fn get_instance_components_wasm(instance_id: &str) -> JsValue {
    let stable_id = crate::document::StableId::new(instance_id);
    crate::SCENE_DOC.with(|s| {
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

/// Validate a SceneInstance's overrides against an asset document.
/// Returns a JSON array of OverrideIssue objects.
#[wasm_bindgen]
pub fn validate_overrides_wasm(instance_json: &str, asset_json: &str) -> Result<String, JsValue> {
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
pub fn effective_values_wasm(instance_json: &str, asset_json: &str) -> Result<String, JsValue> {
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
pub fn try_rebind_wasm(orphaned_override_json: &str, asset_json: &str) -> Result<String, JsValue> {
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
    let reports = crate::asset_state::RESYNC_REPORTS.with(|r| {
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
        target_local_id: LocalId::new(local_id.to_string()),
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

    let snap = super::apply_envelope_internal(&envelope)
        .map_err(|e| JsValue::from_str(&format!("Failed to apply: {e}")))?;
    serde_json::to_string(&snap).map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
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
        target_local_id: LocalId::new(local_id.to_string()),
        component_type_id: ComponentTypeId::new(type_id.to_string()),
        field_path,
    };

    let envelope = CommandEnvelope {
        command,
        metadata: CommandMetadata::now("user"),
    };

    let envelope_json = serde_json::to_string(&envelope)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize envelope: {}", e)))?;

    let snap = super::apply_envelope_internal(&envelope)
        .map_err(|e| JsValue::from_str(&format!("Failed to apply: {e}")))?;
    serde_json::to_string(&snap).map_err(|e| JsValue::from_str(&format!("Serialize: {e}")))
}

// v0.90 PR6: removed pre-existing dead function
// `_ensure_component_instance_linked` (scaffolded, no-op, no callers).
// See debt-report from v0.89.
