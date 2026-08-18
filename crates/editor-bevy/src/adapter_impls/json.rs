//! JSON adapter — wraps all 6 JSON writer sites in `editor-bevy`.
//!
//! Implements [`editor_model::adapter::EditorAdapter`] for [`JsonProjectAdapter`].
//! Declares [`editor_model::adapter::AdapterFidelity::Lossless`] for all supported
//! variants, with one documented caveat:
//!
//! > **Caveat** (`crates/editor-model/src/scene_asset.rs:76`):
//! > `SceneAssetEntity` uses `#[serde(deny_unknown_fields)]`, so unknown JSON fields
//! > in a `SceneAssetDocument` entity will cause decode to fail. All other document
//! > types silently drop unknown fields (via `#[serde(default)]`). This asymmetry means
//! > a `SceneAssetDocument` that round-tripped through a format that added fields would
//! > not be byte-exact. S4 (extension bag) will address this by promoting all types to
//! > true lossless discipline.
//!
//! The 6 wrapped JSON writer sites are:
//! - `crates/editor-bevy/src/lib.rs` — `SceneDocument` save/load (2 sites)
//! - `crates/editor-bevy/src/lib.rs` — `SceneAssetDocument` save/load (2 sites)
//! - `crates/editor-bevy/src/lib.rs` — `WorldDocument` save/load (1 site)
//! - `crates/editor-bevy/src/persistence.rs` — `ProjectMetadata` save/load (1 site)
//!
//! Top-level key sniffing is used for decode dispatch (8 lines):
//! - `"nodes"`, `"edges"`         → `LogicGraphAsset`
//! - `"scene_id"`, `"entities"`    → `SceneDocument`
//! - `"asset_id"`, `"entities"`    → `SceneAssetDocument`
//! - `"schemas"`, `"scenes"`       → `ProjectMetadata`
//! - `"levels"`, `"links"`         → `WorldDocument`

use editor_model::adapter::{AdapterError, AdapterFidelity, EditorAdapter, SemanticModel};
use editor_model::scene_asset::SceneAssetDocument;
use editor_model::{LogicGraphAsset, ProjectMetadata, SceneDocument, WorldDocument};
use serde::Deserialize;
use serde_json::Value;

/// JSON adapter — lossless encode/decode for all 5 semantic model variants.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonProjectAdapter;

impl JsonProjectAdapter {
    /// Constructs a new `JsonProjectAdapter`.
    pub const fn new() -> Self {
        Self
    }
}

impl EditorAdapter for JsonProjectAdapter {
    fn name(&self) -> &str {
        "json.project.v1"
    }

    fn fidelity(&self) -> AdapterFidelity {
        AdapterFidelity::Lossless
    }

    fn encode(&self, model: &SemanticModel<'_>) -> Result<Vec<u8>, AdapterError> {
        let name = self.name();
        match model {
            SemanticModel::Scene(doc) => {
                serde_json::to_string(doc)
                    .map(Into::into)
                    .map_err(|e| AdapterError::Encode {
                        adapter: name.into(),
                        source: e.into(),
                    })
            }
            SemanticModel::SceneAsset(doc) => {
                serde_json::to_string(doc)
                    .map(Into::into)
                    .map_err(|e| AdapterError::Encode {
                        adapter: name.into(),
                        source: e.into(),
                    })
            }
            SemanticModel::LogicGraph(asset) => serde_json::to_string(asset)
                .map(Into::into)
                .map_err(|e| AdapterError::Encode {
                    adapter: name.into(),
                    source: e.into(),
                }),
            SemanticModel::World(doc) => {
                serde_json::to_string(doc)
                    .map(Into::into)
                    .map_err(|e| AdapterError::Encode {
                        adapter: name.into(),
                        source: e.into(),
                    })
            }
            SemanticModel::ProjectMetadata(pm) => serde_json::to_string(pm)
                .map(Into::into)
                .map_err(|e| AdapterError::Encode {
                    adapter: name.into(),
                    source: e.into(),
                }),
        }
    }

