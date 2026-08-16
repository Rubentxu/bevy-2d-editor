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
//! - **ChangeWorkbench pending ChangeSets are session state**: see
//!   [`EditorSession::pending_change_sets_mut`].

use editor_model::time::{Clock, Timestamp};
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use crate::RuntimeDelta;
use crate::ports::project_store::ProjectStore;
use editor_model::PendingChangeSet;

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
// HistoryScope — imported from editor_model::session (the model layer)
// ---------------------------------------------------------------------------

pub use editor_model::session::HistoryScope;

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
// Sub-state types (ADR-0031 — PR2a consolidation)
// ---------------------------------------------------------------------------
//
// PR2a progress: SceneSessionState.document now uses real SceneDocument
// from editor_model. The operation log is kept as Value (OperationLog
// lives in editor-core — full migration to editor_model is future work).
//
// RecentChangeSetsBuffer uses ChangeSetSummary from editor_application::transaction.

use crate::transaction::ChangeSetSummary;
use editor_model::document::SceneDocument;
use editor_model::logic_graph::LogicGraphAsset;
use editor_model::scene_asset::SceneAssetDocument;
use serde_json::Value;

/// Session state for one active scene document (PR2a real types).
///
/// `document` uses the real `SceneDocument` from editor_model.
/// `log` is kept as serialized Value since OperationLog lives in editor-core
/// (future migration to editor_model is tracked separately).
#[derive(Debug, Clone, Default)]
pub struct SceneSessionState {
    /// The active scene document (None when not loaded).
    pub document: Option<SceneDocument>,
    /// Serialised operation log (None when not loaded).
    /// TODO: Replace with real OperationLog after OperationLog moves to editor_model.
    pub log: Option<Value>,
}

/// Session state for the scene asset subsystem (PR2a).
///
/// Uses real types from editor_model where available.
#[derive(Debug, Clone, Default)]
pub struct AssetSessionState {
    /// Active asset document being edited.
    pub active_document: Option<SceneAssetDocument>,
    /// Cached asset bodies (asset_ref → serialized SceneAssetDocument body).
    /// TODO: Replace with real cache after catalog moves to editor_model.
    pub body_cache: BTreeMap<String, SceneAssetDocument>,
    /// Resync reports indexed by stable ID (key is StableId string).
    pub resync_reports: Vec<(String, Value)>,
    /// Validation issues for this asset scope.
    pub validation_issues: Vec<Value>,
}

/// Session state for the logic graph subsystem (PR2a).
///
/// Uses real types from editor_model where available.
#[derive(Debug, Clone, Default)]
pub struct LogicSessionState {
    /// Active logic graph being edited.
    pub active_graph: Option<LogicGraphAsset>,
    /// Logic graph catalog (path → serialized catalog).
    /// TODO: Replace with real catalog after catalog moves to editor_model.
    pub catalog: BTreeMap<String, Value>,
}

/// Session state for the runtime preview inspector (PR2a).
#[derive(Debug, Clone, Default)]
pub struct PreviewInspectorState {
    /// Live runtime metrics (FPS, frame time, rebuild count).
    pub metrics: Value,
    /// Per-instance runtime-to-editor ID mapping.
    pub mapping: Vec<Value>,
    /// Per-StableId provenance records from play mode.
    pub provenance: BTreeMap<String, Value>,
}

/// In-memory cache for source file contents.
#[derive(Debug, Clone, Default)]
pub struct SourceFilesCache {
    /// File path → file content.
    pub files: BTreeMap<String, String>,
}

/// Recent change-set summary buffer (capped at 50 entries per scene path).
///
/// Populated by polling `OperationLog::recent_change_sets_for` per scene path.
/// The UI uses this for the Change Workbench history view.
#[derive(Debug, Clone)]
pub struct RecentChangeSetsBuffer {
    entries: VecDeque<ChangeSetSummary>,
    capacity: usize,
}

impl RecentChangeSetsBuffer {
    /// Construct a new buffer with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            capacity,
        }
    }

    /// Push a new entry, evicting the oldest if over capacity.
    pub fn push(&mut self, summary: ChangeSetSummary) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(summary);
    }

    /// Drain entries and replace with a new set of summaries (used for rebuild).
    pub fn refresh(&mut self, summaries: Vec<ChangeSetSummary>) {
        self.entries.clear();
        for summary in summaries {
            self.push(summary);
        }
    }

    /// Returns a copy of all entries (most recent first).
    pub fn entries(&self) -> Vec<ChangeSetSummary> {
        self.entries.iter().cloned().collect()
    }

    /// Number of entries currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for RecentChangeSetsBuffer {
    fn default() -> Self {
        Self::new(50)
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
    // ─── Sub-state maps (PR2a — ADR-0031 consolidation) ───────────────────────
    /// Per-scene session state: document + operation log (SCENE_DOC + OPERATION_LOG).
    scene_states: BTreeMap<String, SceneSessionState>,
    /// Per-asset-path session state (SCENE_ASSET_CATALOG etc.).
    asset_states: BTreeMap<String, AssetSessionState>,
    /// Per-logic-graph-path session state (LOGIC_GRAPH_DOC etc.).
    logic_states: BTreeMap<String, LogicSessionState>,
    /// Runtime preview inspector state (PREVIEW_METRICS etc.).
    preview_inspector: PreviewInspectorState,
    /// Source file contents cache (SOURCE_FILE_REGISTRY).
    source_files: SourceFilesCache,
    /// Recent change-set summaries per scene path (capped at 50 per scene).
    recent_change_sets: BTreeMap<String, RecentChangeSetsBuffer>,
    /// Runtime delta buffer for play-mode apply-back (capped at 64).
    runtime_delta_buffer: VecDeque<RuntimeDelta>,
    /// Pending ChangeSets awaiting user approval in the ChangeWorkbench (ADR-0039).
    /// Key = change-set ID (e.g. "agent:12345" or "cmd:1234567890").
    pending_change_sets: BTreeMap<String, PendingChangeSet>,
}

