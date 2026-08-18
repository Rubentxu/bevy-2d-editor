//! SEM-5 migration corpus tests (SDD-0046 S3).
//!
//! Historical document shapes are recreated inline as JSON and proven to
//! migrate to the current version with all semantic fields preserved.

use editor_model::migration::migrate;
use editor_model::migration::{MigrationError, parse_version_string};
use editor_model::{ProjectMetadata, SceneDocument};

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
