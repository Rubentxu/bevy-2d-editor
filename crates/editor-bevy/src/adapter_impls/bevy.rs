//! Bevy runtime adapter — encodes editor model types as JSON for Bevy consumption.
//!
//! Implements [`editor_model::adapter::EditorAdapter`] for [`BevyRuntimeAdapter`].
//! Declares [`editor_model::adapter::AdapterFidelity::ExportOnlyLossy`]: encoding
//! to JSON for Bevy consumption is supported, but decoding from Bevy ECS back to
//! the editor model is not possible.
//!
//! The 4 wrapped Bevy projection sites are:
//! - [`crate::dynamic_scene::export_dynamic_scene`] — `SceneDocument` → `DynamicSceneExport`
//! - [`crate::instance_projection::project_instances`] — `SceneDocument` → `Vec<PreviewEntity>`
//! - [`crate::instance_projection::project_instances`] — `SceneAssetDocument` → `Vec<PreviewEntity>`
//! - `preview_runtime.rs` — world-level Bevy entity spawning (not reversible)
//!
//! In S1, this adapter encodes the semantic model as JSON bytes. The actual
//! Bevy ECS projection (entity spawning, component mapping) is performed by the
//! caller using the existing projection functions. S2 will refactor the caller
//! to use the adapter output directly.
//!
//! Decode always returns [`AdapterError::ExportOnly`].

use editor_model::adapter::{AdapterError, AdapterFidelity, EditorAdapter, SemanticModel};

/// Bevy runtime adapter — encode-only adapter for Bevy ECS consumption.
#[derive(Debug, Clone, Copy, Default)]
pub struct BevyRuntimeAdapter;

impl BevyRuntimeAdapter {
    /// Constructs a new `BevyRuntimeAdapter`.
    pub const fn new() -> Self {
        Self
    }
}

impl EditorAdapter for BevyRuntimeAdapter {
    fn name(&self) -> &str {
        "bevy.runtime.v1"
    }

    fn fidelity(&self) -> AdapterFidelity {
        AdapterFidelity::ExportOnlyLossy
    }

    fn encode(&self, model: &SemanticModel) -> Result<Vec<u8>, AdapterError> {
        // Serialize the semantic model as JSON bytes.
        // The caller uses this to drive Bevy ECS projection via the existing
        // projection functions (export_dynamic_scene, project_instances, etc.).
        match model {
            SemanticModel::Scene(doc) => {
                serde_json::to_string(doc)
                    .map(Into::into)
                    .map_err(|e| AdapterError::Encode {
                        adapter: self.name().into(),
                        source: e.into(),
                    })
            }
            SemanticModel::SceneAsset(doc) => {
                serde_json::to_string(doc)
                    .map(Into::into)
                    .map_err(|e| AdapterError::Encode {
                        adapter: self.name().into(),
                        source: e.into(),
                    })
            }
            SemanticModel::LogicGraph(asset) => serde_json::to_string(asset)
                .map(Into::into)
                .map_err(|e| AdapterError::Encode {
                    adapter: self.name().into(),
                    source: e.into(),
                }),
            SemanticModel::World(doc) => {
                serde_json::to_string(doc)
                    .map(Into::into)
                    .map_err(|e| AdapterError::Encode {
                        adapter: self.name().into(),
                        source: e.into(),
                    })
            }
            SemanticModel::ProjectMetadata(pm) => serde_json::to_string(pm)
                .map(Into::into)
                .map_err(|e| AdapterError::Encode {
                    adapter: self.name().into(),
                    source: e.into(),
                }),
        }
    }

    fn decode(&self, _bytes: &[u8]) -> Result<SemanticModel, AdapterError> {
        // Bevy ECS entities carry no editor metadata — projection is one-way only.
        Err(AdapterError::ExportOnly {
            adapter: self.name().into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_model::adapter::SemanticModel;
    use editor_model::scene_asset::SceneAssetRole;
    use std::collections::BTreeMap;

    #[test]
    fn encode_scene_document() {
        let adapter = BevyRuntimeAdapter::new();
        let doc = editor_model::SceneDocument {
            version: "0.1".into(),
            scene_id: "test".into(),
            name: "Test Scene".into(),
            entities: vec![],
            instances: BTreeMap::new(),
        };
        let result = adapter.encode(&SemanticModel::Scene(doc));
        assert!(result.is_ok(), "encode failed: {:?}", result.err());
        let json = String::from_utf8(result.unwrap()).unwrap();
        assert!(json.contains(r#""scene_id":"test""#));
    }

    #[test]
    fn encode_project_metadata() {
        let adapter = BevyRuntimeAdapter::new();
        let pm = editor_model::ProjectMetadata {
            version: "0.1".into(),
            name: "Test Project".into(),
            scenes: vec![],
            schemas: vec![],
            active_scene: None,
            scene_assets: vec![],
            worlds: vec![],
            active_world: None,
        };
        let result = adapter.encode(&SemanticModel::ProjectMetadata(pm));
        assert!(result.is_ok(), "encode failed: {:?}", result.err());
        let json = String::from_utf8(result.unwrap()).unwrap();
        assert!(json.contains(r#""name":"Test Project""#));
    }

    #[test]
    fn decode_is_export_only() {
        let adapter = BevyRuntimeAdapter::new();
        let result = adapter.decode(b"anything");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AdapterError::ExportOnly { .. }));
    }

    #[test]
    fn fidelity_is_export_only_lossy() {
        let adapter = BevyRuntimeAdapter::new();
        assert_eq!(adapter.fidelity(), AdapterFidelity::ExportOnlyLossy);
    }

    #[test]
    fn encode_logic_graph() {
        let adapter = BevyRuntimeAdapter::new();
        let graph = editor_model::LogicGraphAsset {
            asset_id: "lg1".into(),
            logical_path: "logic/test".into(),
            version: 1,
            builtin: false,
            nodes: vec![],
            edges: vec![],
        };
        let result = adapter.encode(&SemanticModel::LogicGraph(graph));
        assert!(result.is_ok(), "encode failed: {:?}", result.err());
    }

    #[test]
    fn encode_world_document() {
        let adapter = BevyRuntimeAdapter::new();
        let world = editor_model::world::WorldDocument {
            id: editor_model::world::WorldId("w1".into()),
            name: "Test World".into(),
            version: 1,
            layout_policy: editor_model::world::LayoutPolicy::Grid { cell_size: 32 },
            levels: vec![],
            links: vec![],
            updated_at: 0,
        };
        let result = adapter.encode(&SemanticModel::World(world));
        assert!(result.is_ok(), "encode failed: {:?}", result.err());
    }
}
