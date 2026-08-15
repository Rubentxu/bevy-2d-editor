//! OPFS-backed [`ProjectStore`] adapter.
//!
//! ## Architecture
//!
//! - `OpfsCore` — pure in-memory mirror + pending-op queue. No JS deps,
//!   compiled on all targets.
//! - `RawStoreBridge` — async I/O abstraction. On wasm32 backed by the
//!   `window.opfs_*` JS bridge via `js_sys::Promise`; on native backed by a
//!   [`MemoryBridge`] fake for tests.
//! - `wasm` module (`#[cfg(target_arch = "wasm32")]`) — wasm32-only bridge ops
//!   and `SysClock` implementations.
//! - [`OpfsProjectStore`] — public struct used at runtime. Holds
//!   `Arc<Mutex<OpfsCore>>` + `Arc<dyn RawStoreBridge>` + `Arc<dyn Clock>`.
//!
//! ## Durability semantics
//!
//! Every [`ProjectStore::write`] / [`ProjectStore::delete`] mutates the in-memory
//! mirror immediately and enqueues a pending operation. The `flush`
//! method drains the entire queue, awaits each operation through the bridge, and
//! only returns `Ok(())` once every operation has resolved. The seven
//! `js_*` wrappers in `editor-core` call `store.write(...).then(store.flush())`
//! so their `await` only resolves after the OPFS write is durable.

use crate::ports::project_store::{ProjectStore, StoreEntry, StoreError};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

// ─────────────────────────────────────────────────────────────────────────────
// core — portable mirror + pending queue (no JS dependencies)
// ─────────────────────────────────────────────────────────────────────────────

/// A pending write or delete that must be flushed to OPFS.
#[derive(Debug, Clone)]
pub(crate) enum PendingOp {
    Write { path: String, bytes: Vec<u8> },
    Delete { path: String },
}

/// Internal state: in-memory mirror + flush queue.
///
/// Compiled on ALL targets (including native) so contract tests can run without
/// a JS runtime.
#[derive(Debug)]
pub struct OpfsCore {
    /// Path → (bytes, modified_ms)
    entries: BTreeMap<String, (Vec<u8>, u64)>,
    /// Ordered queue of operations to flush to OPFS.
    pending: VecDeque<PendingOp>,
}

impl OpfsCore {
    /// Create an empty core with no pending operations.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            pending: VecDeque::new(),
        }
    }

    /// Pop and return all pending operations, leaving the queue empty.
    pub fn take_pending(&mut self) -> Vec<PendingOp> {
        self.pending.drain(..).collect()
    }

    /// Mirror read — used by [`ProjectStore::read`] and [`ProjectStore::exists`].
    pub fn read(&self, path: &str) -> Option<&[u8]> {
        self.entries.get(path).map(|(b, _)| b.as_slice())
    }

    /// Mirror write — mutates the in-memory mirror and enqueues a pending write op.
    pub fn write(&mut self, path: String, bytes: Vec<u8>, modified_ms: u64) {
        self.entries
            .insert(path.clone(), (bytes.clone(), modified_ms));
        self.pending.push_back(PendingOp::Write { path, bytes });
    }

    /// Mirror delete — removes from mirror and enqueues a pending delete op.
    pub fn delete(&mut self, path: String) {
        self.entries.remove(&path);
        self.pending.push_back(PendingOp::Delete { path });
    }

    /// List all entries whose paths start with `prefix`.
    pub fn list(&self, prefix: &str) -> Vec<StoreEntry> {
        self.entries
            .iter()
            .filter(|(p, _)| p.starts_with(prefix))
            .map(|(p, (b, m))| StoreEntry {
                path: p.clone(),
                size: b.len() as u64,
                modified_ms: *m,
            })
            .collect()
    }
}

impl Default for OpfsCore {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// bridge — async I/O abstraction (no wasm_bindgen at crate root per archcheck B2)
// ─────────────────────────────────────────────────────────────────────────────

/// Async I/O operations that the [`OpfsCore`] mirror delegates to.
///
/// On wasm32 this is implemented by [`wasm::JsBridge`] using
/// `js_sys::Promise` returns; on native / tests by a [`MemoryBridge`] fake.
pub(crate) trait RawStoreBridge: Send + Sync {
    /// List all file paths under `dir`. Returns the bare file names (not full paths).
    fn list(&self, dir: &str) -> Result<Vec<String>, String>;

