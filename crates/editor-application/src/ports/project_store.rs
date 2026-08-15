//! ProjectStore port — the file-system abstraction for the editor application.
//!
//! See ADR-0033 (port with OPFS and filesystem adapters) and ADR-0048 (sync v1).

use serde::{Deserialize, Serialize};

/// A single entry in the project store (file + metadata).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreEntry {
    /// The file path.
    pub path: String,
    /// File size in bytes.
    pub size: u64,
    /// Last modified timestamp in milliseconds since epoch.
    pub modified_ms: u64,
}

/// Errors that can occur when operating on the project store.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Entry not found.
    #[error("entry not found: {0}")]
    NotFound(String),
    /// IO error.
    #[error("io error: {0}")]
    Io(String),
    /// Atomic write failed and was rolled back.
    #[error("atomic write failed and was rolled back: {0}")]
    AtomicRollback(String),
    /// A lock was poisoned by a panicked thread.
    #[error("lock poisoned")]
    LockPoisoned,
}

/// ProjectStore — the file-system abstraction for the editor application.
///
/// In v1 this is SYNCHRONOUS per ADR-0048. Async migration deferred to v0.88.
pub trait ProjectStore: Send + Sync {
    /// List all entries under the given prefix.
    fn list(&self, prefix: &str) -> Result<Vec<StoreEntry>, StoreError>;

    /// Read the full contents of a file.
    fn read(&self, path: &str) -> Result<Vec<u8>, StoreError>;

    /// Write contents to a file.
    ///
    /// If `atomic` is true, the write should be atomic (all-or-nothing).
    fn write(&self, path: &str, bytes: &[u8], atomic: bool) -> Result<(), StoreError>;

    /// Delete a file.
    fn delete(&self, path: &str) -> Result<(), StoreError>;

    /// Check if a file exists.
    fn exists(&self, path: &str) -> Result<bool, StoreError>;
}
