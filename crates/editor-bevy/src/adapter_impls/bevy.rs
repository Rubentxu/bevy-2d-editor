//! Bevy runtime adapter — encodes the semantic model to Bevy-compatible artifacts.
//!
//! Implements [`editor_model::adapter::EditorAdapter`] for [`BevyRuntimeAdapter`].
//! Declares [`editor_model::adapter::AdapterFidelity::ExportOnlyLossy`]: encoding
//! is supported, but decoding from Bevy ECS back to the editor model is not
//! possible (Bevy entities carry no editor metadata).
//!
//! # Dispatch (SDD-0046 S2 D3)
//!
//! Only the [`SemanticModel::Scene`] variant is wired to a real Bevy projection:
//!
//! - `Scene(SceneDocument)` → [`crate::dynamic_scene::export_dynamic_scene`] →
//!   `DynamicSceneExport` serialized as JSON bytes.
//!
//! The remaining variants return [`AdapterError::UnsupportedModel`]:
//!
//! - `SceneAsset`, `LogicGraph`, `World`, `ProjectMetadata` — no editor-bevy
//!   projection function accepts these types directly (`export_rust_source` and
//!   `project_instances` both take `SceneDocument`). Wiring them would require
//!   lossy container conversions that S3+ will define; the honest contract for
//!   v0.97.0 is to reject them.
//!
//! The fourth projection site — `rebuild_preview_world` — is a Bevy ECS system
//! with exclusive world access and CANNOT be called from `fn encode`. It keeps
//! running in its own dirty-flag tick (unchanged by S2).
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
        match model {
            SemanticModel::Scene(doc) => {
                // D3 prerequisite: convert the canonical model to the local
                // editor-bevy mirror, then run the real DynamicScene export.
                let bevy_doc = crate::document::SceneDocument::from(doc.clone());
                let export =
                    crate::dynamic_scene::export_dynamic_scene(&bevy_doc).map_err(|e| {
                        AdapterError::Encode {
                            adapter: self.name().into(),
                            source: Box::new(e),
                        }
                    })?;
                serde_json::to_vec(&export).map_err(|e| AdapterError::Encode {
                    adapter: self.name().into(),
                    source: Box::new(e),
                })
            }
            other => {
                let variant = format!("{other:?}");
                Err(AdapterError::UnsupportedModel {
                    adapter: self.name().into(),
                    model: variant,
                })
            }
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
    fn encode_scene_returns_dynamic_scene_bytes() {
        // Spec §sem2-bevy-dispatch scenario 7.
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
        let bytes = result.unwrap();
        let json: serde_json::Value =
            serde_json::from_slice(&bytes).expect("DynamicSceneExport should serialize as JSON");
        // DynamicSceneExport shape: version + entities
        assert!(json.get("version").is_some(), "missing version: {json}");
        assert!(json.get("entities").is_some(), "missing entities: {json}");
    }

    #[test]
    fn encode_scene_asset_unsupported() {
        // Spec deviation: no editor-bevy projection accepts SceneAssetDocument
        // directly (export_rust_source takes SceneDocument). Honest rejection.
        let adapter = BevyRuntimeAdapter::new();
        let asset = editor_model::SceneAssetDocument {
            asset_id: "hero".into(),
            logical_path: "actors/hero".into(),
            role: editor_model::scene_asset::SceneAssetRole::Actor,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: editor_model::scene_asset::SceneAssetMetadata::default(),
            layers: vec![],
        };
        let result = adapter.encode(&SemanticModel::SceneAsset(asset));
        assert!(
            matches!(result, Err(AdapterError::UnsupportedModel { .. })),
            "expected UnsupportedModel, got {result:?}"
        );
    }

    #[test]
    fn encode_logic_graph_unsupported() {
        // Spec deviation: project_instances takes SceneDocument, not
        // LogicGraphAsset. Honest rejection.
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
        assert!(
            matches!(result, Err(AdapterError::UnsupportedModel { .. })),
            "expected UnsupportedModel, got {result:?}"
        );
    }

    #[test]
    fn encode_world_unsupported() {
        // Spec §sem2-bevy-dispatch scenario 10.
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
        assert!(
            matches!(result, Err(AdapterError::UnsupportedModel { .. })),
            "expected UnsupportedModel, got {result:?}"
        );
    }

    #[test]
    fn encode_project_metadata_unsupported() {
        // Spec §sem2-bevy-dispatch scenario 11.
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
        assert!(
            matches!(result, Err(AdapterError::UnsupportedModel { .. })),
            "expected UnsupportedModel, got {result:?}"
        );
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
    fn supports_all_roles() {
        let adapter = BevyRuntimeAdapter::new();
        assert!(adapter.supports(SceneAssetRole::Actor));
        assert!(adapter.supports(SceneAssetRole::Logic));
    }
}