    /// Read the full contents of a file.
    fn read(&self, path: &str) -> Result<Vec<u8>, String>;

    /// Write contents to a file.
    fn write(&self, path: &str, bytes: &[u8]) -> Result<(), String>;

    /// Delete a file.
    fn delete(&self, path: &str) -> Result<(), String>;
}

/// A raw-store bridge backed by an in-memory `BTreeMap` — for unit tests.
#[derive(Debug, Default)]
pub struct MemoryBridge {
    entries: std::sync::RwLock<BTreeMap<String, Vec<u8>>>,
}

impl MemoryBridge {
    /// Create a new empty `MemoryBridge`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl RawStoreBridge for MemoryBridge {
    fn list(&self, dir: &str) -> Result<Vec<String>, String> {
        let entries = self.entries.read().map_err(|_| "lock poisoned")?;
        Ok(entries
            .keys()
            .filter(|p| p.starts_with(dir))
            .map(|p| {
                p.trim_start_matches(dir)
                    .trim_start_matches('/')
                    .to_string()
            })
            .filter(|n| !n.is_empty())
            .collect())
    }

    fn read(&self, path: &str) -> Result<Vec<u8>, String> {
        self.entries
            .read()
            .map_err(|_| "lock poisoned")?
            .get(path)
            .cloned()
            .ok_or_else(|| format!("not found: {}", path))
    }

    fn write(&self, path: &str, bytes: &[u8]) -> Result<(), String> {
        self.entries
            .write()
            .map_err(|_| "lock poisoned")?
            .insert(path.to_string(), bytes.to_vec());
        Ok(())
    }

