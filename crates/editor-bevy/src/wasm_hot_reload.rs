//! HIGH-1 phase 3b: WASM exports for hot-reload queue.
//!
//! Owns the 3 hot-reload-related WASM exports (source / asset / force)
//! plus their native-only test helpers. Each function has a wasm + host
//! pair gated on `target_arch = "wasm32"`. The wasm pair is registered
//! with `wasm-bindgen`; the host pair is for integration tests.
//!
//! **H2.5 Block G**: writes are dual-write — production path is
//! `EditorSession.runtime.hot_reload_requests` (via
//! `with_session_mut`), fallback is `HOT_RELOAD_BUS_FALLBACK`
//! (used by legacy tests + non-session entrypoints).

use crate::hot_reload_state::{HotReloadRequest, HOT_RELOAD_BUS_FALLBACK};
use editor_model::ports::with_session_mut;

fn push_hot_reload(req: HotReloadRequest) {
    // Dual-write: session first, fallback if no session installed.
    let via_session = with_session_mut(|s| {
        s.runtime_hot_reload_requests_mut().push(req.clone());
        true
    });
    if via_session.unwrap_or(false) {
        return;
    }
    HOT_RELOAD_BUS_FALLBACK.with(|bus| {
        bus.borrow_mut().push(req);
    });
}

/// Push a Source hot-reload request onto the hot-reload bus.
/// wasm-bindgen wrapper — callable from TypeScript via `window.hot_reload_source_wasm`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hot_reload_source_wasm(file_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    push_hot_reload(HotReloadRequest::Source {
        file_id: file_id.to_string(),
    });
    Ok(())
}

/// Native-only helper for tests: push a Source request and return bus depth.
#[cfg(not(target_arch = "wasm32"))]
pub fn hot_reload_source_wasm(file_id: &str) {
    push_hot_reload(HotReloadRequest::Source {
        file_id: file_id.to_string(),
    });
}

/// Push an Asset hot-reload request onto the hot-reload bus.
/// wasm-bindgen wrapper — callable from TypeScript via `window.hot_reload_asset_wasm`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hot_reload_asset_wasm(asset_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    push_hot_reload(HotReloadRequest::Asset {
        asset_id: asset_id.to_string(),
    });
    Ok(())
}

/// Native-only helper for tests: push an Asset request.
#[cfg(not(target_arch = "wasm32"))]
pub fn hot_reload_asset_wasm(asset_id: &str) {
    push_hot_reload(HotReloadRequest::Asset {
        asset_id: asset_id.to_string(),
    });
}

/// Push a ForceReloadAll request onto the hot-reload bus.
/// wasm-bindgen wrapper — callable from TypeScript via `window.force_reload_wasm`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn force_reload_wasm() -> Result<(), wasm_bindgen::JsValue> {
    push_hot_reload(HotReloadRequest::ForceReloadAll);
    Ok(())
}

/// Native-only helper for tests: push a ForceReloadAll request.
#[cfg(not(target_arch = "wasm32"))]
pub fn force_reload_wasm() {
    push_hot_reload(HotReloadRequest::ForceReloadAll);
}

/// Returns the current number of entries in the FALLBACK hot-reload bus.
///
/// **H2.5 Block G**: this now reads from `HOT_RELOAD_BUS_FALLBACK`
/// (the dual-write fallback). If a session is installed, the
/// authoritative depth lives in `EditorSession.runtime.hot_reload_requests`;
/// use `with_session_mut(|s| s.runtime_hot_reload_requests_mut().len())`
/// for that. Tests in `hot_reload.rs` exercise the FALLBACK path.
pub fn hot_reload_bus_depth_for_tests() -> usize {
    HOT_RELOAD_BUS_FALLBACK.with(|bus| bus.borrow().len())
}
