//! WASM glue — initializes the OPFS project store and registers it with `editor_model::ports`.
//!
//! `OpfsProjectStore` is constructed here, then registered with the global
//! `PROJECT_STORE` in `editor_model::ports`. Both `editor_core` and
//! `editor_application` access it via `editor_model::ports::with_project_store()`.

use crate::OpfsProjectStore;
use editor_model::ports::register_project_store;
use wasm_bindgen::prelude::*;

/// Initialize the project store (called from TypeScript at WASM startup).
///
/// Creates `OpfsProjectStore`, hydrates from OPFS, then registers it
/// with `editor_model::ports::register_project_store()`.
#[wasm_bindgen]
pub async fn init_project_store() -> Result<(), wasm_bindgen::JsValue> {
    let store = OpfsProjectStore::new();
    store
        .hydrate()
        .await
        .map_err(|e| wasm_bindgen::JsValue::from_str(&*e))?;

    // Wrap in Arc<dyn ProjectStore> and register.
    let arc_store: std::sync::Arc<dyn editor_model::ProjectStore> = std::sync::Arc::new(store);
    register_project_store(arc_store);
    Ok(())
}
