//! Extension-bag tests (SDD-0046 S4, SEM-3).
//!
//! Unknown JSON fields must be PRESERVED (not dropped) in `extension_data`
//! with deterministic BTreeMap ordering.

use editor_model::scene_asset::SceneAssetEntity;
use editor_model::world::WorldDocument;
use editor_model::{LogicGraphAsset, ProjectMetadata, SceneAssetDocument, SceneDocument};

/// Spec §sem4-extension-bag scenarios 1-2: unknown fields land in the bag and
/// survive re-serialization.
#[test]
fn scene_document_unknown_fields_preserved() {
    let json = r#"{
        "version": "0.1",
        "scene_id": "s1",
        "name": "N",
        "entities": [],
        "future_field": {"a": 1},
        "another": 42
    }"#;
    let doc: SceneDocument = serde_json::from_str(json).expect("must parse");
    assert_eq!(doc.extension_data.len(), 2, "both unknown fields captured");
    assert_eq!(doc.extension_data["future_field"]["a"], 1);
    assert_eq!(doc.extension_data["another"], 42);

    let re = serde_json::to_string(&doc).expect("must serialize");
    assert!(
        re.contains(r#""future_field":{"a":1}"#),
        "future_field survived: {re}"
    );
    assert!(re.contains(r#""another":42"#), "another survived: {re}");
}

/// Spec §sem4-extension-bag scenario 3: empty bag adds zero output bytes.
#[test]
fn empty_bag_no_json_noise() {
    let doc = SceneDocument {
        version: "0.1".into(),
        scene_id: "s1".into(),
        name: "N".into(),
        entities: vec![],
        instances: Default::default(),
        extension_data: Default::default(),
    };
    let json = serde_json::to_string(&doc).unwrap();
    assert!(
        !json.contains("extension_data"),
        "empty bag must not appear in output: {json}"
    );
}

/// Spec §sem4-per-type-preservation scenario 4: SceneAssetDocument.
#[test]
fn scene_asset_document_unknown_fields_preserved() {
    let json = r#"{
        "asset_id": "a1",
        "logical_path": "actors/hero",
        "role": "actor",
        "version": 1,
        "entities": [],
        "future_field": true
    }"#;
    let doc: SceneAssetDocument = serde_json::from_str(json).unwrap();
    assert_eq!(doc.extension_data["future_field"], true);
}

/// Spec §sem4-per-type-preservation scenario 5: WorldDocument.
#[test]
fn world_document_unknown_fields_preserved() {
    let json = r#"{
        "id": "w1",
        "name": "W",
        "version": 1,
        "layout_policy": {"kind": "free"},
        "levels": [],
        "links": [],
        "updated_at": 0,
        "future_field": "x"
    }"#;
    let doc: WorldDocument = serde_json::from_str(json).unwrap();
    assert_eq!(doc.extension_data["future_field"], "x");
}

/// Spec §sem4-per-type-preservation scenario 6: LogicGraphAsset.
#[test]
fn logic_graph_asset_unknown_fields_preserved() {
    let json = r#"{
        "asset_id": "lg1",
        "logical_path": "logic/test",
        "version": 1,
        "nodes": [],
        "edges": [],
        "future_field": [1,2,3]
    }"#;
    let doc: LogicGraphAsset = serde_json::from_str(json).unwrap();
    assert_eq!(
        doc.extension_data["future_field"],
        serde_json::json!([1, 2, 3])
    );
}

/// Spec §sem4-per-type-preservation scenario 7: ProjectMetadata.
#[test]
fn project_metadata_unknown_fields_preserved() {
    let json = r#"{
        "version": "0.1",
        "name": "p",
        "scenes": [],
        "schemas": [],
        "active_scene": null,
        "scene_assets": [],
        "worlds": [],
        "active_world": null,
        "future_field": {"deep": {"er": 1}}
    }"#;
    let pm: ProjectMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(pm.extension_data["future_field"]["deep"]["er"], 1);
}

/// Spec §sem4-per-type-preservation scenario 8: Entity (scene) unknown fields.
#[test]
fn entity_unknown_fields_preserved() {
    let json = r#"{
        "version": "0.1",
        "scene_id": "s1",
        "name": "N",
        "entities": [{
            "id": "e1",
            "local_id": "e1",
            "name": "Hero",
            "components": [],
            "entity_future": "kept"
        }],
        "instances": {}
    }"#;
    let doc: SceneDocument = serde_json::from_str(json).unwrap();
    let entity = &doc.entities[0];
    assert_eq!(entity.extension_data["entity_future"], "kept");
}

/// Spec §sem4-per-type-preservation scenario 9: SceneAssetEntity unknown
/// fields are PRESERVED (upgraded from S2's drop behavior).
#[test]
fn scene_asset_entity_unknown_fields_preserved() {
    let json = r#"{
        "local_id": "e1",
        "local_path": "root",
        "name": "Hero",
        "components": [],
        "unknown_field": 42
    }"#;
    let entity: SceneAssetEntity = serde_json::from_str(json).unwrap();
    assert_eq!(entity.extension_data["unknown_field"], 42);

    // Round-trip: the unknown field survives encode→decode.
    let re = serde_json::to_string(&entity).unwrap();
    assert!(re.contains(r#""unknown_field":42"#), "survived: {re}");
}

/// Spec §sem4-determinism scenario 10: different unknown-field ORDER in input
/// produces identical output (BTreeMap sorting).
#[test]
fn unknown_field_order_deterministic() {
    let json_a = r#"{"version":"0.1","scene_id":"s1","name":"N","entities":[],"zeta":1,"alpha":2}"#;
    let json_b = r#"{"version":"0.1","scene_id":"s1","name":"N","entities":[],"alpha":2,"zeta":1}"#;
    let a: SceneDocument = serde_json::from_str(json_a).unwrap();
    let b: SceneDocument = serde_json::from_str(json_b).unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap(),
        "deterministic ordering required (ADR-0045)"
    );
}

/// Spec §sem4-determinism scenario 11: fixed point after first normalize.
#[test]
fn unknown_field_round_trip_fixed_point() {
    let json = r#"{"version":"0.1","scene_id":"s1","name":"N","entities":[],"future":{"x":1}}"#;
    let doc: SceneDocument = serde_json::from_str(json).unwrap();
    let once = serde_json::to_string(&doc).unwrap();
    let twice_json: SceneDocument = serde_json::from_str(&once).unwrap();
    let twice = serde_json::to_string(&twice_json).unwrap();
    assert_eq!(once, twice, "second encode equals first encode");
}

/// Spec §sem4-migration-interplay scenario 12: V0-era doc (no bag) parses
/// with an empty bag — no version bump required.
#[test]
fn v0_doc_parses_with_empty_bag() {
    let json = r#"{"version":"0.1","scene_id":"s1","name":"N","entities":[],"instances":{}}"#;
    let doc: SceneDocument = serde_json::from_str(json).unwrap();
    assert!(doc.extension_data.is_empty());
    assert_eq!(doc.version, "0.1");
}
