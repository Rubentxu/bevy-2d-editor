//! SEM-5 migration corpus tests (SDD-0046 S3).
//!
//! Historical document shapes are recreated inline as JSON and proven to
//! migrate to the current version with all semantic fields preserved.
//!
//! As of v0.110.0 (cycle `g4-roundtrip-corpus`), all 5 durable document
//! types have at least one V0→V1 corpus test. See `docs/v1-format-manifest.md`
//! for the authoritative format registry.

use editor_model::migration::migrate;
use editor_model::migration::{MigrationError, parse_version_string};
use editor_model::{
    LogicGraphAsset, ProjectMetadata, SceneAssetDocument, SceneAssetRole, SceneDocument,
    WorldDocument, WorldId,
};

/// v0.94-era project.json shape: the `worlds` / `active_world` fields added
/// in v0.95.0 (ADR-0037 World Workspace) are absent.
const V0_PROJECT_METADATA_JSON: &str = r#"{
    "version": "0.1",
    "name": "Legacy Project",
    "scenes": ["level_1", "level_2"],
    "schemas": [],
    "active_scene": "level_1",
    "scene_assets": []
}"#;

/// Pre-instances SceneDocument shape: the `instances` map added in v0.88-era
/// (scene instances) is absent.
const V0_SCENE_DOCUMENT_JSON: &str = r#"{
    "version": "0.1",
    "scene_id": "legacy_scene",
    "name": "Legacy Scene",
    "entities": []
}"#;

/// Spec §sem3-corpus scenario 16: v0.94 ProjectMetadata migrates to V1.
#[test]
fn corpus_v0_project_metadata_migrates() {
    let mut pm: ProjectMetadata =
        serde_json::from_str(V0_PROJECT_METADATA_JSON).expect("V0 shape must parse");
    assert_eq!(pm.version, "0.1");

    let v = parse_version_string("ProjectMetadata", &pm.version).unwrap();
    assert_eq!(v, 0);
    migrate::project_metadata(v, &mut pm).unwrap();

    // Materialized defaults
    assert!(pm.worlds.is_empty());
    assert!(pm.active_world.is_none());
    // All other fields preserved
    assert_eq!(pm.name, "Legacy Project");
    assert_eq!(pm.scenes, vec!["level_1", "level_2"]);
    assert_eq!(pm.active_scene, Some("level_1".into()));
}

/// Spec §sem3-corpus scenario 17: pre-instances SceneDocument migrates to V1.
#[test]
fn corpus_v0_scene_document_migrates() {
    let mut doc: SceneDocument =
        serde_json::from_str(V0_SCENE_DOCUMENT_JSON).expect("V0 shape must parse");
    assert_eq!(doc.version, "0.1");

    let v = parse_version_string("SceneDocument", &doc.version).unwrap();
    assert_eq!(v, 0);
    migrate::scene_document(v, &mut doc).unwrap();

    // Materialized defaults
    assert!(doc.instances.is_empty());
    // All other fields preserved
    assert_eq!(doc.scene_id, "legacy_scene");
    assert_eq!(doc.name, "Legacy Scene");
}

/// Spec §sem3-corpus scenario 18: current-version round-trip is a no-op.
///
/// Proves migration does not perturb documents already at CURRENT_VERSION.
#[test]
fn corpus_current_version_round_trip_noop() {
    let mut pm: ProjectMetadata = serde_json::from_str(V0_PROJECT_METADATA_JSON).unwrap();
    let v = parse_version_string("ProjectMetadata", &pm.version).unwrap();
    migrate::project_metadata(v, &mut pm).unwrap();
    let before = pm.clone();
    migrate::project_metadata(1, &mut pm).unwrap();
    assert_eq!(pm, before, "current-version migration must be a no-op");
}

/// Spec §sem3-migrate-functions scenario 1 (corpus-level): future version
/// rejected with UnsupportedVersion.
#[test]
fn corpus_future_version_rejected() {
    let mut pm: ProjectMetadata = serde_json::from_str(V0_PROJECT_METADATA_JSON).unwrap();
    let err = migrate::project_metadata(999, &mut pm).unwrap_err();
    assert!(matches!(
        err,
        MigrationError::UnsupportedVersion {
            type_name: "ProjectMetadata",
            version: 999,
            max: 1
        }
    ));
}

// ---------------------------------------------------------------------------
// Cycle `g4-roundtrip-corpus` (v0.110.0): 3 new corpus tests covering the
// durable document types that were missing from the original corpus
// (SceneAssetDocument, WorldDocument, LogicGraphAsset). All three types
// shipped at v1 (or near it) so the v0→v1 step is a no-op; the tests
// prove the migration API still rejects future versions and that v1 is
// treated as a no-op idempotent step.
// ---------------------------------------------------------------------------

/// Pre-flatten SceneAssetDocument shape: the `extension_data` flatten map
/// (ADR-0046 rule 2) is absent. The v0→v1 step is a no-op (serde defaults
/// materialize the map on parse).
const V0_SCENE_ASSET_DOCUMENT_JSON: &str = r#"{
    "asset_id": "asset_legacy",
    "logical_path": "legacy/door.bsn",
    "role": "screen",
    "version": 1,
    "entities": []
}"#;

