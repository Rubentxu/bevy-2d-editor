//! HIGH-1 phase 3: WASM exports for the runtime preview inspector.
//!
//! Thin wrappers around `crate::preview_inspector` that serialize to JSON
//! for the TypeScript side. Extracted from lib.rs to shrink the god-module
//! and group the 3 preview-related exports together.

use wasm_bindgen::prelude::*;

/// Live preview metrics: total entities, render frame time, etc.
/// Consumed by the Preview Inspector UI on the runtime sidebar.
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