impl std::fmt::Debug for EditorSession {
    /// Prints structural info only — dyn trait contents are not printable.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorSession")
            .field("active_document", &self.active_document)
            .field("history_scopes", &self.history_scopes)
            .field("caches", &self.caches)
            .field("scene_states", &self.scene_states)
            .field("asset_states", &self.asset_states)
            .field("logic_states", &self.logic_states)
            .field("preview_inspector", &self.preview_inspector)
            .field("source_files", &self.source_files)
            .field("recent_change_sets", &self.recent_change_sets)
            .field("runtime_delta_buffer_len", &self.runtime_delta_buffer.len())
            .field("pending_change_sets_len", &self.pending_change_sets.len())
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
            // Sub-state maps (PR2a — initialized empty; populated on first access)
            scene_states: BTreeMap::new(),
            asset_states: BTreeMap::new(),
            logic_states: BTreeMap::new(),
            preview_inspector: PreviewInspectorState::default(),
            source_files: SourceFilesCache::default(),
            recent_change_sets: BTreeMap::new(),
            runtime_delta_buffer: VecDeque::with_capacity(64),
            pending_change_sets: BTreeMap::new(),
        }
    }

    // ─── Sub-state accessors (PR2a) ──────────────────────────────────────────

    /// Returns a mutable reference to the scene session state for the given path,
    /// creating it if absent (idempotent).
    ///
    /// This is the primary entry point for migrating `scene_session.rs` to use
    /// `EditorSession` as the owning store instead of `thread_local!`.
    pub fn scene_state_mut(&mut self, path: &str) -> &mut SceneSessionState {
        self.scene_states
            .entry(path.to_string())
            .or_insert_with(SceneSessionState::default)
    }

    /// Returns a mutable reference to the asset session state for the given path,
    /// creating it if absent.
    pub fn asset_state_mut(&mut self, path: &str) -> &mut AssetSessionState {
        self.asset_states
            .entry(path.to_string())
            .or_insert_with(AssetSessionState::default)
    }

    /// Returns a mutable reference to the logic session state for the given path,
    /// creating it if absent.
    pub fn logic_state_mut(&mut self, path: &str) -> &mut LogicSessionState {
        self.logic_states
            .entry(path.to_string())
            .or_insert_with(LogicSessionState::default)
    }

    /// Returns the recent change-set summaries for the given scene path.
    ///
    /// Returns an empty buffer if no entries have been recorded for this path.
    pub fn recent_change_sets_for(&self, scene_path: &str) -> Vec<ChangeSetSummary> {
        self.recent_change_sets
            .get(scene_path)
            .map(|b| b.entries())
            .unwrap_or_default()
    }

    /// Returns a reference to the runtime delta buffer.
    pub fn runtime_delta_buffer(&self) -> &VecDeque<RuntimeDelta> {
        &self.runtime_delta_buffer
    }

    /// Returns a mutable reference to the runtime delta buffer.
    pub fn runtime_delta_buffer_mut(&mut self) -> &mut VecDeque<RuntimeDelta> {
        // Enforce 64-entry cap
        while self.runtime_delta_buffer.len() > 64 {
            self.runtime_delta_buffer.pop_front();
        }
        &mut self.runtime_delta_buffer
    }

    /// Returns a reference to the source files cache.
    pub fn source_files(&self) -> &SourceFilesCache {
        &self.source_files
    }

    /// Returns a mutable reference to the source files cache.
    pub fn source_files_mut(&mut self) -> &mut SourceFilesCache {
        &mut self.source_files
    }

    /// Returns a reference to the preview inspector state.
    pub fn preview_inspector(&self) -> &PreviewInspectorState {
        &self.preview_inspector
    }

    /// Returns a mutable reference to the preview inspector state.
    pub fn preview_inspector_mut(&mut self) -> &mut PreviewInspectorState {
        &mut self.preview_inspector
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
            .or_insert_with(|| HistoryScope::new());

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

    // ─── ChangeWorkbench pending ChangeSets (ADR-0039) ─────────────────────────

    /// Returns a mutable reference to the pending ChangeSets map.
    ///
    /// ADR-0031 rule: workbench UI state lives in the sanctioned composition root
    /// (`EditorSession`), not in domain modules.
    pub fn pending_change_sets_mut(&mut self) -> &mut BTreeMap<String, PendingChangeSet> {
        &mut self.pending_change_sets
    }

    /// Returns a reference to the pending ChangeSets map.
    pub fn pending_change_sets(&self) -> &BTreeMap<String, PendingChangeSet> {
        &self.pending_change_sets
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
