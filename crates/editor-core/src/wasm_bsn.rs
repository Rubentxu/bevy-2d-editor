//! HIGH-1 phase 3b: WASM exports for BSN (`.bsn`) text format conversion.
//!
//! Owns the 2 round-trip exports:
//! - export_asset_to_bsn_wasm_from_json: SceneAssetDocument JSON → `.bsn` text
//! - import_bsn_text_to_asset_wasm: `.bsn` text → SceneAssetDocument JSON

use wasm_bindgen::prelude::*;

use crate::scene_asset::SceneAssetDocument;

/// Export a `SceneAssetDocument` (as JSON) to `.bsn` text.
///
/// Synchronous version for cases where the caller already has the document JSON.
#[wasm_bindgen]
pub fn export_asset_to_bsn_wasm_from_json(asset_json: &str) -> Result<String, JsValue> {
    let doc: SceneAssetDocument = serde_json::from_str(asset_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid asset JSON: {}", e)))?;
    crate::bsn_export::export_to_bsn_text(&doc)
        .map_err(|e| JsValue::from_str(&format!("BSN export error: {}", e)))
}

/// Parse `.bsn` text into a `SceneAssetDocument` via `BsnIr` round-trip.
/// Returns the document JSON string on success.
///
/// Use this to import `.bsn` files produced by `EditorCoreBsnExporter`
/// (the editor's own export). Import of Bevy-native `.bsn` files from other
/// tools requires type mapping that is not yet implemented.
#[wasm_bindgen]
pub fn import_bsn_text_to_asset_wasm(bsn_text: &str) -> Result<String, JsValue> {
    let ir = crate::bsn_import::parse_bsn_text(bsn_text)
        .map_err(|e| JsValue::from_str(&format!("BSN parse error: {:?}", e)))?;
    let doc = crate::bsn_import::scene_asset_from_bsn_ir(ir);
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}