    fn delete(&self, path: &str) -> Result<(), String> {
        self.entries
            .write()
            .map_err(|_| "lock poisoned")?
            .remove(path);
        Ok(())
    }
}

/// A no-op bridge used as a placeholder on wasm32 where
/// `hydrate` / `flush` call `wasm::list_op` / `wasm::read_op` / `wasm::flush_op`
/// directly instead of going through the bridge.
#[derive(Debug, Default)]
pub struct NoOpBridge;

impl RawStoreBridge for NoOpBridge {
    fn list(&self, _dir: &str) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    fn read(&self, _path: &str) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }
    fn write(&self, _path: &str, _bytes: &[u8]) -> Result<(), String> {
        Ok(())
    }
    fn delete(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// wasm — wasm32-only JS bridge + clock
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    //! wasm32-only inner module — all JS interop lives here (archcheck B2 compliant).
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

    use super::*;
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
            if path.ends_with(".json") || path.ends_with(".txt") || !bytes.iter().any(|&b| b > 127)
            {
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
}

// ─────────────────────────────────────────────────────────────────────────────
// OpfsProjectStore — public API
// ─────────────────────────────────────────────────────────────────────────────

/// OPFS-backed [`ProjectStore`] that delegates to the JS `window.opfs_*` bridge.
///
/// ## Lifecycle
///
/// 1. Construct via `OpfsProjectStore::new()` (wasm32) or
///    `OpfsProjectStore::new_for_tests()` (tests + native).
/// 2. Call [`OpfsProjectStore::hydrate()`] once to populate the in-memory mirror
///    from OPFS — this is the "eager load" step. Binary assets are loaded at
///    startup; sync-access-handles are the future fix (ADR-0033).
/// 3. All [`ProjectStore`] operations are synchronous on the mirror.
/// 4. Call [`OpfsProjectStore::flush()`] to drain and execute all pending ops.
pub struct OpfsProjectStore {
    core: Arc<Mutex<OpfsCore>>,
    bridge: Arc<dyn RawStoreBridge>,
    clock: Arc<dyn editor_model::time::Clock>,
}

impl OpfsProjectStore {
    /// Create with the wasm32 JS bridge and `js_sys::Date` clock.
    /// Panics if called on a non-wasm32 target.
    ///
    /// Note: on wasm32, `hydrate()` and `flush()` call `wasm::list_op` / `wasm::read_op`
    /// / `wasm::flush_op` directly rather than going through `RawStoreBridge`. The
    /// bridge field is a no-op placeholder.
    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Self {
        Self::new_internal(
            Arc::new(Mutex::new(OpfsCore::new())),
            Arc::new(NoOpBridge),
            Arc::new(wasm::SysClock::new()),
        )
    }

    /// Create for tests with a [`MemoryBridge`] and [`editor_model::time::FakeClock`].
    #[cfg(test)]
    pub fn new_for_tests() -> Self {
        Self::new_internal(
            Arc::new(Mutex::new(OpfsCore::new())),
            Arc::new(MemoryBridge::new()),
            Arc::new(editor_model::time::FakeClock::new()),
        )
    }

    fn new_internal(
        core: Arc<Mutex<OpfsCore>>,
        bridge: Arc<dyn RawStoreBridge>,
        clock: Arc<dyn editor_model::time::Clock>,
    ) -> Self {
        Self {
            core,
            bridge,
            clock,
        }
    }

    /// Eagerly hydrate the in-memory mirror from OPFS.
    ///
    /// Lists all paths under "/" via the bridge, reads each file, and populates
    /// the mirror. Called **once** at WASM startup before the editor becomes
    /// interactive.
    ///
    /// ### Eager-load limitation
    ///
    /// All file contents (including binary assets) are loaded into memory at
    /// startup. For large projects this is a known bottleneck. The fix is
    /// deferred to a future iteration: sync-access-handles (ADR-0033) that
    /// lazily read only the bytes needed per operation.
    #[cfg(target_arch = "wasm32")]
    pub async fn hydrate(&self) -> Result<(), String> {
        let paths = wasm::list_op("/")
            .await
            .map_err(|e| format!("hydrate: list failed: {}", e))?;

        for path in paths {
            let bytes: Vec<u8> = match wasm::read_op(&path).await {
                Ok(b) => b,
                Err(_) => {
                    // Skip files we fail to read during hydration.
                    // They may be locked or corrupted.
                    continue;
                }
            };
            let modified_ms = self.clock.now().into_u64();
            // Mutex guard
            if let Ok(mut core) = self.core.try_lock() {
                core.write(path, bytes, modified_ms);
            }
        }
        Ok(())
    }

    /// Eagerly hydrate — non-wasm32 stub that returns `Ok(())`.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn hydrate(&self) -> Result<(), String> {
        Ok(())
    }

    /// Flush all pending operations through the bridge.
    ///
    /// Drains the pending queue and awaits each operation in order.
    /// Returns `Ok(())` only when all operations have resolved.
    #[cfg(target_arch = "wasm32")]
    pub async fn flush(&self) -> Result<(), StoreError> {
        let ops = {
            let mut core = self.core.try_lock().map_err(|_| StoreError::LockPoisoned)?;
            core.take_pending()
        };

        for op in ops {
            let path = match &op {
                PendingOp::Write { path, .. } => path.clone(),
                PendingOp::Delete { path } => path.clone(),
            };
            wasm::flush_op(&path, op)
                .await
                .map_err(|e| StoreError::Io(e))?;
        }
        Ok(())
    }

    /// Flush — non-wasm32 stub for test bridge (blocking).
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn flush(&self) -> Result<(), StoreError> {
        let ops = {
            let mut core = self.core.try_lock().map_err(|_| StoreError::LockPoisoned)?;
            core.take_pending()
        };

        for op in ops {
            match op {
                PendingOp::Write { path, bytes } => {
                    self.bridge
                        .write(&path, &bytes)
                        .map_err(|e| StoreError::Io(e))?;
                }
                PendingOp::Delete { path } => {
                    self.bridge.delete(&path).map_err(|e| StoreError::Io(e))?;
                }
            }
        }
        Ok(())
    }
}

impl ProjectStore for OpfsProjectStore {
    fn list(&self, prefix: &str) -> Result<Vec<StoreEntry>, StoreError> {
        let core = self.core.try_lock().map_err(|_| StoreError::LockPoisoned)?;
        Ok(core.list(prefix))
    }

    fn read(&self, path: &str) -> Result<Vec<u8>, StoreError> {
        let core = self.core.try_lock().map_err(|_| StoreError::LockPoisoned)?;
        core.read(path)
            .map(|b| b.to_vec())
            .ok_or_else(|| StoreError::NotFound(path.to_string()))
    }

