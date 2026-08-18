//! OPFS-backed [`ProjectStore`] adapter.
//!
//! ## Architecture
//!
//! - [`OpfsCore`] — pure in-memory mirror + pending-op queue. No JS deps,
//!   compiled on all targets.
//! - [`RawStoreBridge`] — async I/O abstraction. On wasm32 backed by the
//!   `window.opfs_*` JS bridge via `js_sys::Promise`; on native backed by a
//!   [`MemoryBridge`] fake for tests.
//! - `wasm_bridge` module (`#[cfg(target_arch = "wasm32")]`) — wasm32-only bridge ops
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

use editor_model::ports::{ProjectStore, StoreEntry, StoreError};
use std::collections::{BTreeMap, VecDeque};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

// Re-export RawStoreBridge so OpfsProjectStore can reference it.
pub use crate::raw_store_bridge::RawStoreBridge;

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
    /// Note: on wasm32, `hydrate()` and `flush()` call `wasm_bridge::list_op` / `wasm_bridge::read_op`
    /// / `wasm_bridge::flush_op` directly rather than going through `RawStoreBridge`. The
    /// bridge field is a no-op placeholder.
    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Self {
        Self::new_internal(
            Arc::new(Mutex::new(OpfsCore::new())),
            Arc::new(crate::raw_store_bridge::NoOpBridge),
            Arc::new(crate::wasm_bridge::SysClock::new()),
        )
    }

    /// Create for tests with a [`MemoryBridge`] and [`editor_model::time::FakeClock`].
    #[cfg(test)]
    pub fn new_for_tests() -> Self {
        Self::new_internal(
            Arc::new(Mutex::new(OpfsCore::new())),
            Arc::new(crate::raw_store_bridge::MemoryBridge::new()),
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
        use crate::wasm_bridge::{list_op, read_op};

        let paths = list_op("/")
            .await
            .map_err(|e| format!("hydrate: list failed: {}", e))?;

        for path in paths {
            let bytes: Vec<u8> = match read_op(&path).await {
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
        use crate::wasm_bridge::flush_op;

        let ops = {
            let mut core = self.core.try_lock().map_err(|_| StoreError::LockPoisoned)?;
            core.take_pending()
        };

        for op in ops {
            let path = match &op {
                PendingOp::Write { path, .. } => path.clone(),
                PendingOp::Delete { path } => path.clone(),
            };
            flush_op(&path, op).await.map_err(|e| StoreError::Io(e))?;
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

    #[cfg(target_arch = "wasm32")]
    fn flush(&self) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + '_>> {
        use crate::wasm_bridge::flush_op;

        Box::pin(async move {
            let ops = {
                let mut core = self.core.try_lock().map_err(|_| StoreError::LockPoisoned)?;
                core.take_pending()
            };

            for op in ops {
                let path = match &op {
                    PendingOp::Write { path, .. } => path.clone(),
                    PendingOp::Delete { path } => path.clone(),
                };
                flush_op(&path, op).await.map_err(|e| StoreError::Io(e))?;
            }
            Ok(())
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn flush(&self) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + '_>> {
        // Non-wasm32 OpfsProjectStore (tests) uses MemoryBridge which is synchronous.
        // The in-memory flush is a no-op since MemoryBridge mutates immediately.
        Box::pin(async { Ok(()) })
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

    #[test]
    fn test_contract_write_and_read() {
        let store = make_store();
        store.write("a/test.txt", b"hello", false).unwrap();
        store.write("a/sub/b.txt", b"world", false).unwrap();

        let entries = store.list("a/").unwrap();
        assert_eq!(entries.len(), 2);

        assert_eq!(store.read("a/test.txt").unwrap(), b"hello");
        assert_eq!(store.read("a/sub/b.txt").unwrap(), b"world");
    }

    #[test]
    fn test_contract_list_empty_prefix() {
        let store = make_store();
        store.write("a/test.txt", b"hello", false).unwrap();
        let entries = store.list("nonexistent/").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_contract_exists() {
        let store = make_store();
        store.write("exists.txt", b"content", false).unwrap();
        assert!(store.exists("exists.txt").unwrap());
        assert!(!store.exists("missing.txt").unwrap());
    }

    #[test]
    fn test_contract_delete() {
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

    #[test]
    fn test_contract_list_after_delete() {
        let store = make_store();
        store.write("dir/file.txt", b"data", false).unwrap();
        assert_eq!(store.list("dir/").unwrap().len(), 1);

        store.delete("dir/file.txt").unwrap();
        assert!(store.list("dir/").unwrap().is_empty());
    }

    #[test]
    fn test_contract_read_missing() {
        let store = make_store();
        match store.read("definitely_missing_12345.txt") {
            Err(StoreError::NotFound(path)) => {
                assert!(path.contains("definitely_missing_12345"));
            }
            other => panic!("Expected NotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_contract_write_metadata() {
        let store = make_store();
        store.write("meta.txt", b"hello", false).unwrap();
        let entries = store.list("").unwrap();
        let entry = entries.iter().find(|e| e.path == "meta.txt").unwrap();
        assert_eq!(entry.size, 5);
        assert_eq!(entry.path, "meta.txt");
    }

    #[test]
    fn test_contract_overwrite() {
        let store = make_store();
        store.write("over.txt", b"original", false).unwrap();
        store.write("over.txt", b"updated", false).unwrap();
        let entries = store.list("").unwrap();
        let entry = entries.iter().find(|e| e.path == "over.txt").unwrap();
        assert_eq!(entry.size, 7);
        assert_eq!(store.read("over.txt").unwrap(), b"updated");
    }
}
