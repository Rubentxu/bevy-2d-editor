//! HIGH-1 phase 3d: WASM exports for Scene Instance Layer operations.
//!
//! Owns 3 functions: list/create/delete SceneInstanceLayer entries on
//! a SceneAssetDocument. All operate on a JSON document and return the
//! updated JSON (caller is responsible for persisting via set_asset_document).

use wasm_bindgen::prelude::*;

use crate::scene_asset::{
    LayerId, LevelLayer, SceneAssetDocument, SceneInstanceLayer, SceneInstanceLayerKind,
};

/// Parse the asset JSON once; helper to reduce boilerplate across the 3 fns.
fn parse_asset_doc(asset_json: &str) -> Result<SceneAssetDocument, JsValue> {
    serde_json::from_str(asset_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid asset JSON: {}", e)))
}

/// List Scene Instance Layers of a Scene Asset document.
///
/// Returns a JSON array of `{ id, name, kind, order, instances_count }`,
/// omitting the `instances` vector for brevity at this level.
#[wasm_bindgen]
pub fn list_scene_instance_layers_wasm(asset_json: &str) -> Result<String, JsValue> {
    let doc: SceneAssetDocument = parse_asset_doc(asset_json)?;

    let mut out: Vec<serde_json::Value> = Vec::with_capacity(doc.layers.len());
    for layer in &doc.layers {
        match layer {
            LevelLayer::SceneInstance(scene_layer) => {
                out.push(serde_json::json!({
                    "id": scene_layer.id.as_str(),
                    "name": scene_layer.name,
                    "kind": scene_layer.kind,
                    "order": scene_layer.order,
                    "instances_count": scene_layer.instances.len(),
                }));
            }
            LevelLayer::Tile(_) | LevelLayer::Auto(_) => {
                // Tile and Auto layers are handled separately in their respective APIs
            }
        }
    }
    serde_json::to_string(&out)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize layers: {}", e)))
}

/// Create a new Scene Instance Layer in the asset document and return the
/// updated asset JSON. Rejects unknown `kind` values.
#[wasm_bindgen]
pub fn create_scene_instance_layer_wasm(
    asset_json: &str,
    name: &str,
    kind: &str,
) -> Result<String, JsValue> {
    let mut doc: SceneAssetDocument = parse_asset_doc(asset_json)?;

    // Parse kind
    let parsed_kind: SceneInstanceLayerKind = match kind {
        "actors" => SceneInstanceLayerKind::Actors,
        "props" => SceneInstanceLayerKind::Props,
        "spawns" => SceneInstanceLayerKind::Spawns,
        "triggers" => SceneInstanceLayerKind::Triggers,
        "collision" => SceneInstanceLayerKind::Collision,
        "custom" => SceneInstanceLayerKind::Custom,
        other => {
            return Err(JsValue::from_str(&format!(
                "Unknown layer kind '{}'. Allowed: actors, props, spawns, triggers, collision, custom",
                other
            )))
        }
    };

    // Generate a stable layer id.
    let now = crate::time::now_nanos();
    let new_id = LayerId::new(format!("lyr_{:x}", now));

    // Compute next order = max(order) + 1, falling back to 0.
    let next_order = doc
        .layers
        .iter()
        .filter_map(|l| match l {
            LevelLayer::SceneInstance(s) => Some(s.order),
            LevelLayer::Tile(_) | LevelLayer::Auto(_) => None,
        })
        .max()
        .map(|o| o + 1)
        .unwrap_or(0);

    doc.layers.push(LevelLayer::SceneInstance(SceneInstanceLayer {
        id: new_id,
        name: name.to_string(),
        kind: parsed_kind,
        order: next_order,
        instances: Vec::new(),
    }));

    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize asset: {}", e)))
}

/// Delete a Scene Instance Layer by id and return the updated asset JSON.
/// If the layer id is unknown, the asset is returned unchanged.
#[wasm_bindgen]
pub fn delete_scene_instance_layer_wasm(
    asset_json: &str,
    layer_id: &str,
) -> Result<String, JsValue> {
    let mut doc: SceneAssetDocument = parse_asset_doc(asset_json)?;
    let before = doc.layers.len();
    doc.layers.retain(|l| match l {
        LevelLayer::SceneInstance(s) => s.id.as_str() != layer_id,
        _ => true,
    });
    if doc.layers.len() == before {
        // Unknown id is a no-op; return current asset.
        // Doc comment in spec: "Delete unknown layer is a no-op".
    }
    serde_json::to_string(&doc)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize asset: {}", e)))
}
