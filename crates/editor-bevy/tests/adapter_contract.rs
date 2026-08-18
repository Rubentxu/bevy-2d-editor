//! adapter_contract.rs — Integration tests for the EditorAdapter system (SDD-0046 S1).
//!
//! Verifies:
//! - All three adapters are reachable via `all_adapters()`.
//! - Round-trip encode → decode preserves document identity for Lossless adapters.
//! - Encode-only adapters reject unsupported roles correctly.
//! - Fidelity annotations are consistent with adapter behaviour.

use editor_bevy::adapter_impls::{
    BevyRuntimeAdapter, BsnExportAdapter, JsonProjectAdapter, all_adapters_init,
};
use editor_model::LogicGraphAsset;
use editor_model::adapter::{AdapterFidelity, EditorAdapter, SemanticModel};
use editor_model::ids::SceneAssetLocalId;
use editor_model::scene_asset::{
    SceneAssetDocument, SceneAssetEntity, SceneAssetMetadata, SceneAssetRole,
};

// ---------------------------------------------------------------------------
// Test data helpers
// ---------------------------------------------------------------------------

fn make_actor_asset() -> SceneAssetDocument {
    SceneAssetDocument {
        asset_id: "hero".into(),
        logical_path: "actors/hero".into(),
        role: SceneAssetRole::Actor,
        version: 1,
        entities: vec![SceneAssetEntity {
            local_id: SceneAssetLocalId::new("e1"),
            local_path: "root".into(),
            name: "Hero".into(),
            components: vec![],
        }],
        relationships: vec![],
        exposed_properties: vec![],
        metadata: SceneAssetMetadata {
            tags: Some("player".into()),
            notes: None,
            created_at: None,
            updated_at: None,
        },
        layers: vec![],
    }
}

fn make_logic_asset() -> LogicGraphAsset {
    LogicGraphAsset {
        asset_id: "jump".into(),
        logical_path: "logic/jump".into(),
        version: 1,
        builtin: false,
        nodes: vec![],
        edges: vec![],
    }
}

// ---------------------------------------------------------------------------
// T1.9.1 — all_adapters_init() returns three adapters
// ---------------------------------------------------------------------------

#[test]
fn all_adapters_init_returns_three_adapters() {
    let registry = all_adapters_init();
    assert_eq!(
        registry.len(),
        3,
        "expected 3 adapters: Json, Bsn, BevyRuntime"
    );
}

// ---------------------------------------------------------------------------
// T1.9.2 — JsonProjectAdapter round-trips SceneAssetDocument losslessly
// ---------------------------------------------------------------------------

#[test]
fn json_adapter_round_trips_scene_asset_losslessly() {
    let adapter = JsonProjectAdapter::new();
    let original = make_actor_asset();

    let encoded = adapter
        .encode(&SemanticModel::SceneAsset(original.clone()))
        .unwrap();
    let decoded = adapter.decode(&encoded).unwrap();

    match decoded {
        SemanticModel::SceneAsset(d) => {
            assert_eq!(d.asset_id, original.asset_id);
            assert_eq!(d.logical_path, original.logical_path);
            assert_eq!(d.role, original.role);
            assert_eq!(d.entities.len(), original.entities.len());
        }
        other => panic!("expected SceneAsset, got {other:?}"),
    }
}

#[test]
fn json_adapter_round_trips_logic_graph_losslessly() {
    let adapter = JsonProjectAdapter::new();
    let original = make_logic_asset();

    let encoded = adapter
        .encode(&SemanticModel::LogicGraph(original.clone()))
        .unwrap();
    let decoded = adapter.decode(&encoded).unwrap();

    match decoded {
        SemanticModel::LogicGraph(d) => {
            assert_eq!(d.asset_id, original.asset_id);
            assert_eq!(d.logical_path, original.logical_path);
        }
        other => panic!("expected LogicGraph, got {other:?}"),
    }
}

#[test]
fn json_adapter_has_lossless_fidelity() {
    let adapter = JsonProjectAdapter::new();
    assert_eq!(adapter.fidelity(), AdapterFidelity::Lossless);
}

// ---------------------------------------------------------------------------
// T1.9.3 — BsnExportAdapter encodes SceneAssetDocument as semantic .bsn text
// ---------------------------------------------------------------------------