    fn decode(&self, bytes: &[u8]) -> Result<SemanticModel<'static>, AdapterError> {
        let name = self.name();
        let json: Value = serde_json::from_slice(bytes).map_err(|e| AdapterError::Decode {
            adapter: name.into(),
            source: e.into(),
        })?;

        let obj = json.as_object().ok_or_else(|| AdapterError::Decode {
            adapter: name.into(),
            source: "JSON value is not an object".into(),
        })?;

        // Key sniff — 8 lines, per D5.
        // Deserialize to owned value, leak to get &'static ref for SemanticModel.
        if obj.contains_key("nodes") && obj.contains_key("edges") {
            let owned: LogicGraphAsset =
                serde_json::from_value(json.clone()).map_err(|e| AdapterError::Decode {
                    adapter: name.into(),
                    source: e.into(),
                })?;
            let leaked: &'static LogicGraphAsset = Box::leak(Box::new(owned));
            return Ok(SemanticModel::LogicGraph(leaked));
        }
        if obj.contains_key("scenes") && obj.contains_key("schemas") {
            let owned: ProjectMetadata =
                serde_json::from_value(json.clone()).map_err(|e| AdapterError::Decode {
                    adapter: name.into(),
                    source: e.into(),
                })?;
            let leaked: &'static ProjectMetadata = Box::leak(Box::new(owned));
            return Ok(SemanticModel::ProjectMetadata(leaked));
        }
        if obj.contains_key("levels") && obj.contains_key("links") {
            let owned: WorldDocument =
                serde_json::from_value(json.clone()).map_err(|e| AdapterError::Decode {
                    adapter: name.into(),
                    source: e.into(),
                })?;
            let leaked: &'static WorldDocument = Box::leak(Box::new(owned));
            return Ok(SemanticModel::World(leaked));
        }
        if obj.contains_key("scene_id") && obj.contains_key("entities") {
            let owned: SceneDocument =
                serde_json::from_value(json.clone()).map_err(|e| AdapterError::Decode {
                    adapter: name.into(),
                    source: e.into(),
                })?;
            let leaked: &'static SceneDocument = Box::leak(Box::new(owned));
            return Ok(SemanticModel::Scene(leaked));
        }
        if obj.contains_key("asset_id") && obj.contains_key("entities") {
            let owned: SceneAssetDocument =
                serde_json::from_value(json.clone()).map_err(|e| AdapterError::Decode {
                    adapter: name.into(),
                    source: e.into(),
                })?;
            let leaked: &'static SceneAssetDocument = Box::leak(Box::new(owned));
            return Ok(SemanticModel::SceneAsset(leaked));
        }

