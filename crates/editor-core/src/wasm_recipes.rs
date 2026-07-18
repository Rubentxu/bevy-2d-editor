//! HIGH-1 phase 3d: WASM export for built-in logic recipes.
//!
//! Owns the single export `list_builtin_recipes_wasm` that returns
//! metadata for all immutable recipes embedded in the editor.

use wasm_bindgen::prelude::*;

/// List all built-in immutable recipes with metadata.
/// Returns JSON array of { asset_id, name, description, node_count }.
#[wasm_bindgen]
pub fn list_builtin_recipes_wasm() -> Result<JsValue, JsValue> {
    let recipes = crate::logic_recipes::list_builtin_recipes();
    serde_wasm_bindgen::to_value(&recipes)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize recipes: {}", e)))
}
