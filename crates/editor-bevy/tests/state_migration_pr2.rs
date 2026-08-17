//! v0.91 PR2 (MUST) — `SCENE_ASSET_CATALOG` and `SCENE_ASSET_CATALOG_WARNINGS`
//! thread_locals in editor-core are gone; both now read from / write to
//! `EditorSession` via the `EditorSessionPort` trait.

#[path = "support/mod.rs"]
mod support;

use editor_model::EditorSessionPort;
use editor_model::scene_asset_catalog::{CatalogWarning, SceneAssetCatalog};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

// Uses support::FakeSessionWithDefaults as base (provides all 11 fields + impl).
// Only overrides recent_change_sets_for/all_recent_change_sets/push_recent_change_set
// to return empty/nop as required by this test's isolation checks.

struct FakeSession {
    inner: support::FakeSession,
}

impl EditorSessionPort for FakeSession {
    fn scene_state_mut(&mut self, path: &str) -> &mut support::SceneSessionState {
        self.inner.scene_state_mut(path)
    }
    fn asset_state_mut(&mut self, path: &str) -> &mut support::AssetSessionState {
        self.inner.asset_state_mut(path)
    }
    fn logic_state_mut(&mut self, path: &str) -> &mut support::LogicSessionState {
        self.inner.logic_state_mut(path)
    }
    fn tunable_baselines_mut(
        &mut self,
    ) -> &mut std::collections::BTreeMap<String, serde_json::Value> {
        self.inner.tunable_baselines_mut()
    }
    fn last_rebuild_cause_mut(&mut self) -> &mut Option<support::RebuildCause> {
        self.inner.last_rebuild_cause_mut()
    }
    fn pending_causality_edges_mut(
        &mut self,
    ) -> &mut std::collections::BTreeMap<support::StableId, Vec<support::CausalityEdge>> {
        self.inner.pending_causality_edges_mut()
    }
    fn runtime_delta_buffer_mut(
        &mut self,
    ) -> &mut std::collections::VecDeque<support::RuntimeDelta> {
        self.inner.runtime_delta_buffer_mut()
    }
    fn preview_inspector_mut(&mut self) -> &mut support::PreviewInspectorState {
        self.inner.preview_inspector_mut()
    }
    fn source_files_mut(&mut self) -> &mut support::SourceFilesCache {
        self.inner.source_files_mut()
    }
    fn logic_activation_ring_mut(
        &mut self,
    ) -> &mut std::collections::VecDeque<support::LogicActivationEvent> {
        self.inner.logic_activation_ring_mut()
    }
    // Override to return empty as required by this test's isolation semantics
    fn recent_change_sets_for(&self, _scene_path: &str) -> Vec<support::ChangeSetSummary> {
        Vec::new()
    }
    fn all_recent_change_sets(&self) -> Vec<support::ChangeSetSummary> {
        Vec::new()
    }
    fn active_document_path(&self) -> Option<&str> {
        None
    }
    fn push_recent_change_set(&mut self, _scene_path: &str, _summary: support::ChangeSetSummary) {}
}

fn fresh_session() {
    let session = FakeSession {
        inner: support::FakeSession::new(),
    };
    let arc: Arc<Mutex<dyn EditorSessionPort>> = Arc::new(Mutex::new(session));
    editor_model::ports::register_editor_session(arc);
}

#[test]
fn scene_asset_catalog_writes_via_session() {
    fresh_session();
    // with_asset_catalog_mut writes to the session.
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(editor_bevy::asset_state::ACTIVE_ASSET_PATH)
            .catalog = Some(SceneAssetCatalog::new());
    });
    let catalog_present: Option<bool> = editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(editor_bevy::asset_state::ACTIVE_ASSET_PATH)
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
        sess.asset_state_mut(editor_bevy::asset_state::ACTIVE_ASSET_PATH)
            .catalog_warnings
            .push(warning.clone());
    });

    // Read it back via the editor-core API.
    let warnings = editor_bevy::asset_state::get_asset_catalog_warnings();
    assert_eq!(warnings.len(), 1, "session should have one warning");
    assert_eq!(warnings[0].code, "test_warning");
}

#[test]
fn clear_asset_catalog_warnings_clears_session() {
    fresh_session();
    // Push a warning, then clear.
    let _ = editor_model::ports::with_session_mut(|sess| {
        sess.asset_state_mut(editor_bevy::asset_state::ACTIVE_ASSET_PATH)
            .catalog_warnings
            .push(CatalogWarning {
                code: "to_be_cleared".to_string(),
                message: "".to_string(),
                asset_id: None,
                logical_path: None,
            });
    });
    assert_eq!(
        editor_bevy::asset_state::get_asset_catalog_warnings().len(),
        1,
        "warning present"
    );

    editor_bevy::asset_state::clear_asset_catalog_warnings();
    assert_eq!(
        editor_bevy::asset_state::get_asset_catalog_warnings().len(),
        0,
        "warning cleared"
    );
}
