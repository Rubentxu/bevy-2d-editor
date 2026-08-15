//! Explicit application-level owner of mutable editing state (ADR-0031).
//!
//! # ADR-0031 Rules Honored
//!
//! - **Caches have named owners and invalidation methods**: see [`CacheEntry`].
//! - **Active document selection is session state**: see [`DocumentSelection`],
//!   [`EditorSession::activate_document`], [`EditorSession::deactivate_document`].
//! - **Operation histories are scoped explicitly**: see [`HistoryScope`],
//!   [`EditorSession::history_scope_mut`]. History scopes survive deselection.
//! - **Test code creates isolated sessions**: each test constructs its own
//!   [`EditorSession`] with its own [`InMemoryProjectStore`](crate::adapters::InMemoryProjectStore)
//!   + [`FakeClock`](editor_model::time::FakeClock).

use editor_model::time::{Clock, Timestamp};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::ports::project_store::ProjectStore;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Active document selection.
///
/// The selected document path IS session state (ADR-0031 rule: "active document
/// selection is part of session state"). Constructed by
/// [`EditorSession::activate_document`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSelection {
    /// Logical path of the selected document.
    path: String,
    /// Timestamp when the document was activated.
    activated_at: Timestamp,
}

impl DocumentSelection {
    /// Construct a new selection for the given path.
    fn new(path: String, activated_at: Timestamp) -> Self {
        Self { path, activated_at }
    }

    /// Returns the selected document path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the timestamp when this document was activated.
    pub fn activated_at(&self) -> Timestamp {
        self.activated_at
    }
}

// ---------------------------------------------------------------------------

/// Minimal explicit per-document history scope.
///
/// In v1 this holds only a revision counter. PR E (TransactionKernel) will
/// attach the real history container. The type exists so that history ownership
/// is session-scoped from day one (ADR-0031 rule: "operation histories are
/// scoped explicitly").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryScope {
    revision: u64,
}

impl HistoryScope {
    /// Returns the current revision number.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the next revision number and increments the stored value.
    pub fn next_revision(&mut self) -> u64 {
        let next = self.revision + 1;
        self.revision = next;
        next
    }
}

// ---------------------------------------------------------------------------

/// A named cache entry with owner tracking and generation-based invalidation.
///
/// ADR-0031 rule: "caches have named owners and invalidation methods".
/// The `owner` field identifies which component or service created the cache;
/// `generation` is bumped on every [`CacheEntry::invalidate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    owner: String,
    generation: u64,
}

impl CacheEntry {
    /// Construct a new cache entry with generation 0.
    fn new(owner: String) -> Self {
        Self {
            owner,
            generation: 0,
        }
    }

    /// Returns the owner identifier of this cache.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Returns the current generation number.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Bump the generation to the next number, signalling cache invalidation.
    pub fn invalidate(&mut self) {
        self.generation += 1;
    }
}

// ---------------------------------------------------------------------------
// EditorSession
// ---------------------------------------------------------------------------

/// Explicit application-level owner of mutable editing state.
///
/// Replaces the 14+ scattered `thread_local!` stores that currently hold mutable
/// editing state throughout `editor-core`. See ADR-0031.
///
/// The WASM composition root holds exactly one `EditorSession` — it is not
/// shared, not cloned, and not registered in any global store.
///
/// # Example
///
/// ```ignore
/// use editor_application::session::EditorSession;
/// use editor_application::adapters::InMemoryProjectStore;
/// use editor_model::time::FakeClock;
///
/// let store = Arc::new(InMemoryProjectStore::new());
/// let clock = Arc::new(FakeClock::new());
/// let mut session = EditorSession::new(store.clone(), clock.clone());
///
/// session.activate_document("my-scene.json");
/// assert!(session.active_document().is_some());
/// ```
///
/// # Invariants
///
/// - `store` and `clock` are always present — they are supplied at construction
///   and the session owns them for its lifetime.
/// - `active_document` is `None` until a document is explicitly activated.
/// - `history_scopes` entries are keyed by logical document path and survive
///   calls to [`deactivate_document`](EditorSession::deactivate_document).
pub struct EditorSession {
    store: Arc<dyn ProjectStore>,
    clock: Arc<dyn Clock>,
    active_document: Option<DocumentSelection>,
    /// Keyed by logical document path (ADR-0031 rule: "operation histories are
    /// scoped explicitly").
    history_scopes: BTreeMap<String, HistoryScope>,
    /// Named caches with owner tracking (ADR-0031 rule: "caches have named
    /// owners and invalidation methods").
    caches: BTreeMap<String, CacheEntry>,
}

