//! Bevy runtime adapter — encodes editor model types to Bevy ECS projections.
//!
//! Implements [`editor_model::adapter::EditorAdapter`] for [`BevyRuntimeAdapter`].
//! Declares [`editor_model::adapter::AdapterFidelity::ExportOnlyLossy`]: encoding
//! to Bevy runtime projections is supported, but decoding from Bevy ECS back to
//! the editor model is not possible.
//!
//! The 4 wrapped Bevy projection sites are:
//! - [`crate::dynamic_scene::export_dynamic_scene`] — `SceneDocument` → `DynamicSceneExport`
//! - [`crate::instance_projection::project_instances`] — `SceneDocument` → `Vec<PreviewEntity>`
//! - [`crate::instance_projection::project_instances`] — `SceneAssetDocument` → `Vec<PreviewEntity>`
//! - `preview_runtime.rs` — world-level Bevy entity spawning (not reversible)
//!
//! Decode always returns [`AdapterError::ExportOnly`].

use editor_model::adapter::{AdapterError, AdapterFidelity, EditorAdapter, SemanticModel};
use crate::dynamic_scene::{DynamicSceneExport, EntityExport, ExportWarning};
use crate::instance_projection::project_instances;
use crate::scene_asset::SceneAssetDocument;
use editor_model::SceneDocument;

/// Bevy runtime adapter — encode-only projection of editor model to Bevy ECS.
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

    fn encode(&self, model: &SemanticModel<'_>) -> Result<Vec<u8>, AdapterError> {
        match model {
            SemanticModel::Scene(doc) => {
                let name = self.name();
                // Project to DynamicSceneExport (no resolver — None paths are pruned).
                let export = crate::dynamic_scene::export_dynamic_scene(doc)
                    .map_err(|e| AdapterError::Encode {
                        adapter: name.into(),
                        source: e.into(),
                    })?;
                // Encode as JSON bytes.
                serde_json::to_string(&export)
                    .map(Into::into)
                    .map_err(|e| AdapterError::Encode {
                        adapter: name.into(),
                        source: e.into(),
                    })
            }
            SemanticModel::SceneAsset(doc) => {
                let name = self.name();
                // Project to Vec<PreviewEntity> using the empty resolver.
                let resolver = |_ref: &editor_model::scene_asset::AssetReference| None;
                let projected = project_instances(doc, &resolver);
                serde_json::to_string(&projected)
                    .map(Into::into)
                    .map_err(|e| AdapterError::Encode {
                        adapter: name.into(),
                        source: e.into(),
                    })
            }
            // Other variants are not reachable from Bevy runtime.
            other => {
                let variant = format!("{:?}", other);
                Err(AdapterError::UnsupportedModel {
                    adapter: self.name().into(),
                    model: variant,
                })
            }
        }
    }

    fn decode(&self, _bytes: &[u8]) -> Result<SemanticModel<'static>, AdapterError> {
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
        let doc = SceneDocument {
            version: "0.1".into(),
            scene_id: "test".into(),
            name: "Test Scene".into(),
            entities: vec![],
            instances: BTreeMap::new(),
        };
        let result = adapter.encode(&SemanticModel::Scene(&doc));
        assert!(result.is_ok(), "encode failed: {:?}", result.err());
        let json = String::from_utf8(result.unwrap()).unwrap();
        assert!(json.contains(r#""source_scene_id":"test""#));
    }

    #[test]
    fn encode_scene_asset() {
        let adapter = BevyRuntimeAdapter::new();
        let doc = SceneAssetDocument {
            asset_id: "a1".into(),
            logical_path: "actors/hero".into(),
            role: SceneAssetRole::Actor,
            version: 1,
            entities: vec![],
            relationships: vec![],
            exposed_properties: vec![],
            metadata: editor_model::scene_asset::SceneAssetMetadata::default(),
            layers: vec![],
        };
        let result = adapter.encode(&SemanticModel::SceneAsset(&doc));
        assert!(result.is_ok(), "encode failed: {:?}", result.err());
        // Vec<PreviewEntity> serializes as a JSON array.
        let json = String::from_utf8(result.unwrap()).unwrap();
        assert!(json.starts_with("["), "expected JSON array, got: {json}");
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
    fn unsupported_model_project_metadata() {
        let adapter = BevyRuntimeAdapter::new();
        let pm = editor_model::ProjectMetadata {
            version: "0.1".into(),
            name: "Test".into(),
            scenes: vec![],
            schemas: vec![],
            active_scene: None,
            scene_assets: vec![],
            worlds: vec![],
            active_world: None,
        };
        let result = adapter.encode(&SemanticModel::ProjectMetadata(&pm));
        assert!(matches!(result, Err(AdapterError::UnsupportedModel { .. })));
    }
}
