//! HIGH-1 phase 3d: WASM exports for scene-document export.
//!
//! Owns `export_dynamic_scene_wasm` which converts a SceneDocument JSON
//! to Bevy's DynamicScene format (the runtime preview format).

use wasm_bindgen::prelude::*;

use crate::document::SceneDocument;

/// Export a SceneDocument JSON to a Bevy DynamicScene format.
///
/// Returns a JSON string envelope of shape `{ json: String, warnings: ExportWarning[] }`.
/// The inner `DynamicSceneExport` is serialized separately because it contains
/// nested `serde_json::Value` inside `BTreeMap` values that are easier to
/// round-trip as a JSON string than via `serde_wasm_bindgen`.
///
/// Returns a JsValue error (thrown as exception on the JS side) if the input
/// is not valid SceneDocument JSON.
#[wasm_bindgen]
pub fn export_dynamic_scene_wasm(doc_json: &str) -> Result<JsValue, JsValue> {
    let doc: SceneDocument = serde_json::from_str(doc_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    let export = crate::dynamic_scene::export_dynamic_scene(&doc)
        .map_err(|e| JsValue::from_str(&format!("Export error: {}", e)))?;

    // Marshal the response as `{ json: String, warnings: ExportWarning[] }`.
    // We re-use the JSON string approach for the inner DynamicSceneExport
    // because it contains nested serde_json::Value inside BTreeMap values.
    let export_json = serde_json::to_string(&export)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize export: {}", e)))?;

    let envelope = serde_json::json!({
        "json": export_json,
        "warnings": export.warnings,
    });
    let envelope_str = serde_json::to_string(&envelope)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize envelope: {}", e)))?;
    Ok(JsValue::from_str(&envelope_str))
}
