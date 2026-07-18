//! HIGH-1 phase 3b: WASM exports for hot-reload queue.
//!
//! Owns the 3 hot-reload-related WASM exports (source / asset / force)
//! plus their native-only test helpers. Each function has a wasm + host
//! pair gated on `target_arch = "wasm32"`. The wasm pair is registered
//! with `wasm-bindgen`; the host pair is for integration tests.

use crate::hot_reload_state::{HotReloadRequest, HOT_RELOAD_BUS};

/// Push a Source hot-reload request onto the HOT_RELOAD_BUS.
/// wasm-bindgen wrapper — callable from TypeScript via `window.hot_reload_source_wasm`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hot_reload_source_wasm(file_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    HOT_RELOAD_BUS.with(|bus| {
        bus.borrow_mut().push(HotReloadRequest::Source {
            file_id: file_id.to_string(),
        });
    });
    Ok(())
}

/// Native-only helper for tests: push a Source request and return bus depth.
#[cfg(not(target_arch = "wasm32"))]
pub fn hot_reload_source_wasm(file_id: &str) {
    HOT_RELOAD_BUS.with(|bus| {
        bus.borrow_mut().push(HotReloadRequest::Source {
            file_id: file_id.to_string(),
        });
    });
}

/// Push an Asset hot-reload request onto the HOT_RELOAD_BUS.
/// wasm-bindgen wrapper — callable from TypeScript via `window.hot_reload_asset_wasm`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hot_reload_asset_wasm(asset_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    HOT_RELOAD_BUS.with(|bus| {
        bus.borrow_mut().push(HotReloadRequest::Asset {
            asset_id: asset_id.to_string(),
        });
    });
    Ok(())
}

/// Native-only helper for tests: push an Asset request.
#[cfg(not(target_arch = "wasm32"))]
pub fn hot_reload_asset_wasm(asset_id: &str) {
    HOT_RELOAD_BUS.with(|bus| {
        bus.borrow_mut().push(HotReloadRequest::Asset {
            asset_id: asset_id.to_string(),
        });
    });
}

/// Push a ForceReloadAll request onto the HOT_RELOAD_BUS.
/// wasm-bindgen wrapper — callable from TypeScript via `window.force_reload_wasm`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn force_reload_wasm() -> Result<(), wasm_bindgen::JsValue> {
    HOT_RELOAD_BUS.with(|bus| {
        bus.borrow_mut().push(HotReloadRequest::ForceReloadAll);
    });
    Ok(())
}

/// Native-only helper for tests: push a ForceReloadAll request.
#[cfg(not(target_arch = "wasm32"))]
pub fn force_reload_wasm() {
    HOT_RELOAD_BUS.with(|bus| {
        bus.borrow_mut().push(HotReloadRequest::ForceReloadAll);
    });
}

/// Returns the current number of entries in HOT_RELOAD_BUS.
/// Used by integration tests to assert bus depth.
pub fn hot_reload_bus_depth_for_tests() -> usize {
    HOT_RELOAD_BUS.with(|bus| bus.borrow().len())
}