#[test]
fn bsn_adapter_encodes_actor_asset_to_bsn_text() {
    let adapter = BsnExportAdapter::new();
    let asset = make_actor_asset();

    let result = adapter.encode(&SemanticModel::SceneAsset(asset.clone()));
    assert!(
        result.is_ok(),
        "encode should succeed for Actor role, got {result:?}"
    );
    let bytes = result.unwrap();
    let text = String::from_utf8(bytes).expect("BSN output should be valid UTF-8");
    assert!(!text.is_empty(), "BSN output should not be empty");
}

#[test]
fn bsn_adapter_rejects_logic_graph() {
    let adapter = BsnExportAdapter::new();
    let logic = make_logic_asset();

    let result = adapter.encode(&SemanticModel::LogicGraph(logic.clone()));
    assert!(
        matches!(
            result,
            Err(editor_model::adapter::AdapterError::UnsupportedModel { .. })
        ),
        "encode should reject LogicGraph model, got {result:?}"
    );
}

#[test]
fn bsn_adapter_has_semantic_lossless_fidelity() {
    let adapter = BsnExportAdapter::new();
    assert_eq!(
        adapter.fidelity(),
        AdapterFidelity::SemanticLossless,
        "BSN export preserves scene structure semantically"
    );
}

// ---------------------------------------------------------------------------
// T1.9.4 — BevyRuntimeAdapter encodes all variants as lossy JSON
// ---------------------------------------------------------------------------

#[test]
fn bevy_runtime_adapter_encodes_scene_asset() {
    let adapter = BevyRuntimeAdapter::new();
    let asset = make_actor_asset();

    let result = adapter.encode(&SemanticModel::SceneAsset(asset.clone()));
    assert!(
        result.is_ok(),
        "BevyRuntimeAdapter should encode SceneAsset, got {result:?}"
    );
}

#[test]
fn bevy_runtime_adapter_encodes_logic_graph() {
    let adapter = BevyRuntimeAdapter::new();
    let logic = make_logic_asset();

    let result = adapter.encode(&SemanticModel::LogicGraph(logic.clone()));
    assert!(
        result.is_ok(),
        "BevyRuntimeAdapter should encode LogicGraph, got {result:?}"
    );
}

#[test]
fn bevy_runtime_adapter_has_export_only_lossy_fidelity() {
    let adapter = BevyRuntimeAdapter::new();
    assert_eq!(
        adapter.fidelity(),
        AdapterFidelity::ExportOnlyLossy,
        "BevyRuntime is encode-only lossy"
    );
}

// ---------------------------------------------------------------------------
// T1.9.5 — Fidelity descriptions are non-empty
// ---------------------------------------------------------------------------

#[test]
fn all_fidelities_have_non_empty_descriptions() {
    let adapters = all_adapters_init();
    for boxed in adapters.iter() {
        let desc = boxed.fidelity().description();
        assert!(
            !desc.is_empty(),
            "fidelity description for {} should not be empty",
            boxed.name()
        );
    }
}

// ---------------------------------------------------------------------------
// T1.9.6 — Adapter names are stable strings
// ---------------------------------------------------------------------------

#[test]
fn adapter_names_are_stable_and_distinct() {
    let adapters = all_adapters_init();
    let names: Vec<_> = adapters.iter().map(|a| a.name()).collect();
    assert_eq!(names.len(), 3, "should have exactly 3 adapters");
    assert!(
        names.iter().all(|n| !n.is_empty()),
        "no adapter should have an empty name"
    );

    // All names should be distinct
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        names.len(),
        "adapter names must be distinct: {names:?}"
    );
}

// ---------------------------------------------------------------------------
// SDD-0046 S2 scenario 16 — initialized registry is observable from any thread
// ---------------------------------------------------------------------------

#[test]
fn initialized_registry_is_observable_from_any_thread() {
    // One-shot init per test binary (OnceLock::set panics on double call).
    // This is the ONLY test in this binary that calls init_registry.
    editor_model::adapter::init_registry(all_adapters_init());

    let expected: Vec<&str> = vec!["json.project.v1", "bsn.export.v1", "bevy.runtime.v1"];
    let handles: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| {
                let adapters = editor_model::adapter::all_adapters();
                adapters
                    .iter()
                    .map(|a| a.name().to_string())
                    .collect::<Vec<_>>()
            })
        })
        .collect();

    for handle in handles {
        let names = handle.join().expect("thread should not panic");
        assert_eq!(names.len(), 3, "all threads see 3 adapters: {names:?}");
        for name in &expected {
            assert!(
                names.iter().any(|n| n == name),
                "missing {name} in {names:?}"
            );
        }
    }
}