impl std::fmt::Debug for EditorSession {
    /// Prints structural info only — dyn trait contents are not printable.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorSession")
            .field("active_document", &self.active_document)
            .field("history_scopes", &self.history_scopes)
            .field("caches", &self.caches)
            .finish()
    }
}

impl EditorSession {
    /// Construct a new session with the given store and clock.
    ///
    /// The session starts with no active document and no history scopes.
    pub fn new(store: Arc<dyn ProjectStore>, clock: Arc<dyn Clock>) -> Self {
        Self {
            store,
            clock,
            active_document: None,
            history_scopes: BTreeMap::new(),
            caches: BTreeMap::new(),
        }
    }

    /// Returns a reference to the project store.
    pub fn store(&self) -> &dyn ProjectStore {
        &*self.store
    }

    /// Returns a reference to the clock.
    pub fn clock(&self) -> &dyn Clock {
        &*self.clock
    }

    /// Returns the current timestamp from the session's clock.
    pub fn now(&self) -> Timestamp {
        self.clock.now()
    }

    /// Activate the document at the given path.
    ///
    /// Sets `active_document` with the current clock value. Creates the
    /// [`HistoryScope`] for this path if it does not already exist.
    /// Re-activating the same path updates `activated_at` but does **not**
    /// reset the revision counter or destroy the history scope.
    pub fn activate_document(&mut self, path: impl Into<String>) {
        let path = path.into();
        let now = self.clock.now();

        // Create history scope if absent (idempotent).
        self.history_scopes
            .entry(path.clone())
            .or_insert_with(|| HistoryScope { revision: 0 });

        self.active_document = Some(DocumentSelection::new(path, now));
    }

    /// Returns the currently active document selection, if any.
    pub fn active_document(&self) -> Option<&DocumentSelection> {
        self.active_document.as_ref()
    }

    /// Deactivate the current document, clearing the active selection.
    ///
    /// History scopes are **not** destroyed — undo/redo history survives
    /// deselection (ADR-0031 rule: "operation histories are scoped
    /// explicitly").
    pub fn deactivate_document(&mut self) {
        self.active_document = None;
    }

    /// Returns a mutable reference to the history scope for the given path.
    ///
    /// Returns `None` if no history scope exists for this path (no scope is
    /// created by this method — use [`activate_document`](EditorSession::activate_document)
    /// first).
    pub fn history_scope_mut(&mut self, path: &str) -> Option<&mut HistoryScope> {
        self.history_scopes.get_mut(path)
    }

    /// Register a named cache, creating it with generation 0 if absent.
    ///
    /// Registration is idempotent: if the cache already exists this is a no-op.
    pub fn register_cache(&mut self, name: impl Into<String>, owner: impl Into<String>) {
        let name = name.into();
        let owner = owner.into();
        self.caches
            .entry(name)
            .or_insert_with(|| CacheEntry::new(owner));
    }

    /// Invalidate the named cache, bumping its generation.
    ///
    /// Returns `true` if the cache existed and was invalidated; returns
    /// `false` if the cache name was not registered.
    pub fn invalidate_cache(&mut self, name: &str) -> bool {
        match self.caches.get_mut(name) {
            Some(entry) => {
                entry.invalidate();
                true
            }
            None => false,
        }
    }

