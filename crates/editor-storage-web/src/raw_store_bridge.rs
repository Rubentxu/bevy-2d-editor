//! Low-level raw store operations that can be implemented by OPFS or memory.

use editor_model::ports::StoreError;
use std::collections::BTreeMap;
use std::sync::RwLock;

// ─────────────────────────────────────────────────────────────────────────────
// RawStoreBridge trait
// ─────────────────────────────────────────────────────────────────────────────

/// Async I/O operations that the [`OpfsCore`](super::opfs_core::OpfsCore) mirror delegates to.
///
/// On wasm32 this is implemented by [`wasm_bridge`] using
/// `js_sys::Promise` returns; on native / tests by a [`MemoryBridge`] fake.
pub trait RawStoreBridge: Send + Sync {
    /// List all file paths under `dir`. Returns the bare file names (not full paths).
    fn list(&self, dir: &str) -> Result<Vec<String>, String>;

    /// Read the full contents of a file.
    fn read(&self, path: &str) -> Result<Vec<u8>, String>;

    /// Write contents to a file.
    fn write(&self, path: &str, bytes: &[u8]) -> Result<(), String>;

    /// Delete a file.
    fn delete(&self, path: &str) -> Result<(), String>;
}

// ─────────────────────────────────────────────────────────────────────────────
// MemoryBridge — in-memory test fake
// ─────────────────────────────────────────────────────────────────────────────

/// A raw-store bridge backed by an in-memory `BTreeMap` — for unit tests.
#[derive(Debug, Default)]
pub struct MemoryBridge {
    entries: RwLock<BTreeMap<String, Vec<u8>>>,
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

// ─────────────────────────────────────────────────────────────────────────────
// NoOpBridge — wasm32 placeholder
// ─────────────────────────────────────────────────────────────────────────────

/// A no-op bridge used as a placeholder on wasm32 where
/// `hydrate` / `flush` call `wasm_bridge::list_op` / `wasm_bridge::read_op` /
/// `wasm_bridge::flush_op` directly instead of going through the bridge.
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
