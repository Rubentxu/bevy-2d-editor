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
use std::sync::{Arc, Mutex};

use crate::RuntimeDelta;
use crate::ports::project_store::ProjectStore;
use crate::extension::ExtensionRegistry;
use crate::importer_registry::ImporterRegistry;
use editor_model::CausalityEdge;
use editor_model::EditorSessionPort;
use editor_model::PendingChangeSet;
use editor_model::RebuildCause;
use editor_model::StableId;
use editor_model::logic_activation::{LogicActivationEvent, LogicActivationRing, ring_push};
use editor_model::ports::ExtensionRegistryPort;
use editor_model::ports::ImporterRegistryPort;

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
// PR2a progress: LocalSceneSessionState.document now uses real SceneDocument
// from editor_model. The operation log is kept as Value (OperationLog
// lives in editor-core — full migration to editor_model is future work).
//
// RecentChangeSetsBuffer uses ChangeSetSummary from editor_application::transaction.

use crate::transaction::ChangeSetSummary;

/// Session state for one active scene document (PR2a real types).
///
/// v0.90 PR4: this type is now defined in `editor_model::session` (single
/// source of truth). The local struct is removed; any downstream consumer
/// that used the old field names must be updated to the new ones
/// (`scene_doc`, `reload_count` instead of `document`, `log`).
pub type LocalSceneSessionState = editor_model::SceneSessionState;

/// Session state for the scene asset subsystem (PR2a).
///
/// v0.90 PR4: type now defined in `editor_model::session`. Old field names
/// (`active_document`, `body_cache`, `resync_reports`, `validation_issues`)
/// are replaced by `asset_bodies`.
pub type LocalAssetSessionState = editor_model::AssetSessionState;

/// Session state for the logic graph subsystem (PR2a).
///
/// v0.90 PR4: type now defined in `editor_model::session`. Old field names
/// (`active_graph`, `catalog`) are replaced by `graph_docs`.
pub type LocalLogicSessionState = editor_model::LogicSessionState;

/// Recent change-set summary buffer (capped at 50 entries per scene path).
///
/// Populated by polling `OperationLog::recent_change_sets_for` per scene path.
/// The UI uses this for the Change Workbench history view.
#[derive(Debug, Clone)]
pub struct RecentChangeSetsBuffer {
    entries: VecDeque<ChangeSetSummary>,
    capacity: usize,
    /// Cursor: the highest `change_id` already pushed to the buffer.
    /// New entries with `change_id <= last_seen_change_id` are skipped (dedup).
    last_seen_change_id: Option<u64>,
}