/// Spec §g4-corpus scenario 1: v0 SceneAssetDocument migrates to V1.
#[test]
fn corpus_v0_scene_asset_document_migrates() {
    let mut doc: SceneAssetDocument =
        serde_json::from_str(V0_SCENE_ASSET_DOCUMENT_JSON).expect("V0 shape must parse");

    let v = parse_version_string("SceneAssetDocument", "0.1").unwrap();
    assert_eq!(v, 0);
    migrate::scene_asset_document(v, &mut doc).unwrap();

    // No-op step: all fields preserved verbatim.
    assert_eq!(doc.asset_id, "asset_legacy");
    assert_eq!(doc.logical_path, "legacy/door.bsn");
    assert_eq!(doc.role, SceneAssetRole::Screen);
    assert_eq!(doc.version, 1);
    assert!(doc.entities.is_empty());
    assert!(doc.extension_data.is_empty());

    // Current-version migration is also a no-op (idempotent).
    let before = doc.clone();
    migrate::scene_asset_document(1, &mut doc).unwrap();
    assert_eq!(doc, before);
}

/// Pre-ADR-0037 WorldDocument shape: the `updated_at` field added in
/// v0.95.0 is absent. The v0→v1 step is a no-op (serde defaults).
const V0_WORLD_DOCUMENT_JSON: &str = r#"{
    "id": "world_legacy",
    "name": "Legacy World",
    "version": 1,
    "layout_policy": { "kind": "grid", "cell_size": 64, "cols": 4, "rows": 4 },
    "levels": [],
    "links": [],
    "updated_at": 0
}"#;

/// Spec §g4-corpus scenario 2: v0 WorldDocument migrates to V1.
#[test]
fn corpus_v0_world_document_migrates() {
    let mut doc: WorldDocument =
        serde_json::from_str(V0_WORLD_DOCUMENT_JSON).expect("V0 shape must parse");

    let v = parse_version_string("WorldDocument", "0.1").unwrap();
    assert_eq!(v, 0);
    migrate::world_document(v, &mut doc).unwrap();

    // No-op step: id + name preserved; layout_policy parses to default.
    assert_eq!(doc.id, WorldId("world_legacy".into()));
    assert_eq!(doc.name, "Legacy World");
    assert_eq!(doc.version, 1);
    assert!(doc.levels.is_empty());
    assert!(doc.links.is_empty());
    assert!(doc.extension_data.is_empty());

    // Idempotency.
    let before = doc.clone();
    migrate::world_document(1, &mut doc).unwrap();
    assert_eq!(doc, before);
}

/// Pre-builtin LogicGraphAsset shape: the `builtin` flag added later is
/// absent (defaults to false via serde). The v0→v1 step is a no-op.
const V0_LOGIC_GRAPH_ASSET_JSON: &str = r#"{
    "asset_id": "graph_legacy",
    "logical_path": "legacy/spawn_enemy.graph",
    "version": 1,
    "nodes": [],
    "edges": []
}"#;

/// Spec §g4-corpus scenario 3: v0 LogicGraphAsset migrates to V1.
#[test]
fn corpus_v0_logic_graph_asset_migrates() {
    let mut doc: LogicGraphAsset =
        serde_json::from_str(V0_LOGIC_GRAPH_ASSET_JSON).expect("V0 shape must parse");

    let v = parse_version_string("LogicGraphAsset", "0.1").unwrap();
    assert_eq!(v, 0);
    migrate::logic_graph_asset(v, &mut doc).unwrap();

    // No-op step: all fields preserved; builtin defaults to false.
    assert_eq!(doc.asset_id, "graph_legacy");
    assert_eq!(doc.logical_path, "legacy/spawn_enemy.graph");
    assert_eq!(doc.version, 1);
    assert!(!doc.builtin);
    assert!(doc.nodes.is_empty());
    assert!(doc.edges.is_empty());
    assert!(doc.extension_data.is_empty());

    // Idempotency.
    let before = doc.clone();
    migrate::logic_graph_asset(1, &mut doc).unwrap();
    assert_eq!(doc, before);
}

/// Spec §g4-corpus scenario 4 (negative): future version of every
/// migrated type is rejected with UnsupportedVersion. Proves the
/// `other => Err` arm fires for each of the 5 types.
#[test]
fn corpus_all_types_reject_future_version() {
    // Build minimal v1 documents of each type via the JSON fixtures.
    let mut sd: SceneDocument = serde_json::from_str(V0_SCENE_DOCUMENT_JSON).unwrap();
    let mut sad: SceneAssetDocument =
        serde_json::from_str(V0_SCENE_ASSET_DOCUMENT_JSON).unwrap();
    let mut wd: WorldDocument = serde_json::from_str(V0_WORLD_DOCUMENT_JSON).unwrap();
    let mut lg: LogicGraphAsset = serde_json::from_str(V0_LOGIC_GRAPH_ASSET_JSON).unwrap();
    let mut pm: ProjectMetadata = serde_json::from_str(V0_PROJECT_METADATA_JSON).unwrap();

    // All five should reject version 999.
    for (name, err) in [
        (
            "SceneDocument",
            migrate::scene_document(999, &mut sd).unwrap_err(),
        ),
        (
            "SceneAssetDocument",
            migrate::scene_asset_document(999, &mut sad).unwrap_err(),
        ),
        (
            "WorldDocument",
            migrate::world_document(999, &mut wd).unwrap_err(),
        ),
        (
            "LogicGraphAsset",
            migrate::logic_graph_asset(999, &mut lg).unwrap_err(),
        ),
        (
            "ProjectMetadata",
            migrate::project_metadata(999, &mut pm).unwrap_err(),
        ),
    ] {
        assert!(
            matches!(
                err,
                MigrationError::UnsupportedVersion {
                    version: 999,
                    max: 1,
                    ..
                }
            ),
            "{name} future-version rejection failed"
        );
    }
}
