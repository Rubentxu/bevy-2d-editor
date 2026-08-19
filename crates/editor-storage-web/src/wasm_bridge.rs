//! wasm32-only inner module — all JS interop lives here (archcheck B9 compliant).
//!
//! ## Design note
//!
//! The async JS operations (list, read, write, delete) are exposed as
//! standalone async functions in this module (`list_op`, `read_op`, `write_op`,
//! `delete_op`, `flush_op`). They use `JsFuture::from(promise).await` directly.
//!
//! `OpfsProjectStore::hydrate()` and `OpfsProjectStore::flush()` call these
//! functions directly rather than going through `RawStoreBridge`, which stays
//! synchronous for the native `MemoryBridge` implementation.
//!
//! `JsBridge` is kept as a stub so the `#[wasm_bindgen] extern "C"` block
//! can compile (the externs are referenced by the async functions below).

use crate::opfs_core::PendingOp;
use editor_model::time::{Clock, Timestamp};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// `window.opfs_*` extern declarations — mirror of `editor-core/src/lib.rs`.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = opfs_save_file)]
    fn opfs_save_file_raw(path: &str, contents: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_load_file)]
    fn opfs_load_file_raw(path: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_list_files)]
    fn opfs_list_files_raw(path: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_list_tree)]
    fn opfs_list_tree_raw(path: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_exists)]
    fn opfs_exists_raw(path: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_delete_file)]
    fn opfs_delete_file_raw(path: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_save_binary)]
    fn opfs_save_binary_raw(path: &str, contents: &js_sys::Uint8Array) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_load_binary)]
    fn opfs_load_binary_raw(path: &str) -> js_sys::Promise;
}

/// Parse a `{ok, error?}` JSON response.
fn parse_ok_response(val: serde_json::Value) -> Result<(), String> {
    if val.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Err(val
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error")
            .to_string());
    }
    Ok(())
}

/// Parse a `{ok, value?, error?}` response and extract a String array.
fn parse_string_array(val: serde_json::Value) -> Result<Vec<String>, String> {
    if val.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Err(val
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error")
            .to_string());
    }
    let arr = val
        .get("value")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Missing value array".to_string())?;
    Ok(arr
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect())
}

/// Async list operation — lists files under a directory prefix.
pub async fn list_op(dir: &str) -> Result<Vec<String>, String> {
    let promise = opfs_list_files_raw(dir);
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("JS promise rejected: {:?}", e))?;
    let val: serde_json::Value = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Bad bridge response: {}", e))?;
    parse_string_array(val)
}

/// Async recursive list operation — lists ALL files under a directory,
/// including files inside subdirectories (paths are "/"-joined, relative to
/// the OPFS namespace root, e.g. `schemas/game.PlayerHealth.schema.json`).
pub async fn list_tree_op(dir: &str) -> Result<Vec<String>, String> {
    let promise = opfs_list_tree_raw(dir);
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("JS promise rejected: {:?}", e))?;
    let val: serde_json::Value = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Bad bridge response: {}", e))?;
    parse_string_array(val)
}

/// Async read operation — reads binary file contents.
pub async fn read_op(path: &str) -> Result<Vec<u8>, String> {
    let promise = opfs_load_binary_raw(path);
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("JS promise rejected: {:?}", e))?;
    // Binary response: {ok: true, value: Uint8Array}
    let obj = js_sys::Object::from(result);
    let ok = js_sys::Reflect::get(&obj, &"ok".into())
        .map_err(|e| format!("Reflect::get('ok') failed: {:?}", e))?;
    if ok.as_bool() != Some(true) {
        let err = js_sys::Reflect::get(&obj, &"error".into())
            .map_err(|e| format!("Reflect::get('error') failed: {:?}", e))?;
        return Err(err
            .as_string()
            .unwrap_or_else(|| "Unknown error".to_string()));
    }
    let value = js_sys::Reflect::get(&obj, &"value".into())
        .map_err(|e| format!("Reflect::get('value') failed: {:?}", e))?;
    let bytes: Vec<u8> = js_sys::Uint8Array::new(&value).to_vec();
    Ok(bytes)
}

/// Async write operation — saves bytes to a file.
pub async fn write_op(path: &str, bytes: &[u8]) -> Result<(), String> {
    let promise =
        if path.ends_with(".json") || path.ends_with(".txt") || !bytes.iter().any(|&b| b > 127) {
            // Text mode
            let text =
                String::from_utf8(bytes.to_vec()).map_err(|e| format!("Not UTF-8: {}", e))?;
            opfs_save_file_raw(path, &text)
        } else {
            // Binary mode
            let js_bytes = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
            js_bytes.copy_from(bytes);
            opfs_save_binary_raw(path, &js_bytes)
        };
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("JS promise rejected: {:?}", e))?;
    let val: serde_json::Value = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Bad bridge response: {}", e))?;
    parse_ok_response(val)
}

/// Async delete operation — removes a file.
pub async fn delete_op(path: &str) -> Result<(), String> {
    let promise = opfs_delete_file_raw(path);
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("JS promise rejected: {:?}", e))?;
    let val: serde_json::Value = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Bad bridge response: {}", e))?;
    parse_ok_response(val)
}

/// Flush a single [`PendingOp`] through the JS bridge.
///
/// Used by [`super::OpfsProjectStore::flush`].
pub async fn flush_op(_path: &str, op: PendingOp) -> Result<(), String> {
    match op {
        PendingOp::Write { path, bytes } => write_op(&path, &bytes).await,
        PendingOp::Delete { path } => delete_op(&path).await,
    }
}

/// Clock using `js_sys::Date.now()` — production WASM clock.
#[derive(Debug, Default)]
pub struct SysClock;

impl SysClock {
    /// Create a new `SysClock`.
    pub fn new() -> Self {
        Self
    }
}

impl Clock for SysClock {
    fn now(&self) -> Timestamp {
        Timestamp(js_sys::Date::now() as u64)
    }
}
