//! v0.91 PR2 (MUST) — `SCENE_ASSET_CATALOG` and `SCENE_ASSET_CATALOG_WARNINGS`
//! thread_locals in editor-core are gone; both now read from / write to
//! `EditorSession` via the `EditorSessionPort` trait.

use editor_model::EditorSessionPort;
use editor_model::scene_asset_catalog::{CatalogWarning, SceneAssetCatalog};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

struct FakeSession {
    tunable_baselines: std::collections::BTreeMap<String, serde_json::Value>,
    runtime_delta_buffer: std::collections::VecDeque<editor_model::RuntimeDelta>,
    pending_causality_edges:
        std::collections::BTreeMap<editor_model::StableId, Vec<editor_model::CausalityEdge>>,
    last_rebuild_cause: Option<editor_model::RebuildCause>,
    scene_states: std::collections::BTreeMap<String, editor_model::SceneSessionState>,
    asset_states: std::collections::BTreeMap<String, editor_model::AssetSessionState>,
    logic_states: std::collections::BTreeMap<String, editor_model::LogicSessionState>,
    preview_inspector: editor_model::PreviewInspectorState,
    source_files: editor_model::SourceFilesCache,
    recent_change_sets: std::collections::BTreeMap<String, Vec<editor_model::ChangeSetSummary>>,
    logic_activation_ring:
        std::collections::VecDeque<editor_model::logic_activation::LogicActivationEvent>,
}

impl EditorSessionPort for FakeSession {
    fn tunable_baselines_mut(
        &mut self,
    ) -> &mut std::collections::BTreeMap<String, serde_json::Value> {
        &mut self.tunable_baselines
    }
    fn last_rebuild_cause_mut(&mut self) -> &mut Option<editor_model::RebuildCause> {
        &mut self.last_rebuild_cause
    }
    fn pending_causality_edges_mut(
        &mut self,
    ) -> &mut std::collections::BTreeMap<editor_model::StableId, Vec<editor_model::CausalityEdge>>
    {
        &mut self.pending_causality_edges
    }
    fn runtime_delta_buffer_mut(
        &mut self,
    ) -> &mut std::collections::VecDeque<editor_model::RuntimeDelta> {
        &mut self.runtime_delta_buffer
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
        &mut self.preview_inspector
    }
    fn source_files_mut(&mut self) -> &mut editor_model::SourceFilesCache {
        &mut self.source_files
    }
    fn recent_change_sets_for(&self, _scene_path: &str) -> Vec<editor_model::ChangeSetSummary> {
        Vec::new()
    }
    fn push_recent_change_set(
        &mut self,
        _scene_path: &str,
        _summary: editor_model::ChangeSetSummary,
    ) {
    }
    fn logic_activation_ring_mut(
        &mut self,
    ) -> &mut std::collections::VecDeque<editor_model::logic_activation::LogicActivationEvent> {
        &mut self.logic_activation_ring
    }
    fn all_recent_change_sets(&self) -> Vec<editor_model::ChangeSetSummary> {
        Vec::new()
    }
}

fn fresh_session() {
    let session = FakeSession {
        tunable_baselines: std::collections::BTreeMap::new(),
        runtime_delta_buffer: std::collections::VecDeque::with_capacity(64),
        pending_causality_edges: std::collections::BTreeMap::new(),
        last_rebuild_cause: None,
        scene_states: std::collections::BTreeMap::new(),
        asset_states: std::collections::BTreeMap::new(),
        logic_states: std::collections::BTreeMap::new(),
        preview_inspector: editor_model::PreviewInspectorState::default(),
        source_files: editor_model::SourceFilesCache::default(),
        recent_change_sets: std::collections::BTreeMap::new(),
        logic_activation_ring: std::collections::VecDeque::with_capacity(64),
    };
    let arc: Arc<Mutex<dyn EditorSessionPort>> = Arc::new(Mutex::new(session));
    editor_model::ports::register_editor_session(arc);
}

#[test]
fn scene_asset_catalog_writes_via_session() {
    fresh_session();
    // with_asset_catalog_mut writes to the session.
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(editor_core::asset_state::ACTIVE_ASSET_PATH)
            .catalog = Some(SceneAssetCatalog::new());
    });
    let catalog_present: Option<bool> = editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(editor_core::asset_state::ACTIVE_ASSET_PATH)
            .catalog
            .is_some()
    });
    assert_eq!(catalog_present, Some(true), "session must hold the catalog");
}

#[test]
fn scene_asset_catalog_warnings_via_session() {
    fresh_session();
    // Push a warning to the session via the editor-core API.
    let warning = CatalogWarning {
        code: "test_warning".to_string(),
        message: "test message".to_string(),
        asset_id: None,
        logical_path: None,
    };
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(editor_core::asset_state::ACTIVE_ASSET_PATH)
            .catalog_warnings
            .push(warning.clone());
    });

    // Read it back via the editor-core API.
    let warnings = editor_core::asset_state::get_asset_catalog_warnings();
    assert_eq!(warnings.len(), 1, "session should have one warning");
    assert_eq!(warnings[0].code, "test_warning");
}

#[test]
fn clear_asset_catalog_warnings_clears_session() {
    fresh_session();
    // Push a warning, then clear.
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(editor_core::asset_state::ACTIVE_ASSET_PATH)
            .catalog_warnings
            .push(CatalogWarning {
                code: "to_be_cleared".to_string(),
                message: "".to_string(),
                asset_id: None,
                logical_path: None,
            });
    });
    assert_eq!(
        editor_core::asset_state::get_asset_catalog_warnings().len(),
        1,
        "warning present"
    );

    editor_core::asset_state::clear_asset_catalog_warnings();
    assert_eq!(
        editor_core::asset_state::get_asset_catalog_warnings().len(),
        0,
        "warning cleared"
    );
}
