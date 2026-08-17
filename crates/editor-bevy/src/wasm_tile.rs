//! HIGH-1 phase 3d: WASM exports for tile paint/erase operations.
//!
//! Owns `paint_tile_wasm` and `erase_tile_wasm`, both of which route
//! through the AssetCommand surface (HIGH-10) for proper undo/redo.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::asset_command::AssetCommand;
#[cfg(target_arch = "wasm32")]
use crate::asset_state::{with_asset_body_cache, with_asset_body_cache_mut, with_asset_log_mut};
#[cfg(target_arch = "wasm32")]
use crate::scene_asset::SceneAssetDocument;
#[cfg(target_arch = "wasm32")]
use crate::tile_layer::TileLayerId;

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

    let mut doc = doc_opt.ok_or_else(|| JsValue::from_str("Scene asset not found"))?;

    // Apply PaintTile through the command surface (captures inverse)
    let cmd = AssetCommand::PaintTile {
        layer_id: TileLayerId::from(layer_id),
        x,
        y,
        old_tile: None,
        tileset_id: tileset_id.to_string(),
        local_index,
    };
    let inverse = crate::asset_command::apply(&mut doc, &cmd)
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
    super::set_asset_document_wasm(&doc_json)?;

    Ok(JsValue::NULL)
}

/// Erase a tile from a TileLayer.
///
/// HIGH-10: routes through the AssetCommand surface so undo/redo captures
/// the erased TileRef via the inverse `PaintTile`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn erase_tile(asset_ref: &str, layer_id: &str, x: i32, y: i32) -> Result<JsValue, JsValue> {
    // Load the SceneAssetDocument from cache
    let mut doc_opt: Option<SceneAssetDocument> = None;
    with_asset_body_cache(|cache| {
        doc_opt = cache.get(asset_ref).cloned();
    });

    let mut doc = doc_opt.ok_or_else(|| JsValue::from_str("Scene asset not found"))?;

    // Apply EraseTile through the command surface (captures inverse)
    let cmd = AssetCommand::EraseTile {
        layer_id: TileLayerId::from(layer_id),
        x,
        y,
        erased_tile: None,
    };
    let inverse = crate::asset_command::apply(&mut doc, &cmd)
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
    super::set_asset_document_wasm(&doc_json)?;

    Ok(JsValue::NULL)
}
