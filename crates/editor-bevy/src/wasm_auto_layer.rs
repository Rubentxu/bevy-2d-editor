//! HIGH-1 phase 3d: WASM exports for AutoLayer operations.
//!
//! Owns 6 functions: is_auto_layer_stale_wasm (read-only stale check),
//! regenerate_auto_layer_wasm (regen via dispatch_asset_command),
//! and add/update/remove_auto_rule_wasm (rule mutations through the
//! AssetCommand surface + operation log).

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::asset_command::AssetCommand;
#[cfg(target_arch = "wasm32")]
use crate::asset_state::{with_asset_body_cache, with_asset_body_cache_mut, with_asset_log_mut};
#[cfg(target_arch = "wasm32")]
use crate::auto_layer::AutoLayerId;

/// Check if an AutoLayer's cached grid is stale — i.e., whether the source
/// TileLayer has been modified since the cache was last built.
///
/// Returns `true` if stale, `false` if the cache is up-to-date.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn is_auto_layer_stale_wasm(asset_ref: &str, layer_id: &str) -> Result<bool, JsValue> {
    use crate::scene_asset::LevelLayer;

    // Load from asset_body_cache
    let doc = with_asset_body_cache(|cache| cache.get(asset_ref).cloned())
        .ok_or_else(|| JsValue::from_str("Scene asset not found"))?;

    // Find the AutoLayer
    let auto_layer = doc
        .layers
        .iter()
        .find(|l| matches!(l, LevelLayer::Auto(al) if al.id.as_str() == layer_id))
        .ok_or_else(|| JsValue::from_str("AutoLayer not found"))?;

    let LevelLayer::Auto(al) = auto_layer else {
        return Err(JsValue::from_str("Layer is not an AutoLayer"));
    };

    // Find the source TileLayer
    let source_tl = doc
        .layers
        .iter()
        .find(
            |l| matches!(l, LevelLayer::Tile(tl) if tl.id.as_str() == al.source_layer_id.as_str()),
        )
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
pub fn regenerate_auto_layer_wasm(asset_ref: &str, layer_id: &str) -> Result<String, JsValue> {
    use crate::scene_asset::{LayerId, LevelLayer};

    // Load the doc from cache to find the AutoLayer
    let doc_for_layer = with_asset_body_cache(|cache| cache.get(asset_ref).cloned())
        .ok_or_else(|| JsValue::from_str("Scene asset not found"))?;

    // Find AutoLayer and capture old cached/source_generation for the command
    let (old_cached, old_source_generation) = match doc_for_layer
        .layers
        .iter()
        .find(|l| matches!(l, LevelLayer::Auto(al) if al.id.as_str() == layer_id))
    {
        Some(LevelLayer::Auto(al)) => (al.cached.clone(), al.source_generation),
        _ => return Err(JsValue::from_str("AutoLayer not found")),
    };

    // Build the RegenerateAutoLayer command
    let cmd = AssetCommand::RegenerateAutoLayer {
        layer_id: LayerId::new(layer_id.to_string()),
        old_cached,
        old_source_generation,
    };
    let cmd_json = serde_json::to_string(&cmd)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize command: {}", e)))?;

    // Route through dispatch_asset_command for operation log recording
    super::dispatch_asset_command(&cmd_json)?;

    // Fetch the updated doc and sync to asset_body_cache and SCENE_ASSET_DOC
    let updated_doc = crate::asset_state::with_asset_doc(|doc_opt| doc_opt.clone())
        .ok_or_else(|| JsValue::from_str("No asset open — asset doc was not set"))?;

    // Update asset_body_cache
    with_asset_body_cache_mut(|cache| {
        cache.insert(asset_ref.to_string(), updated_doc.clone());
    });

    // Sync to SCENE_ASSET_DOC via set_asset_document_wasm
    let updated_json = serde_json::to_string(&updated_doc)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
    super::set_asset_document_wasm(&updated_json)?;

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
    let inverse = crate::asset_command::apply(&mut doc, &cmd)
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
    super::set_asset_document_wasm(&doc_json)?;
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
    let inverse = crate::asset_command::apply(&mut doc, &cmd)
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
    super::set_asset_document_wasm(&doc_json)?;
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
    let inverse = crate::asset_command::apply(&mut doc, &cmd)
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
    super::set_asset_document_wasm(&doc_json)?;
    Ok(doc_json)
}
