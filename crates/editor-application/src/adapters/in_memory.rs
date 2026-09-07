//! In-memory implementation of [`ProjectStore`] for testing and development.

use crate::ports::project_store::{ProjectStore, StoreEntry, StoreError};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// In-memory project store backed by a `RwLock<HashMap>`.
#[derive(Debug, Default)]
pub struct InMemoryProjectStore {
    entries: RwLock<HashMap<String, (Vec<u8>, u64)>>,
    next_modified_ms: RwLock<u64>,
}

impl InMemoryProjectStore {
    /// Create a new empty in-memory project store.
    pub fn new() -> Self {
        Self::default()
    }

    fn now_ms(&self) -> Result<u64, StoreError> {
        let mut next = self.next_modified_ms.write().map_err(lock_poisoned)?;
        let current = *next;
        *next += 1;
        Ok(current)
    }
}

/// Map a lock poisoning error to [`StoreError::LockPoisoned`].
fn lock_poisoned<T>(_: std::sync::PoisonError<RwLockWriteGuard<'_, T>>) -> StoreError {
    StoreError::LockPoisoned
}

/// Map a read-lock poisoning error to [`StoreError::LockPoisoned`].
fn lock_poisoned_read<T>(_: std::sync::PoisonError<RwLockReadGuard<'_, T>>) -> StoreError {
    StoreError::LockPoisoned
}

impl ProjectStore for InMemoryProjectStore {
    fn list(&self, prefix: &str) -> Result<Vec<StoreEntry>, StoreError> {
        let entries = self.entries.read().map_err(lock_poisoned_read)?;
        Ok(entries
            .iter()
            .filter(|(p, _)| p.starts_with(prefix))
            .map(|(p, (b, m))| StoreEntry {
                path: p.clone(),
                size: b.len() as u64,
                modified_ms: *m,
            })
            .collect())
    }

    fn read(&self, path: &str) -> Result<Vec<u8>, StoreError> {
        self.entries
            .read()
            .map_err(lock_poisoned_read)?
            .get(path)
            .map(|(b, _)| b.clone())
            .ok_or_else(|| StoreError::NotFound(path.to_string()))
    }

    fn write(&self, path: &str, bytes: &[u8], atomic: bool) -> Result<(), StoreError> {
        // Mirror of `editor_storage_web::memory_bridge::InMemoryProjectStore`:
        // the atomic flag is observable at the API surface only, since
        // in-memory writes are infallible. Real rollback semantics live in
        // `OpfsProjectStore` flush-time.
        let _ = atomic;
        let mut entries = self.entries.write().map_err(lock_poisoned)?;
        entries.insert(path.to_string(), (bytes.to_vec(), self.now_ms()?));
        Ok(())
    }

    fn delete(&self, path: &str) -> Result<(), StoreError> {
        self.entries.write().map_err(lock_poisoned)?.remove(path);
        Ok(())
    }

    fn exists(&self, path: &str) -> Result<bool, StoreError> {
        Ok(self
            .entries
            .read()
            .map_err(lock_poisoned_read)?
            .contains_key(path))
    }

    fn flush(&self) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + '_>> {
        Box::pin(async { Ok(()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::project_store::ProjectStore;

    #[test]
    fn write_then_read() {
        let s = InMemoryProjectStore::new();
        s.write("a.txt", b"hello", false).unwrap();
        assert_eq!(s.read("a.txt").unwrap(), b"hello");
    }

    #[test]
    fn list_filters_by_prefix() {
        let s = InMemoryProjectStore::new();
        s.write("a/b.txt", b"1", false).unwrap();
        s.write("a/c.txt", b"2", false).unwrap();
        s.write("d.txt", b"3", false).unwrap();
        let xs = s.list("a/").unwrap();
        assert_eq!(xs.len(), 2);
    }

    #[test]
    fn delete_removes_entry() {
        let s = InMemoryProjectStore::new();
        s.write("a.txt", b"hello", false).unwrap();
        s.delete("a.txt").unwrap();
        assert!(!s.exists("a.txt").unwrap());
    }

    #[test]
    fn exists_correctly_reports_missing() {
        let s = InMemoryProjectStore::new();
        assert!(!s.exists("nope.txt").unwrap());
    }

    #[test]
    fn read_missing_returns_not_found() {
        let s = InMemoryProjectStore::new();
        assert!(matches!(s.read("nope.txt"), Err(StoreError::NotFound(_))));
    }

    #[test]
    fn atomic_write_does_not_lose_data_on_collision() {
        let s = InMemoryProjectStore::new();
        s.write("a.txt", b"original", true).unwrap();
        assert_eq!(s.read("a.txt").unwrap(), b"original");
        s.write("a.txt", b"updated", true).unwrap();
        assert_eq!(s.read("a.txt").unwrap(), b"updated");
    }

    #[test]
    fn atomic_write_read_after_write_is_visible_immediately() {
        let s = InMemoryProjectStore::new();
        s.write("rt.txt", b"first", true).unwrap();
        assert_eq!(s.read("rt.txt").unwrap(), b"first");
        s.write("rt.txt", b"second", true).unwrap();
        assert_eq!(s.read("rt.txt").unwrap(), b"second");
        let entries = s.list("").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "rt.txt");
        assert_eq!(entries[0].size, b"second".len() as u64);
    }

    #[test]
    fn atomic_and_non_atomic_converge_on_same_mirror_state() {
        let a = InMemoryProjectStore::new();
        let b = InMemoryProjectStore::new();
        a.write("x.txt", b"payload", true).unwrap();
        b.write("x.txt", b"payload", false).unwrap();
        assert_eq!(a.read("x.txt").unwrap(), b.read("x.txt").unwrap());
    }

    #[test]
    fn write_returns_correct_entry_metadata() {
        let s = InMemoryProjectStore::new();
        s.write("a.txt", b"hello", false).unwrap();
        let xs = s.list("").unwrap();
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].path, "a.txt");
        assert_eq!(xs[0].size, 5);
    }
}