        Err(AdapterError::Decode {
            adapter: name.into(),
            source: "unrecognised JSON top-level keys".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_model::ComponentInstance;
    use editor_model::adapter::SemanticModel;
    use editor_model::ids::StableId;
    use editor_model::scene_asset::{SceneAssetEntity, SceneAssetMetadata, SceneAssetRole};
    use std::collections::BTreeMap;

    #[test]
    fn encode_scene_document() {
        let adapter = JsonProjectAdapter::new();
        let doc = SceneDocument {
            version: "0.1".into(),
            scene_id: "test-scene".into(),
            name: "Test Scene".into(),
            entities: vec![],
            instances: BTreeMap::new(),
        };
        let bytes = adapter.encode(&SemanticModel::Scene(&doc)).unwrap();
        let json_str = String::from_utf8(bytes).unwrap();
        assert!(json_str.contains(r#""scene_id":"test-scene""#));
        assert!(json_str.contains(r#""version":"0.1""#));
    }

    #[test]
    fn decode_scene_document() {
        let adapter = JsonProjectAdapter::new();
        let json =
            r#"{"version":"0.1","scene_id":"s1","name":"My Scene","entities":[],"instances":{}}"#;
        let model = adapter.decode(json.as_bytes()).unwrap();
        match model {
            SemanticModel::Scene(doc) => {
                assert_eq!(doc.scene_id, "s1");
                assert_eq!(doc.name, "My Scene");
            }
            _ => panic!("expected Scene, got {:?}", model),
        }
    }

    #[test]
    fn decode_project_metadata() {
        let adapter = JsonProjectAdapter::new();
        let json = r#"{"version":"0.1","name":"My Project","scenes":["a"],"schemas":[],"scene_assets":[],"worlds":[]}"#;
        let model = adapter.decode(json.as_bytes()).unwrap();
        match model {
            SemanticModel::ProjectMetadata(pm) => {
                assert_eq!(pm.name, "My Project");
                assert_eq!(pm.scenes, vec!["a"]);
            }
            _ => panic!("expected ProjectMetadata, got {:?}", model),
        }
    }

    #[test]
    fn decode_world_document() {
        let adapter = JsonProjectAdapter::new();
        let json = r#"{"id":"w1","name":"World","version":1,"layout_policy":{"kind":"grid","cell_size":32},"levels":[],"links":[],"updated_at":0}"#;
        let model = adapter.decode(json.as_bytes()).unwrap();
        match model {
            SemanticModel::World(doc) => {
                assert_eq!(doc.id.as_str(), "w1");
                assert_eq!(doc.name, "World");
            }
            _ => panic!("expected World, got {:?}", model),
        }
    }

    #[test]
    fn decode_logic_graph_asset() {
        let adapter = JsonProjectAdapter::new();
        let json = r#"{"asset_id":"lg1","logical_path":"logic/test","version":1,"builtin":false,"nodes":[],"edges":[]}"#;
        let model = adapter.decode(json.as_bytes()).unwrap();
        match model {
            SemanticModel::LogicGraph(doc) => {
                assert_eq!(doc.asset_id, "lg1");
            }
            _ => panic!("expected LogicGraph, got {:?}", model),
        }
    }

    #[test]
    fn round_trip_scene_document() {
        let adapter = JsonProjectAdapter::new();
        let doc = SceneDocument {
            version: "0.1".into(),
            scene_id: "roundtrip".into(),
            name: "Round Trip".into(),
            entities: vec![],
            instances: BTreeMap::new(),
        };
        let encoded = adapter.encode(&SemanticModel::Scene(&doc)).unwrap();
        let decoded = adapter.decode(&encoded).unwrap();
        match decoded {
            SemanticModel::Scene(d) => {
                assert_eq!(d.scene_id, "roundtrip");
                assert_eq!(d.name, "Round Trip");
            }
            _ => panic!("expected Scene"),
        }
    }

    #[test]
    fn round_trip_project_metadata() {
        let adapter = JsonProjectAdapter::new();
        let pm = ProjectMetadata {
            version: "0.1".into(),
            name: "Round Trip Project".into(),
            scenes: vec!["s1".into(), "s2".into()],
            schemas: vec![],
            active_scene: Some("s1".into()),
            scene_assets: vec![],
            worlds: vec![],
            active_world: None,
        };
        let encoded = adapter
            .encode(&SemanticModel::ProjectMetadata(&pm))
            .unwrap();
        let decoded = adapter.decode(&encoded).unwrap();
        match decoded {
            SemanticModel::ProjectMetadata(d) => {
                assert_eq!(d.name, "Round Trip Project");
                assert_eq!(d.scenes, vec!["s1", "s2"]);
                assert_eq!(d.active_scene, Some("s1".into()));
            }
            _ => panic!("expected ProjectMetadata"),
        }
    }

    #[test]
    fn decode_scene_asset_document() {
        let adapter = JsonProjectAdapter::new();
        let json = r#"{"asset_id":"a1","logical_path":"actors/hero","role":"actor","version":1,"entities":[],"relationships":[],"metadata":{"tags":null,"notes":""},"layers":[]}"#;
        let model = adapter.decode(json.as_bytes()).unwrap();
        match model {
            SemanticModel::SceneAsset(doc) => {
                assert_eq!(doc.asset_id, "a1");
                assert_eq!(doc.logical_path, "actors/hero");
            }
            _ => panic!("expected SceneAsset, got {:?}", model),
        }
    }

    #[test]
    fn unsupported_json_error() {
        let adapter = JsonProjectAdapter::new();
        let json = r#"{"unknown_top_level":"value"}"#;
        let result = adapter.decode(json.as_bytes());
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("unrecognised JSON top-level keys"),
            "got: {msg}"
        );
    }
}