    fn write(&self, path: &str, bytes: &[u8], _atomic: bool) -> Result<(), StoreError> {
        let mut core = self.core.try_lock().map_err(|_| StoreError::LockPoisoned)?;
        let modified_ms = self.clock.now().into_u64();
        core.write(path.to_string(), bytes.to_vec(), modified_ms);
        Ok(())
    }

    fn delete(&self, path: &str) -> Result<(), StoreError> {
        let mut core = self.core.try_lock().map_err(|_| StoreError::LockPoisoned)?;
        core.delete(path.to_string());
        Ok(())
    }

    fn exists(&self, path: &str) -> Result<bool, StoreError> {
        let core = self.core.try_lock().map_err(|_| StoreError::LockPoisoned)?;
        Ok(core.read(path).is_some())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── OpfsCore mirror-only tests ─────────────────────────────────────────

    #[test]
    fn test_opfs_core_write_read() {
        let mut core = OpfsCore::new();
        core.write("test.txt".to_string(), b"hello".to_vec(), 1000);
        assert_eq!(core.read("test.txt"), Some(&b"hello"[..]));
    }

    #[test]
    fn test_opfs_core_list() {
        let mut core = OpfsCore::new();
        core.write("a/b.txt".to_string(), b"1".to_vec(), 1000);
        core.write("a/c.txt".to_string(), b"2".to_vec(), 1000);
        core.write("d.txt".to_string(), b"3".to_vec(), 1000);
        let entries = core.list("a/");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_opfs_core_delete() {
        let mut core = OpfsCore::new();
        core.write("del.txt".to_string(), b"temp".to_vec(), 1000);
        assert!(core.read("del.txt").is_some());
        core.delete("del.txt".to_string());
        assert!(core.read("del.txt").is_none());
    }

    #[test]
    fn test_opfs_core_take_pending() {
        let mut core = OpfsCore::new();
        core.write("p1.txt".to_string(), b"a".to_vec(), 1000);
        core.write("p2.txt".to_string(), b"b".to_vec(), 1000);
        core.delete("p1.txt".to_string());
        let pending = core.take_pending();
        assert_eq!(pending.len(), 3); // 2 writes + 1 delete
        // Second take should be empty
        assert!(core.take_pending().is_empty());
    }

    #[test]
    fn test_opfs_core_exists() {
        let mut core = OpfsCore::new();
        core.write("ex.txt".to_string(), b"x".to_vec(), 1000);
        assert!(core.read("ex.txt").is_some());
        assert!(core.read("nx.txt").is_none());
    }

    // ── OpfsProjectStore + MemoryBridge contract tests ─────────────────────

    fn make_store() -> OpfsProjectStore {
        OpfsProjectStore::new_for_tests()
    }

    #[tokio::test]
    async fn test_contract_write_and_read() {
        let store = make_store();
        store.write("a/test.txt", b"hello", false).unwrap();
        store.write("a/sub/b.txt", b"world", false).unwrap();

        let entries = store.list("a/").unwrap();
        assert_eq!(entries.len(), 2);

        assert_eq!(store.read("a/test.txt").unwrap(), b"hello");
        assert_eq!(store.read("a/sub/b.txt").unwrap(), b"world");
    }

    #[tokio::test]
    async fn test_contract_list_empty_prefix() {
        let store = make_store();
        store.write("a/test.txt", b"hello", false).unwrap();
        let entries = store.list("nonexistent/").unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_contract_exists() {
        let store = make_store();
        store.write("exists.txt", b"content", false).unwrap();
        assert!(store.exists("exists.txt").unwrap());
        assert!(!store.exists("missing.txt").unwrap());
    }

    #[tokio::test]
    async fn test_contract_delete() {
        let store = make_store();
        store.write("to_delete.txt", b"temp", false).unwrap();
        assert!(store.exists("to_delete.txt").unwrap());

        store.delete("to_delete.txt").unwrap();
        assert!(!store.exists("to_delete.txt").unwrap());

        match store.read("to_delete.txt") {
            Err(StoreError::NotFound(_)) => {}
            other => panic!("Expected NotFound after delete, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_contract_list_after_delete() {
        let store = make_store();
        store.write("dir/file.txt", b"data", false).unwrap();
        assert_eq!(store.list("dir/").unwrap().len(), 1);

        store.delete("dir/file.txt").unwrap();
        assert!(store.list("dir/").unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_contract_read_missing() {
        let store = make_store();
        match store.read("definitely_missing_12345.txt") {
            Err(StoreError::NotFound(path)) => {
                assert!(path.contains("definitely_missing_12345"));
            }
            other => panic!("Expected NotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_contract_write_metadata() {
        let store = make_store();
        store.write("meta.txt", b"hello", false).unwrap();
        let entries = store.list("").unwrap();
        let entry = entries.iter().find(|e| e.path == "meta.txt").unwrap();
        assert_eq!(entry.size, 5);
        assert_eq!(entry.path, "meta.txt");
    }

    #[tokio::test]
    async fn test_contract_overwrite() {
        let store = make_store();
        store.write("over.txt", b"original", false).unwrap();
        store.write("over.txt", b"updated", false).unwrap();
        let entries = store.list("").unwrap();
        let entry = entries.iter().find(|e| e.path == "over.txt").unwrap();
        assert_eq!(entry.size, 7);
        assert_eq!(store.read("over.txt").unwrap(), b"updated");
    }

    // ── InMemoryProjectStore contract tests ────────────────────────────────

    use crate::InMemoryProjectStore;

    fn make_inmemory() -> InMemoryProjectStore {
        InMemoryProjectStore::new()
    }

    #[test]
    fn test_inmemory_write_and_read() {
        let store = make_inmemory();
        store.write("a/test.txt", b"hello", false).unwrap();
        store.write("a/sub/b.txt", b"world", false).unwrap();

        let entries = store.list("a/").unwrap();
        assert_eq!(entries.len(), 2);

        assert_eq!(store.read("a/test.txt").unwrap(), b"hello");
        assert_eq!(store.read("a/sub/b.txt").unwrap(), b"world");
    }

    #[test]
    fn test_inmemory_list_empty_prefix() {
        let store = make_inmemory();
        store.write("a/test.txt", b"hello", false).unwrap();
        let entries = store.list("nonexistent/").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_inmemory_exists() {
        let store = make_inmemory();
        store.write("exists.txt", b"content", false).unwrap();
        assert!(store.exists("exists.txt").unwrap());
        assert!(!store.exists("missing.txt").unwrap());
    }

    #[test]
    fn test_inmemory_delete() {
        let store = make_inmemory();
        store.write("to_delete.txt", b"temp", false).unwrap();
        assert!(store.exists("to_delete.txt").unwrap());

        store.delete("to_delete.txt").unwrap();
        assert!(!store.exists("to_delete.txt").unwrap());

        match store.read("to_delete.txt") {
            Err(StoreError::NotFound(_)) => {}
            other => panic!("Expected NotFound after delete, got {:?}", other),
        }
    }

    #[test]
    fn test_inmemory_list_after_delete() {
        let store = make_inmemory();
        store.write("dir/file.txt", b"data", false).unwrap();
        assert_eq!(store.list("dir/").unwrap().len(), 1);

        store.delete("dir/file.txt").unwrap();
        assert!(store.list("dir/").unwrap().is_empty());
    }

    #[test]
    fn test_inmemory_read_missing() {
        let store = make_inmemory();
        match store.read("definitely_missing_12345.txt") {
            Err(StoreError::NotFound(path)) => {
                assert!(path.contains("definitely_missing_12345"));
            }
            other => panic!("Expected NotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_inmemory_write_metadata() {
        let store = make_inmemory();
        store.write("meta.txt", b"hello", false).unwrap();
        let entries = store.list("").unwrap();
        let entry = entries.iter().find(|e| e.path == "meta.txt").unwrap();
        assert_eq!(entry.size, 5);
        assert_eq!(entry.path, "meta.txt");
    }

    #[test]
    fn test_inmemory_overwrite() {
        let store = make_inmemory();
        store.write("over.txt", b"original", false).unwrap();
        store.write("over.txt", b"updated", false).unwrap();
        let entries = store.list("").unwrap();
        let entry = entries.iter().find(|e| e.path == "over.txt").unwrap();
        assert_eq!(entry.size, 7);
        assert_eq!(store.read("over.txt").unwrap(), b"updated");
    }
}