impl RecentChangeSetsBuffer {
    /// Construct a new buffer with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            capacity,
            last_seen_change_id: None,
        }
    }

    /// Push a new entry, evicting the oldest if over capacity.
    /// Skips entries whose `change_id` is at or below `last_seen_change_id`.
    pub fn push(&mut self, summary: ChangeSetSummary) {
        let change_id = summary.change_id;
        if let Some(cutoff) = self.last_seen_change_id {
            if change_id <= cutoff {
                return; // dedup — already seen this entry
            }
        }
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(summary);
        self.last_seen_change_id = Some(change_id);
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
// Sub-state structs (ADR-0031 — MEDIUM-5 god-class split)
// ---------------------------------------------------------------------------
//
// Split from EditorSession to reduce its 16-field / 43-method surface area.
// Each sub-struct groups fields that share a domain theme; EditorSession
// holds them as public fields so call sites access session.preview.xxx directly.
//
// | Sub-struct              | Fields grouped                        |
// |-------------------------|---------------------------------------|
// | PreviewSessionState     | preview_inspector + source_files      |
// | ChangeSetsSessionState  | recent_change_sets + pending_*       |
// | RuntimeSessionState     | runtime_delta_buffer + tunable_baselines |

/// Session state for the runtime preview inspector and source-file cache.
///
/// Groups the preview inspector (live runtime data) with the source-file
/// contents cache, both of which are read frequently during play-mode.
#[derive(Debug, Clone, Default)]
pub struct PreviewSessionState {
    /// Live runtime metrics, ID mapping, and provenance.
    pub preview_inspector: editor_model::PreviewInspectorState,
    /// In-memory contents of source files (keyed by project-relative path).
    pub source_files: editor_model::SourceFilesCache,
}

impl PreviewSessionState {
    /// Construct a default preview state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the preview inspector state.
    pub fn preview_inspector(&self) -> &editor_model::PreviewInspectorState {
        &self.preview_inspector
    }

    /// Returns a mutable reference to the preview inspector state.
    pub fn preview_inspector_mut(&mut self) -> &mut editor_model::PreviewInspectorState {
        &mut self.preview_inspector
    }

    /// Returns a reference to the source files cache.
    pub fn source_files(&self) -> &editor_model::SourceFilesCache {
        &self.source_files
    }

    /// Returns a mutable reference to the source files cache.
    pub fn source_files_mut(&mut self) -> &mut editor_model::SourceFilesCache {
        &mut self.source_files
    }

    /// Returns the last rebuild cause recorded by §6.
    pub fn last_rebuild_cause(&self) -> Option<&RebuildCause> {
        self.preview_inspector.last_rebuild_cause.as_ref()
    }

    /// Records a rebuild cause (§6).
    pub fn set_last_rebuild_cause(&mut self, cause: RebuildCause) {
        self.preview_inspector.last_rebuild_cause = Some(cause);
    }
}

// ---------------------------------------------------------------------------
// ChangeSetsSessionState — recent change sets + pending causality + pending changes
// ---------------------------------------------------------------------------

/// Session state for ChangeWorkbench and causality tracking.
///
/// Groups three related concerns: per-scene recent-change-set buffers,
/// pending causality edges from play-mode, and pending ChangeSets awaiting
/// user approval in the ChangeWorkbench (ADR-0039).
#[derive(Debug, Clone, Default)]
pub struct ChangeSetsSessionState {
    /// Recent change-set summaries per scene path (capped at 50 per scene).
    pub recent_change_sets: BTreeMap<String, RecentChangeSetsBuffer>,
    /// Pending causality edges collected during a preview rebuild.
    /// Keyed by target StableId; drained and applied to `PreviewProvenance`
    /// at the end of each rebuild.
    pub pending_causality_edges: BTreeMap<StableId, Vec<CausalityEdge>>,
    /// Pending ChangeSets awaiting user approval in the ChangeWorkbench (ADR-0039).
    /// Key = change-set ID (e.g. "agent:12345" or "cmd:1234567890").
    pub pending_change_sets: BTreeMap<String, PendingChangeSet>,
}

impl ChangeSetsSessionState {
    /// Construct a default instance with empty maps.
    pub fn new() -> Self {
        Self::default()
    }

    // ─── Recent change sets ─────────────────────────────────────────────────

    /// Returns the recent change-set summaries for the given scene path.
    /// Returns an empty buffer if no entries have been recorded for this path.
    pub fn recent_change_sets_for(&self, scene_path: &str) -> Vec<ChangeSetSummary> {
        self.recent_change_sets
            .get(scene_path)
            .map(|b| b.entries())
            .unwrap_or_default()
    }

    /// Push a `ChangeSetSummary` to the per-scene buffer.
    /// The buffer is capped at 50 entries per scene path; the oldest entry
    /// is evicted on overflow.
    pub fn push_recent_change_set(&mut self, scene_path: &str, summary: ChangeSetSummary) {
        self.recent_change_sets
            .entry(scene_path.to_string())
            .or_insert_with(|| RecentChangeSetsBuffer::new(50))
            .push(summary);
    }

    /// Returns all `ChangeSetSummary` entries across all scene paths
    /// (most recent first).
    pub fn all_recent_change_sets(&self) -> Vec<editor_model::ChangeSetSummary> {
        let mut all = Vec::new();
        for buf in self.recent_change_sets.values() {
            all.extend(buf.entries());
        }
        all
    }

    // ─── Pending causality edges ─────────────────────────────────────────────

    /// Returns a mutable reference to the pending causality edges map.
    pub fn pending_causality_edges_mut(
        &mut self,
    ) -> &mut BTreeMap<StableId, Vec<CausalityEdge>> {
        &mut self.pending_causality_edges
    }

    // ─── Pending ChangeSets ─────────────────────────────────────────────────

    /// Returns a mutable reference to the pending ChangeSets map.
    pub fn pending_change_sets_mut(&mut self) -> &mut BTreeMap<String, PendingChangeSet> {
        &mut self.pending_change_sets
    }

    /// Returns an immutable reference to the pending ChangeSets map.
    pub fn pending_change_sets(&self) -> &BTreeMap<String, PendingChangeSet> {
        &self.pending_change_sets
    }
}

// ---------------------------------------------------------------------------
// RuntimeSessionState — runtime deltas + tunable baselines
// ---------------------------------------------------------------------------

/// Session state for play-mode runtime data.
///
/// Groups the runtime delta buffer (play-mode apply-back) with the tunable
/// baselines captured on PlayModeEnter.
#[derive(Debug, Clone, Default)]
pub struct RuntimeSessionState {
    /// Runtime delta buffer for play-mode apply-back (capped at RUNTIME_DELTA_BUFFER_CAP).
    pub runtime_delta_buffer: VecDeque<RuntimeDelta>,
    /// Baseline values for Tunable fields, captured on PlayModeEnter.
    /// Key = composite `"instance_id|component_type_id|field_path"`.
    pub tunable_baselines: BTreeMap<String, serde_json::Value>,
}

impl RuntimeSessionState {
    /// Construct with capacity pre-allocated for the delta buffer.
    pub fn new() -> Self {
        Self {
            runtime_delta_buffer: VecDeque::with_capacity(crate::runtime_delta::RUNTIME_DELTA_BUFFER_CAP),
            tunable_baselines: BTreeMap::new(),
        }
    }

    // ─── Runtime delta buffer ────────────────────────────────────────────────

    /// Returns a reference to the delta buffer.
    pub fn runtime_delta_buffer(&self) -> &VecDeque<RuntimeDelta> {
        &self.runtime_delta_buffer
    }

    /// Returns a mutable reference to the delta buffer, with cap enforcement.
    pub fn runtime_delta_buffer_mut(&mut self) -> &mut VecDeque<RuntimeDelta> {
        while self.runtime_delta_buffer.len() > crate::runtime_delta::RUNTIME_DELTA_BUFFER_CAP {
            self.runtime_delta_buffer.pop_front();
        }
        &mut self.runtime_delta_buffer
    }

    // ─── Tunable baselines ───────────────────────────────────────────────────

    /// Capture the authoring baseline for every field marked Tunable.
    pub fn snapshot_tunable_baselines(&mut self, baselines: BTreeMap<String, serde_json::Value>) {
        self.tunable_baselines = baselines;
    }

    /// Returns a reference to the tunable baselines map.
    pub fn tunable_baselines(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.tunable_baselines
    }

    /// Returns a mutable reference to the tunable baselines map.
    pub fn tunable_baselines_mut(&mut self) -> &mut BTreeMap<String, serde_json::Value> {
        &mut self.tunable_baselines
    }

    /// Clear all tunable baselines.
    pub fn clear_tunable_baselines(&mut self) {
        self.tunable_baselines.clear();
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
    scene_states: BTreeMap<String, LocalSceneSessionState>,
    /// Per-asset-path session state (SCENE_ASSET_CATALOG etc.).
    asset_states: BTreeMap<String, LocalAssetSessionState>,
    /// Per-logic-graph-path session state (LOGIC_GRAPH_DOC etc.).
    logic_states: BTreeMap<String, LocalLogicSessionState>,
    /// Runtime preview inspector + source files (PREVIEW_METRICS etc.).
    preview_state: PreviewSessionState,
    /// Change-workbench and causality tracking state.
    change_sets: ChangeSetsSessionState,
    /// Runtime delta buffer and tunable baselines.
    runtime: RuntimeSessionState,
    /// Logic activation event ring — capped at 64 entries (§6).
    logic_activation_ring: LogicActivationRing,
    /// Extension registry (ADR-0040 — v0.92 SDK).
    extension_registry: Arc<Mutex<dyn ExtensionRegistryPort>>,
    /// Importer registry (ADR-0040 step 3 + ADR-0041 — v0.93 external source importers).
    importer_registry: Arc<Mutex<dyn ImporterRegistryPort>>,
}

impl std::fmt::Debug for EditorSession {
    /// Prints structural info only — dyn trait contents are not printable.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ext_count = self.extension_registry.lock()
            .map(|r| r.list().len())
            .unwrap_or(0);
        let imp_count = self.importer_registry.lock()
            .map(|r| {
                use editor_model::external_source::ExternalSourceKind;
                r.list_by_kind(&ExternalSourceKind::Aseprite).len()
                    + r.list_by_kind(&ExternalSourceKind::Ldtk).len()
                    + r.list_by_kind(&ExternalSourceKind::Tiled).len()
            })
            .unwrap_or(0);
        f.debug_struct("EditorSession")
            .field("active_document", &self.active_document)
            .field("history_scopes", &self.history_scopes)
            .field("caches", &self.caches)
            .field("scene_states", &self.scene_states)
            .field("asset_states", &self.asset_states)
            .field("logic_states", &self.logic_states)
            .field("preview_state", &self.preview_state)
            .field("change_sets", &self.change_sets)
            .field("runtime", &self.runtime)
            .field(
                "logic_activation_ring_len",
                &self.logic_activation_ring.len(),
            )
            .field("extension_registry_len", &ext_count)
            .field("importer_registry_len", &imp_count)
            .finish()
    }
}

impl EditorSession {
    /// Construct a new session with the given store and clock.
    ///
    /// The session starts with no active document and no history scopes.
    /// Extension registry and importer registry are empty (for tests that don't
    /// want built-in pre-seeding).
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
            preview_state: PreviewSessionState::new(),
            change_sets: ChangeSetsSessionState::new(),
            runtime: RuntimeSessionState::new(),
            logic_activation_ring: VecDeque::with_capacity(
                editor_model::logic_activation::LOGIC_ACTIVATION_RING_CAP,
            ),
            extension_registry: Arc::new(Mutex::new(ExtensionRegistry::empty())),
            importer_registry: Arc::new(Mutex::new(ImporterRegistry::empty())),
        }
    }

    /// Construct a new session with built-in extensions pre-registered.
    ///
    /// This is the canonical production constructor. Built-in extensions
    /// (`builtin.logic-bricks.controllers`, `builtin.logic-recipes`,
    /// `builtin.scene-validator`) and built-in importers (Aseprite, LDtk,
    /// Tiled) are pre-registered at composition time.
    pub fn with_builtins(store: Arc<dyn ProjectStore>, clock: Arc<dyn Clock>) -> Self {
        Self {
            store,
            clock,
            active_document: None,
            history_scopes: BTreeMap::new(),
            caches: BTreeMap::new(),
            scene_states: BTreeMap::new(),
            asset_states: BTreeMap::new(),
            logic_states: BTreeMap::new(),
            preview_state: PreviewSessionState::new(),
            change_sets: ChangeSetsSessionState::new(),
            runtime: RuntimeSessionState::new(),
            logic_activation_ring: VecDeque::with_capacity(
                editor_model::logic_activation::LOGIC_ACTIVATION_RING_CAP,
            ),
            extension_registry: Arc::new(Mutex::new(ExtensionRegistry::with_builtins())),
            importer_registry: Arc::new(Mutex::new(ImporterRegistry::with_builtins())),
        }
    }

    /// Returns the extension registry accessor (shared, read-only).
    ///
    /// Returns `Arc<Mutex<dyn ExtensionRegistryPort>>` so callers can hold the
    /// lock across multiple calls without borrowing `&mut self`.
    pub fn extension_registry(&self) -> Arc<Mutex<dyn ExtensionRegistryPort>> {
        Arc::clone(&self.extension_registry)
    }

    /// Returns a mutable reference to the extension registry.
    ///
    /// Used by WASM exports that need to register/unregister extensions.
    pub(crate) fn extension_registry_mut(
        &mut self,
    ) -> &mut Arc<Mutex<dyn ExtensionRegistryPort>> {
        &mut self.extension_registry
    }

    /// Returns the importer registry accessor (shared).
    ///
    /// Returns `Arc<Mutex<dyn ImporterRegistryPort>>` so callers can hold the
    /// lock across multiple calls without borrowing `&mut self`.
    pub fn importer_registry(&self) -> Arc<Mutex<dyn ImporterRegistryPort>> {
        Arc::clone(&self.importer_registry)
    }

    /// Returns a mutable reference to the importer registry.
    ///
    /// Used by WASM exports that need to register/unregister importers.
    pub(crate) fn importer_registry_mut(
        &mut self,
    ) -> &mut Arc<Mutex<dyn ImporterRegistryPort>> {
        &mut self.importer_registry
    }

    // ─── Sub-state accessors (PR2a) ──────────────────────────────────────────

    /// Returns a mutable reference to the scene session state for the given path,
    /// creating it if absent (idempotent).
    ///
    /// This is the primary entry point for migrating `scene_session.rs` to use
    /// `EditorSession` as the owning store instead of `thread_local!`.
    pub fn scene_state_mut(&mut self, path: &str) -> &mut LocalSceneSessionState {
        self.scene_states
            .entry(path.to_string())
            .or_insert_with(LocalSceneSessionState::default)
    }

    /// Returns a mutable reference to the asset session state for the given path,
    /// creating it if absent.
    pub fn asset_state_mut(&mut self, path: &str) -> &mut LocalAssetSessionState {
        self.asset_states
            .entry(path.to_string())
            .or_insert_with(LocalAssetSessionState::default)
    }

    /// Returns a mutable reference to the logic session state for the given path,
    /// creating it if absent.
    pub fn logic_state_mut(&mut self, path: &str) -> &mut LocalLogicSessionState {
        self.logic_states
            .entry(path.to_string())
            .or_insert_with(LocalLogicSessionState::default)
    }

    /// Returns the recent change-set summaries for the given scene path.
    ///
    /// Returns an empty buffer if no entries have been recorded for this path.
    pub fn recent_change_sets_for(&self, scene_path: &str) -> Vec<ChangeSetSummary> {
        self.change_sets.recent_change_sets_for(scene_path)
    }

    /// Returns a reference to the runtime delta buffer.
    pub fn runtime_delta_buffer(&self) -> &VecDeque<RuntimeDelta> {
        RuntimeSessionState::runtime_delta_buffer(&self.runtime)
    }

    /// Returns a mutable reference to the runtime delta buffer.
    pub fn runtime_delta_buffer_mut(&mut self) -> &mut VecDeque<RuntimeDelta> {
        self.runtime.runtime_delta_buffer_mut()
    }

    /// Capture the authoring baseline for every field marked Tunable.
    ///
    /// Called by `process_play_mode_request(PlayModeEnter)`. The captured
    /// baseline is later used to compute `RuntimeDelta` on `PlayModeExit`.
    pub fn snapshot_tunable_baselines(&mut self, baselines: BTreeMap<String, serde_json::Value>) {
        self.runtime.snapshot_tunable_baselines(baselines);
    }

    /// Returns a reference to the tunable baselines map.
    pub fn tunable_baselines(&self) -> &BTreeMap<String, serde_json::Value> {
        RuntimeSessionState::tunable_baselines(&self.runtime)
    }

    /// Clear all tunable baselines (called after PlayModeExit delta computation).
    pub fn clear_tunable_baselines(&mut self) {
        self.runtime.clear_tunable_baselines();
    }

    /// Returns a reference to the source files cache.
    pub fn source_files(&self) -> &editor_model::SourceFilesCache {
        self.preview_state.source_files()
    }

    /// Returns a mutable reference to the source files cache.
    pub fn source_files_mut(&mut self) -> &mut editor_model::SourceFilesCache {
        self.preview_state.source_files_mut()
    }

    /// Returns a reference to the preview inspector state.
    pub fn preview_inspector(&self) -> &editor_model::PreviewInspectorState {
        self.preview_state.preview_inspector()
    }

    /// Returns a mutable reference to the preview inspector state.
    pub fn preview_inspector_mut(&mut self) -> &mut editor_model::PreviewInspectorState {
        self.preview_state.preview_inspector_mut()
    }

    // ─── §6 Runtime Causality accessors ────────────────────────────────────────

    /// Returns a reference to the logic activation ring.
    pub fn logic_activation_ring(&self) -> &LogicActivationRing {
        &self.logic_activation_ring
    }

    /// Push an event onto the logic activation ring, evicting the oldest if at cap.
    pub fn push_logic_activation(&mut self, event: LogicActivationEvent) {
        ring_push(&mut self.logic_activation_ring, event);
    }

    /// Push a `ChangeSetSummary` to the per-scene buffer (v0.91 PR1).
    ///
    /// Called by the `poll_recent_change_sets` Bevy system in editor-core
    /// after each successful `TransactionKernel::apply_atomic` (or directly
    /// from tests). The buffer is capped at 50 entries per scene path; the
    /// oldest entry is evicted on overflow.
    pub fn push_recent_change_set(&mut self, scene_path: &str, summary: ChangeSetSummary) {
        self.change_sets.push_recent_change_set(scene_path, summary);
    }

    /// Returns all `ChangeSetSummary` entries for the given scene path
    /// (most recent first). Empty Vec if no entries (or session not initialized).
    pub fn all_recent_change_sets(&self) -> Vec<editor_model::ChangeSetSummary> {
        self.change_sets.all_recent_change_sets()
    }

    /// Returns the last rebuild cause recorded by §6.
    pub fn last_rebuild_cause(&self) -> Option<&RebuildCause> {
        self.preview_state.last_rebuild_cause()
    }

    /// Records a rebuild cause (§6).
    pub fn set_last_rebuild_cause(&mut self, cause: RebuildCause) {
        self.preview_state.set_last_rebuild_cause(cause);
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
        self.change_sets.pending_change_sets_mut()
    }

    /// Returns a reference to the pending ChangeSets map.
    pub fn pending_change_sets(&self) -> &BTreeMap<String, PendingChangeSet> {
        self.change_sets.pending_change_sets()
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

// ─────────────────────────────────────────────────────────────────────────────
// EditorSessionPort impl (v0.90 PR1)
// ─────────────────────────────────────────────────────────────────────────────
//
// editor-core (Bevy systems) accesses the session through this trait via
// `editor_model::ports::with_session_mut(|s| { ... })`. The trait is in
// editor-model so editor-core can import it without depending on
// editor-application. The dyn-trait object also lets the global registry
// hold a type-erased session.

impl EditorSessionPort for EditorSession {
    fn tunable_baselines_mut(&mut self) -> &mut BTreeMap<String, serde_json::Value> {
        self.runtime.tunable_baselines_mut()
    }

    fn last_rebuild_cause_mut(&mut self) -> &mut Option<RebuildCause> {
        &mut self.preview_state.preview_inspector.last_rebuild_cause
    }

    fn pending_causality_edges_mut(&mut self) -> &mut BTreeMap<StableId, Vec<CausalityEdge>> {
        self.change_sets.pending_causality_edges_mut()
    }

    fn runtime_delta_buffer_mut(&mut self) -> &mut VecDeque<RuntimeDelta> {
        self.runtime.runtime_delta_buffer_mut()
    }

    fn scene_state_mut(&mut self, path: &str) -> &mut editor_model::SceneSessionState {
        self.scene_states
            .entry(path.to_string())
            .or_insert_with(editor_model::SceneSessionState::default)
    }

    fn asset_state_mut(&mut self, path: &str) -> &mut editor_model::AssetSessionState {
        self.asset_states
            .entry(path.to_string())
            .or_insert_with(editor_model::AssetSessionState::default)
    }

    fn logic_state_mut(&mut self, path: &str) -> &mut editor_model::LogicSessionState {
        self.logic_states
            .entry(path.to_string())
            .or_insert_with(editor_model::LogicSessionState::default)
    }

    fn preview_inspector_mut(&mut self) -> &mut editor_model::PreviewInspectorState {
        self.preview_state.preview_inspector_mut()
    }

    fn source_files_mut(&mut self) -> &mut editor_model::SourceFilesCache {
        self.preview_state.source_files_mut()
    }

    fn recent_change_sets_for(&self, scene_path: &str) -> Vec<editor_model::ChangeSetSummary> {
        self.change_sets.recent_change_sets_for(scene_path)
    }

    fn all_recent_change_sets(&self) -> Vec<editor_model::ChangeSetSummary> {
        self.change_sets.all_recent_change_sets()
    }

    fn active_document_path(&self) -> Option<&str> {
        self.active_document.as_ref().map(|sel| sel.path())
    }

    fn push_recent_change_set(
        &mut self,
        scene_path: &str,
        summary: editor_model::ChangeSetSummary,
    ) {
        self.change_sets.push_recent_change_set(scene_path, summary);
    }

    fn logic_activation_ring_mut(
        &mut self,
    ) -> &mut VecDeque<editor_model::logic_activation::LogicActivationEvent> {
        &mut self.logic_activation_ring
    }
}