    /// Returns the current generation of the named cache.
    ///
    /// Returns `None` if the cache is not registered.
    pub fn cache_generation(&self, name: &str) -> Option<u64> {
        self.caches.get(name).map(|e| e.generation())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::InMemoryProjectStore;
    use editor_model::time::FakeClock;

    // -------------------------------------------------------------------------
    // Helper
    // -------------------------------------------------------------------------
    fn make_session() -> (EditorSession, Arc<InMemoryProjectStore>, Arc<FakeClock>) {
        let store = Arc::new(InMemoryProjectStore::new());
        let clock = Arc::new(FakeClock::new());
        let session = EditorSession::new(store.clone(), clock.clone());
        (session, store, clock)
    }

    // -------------------------------------------------------------------------
    // Test 1: new wires store+clock; now() returns FakeClock-injected value
    // -------------------------------------------------------------------------
    #[test]
    fn test_new_wires_store_and_clock() {
        let (session, _store, clock) = make_session();
        clock.set(Timestamp(1_700_000_000_000_u64));

        // clock is wired through now()
        assert_eq!(session.now().0, 1_700_000_000_000_u64);
        assert_eq!(session.now().into_u64(), 1_700_000_000_000_u64);

        // store is accessible — confirm it returns false for non-existent file
        assert!(!session.store().exists("nonexistent").unwrap());
    }

    // -------------------------------------------------------------------------
    // Test 2: activate_document sets selection with clock value and creates scope
    // -------------------------------------------------------------------------
    #[test]
    fn test_activate_document_sets_selection_and_scope() {
        let (mut session, _store, clock) = make_session();
        clock.set(Timestamp(1_700_000_000_000_u64));

        session.activate_document("scene.json");

        let sel = session
            .active_document()
            .expect("document should be active");
        assert_eq!(sel.path(), "scene.json");
        assert_eq!(sel.activated_at().0, 1_700_000_000_000_u64);

        // History scope was created
        let scope = session
            .history_scope_mut("scene.json")
            .expect("scope should exist");
        assert_eq!(scope.revision(), 0);
    }

    // -------------------------------------------------------------------------
    // Test 3: re-activating same path updates activated_at but not revision
    // -------------------------------------------------------------------------
    #[test]
    fn test_reactivate_same_path_preserves_history_scope() {
        let (mut session, _store, clock) = make_session();
        clock.set(Timestamp(1_700_000_000_000_u64));

        session.activate_document("scene.json");
        // Advance the clock
        clock.set(Timestamp(1_700_000_000_100_u64));
        // Re-activate the same path
        session.activate_document("scene.json");

        let sel = session
            .active_document()
            .expect("document should still be active");
        assert_eq!(sel.path(), "scene.json");
        assert_eq!(sel.activated_at().0, 1_700_000_000_100_u64);

        // Revision is still 0 — not reset
        let scope = session
            .history_scope_mut("scene.json")
            .expect("scope should still exist");
        assert_eq!(scope.revision(), 0);
    }

    // -------------------------------------------------------------------------
    // Test 4: deactivate_document clears selection; scope survives
    // -------------------------------------------------------------------------
    #[test]
    fn test_deactivate_clears_selection_scope_survives() {
        let (mut session, _store, _clock) = make_session();
        session.activate_document("scene.json");

        // Bump revision
        {
            let scope = session.history_scope_mut("scene.json").unwrap();
            scope.next_revision();
            assert_eq!(scope.revision(), 1);
        }

        session.deactivate_document();

        assert!(session.active_document().is_none());
        // Scope still present
        let scope = session
            .history_scope_mut("scene.json")
            .expect("scope should survive deselection");
        assert_eq!(scope.revision(), 1);
    }

    // -------------------------------------------------------------------------
    // Test 5: two isolated sessions do not share state
    // -------------------------------------------------------------------------
    #[test]
    fn test_isolated_sessions_are_independent() {
        let (mut s1, store1, _clock1) = make_session();
        let (mut s2, _store2, _clock2) = make_session();

        s1.activate_document("doc-a.json");
        s2.activate_document("doc-b.json");

        // Each session has its own active document
        assert_eq!(s1.active_document().unwrap().path(), "doc-a.json");
        assert_eq!(s2.active_document().unwrap().path(), "doc-b.json");

        // Store is also independent (each got its own InMemoryProjectStore)
        store1
            .write("独占", b"data", false)
            .expect("write should succeed");
        assert!(s1.store().exists("独占").unwrap());
        assert!(!s2.store().exists("独占").unwrap());
    }

    // -------------------------------------------------------------------------
    // Test 6: cache register -> generation 0 -> invalidate bumps to 1 -> unknown returns false
    // -------------------------------------------------------------------------
    #[test]
    fn test_cache_registration_and_invalidation() {
        let (mut session, _store, _clock) = make_session();

        // Unknown cache returns false
        assert!(!session.invalidate_cache("unknown"));
        assert!(session.cache_generation("unknown").is_none());

        // Register
        session.register_cache("render-cache", "Renderer");

        // Initial generation is 0
        assert_eq!(session.cache_generation("render-cache").unwrap(), 0);

        // First invalidate bumps to 1
        assert!(session.invalidate_cache("render-cache"));
        assert_eq!(session.cache_generation("render-cache").unwrap(), 1);

        // Second invalidate bumps to 2
        assert!(session.invalidate_cache("render-cache"));
        assert_eq!(session.cache_generation("render-cache").unwrap(), 2);

        // Re-register is idempotent — does not reset generation
        session.register_cache("render-cache", "Renderer");
        assert_eq!(session.cache_generation("render-cache").unwrap(), 2);

        // Invalidate of unknown still returns false
        assert!(!session.invalidate_cache("completely-unknown"));
    }

    // -------------------------------------------------------------------------
    // Test 7: store accessor roundtrip — write via session.store(), read back
    // -------------------------------------------------------------------------
    #[test]
    fn test_store_accessor_roundtrip() {
        let (session, _store, _clock) = make_session();

        session
            .store()
            .write("roundtrip/test.txt", b"hello world", false)
            .expect("write should succeed");

        let bytes = session
            .store()
            .read("roundtrip/test.txt")
            .expect("read should succeed");
        assert_eq!(bytes, b"hello world");
    }
}
